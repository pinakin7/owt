---
type: lld
status: draft
owner: pinakin
summary: "LLD for PostgreSQL/Timescale: logical schema, hypertables, indexes, migrations, retention, and capacity math."
tags: [area/storage, release/mvp]
related:
  - ../architecture/data-model.md
  - ../adr/0004-postgres-timescale-primary-store.md
  - normalization.md
  - search.md
---

# Storage & retention

One Postgres 16 + TimescaleDB instance is the system of record ([ADR-0004](../adr/0004-postgres-timescale-primary-store.md)). Everything else is rebuildable from it: Typesense collections, candles (in-DB aggregates), bus state. The raw-envelope archive lives in object storage, not here.

## Store responsibility matrix

| Store | Holds | Explicitly does not hold |
|---|---|---|
| Postgres/Timescale | reference data, canonical facts, workspace state, ops state, candles | raw envelopes (archive), search ranking state |
| Typesense | derived search documents | anything unrebuildable |
| Object store / filesystem | raw envelope archive, exports | queryable data |
| In-process cache (moka) | hot reference lookups (token→market), completed pages | anything whose loss matters |

## Logical schema

### Reference (mutable snapshots)

| Table | Key | Notes |
|---|---|---|
| `events` | `event_id` PK | slug unique; tags text[] |
| `markets` | `market_id` PK | `event_id` FK; slug unique; `condition_id` indexed; status enum |
| `market_tokens` | `token_id` PK | `market_id` FK, outcome name, index-in-market |
| `market_state` | `market_id` PK | best_bid/ask, last_trade_price, spread, volume_24h, liquidity_score, `updated_at` — the hot snapshot row |
| `tags` | slug PK | display name, kind |
| `entities` | `entity_id` PK | kind, name, description (nullable, human-curated — [ADR-0011](../adr/0011-topic-lookup-entity-composite.md)) |
| `entity_aliases` | (alias_norm, entity_id) PK | match_policy, confidence, added_by |
| `entity_links` | (entity_id, kind, ref_id) PK | entity→market/event mappings |

### Facts (append-only; hypertables marked ⏳)

| Table | Natural key | ⏳ | Chunk | Compress after | Retain |
|---|---|---|---|---|---|
| `price_ticks` | (token_id, ts, kind, source) | ⏳ | 1 d | 7 d | indefinite |
| `trades` | trade_id | ⏳ | 1 d | 7 d | indefinite |
| `onchain_fills` *(v1)* | fill_id | ⏳ | 1 d | 7 d | indefinite |
| `book_snapshots` | (token_id, ts) | ⏳ | 1 d | 2 d | **14 d** (drop policy) |
| `news_items` | news_id | ⏳ | 7 d | 30 d | indefinite (see license note) |
| `comments` *(v1)* | comment_id | ⏳ | 7 d | 30 d | indefinite |
| `timeline_items` | timeline_id | ⏳ | 7 d | 30 d | indefinite |
| `social_posts` *(v2)* | post_id | ⏳ | 7 d | 30 d | per X ToS |

`news_items.body_text` is stored only where `license_class` permits; a scheduled job nulls bodies whose license class demands expiry ([security-and-privacy](../ops/security-and-privacy.md)).

### Candles (continuous aggregates)

`candles_1m` is a continuous aggregate over `price_ticks` (kind = trade/history), cascading to `candles_5m`, `candles_1h`, `candles_1d`. Refresh policies: 1m every 1 min (lag 1 min), higher tiers every 10 min. **Candles are computed in Timescale, never in application code.** API reads candles, not raw ticks, for any range > 6 h.

### Forecast derivations *(v1 — schema slots at MVP)*

Derived data with a rebuild guarantee, written by `owt-forecast` ([ADR-0012](../adr/0012-forecast-derived-data-module.md), [forecasts](forecasts.md)):

| Table | Kind | Key | Notes |
|---|---|---|---|
| `forecasts` | ⏳ hypertable | (model_id, scope_kind, scope_id, horizon, ts) | `ts` = as-of time of the input window; chunk 1 d, compress 7 d, retain **180 d** |
| `forecast_state` | snapshot | (model_id, scope_kind, scope_id, horizon) | latest value per series — hot row, analogous to `market_state` |
| `entity_odds` | snapshot | entity_id | materialized EntityOdds document |
| `model_registry` | ops | (model_id, model_version) | params JSONB, activated_at — rebuild bookkeeping, analogous to `normalizer_versions` |
| `calibration_bins` | derived | (price_bin, ttr_bucket, liquidity_tier) | rebuilt nightly from resolved markets (v2) |

