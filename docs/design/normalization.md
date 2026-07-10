---
type: lld
status: draft
owner: pinakin
summary: "LLD for source-to-canonical mapping, cross-source dedupe and precedence, entity resolution, and timeline construction."
tags: [area/data-model, area/ingestion, release/mvp]
related:
  - ../architecture/data-model.md
  - ../architecture/data-sources.md
  - ingestion.md
  - storage.md
---

# Normalization & entity resolution

`owt-normalize` consumes `raw.*` envelopes and emits canonical records to `canon.v1.*`. It is the only code allowed to interpret vendor payloads, and it must be **deterministic**: same envelope + same `ingest_version` ⇒ byte-identical canonical output (property-tested; this is what makes replay meaningful).

## Pipeline

```
raw.* envelope
  → deserialize (tolerant)      — unknown fields accepted, drift counter on new ones
  → map to canonical record     — tables below
  → complete entity_keys        — token→market→event lookups from reference cache
  → dedupe / precedence check   — cross-source rules below
  → entity resolution           — news/comments only
  → emit canon.v1.* envelope    — same envelope_id lineage, ingest_version stamped
```

Failures follow the [ingestion failure matrix](ingestion.md#failure-mode-matrix): deserialization failure → DLQ + drift counter; *partial* normalization (record ok, enrichment failed) emits the record with the enrichment fields null and a warning metric — never drops data for a missing nicety.

## Mapping tables (normative once fixtures land)

Columns: source field → canonical field · transform · null policy. Payload shapes marked ⚠ in [data-sources](../architecture/data-sources.md) are pinned by recorded fixtures before these tables flip to `approved`.

### Gamma market → Market

| Source | Canonical | Transform / rule |
|---|---|---|
| `id` | `market_id` | verbatim string |
| `slug` | `market_slug` | verbatim |
| `events[0].id` / event ref | `event_id` | required; DLQ if absent |
| `conditionId` | `condition_id` | verbatim hex |
| `questionID` | `question_id` | verbatim hex; nullable |
| `question`/`title` | `title` | prefer `question`; trim |
| `outcomes` **(stringified JSON array)** | `outcomes[].name` | parse JSON string → array; DLQ on parse fail |
| `outcomePrices` **(stringified)** | `outcomes[].price` | parse; f64 in [0,1]; zip by index with names |
| `clobTokenIds` **(stringified)** | `outcomes[].token_id` | parse; zip by index; lengths must match — else DLQ |
| `enableOrderBook` | `enable_order_book` | bool, default false |
| `active`/`closed`/`archived` | `status` | fold to `active`\|`closed`\|`archived` |
| `tags[]` | `tags` | slug-normalize, dedupe |

### CLOB → PricePoint / BookSnapshot

| Source | Canonical | Rule |
|---|---|---|
| WS `price_change` | PricePoint`{kind:best_bid/best_ask}` | one point per side present; best bid/ask read from `price_change` fields (no separate `best_bid_ask` message assumed — ⚠ fixture-verify) |
| WS `last_trade_price` | PricePoint`{kind:trade}` | price+size |
| WS `book` | BookSnapshot | top-N levels, keep `seq` if present |
| REST `prices-history` point | PricePoint`{kind:history_backfill}` | `(t, p)` → ts, price |
| WS `tick_size_change` | Market snapshot patch | update `tick_size` |

### Data API trade → Trade

`id→trade_id` verbatim · market/token ids verbatim · `side` uppercased BUY/SELL · `price`,`size` f64 · `timestamp` → `ts` (epoch→RFC3339 UTC).

### Goldsky `order_filled` → OrderFill (v1)

`transaction_hash + log_index → fill_id` · block fields verbatim · maker/taker lowercased hex · amounts → `amount_shares`, **`amount_collateral` + `collateral_token:"pUSD"`** · `price = amount_collateral / amount_shares` when not explicit (guard div-by-zero → DLQ).

### RSS item → NewsItem

| Source | Canonical | Rule |
|---|---|---|
| `link` | `canonical_url` → `news_id` | canonicalize (below) then sha256 |
| `title` | `headline` | trim, collapse whitespace |
| `description`/`summary` | `summary` | strip HTML; ≤ 2,000 chars |
| full content | `body_text` | **only if** feed's `license_class` permits, else null |
| `pubDate` | `published_at` | parse RFC822/RFC3339; fallback `received_at` with flag |
| feed registry | `publisher`, `source_kind:"rss"`, `license_class` | from config |

**URL canonicalization:** lowercase scheme+host · strip fragments · strip tracking params (`utm_*`, `fbclid`, `gclid`, …) · resolve to publisher canonical (`<link rel=canonical>` when body fetched) · sort remaining query params. Deterministic and version-locked — changing it is an `ingest_version` **major** (it re-keys news).

### RTDS comment → Comment (v1)

id/market/ts verbatim · `body` length-capped (10 k) · author handle verbatim.

## Cross-source dedupe & precedence

Authority ranking comes from [data-sources](../architecture/data-sources.md#authority-hierarchy); ties broken field-wise:

- **Trade vs OrderFill (v1):** match on `(token_id, ts±2s, size, price)`; when matched, the Trade row gains `fill_id` linkage. On-chain wins for settlement facts (amounts, addresses); Data API wins for tape latency. Unmatched fills after 5 min are kept standalone (they're truth).
- **News:** primary key is canonical-URL hash. Near-dupes across publishers (wire stories) additionally cluster on `(normalized_title_simhash, published_at±2h)` — clustered items keep distinct `news_id`s but share a `cluster_id` so timelines show one item with N sources.
- **Market metadata:** Gamma wins over anything derived; a WS-derived state never overwrites Gamma reference fields.
- Suppression counted: `dedupe_suppressed_total{kind}`.

## Entity resolution (MVP: dictionary + rules)

1. **Dictionary:** `entities` + `entity_aliases` tables, seeded from market/event tags and a hand-curated starter pack (Fed/FOMC/Powell, elections, major geopolitics). Every alias row: `(alias, entity_id, match_policy, confidence, added_by)`. Entity `description` blurbs are part of the curated dictionary — human-written (starter pack or contributor-added), **never generated** ([ADR-0011](../adr/0011-topic-lookup-entity-composite.md)); they are user-facing on the topic page.
2. **Matcher:** exact alias → case/punctuation-normalized alias → token-boundary phrase match, longest-alias-first, per-alias `match_policy` can require context words (e.g. "Fed" requires a finance-context token nearby to avoid "federal court").
3. **Output:** `entities[]` on NewsItems/Comments; `market_links`/`event_links` via entity→market mappings (markets inherit entities from their tags + title matching at sweep time).
4. **Auditability:** resolution decisions are reproducible from the dictionary version; `owtd` exposes `explain-entity <news_id>` (debug subcommand) showing which aliases fired.
5. **Deferred:** model-assisted linking and `sentiment`/`importance` scoring are a v2 offline enrichment job; fields stay null at MVP ([data-model](../architecture/data-model.md#newsitem-fact)).
6. **Not a normalizer job:** the aggregated-odds derivation on topic pages is read-time API logic (MVP) and later the forecast module (v1) — it consumes entity links, it does not produce canonical records ([topic-lookup](topic-lookup.md)).

## Timeline construction

The timeline builder (module of `owt-normalize`, own durable consumer) emits TimelineItems per event:

| Kind | Trigger | numeric_delta |
|---|---|---|
| `price_move` | \|Δprice\| ≥ 3¢ within ≤ 30 min window on any member market (both thresholds config) | before/after/bp/window |
| `news` | NewsItem with `event_links` ∋ event | price context of nearest member market over ±30 min |
| `trade_burst` | trades/min > z-score 3 vs trailing 24 h | volume before/after |
| `comment_spike` *(v1)* | comments/10 min > z-score 3 | count delta |
| `resolution` | market status → resolved | final price |

- **Confidence** (0–1): for `news` items, proximity-weighted — time distance to the nearest qualifying price move, entity-match strength, publisher weight. Formula versioned with `ingest_version`; documented inline in code.
- **Ordering & ties:** `(ts, kind_priority, timeline_id)`; `timeline_id` hash makes emission idempotent.
- Items are append-only; a correction emits a superseding item (`refs.supersedes`), never an update.

## Fixtures & tests

- `owt-testkit` fixture corpus: `fixtures/{source}/{endpoint_or_kind}/{name}.json` — one pristine capture + edge cases (nulls, drift, stringified-array oddities) per mapping row.
- Property tests: determinism (envelope → canonical, twice, byte-equal), outcome zip integrity (names/prices/tokens same length), URL canonicalization idempotence.
- Golden files: canonical outputs checked in; a mapping change that alters goldens must bump `ingest_version` or fail CI.

## Related

- [../architecture/data-model.md](../architecture/data-model.md)
- [../architecture/data-sources.md](../architecture/data-sources.md)
- [ingestion.md](ingestion.md)
- [storage.md](storage.md)
