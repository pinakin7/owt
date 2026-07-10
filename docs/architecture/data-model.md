---
type: hld
status: draft
owner: pinakin
summary: "Canonical entities, identifier grammar, the normalized ingest envelope, and schema versioning policy."
tags: [area/data-model, release/mvp]
related:
  - data-sources.md
  - ../design/normalization.md
  - ../design/storage.md
  - ../product/glossary.md
---

# Data model

The contract every pipeline honors. Three layers, strictly separated:

1. **Source payloads** — vendor-shaped JSON, kept verbatim inside envelopes; never queried directly.
2. **Canonical records** — owt-shaped facts and snapshots defined here; the durable product data.
3. **Derived documents** — search docs, candles, timeline projections, forecast points, and aggregated entity odds ([ADR-0012](../adr/0012-forecast-derived-data-module.md)); always rebuildable from layer 2.

Facts (ticks, trades, fills, news, comments, timeline items) are **append-only**. Snapshots (`market_state`, market/event metadata) are **mutable with `updated_at`**. Rust definitions live in `owt-domain` and are the normative encoding; the JSON here is documentation.

## Identifier grammar

| ID | Format | Source of truth |
|---|---|---|
| `event_id`, `event_slug` | Polymarket-native string / slug, verbatim | Gamma |
| `market_id`, `market_slug` | Polymarket-native string / slug, verbatim | Gamma |
| `condition_id`, `question_id` | 0x-hex, verbatim | Gamma/CLOB |
| `token_id` | Polymarket-native decimal string, verbatim | CLOB |
| `entity_id` | `ent_{kebab-name}` owt-assigned, stable | entity dictionary |
| `news_id` | `sha256(canonical_url)` hex | normalizer |
| `trade_id` | source-native ID, verbatim | Data API |
| `fill_id` | `{tx_hash}:{log_index}` | Goldsky (v1) |
| `comment_id` | source-native ID, verbatim | RTDS (v1) |
| `timeline_id` | `event:{event_id}:{ts_rfc3339}:{kind}:{hash8}` | timeline builder |
| `model_id` | `{name}.v{major}` owt-assigned (e.g. `momentum.v1`) | forecast engine (v1, [forecasts](../design/forecasts.md)) |
| `envelope_id` | UUIDv7 | adapter at receipt |
| internal rows | UUIDv7 surrogate keys where a natural key is absent | store |

Rules: source-native IDs are never rewritten; owt-assigned IDs are deterministic where possible (replay yields identical IDs); every canonical record carries `source_refs: ["gamma:market:12345", "clob:token:4833…"]` — provenance strings of the form `{source}:{kind}:{native_id}`.

## The ingest envelope (normative)

Every payload entering the bus is wrapped exactly like this — the shared contract between adapters, normalizer, replay, and archive:

```json
{
  "envelope_id": "0197c9f2-7b3a-7cc1-a1e2-3f6b9d2f4a11",
  "source": "pm_ws",
  "kind": "market.price_change",
  "ts": "2026-07-07T10:14:02.184Z",
  "received_at": "2026-07-07T10:14:02.391Z",
  "entity_keys": { "token_id": "4833…", "market_id": "12345", "event_id": "678" },
  "partition_key": "12345",
  "ingest_version": "1.2.0",
  "schema_version": 1,
  "payload": { "…vendor JSON, verbatim…": true }
}
```

| Field | Meaning |
|---|---|
| `envelope_id` | UUIDv7, minted once at receipt; JetStream dedupe key (`Nats-Msg-Id`); archive object name |
| `source` | adapter identity: `gamma`, `clob`, `data_api`, `pm_ws`, `pm_rtds`, `goldsky`, `rss`, `gdelt`, `x` |
| `kind` | source-scoped event kind, dotted (`market.book`, `news.item`) |
| `ts` | upstream event time (best available); `received_at` — our clock at receipt |
| `entity_keys` | canonical keys known at receipt; may be partial — the normalizer completes them |
| `partition_key` | ordering key (usually `market_id`); maps to subject token |
| `ingest_version` | semver of the *adapter+normalizer* pipeline that produced/will produce canonical output |
| `schema_version` | envelope schema major (this is 1) |
| `payload` | vendor JSON **verbatim** — no cleanup before the bus, ever |

## Canonical entities

### Market (snapshot)

```json
{
  "market_id": "12345",
  "market_slug": "will-fed-cut-rates-in-september",
  "event_id": "678",
  "condition_id": "0x…",
  "question_id": "0x…",
  "title": "Will the Fed cut rates in September?",
  "description": "…",
  "outcomes": [
    { "name": "Yes", "token_id": "4833…", "price": 0.42 },
    { "name": "No",  "token_id": "9321…", "price": 0.58 }
  ],
  "enable_order_book": true,
  "status": "active",
  "tags": ["economics", "fed"],
  "resolution_source": "official-market-rules",
  "created_at": "2026-07-01T12:00:00Z",
  "updated_at": "2026-07-07T10:15:01Z",
  "source_refs": ["gamma:market:12345"]
}
```

Note: outcomes are structured objects (name + token + price together), not the parallel `outcomes`/`outcomePrices` string arrays Gamma serves — the normalizer zips them ([normalization](../design/normalization.md)). Live top-of-book state (best bid/ask, last trade, 24h volume) lives in the separate `market_state` snapshot, not here.

### Event (snapshot)

`event_id`, `event_slug`, `title`, `description`, `tags`, `market_ids[]`, `status`, timestamps, `source_refs`. One event groups one or more markets.