Pure windowed inputs (return-variance, trade-rate buckets) are additional continuous aggregates over `price_ticks`/`trades`, per the candles rule. **At MVP, aggregated topic odds are computed on read** from `entity_links ⋈ market_state` — no odds table exists until the v1 module lands ([topic-lookup](topic-lookup.md#aggregated-odds-entityodds)).

### Workspace (mutable; `workspace_id` from day one)

| Table | Key | Notes |
|---|---|---|
| `workspaces` | `workspace_id` PK | single row `default` until v2 auth ([ADR-0008](../adr/0008-read-only-through-v1.md)) |
| `watchlists` | id PK | workspace FK, name unique per workspace |
| `watchlist_items` | (watchlist_id, market_id) PK | position/order |
| `saved_views` | id PK | workspace FK, name, screen kind, query, layout JSONB |
| `alert_rules` *(v1)* | id PK | workspace FK; [alerts](alerts.md) |
| `alert_events` *(v1)* | id PK | ⏳ hypertable, 90 d retention |

### Ops

| Table | Purpose |
|---|---|
| `ingest_checkpoints` | (source, endpoint, partition) → cursor, ts, ingest_version |
| `ingest_runs` | backfill/sweep audit: job, phase, calls, duration, outcome |
| `dlq_quarantine` | poisoned envelopes: envelope JSONB, error, attempts, status |
| `normalizer_versions` | ingest_version history with deploy ts — replay bookkeeping |
| `schema_migrations` | sqlx-managed |

## Index inventory (beyond PKs)

- `markets(event_id)`, `markets(condition_id)`, unique `markets(market_slug)`, `markets(status) WHERE status='active'`
- unique `market_tokens(token_id)`; `market_tokens(market_id)`
- `price_ticks(token_id, ts DESC)`; `trades(market_id, ts DESC)`; `onchain_fills(market_id, ts DESC)`
- unique `news_items(news_id)`; `news_items(published_at DESC)`; GIN `news_items(entities)`; `news_items(cluster_id)`
- `timeline_items(event_id, ts DESC)`; `timeline_items(market_id, ts DESC)`
- `entity_aliases(alias_norm)`; GIN on `events.tags`/`markets.tags`; `entity_links(entity_id)` covering — the topic-composite join
- `forecasts(scope_id, ts DESC)` *(v1)* — series reads per market/entity
- pg_trgm GIN on `markets.title`, `events.title`, `news_items.headline` — the search degradation path ([search](search.md))

Hypertable composite indexes lead with the series key (`token_id`/`market_id`), then `ts DESC` — matching every hot query shape (`WHERE key = ? ORDER BY ts DESC LIMIT n`).

## Write path

- Store-writer consumes `canon.v1.>` in batches: **500 rows or 200 ms**, whichever first; one transaction per batch per table family.
- Facts: `INSERT … ON CONFLICT DO NOTHING` (natural key). Snapshots: `ON CONFLICT DO UPDATE … WHERE excluded.updated_at > current.updated_at` (monotonic guard).
- Checkpoint rows commit **in the same transaction** as their batch ([ingestion](ingestion.md#checkpoints)).
- Sustained target ≥ 5k rows/s batched on laptop-class hardware; burst absorption via bus backlog, not bigger transactions.

## Migrations

- SQL-first, embedded via `sqlx::migrate!`, applied **only** by explicit `owtd migrate` (CI/dev may use `--auto-migrate`; production never).
- Forward-only; no down-migrations. Pre-1.0, squash policy: migrations may be consolidated at minor releases while `owtd migrate --from-scratch` reproduces byte-identical schema.
- Timescale caveats documented per migration: hypertable PKs must include the partition column; `create_hypertable`, compression, retention, and continuous-aggregate policies are created in plain SQL migrations (no ORM indirection).
- CI gate: fresh database → all migrations → seed fixtures → smoke queries ([testing-strategy](../ops/testing-strategy.md)).

## Pooling & sizing

- Pools per role: api 20 / writer 10 / scheduler 5; `statement_timeout` 5 s (api), 30 s (writer); writer uses `synchronous_commit = on` (durability over throughput — bus absorbs bursts).
- **Capacity math (design point, top-200 watched markets):** price ticks ~2/s/market avg ⇒ ~35 M rows/mo (~3 GB/mo pre-compression, ~10× compression after 7 d); trades ~0.5/s/market ⇒ ~9 M rows/mo; news ~5 k items/day ⇒ trivial; forecasts (v1) ≤ a few rows/market/min with 180 d retention ⇒ negligible next to ticks. **12-month estimate ≤ ~15 GB compressed** — one modest volume; ClickHouse trigger is nowhere in sight at this scale ([system-overview § scale-up](../architecture/system-overview.md#scale-up-triggers)).

## Backup & restore

- Self-host default: nightly `pg_dump` + WAL archiving off; documented restore drill in [runbook](../ops/runbook.md).
- Production-initial: managed Postgres PITR.
- Restore ordering: Postgres → `owtd reindex` (Typesense rebuilds from canonical) → bus streams recreate empty (they're a window, not truth) → archive untouched.

## Related

- [../architecture/data-model.md](../architecture/data-model.md)
- [../adr/0004-postgres-timescale-primary-store.md](../adr/0004-postgres-timescale-primary-store.md)
- [normalization.md](normalization.md)
- [search.md](search.md)
