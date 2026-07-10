---
type: lld
status: draft
owner: pinakin
summary: "LLD for NATS JetStream: subject taxonomy, stream and consumer configs, replay and rehydration, and fanout."
tags: [area/realtime, area/ingestion, release/mvp]
related:
  - ../adr/0005-nats-jetstream-bus.md
  - ingestion.md
  - ../architecture/data-model.md
  - query-api.md
---

# Realtime bus (NATS JetStream)

The only transport between backend modules ([ADR-0005](../adr/0005-nats-jetstream-bus.md)). Subjects, streams, and consumers are **code** (`owt-bus`), applied idempotently at startup — never hand-configured. All bus subjects are **private and unstable**: the public realtime contract is the API's WS topics ([query-api](query-api.md)), full stop.

## Subject taxonomy (normative)

### Raw (pre-normalization)

```
raw.{source}.{channel}[.{key}]
```

| Subject | Payload | Emitter |
|---|---|---|
| `raw.pm_ws.market.{token_id}` | envelope: book/price_change/last_trade/tick_size | PM WS adapter |
| `raw.pm_rtds.comment` *(v1)* | envelope: comment | RTDS adapter |
| `raw.gamma.market` / `raw.gamma.event` | envelope: REST sweep page items (one per object) | Gamma adapter |
| `raw.clob.prices_history.{token_id}` | envelope: history batch | CLOB adapter |
| `raw.data_api.trade.{market_id}` | envelope: trade | Data adapter |
| `raw.rss.{feed_id}` | envelope: feed item | RSS adapter |
| `raw.gdelt.article` | envelope: article ref | GDELT adapter |
| `raw.goldsky.{dataset}` *(v1)* | envelope: dataset row | Goldsky adapter |

### Canonical (post-normalization; version token = envelope schema major)

```
canon.v1.{domain}.{key}.{kind}
```

| Subject | Payload |
|---|---|
| `canon.v1.market.{market_id}.price` | PricePoint |
| `canon.v1.market.{market_id}.book` | BookSnapshot |
| `canon.v1.market.{market_id}.trade` | Trade |
| `canon.v1.market.{market_id}.state` | MarketState delta |
| `canon.v1.market.{market_id}.meta` | Market snapshot change |
| `canon.v1.event.{event_id}.timeline` | TimelineItem |
| `canon.v1.news.item.{news_id}` | NewsItem |
| `canon.v1.comment.{market_id}` *(v1)* | Comment |
| `canon.v1.onchain.fill.{market_id}` *(v1)* | OrderFill |
| `canon.v1.alert.{rule_id}` *(v1)* | AlertEvent |
| `canon.v1.forecast.{market_id}.{model_id}` *(v1)* | ForecastPoint |
| `canon.v1.forecast.{entity_id}.odds` *(v1)* | EntityOdds snapshot |

### Dead-letter

```
dlq.{stage}.{source}     stages: ingest | normalize | write
```

**Rules:** keys are canonical IDs only (never slugs — slugs can change); one payload type per subject; a new `canon.v2.*` namespace appears only on an envelope-schema major bump and runs in parallel during migration.

## Streams

| Stream | Subjects | Storage | Retention | Dedupe window | Purpose |
|---|---|---|---|---|---|
| `RAW` | `raw.>` | file | max_age **7 d** (config) | 2 min on `Nats-Msg-Id`=`envelope_id` | replay window |
| `CANON` | `canon.>` | file | max_age **30 d** | 2 min | consumer feed + short replay |
| `DLQ` | `dlq.>` | file | max_age 30 d | — | failure parking |

Replicas: 1 self-host, 3 clustered ([deployment](../ops/deployment.md)). `max_bytes` caps sized to disk (defaults: RAW 20 GiB, CANON 10 GiB, discard=old) — **Postgres is durable truth; these are windows**, so discarding old bus data loses nothing that the store and archive don't hold.

## Durable consumers