### PricePoint (fact)

`token_id`, `market_id`, `ts`, `price`, `size?`, `kind` (`trade` | `midpoint` | `best_bid` | `best_ask` | `history_backfill`), `source_refs`. Feeds `price_ticks` and the candle aggregates.

### BookSnapshot (fact, short retention)

`token_id`, `market_id`, `ts`, `bids[{price,size}]`, `asks[{price,size}]`, `seq?`. Top-N levels only; full-depth history is deliberately not kept ([storage](../design/storage.md)).

### Trade (fact — Data API tape)

`trade_id`, `market_id`, `token_id`, `ts`, `side`, `price`, `size`, `taker?`, `source_refs`.

### OrderFill (fact — Goldsky, v1)

```json
{
  "fill_id": "0xabc…:17",
  "block_number": 12345678,
  "block_timestamp": "2026-07-07T09:21:42Z",
  "transaction_hash": "0xabc…",
  "maker": "0xdef…",
  "taker": "0x123…",
  "token_id": "4833…",
  "market_id": "12345",
  "side": "BUY",
  "amount_shares": 100.0,
  "amount_collateral": 42.0,
  "collateral_token": "pUSD",
  "price": 0.42,
  "fee": 0.03,
  "source_refs": ["goldsky:order_filled:0xabc…:17"]
}
```

`amount_collateral` + `collateral_token` (pUSD era) — **not** the report's `amount_usdc`. Reconciliation with Data-API trades: [normalization](../design/normalization.md).

### NewsItem (fact)

```json
{
  "news_id": "sha256(canonical_url)",
  "source_kind": "rss",
  "publisher": "Reuters",
  "headline": "Fed official signals caution on cuts",
  "summary": "…publisher summary…",
  "body_text": null,
  "canonical_url": "https://…",
  "published_at": "2026-07-07T09:22:00Z",
  "language": "en",
  "entities": ["ent_federal-reserve", "ent_jerome-powell"],
  "event_links": ["678"],
  "market_links": ["12345"],
  "sentiment": null,
  "importance": null,
  "license_class": "metadata_excerpt",
  "source_refs": ["rss:item:…"]
}
```

`body_text` only where `license_class` permits; `sentiment`/`importance` are **nullable and null at MVP** — dictionary-based linking first, model-assisted enrichment later ([roadmap divergence 9](../product/roadmap.md#divergences-from-the-research-report)).

### Comment (fact, v1)

`comment_id`, `market_id`/`event_id`, `ts`, `author_handle`, `body`, `source_refs`.

### Entity + EntityAlias (dictionary)

`entity_id`, `kind` (`person` | `org` | `topic` | `place`), `name`, `description` (nullable; human-curated context blurb, never generated — [ADR-0011](../adr/0011-topic-lookup-entity-composite.md)), `aliases[]` (with per-alias match policy and confidence), `external_refs`. Human-auditable: every alias row records who/what added it. Entities are the topic-lookup surface: the composite entity read plus the derived `EntityOdds` and `ForecastPoint` documents (layer 3) power the topic page ([topic-lookup](../design/topic-lookup.md)).

### TimelineItem (fact)

```json
{
  "timeline_id": "event:678:2026-07-07T09:22:00Z:news:1f2a3b4c",
  "event_id": "678",
  "market_id": "12345",
  "ts": "2026-07-07T09:22:00Z",
  "kind": "news",
  "title": "Fed official signals caution on cuts",
  "body_preview": "…",
  "numeric_delta": { "price_before": 0.39, "price_after": 0.42, "delta_bp": 300, "window_s": 1080 },
  "confidence": 0.68,
  "refs": { "news_id": "sha256(…)" },
  "source_refs": ["rss:item:…", "clob:token:4833…"]
}
```

`kind` ∈ `price_move` | `news` | `trade_burst` | `comment_spike` (v1) | `resolution`. Construction and confidence rules: [normalization](../design/normalization.md).

### Workspace records (mutable)

`workspaces`, `watchlists`, `saved_views` — all carry `workspace_id` from day one; a single `default` workspace exists until v2 auth ([ADR-0008](../adr/0008-read-only-through-v1.md)). Reserved for later: `social_posts` (v2), `alert_rules`/`alert_events` (v1, [alerts](../design/alerts.md)).

## Versioning

- **`schema_version`** (envelope): major only; a bump means consumers must be updated before adapters — expected to be rare.
- **`ingest_version`** (pipeline semver): patch = bugfix with identical output; minor = output enriched (new nullable fields); major = mapping semantics changed → **triggers replay** of affected `raw.*` ranges ([realtime-bus](../design/realtime-bus.md)).
- **Canonical record evolution:** additive-only within a `schema_version`; renames/retypes require a major bump + replay + an ADR.
- **`model_version`** (forecast models, v1): mirrors `ingest_version` — minor = enriched output; major = semantics changed → **triggers rebuild** of that model's derived rows, recorded in `model_registry` ([forecasts](../design/forecasts.md), [ADR-0012](../adr/0012-forecast-derived-data-module.md)).
- **Determinism requirement:** same envelope + same `ingest_version` ⇒ byte-identical canonical record (property-tested; [testing-strategy](../ops/testing-strategy.md)).

## Related

- [data-sources.md](data-sources.md)
- [../design/normalization.md](../design/normalization.md)
- [../design/storage.md](../design/storage.md)
- [../product/glossary.md](../product/glossary.md)