| Consumer | Filter | Ack | max_ack_pending | max_deliver | Owner |
|---|---|---|---|---|---|
| `normalizer` | `raw.>` | explicit | 2,000 | 5 → `dlq.normalize.*` | owt-normalize |
| `archive-writer` | `raw.>` | explicit | 5,000 | 5 | raw archive ([ingestion](ingestion.md#raw-archive)) |
| `store-writer` | `canon.>` minus alerts | explicit | 5,000 | 5 → `dlq.write.*` | owt-store |
| `search-indexer` | `canon.v1.news.>`, `canon.v1.market.*.meta`, `…state` | explicit | 2,000 | 5 | owt-search |
| `timeline-builder` | `canon.v1.news.>`, `…price`, `…trade`, `…comment` | explicit | 2,000 | 5 | owt-normalize |
| `alert-engine` *(v1)* | `canon.v1.>` | explicit | 2,000 | 5 | owt-alerts |
| `forecast-engine` *(v1)* | `canon.v1.market.>` | explicit | 2,000 | 5 | owt-forecast |
| API fanout | `canon.v1.>` (ephemeral, per-instance) | none | — | — | owt-api |

`ack_wait` 30 s everywhere; consumers are idempotent by construction (natural-key upserts), so redelivery is harmless. The API fanout subscriber is deliberately **ephemeral** — a restarted API instance resnapshots clients from the store, it does not need bus history. Forecast subjects flow to the store-writer like any canonical subject, but forecast rows are recovered by `owtd forecast rebuild`, not bus replay ([ADR-0012](../adr/0012-forecast-derived-data-module.md)).

## Ordering & consistency

- Ordering is guaranteed **per subject** only; the partition key (usually `market_id`) is baked into the subject, so per-market order holds and cross-market order is undefined — consumers must not assume it.
- Book consistency uses snapshot + sequence: `BookSnapshot.seq` monotonic per token; a gap observed by any consumer triggers the adapter's REST resync ([ingestion](ingestion.md#websocket--rtds-connection-management)).
- Consumers publish no cross-subject transactions; anything needing atomicity (fact + checkpoint) happens in Postgres, not on the bus.

## Replay & rehydration

**Replay (within RAW window):** normalizer mapping change ⇒ `ingest_version` major/minor bump ⇒

```
owtd replay --subjects 'raw.rss.>' --from 2026-07-01T00:00:00Z --ingest-version 2.0.0
```

spawns a temporary consumer from the requested start; canonical upserts are idempotent, so re-derived records overwrite-or-skip deterministically. `normalizer_versions` table records the run.

**Rehydration (beyond window):** `owtd replay --from-archive raw/rss/2026/06/** …` streams archived envelopes from object storage back through the same code path (direct in-process feed; a `--via-bus` mode republished under `rehydrate.>` exists for multi-consumer replays). Determinism check: replaying an already-processed range must produce zero row changes — verified in CI ([testing-strategy](../ops/testing-strategy.md)).

## Fanout to clients

Lives in `owt-api` (no separate gateway — [roadmap divergence 5](../product/roadmap.md#divergences-from-the-research-report)):

1. One ephemeral subscription set per API instance over `canon.v1.>`.
2. In-process topic router maps bus subjects → public WS topics (`canon.v1.market.12345.state` → `market:{slug}:state` via the reference cache).
3. Per-client bounded queues with conflation for book/state topics (coalesce to latest, flush 4–10 Hz), drop-oldest + `lagged` marker on overflow ([query-api § WebSocket](query-api.md#websocket-protocol)).

## Metrics

`bus_consumer_lag{consumer}` (num_pending), `bus_redeliveries_total{consumer}`, `dlq_depth{stage}`, `bus_publish_dedupe_hits_total`, `stream_bytes{stream}`, `fanout_conflation_ratio`, `fanout_dropped_frames_total`. Chaos: Toxiproxy latency/partition injection between owtd and NATS is a standing test scenario.

## Related

- [../adr/0005-nats-jetstream-bus.md](../adr/0005-nats-jetstream-bus.md)
- [ingestion.md](ingestion.md)
- [../architecture/data-model.md](../architecture/data-model.md)
- [query-api.md](query-api.md)
