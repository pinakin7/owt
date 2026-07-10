---
type: lld
status: draft
owner: pinakin
summary: "LLD for the v1 forecast engine: the owt-forecast module, model catalog v0, forecast storage and bus subjects, odds materialization, evaluation, and rebuild determinism."
tags: [area/forecasts, area/data-model, release/v1]
related:
  - ../adr/0012-forecast-derived-data-module.md
  - storage.md
  - realtime-bus.md
  - query-api.md
  - topic-lookup.md
---

# Forecasts (v1)

Designed now, built in v1 ([roadmap](../product/roadmap.md#v1--realtime-everywhere--alerts)); MVP ships schema slots, reserved subjects, and the crate row only — the [alerts](alerts.md) precedent. Governing decision: [ADR-0012](../adr/0012-forecast-derived-data-module.md) — forecasts are **derived data with a rebuild guarantee**, statistical models only, always disclosed.

## Module: `owt-forecast`

- Dependency edges `forecast → domain & bus & store` — the same shape as `owt-alerts` ([cargo-workspace](../architecture/cargo-workspace.md)).
- **Nowcast path:** durable consumer `forecast-engine` over `canon.v1.market.>` (price/book/trade/state); per-market debounce ≥ 1 s; ring-buffer windowed state in memory, rebuilt from `candles_1m` + recent ticks on restart (stateless-on-disk, like the alert engine).
- **Scheduled path:** windowed models run every 1 m / 5 m against candles and the forecast continuous aggregates in Postgres.
- **Rebuild:** `owtd forecast rebuild --model <id> --from <ts> [--to <ts>]` recomputes from facts; upserts are idempotent because `ts` is the as-of time of the input window, not wall clock. Same `model_version` + same input rows ⇒ identical output rows — property-tested with fixed evaluation order (no parallel-reduction float nondeterminism).
- Shared rolling-stat primitives (EWMA, z-score, ring buffers) live in `owt-domain`, reused by [alerts](alerts.md).

**In-DB vs module split** ([ADR-0004](../adr/0004-postgres-timescale-primary-store.md)): pure windowed aggregations over one hypertable — per-market return-variance buckets and trade-rate baselines — are Timescale continuous aggregates. Anything combining inputs, applying a model transform, or carrying a `model_version` is computed here.

## Storage

Tables (full DDL context in [storage](storage.md)):

| Table | Kind | Key | Notes |
|---|---|---|---|
| `forecasts` | hypertable | `(model_id, scope_kind, scope_id, horizon, ts)` | `scope_kind` ∈ `market` \| `entity`; `kind` ∈ `signal` \| `forecast`; `value`, nullable `interval_low`/`interval_high`, `model_version`, `n_inputs`, `inputs` JSONB (window bounds — auditability), `computed_at`. Chunk 1 d, compress 7 d, retain 180 d |
| `forecast_state` | snapshot | `(model_id, scope_kind, scope_id, horizon)` | latest value per series — the hot row API and fanout read, analogous to `market_state` |
| `entity_odds` | snapshot | `entity_id` | materialized `EntityOdds` document ([topic-lookup](topic-lookup.md#aggregated-odds-entityodds)) |
| `model_registry` | ops | `(model_id, model_version)` | params JSONB, activated_at — rebuild bookkeeping, analogous to `normalizer_versions` |
| `calibration_bins` | derived | `(price_bin, ttr_bucket, liquidity_tier)` | n, empirical frequency, Wilson interval; rebuilt nightly from resolved markets (v2) |

`model_version` semantics mirror `ingest_version`: minor = enriched output; major = semantics changed → rebuild of that model's rows, recorded in `model_registry` ([data-model § versioning](../architecture/data-model.md#versioning)).

## Bus

New canonical subjects ([realtime-bus](realtime-bus.md#subject-taxonomy-normative)):

| Subject | Payload |
|---|---|
| `canon.v1.forecast.{market_id}.{model_id}` | ForecastPoint |
| `canon.v1.forecast.{entity_id}.odds` | EntityOdds snapshot |

The store-writer's filter gains `canon.v1.forecast.>` → `forecasts` + snapshots; the API fanout maps these to the public WS topics below. Forecast rows are **not** re-derived from bus replay — the rebuild command is their recovery path ([ADR-0012](../adr/0012-forecast-derived-data-module.md)).

## Model catalog v0

Honest labels: a *signal* describes the present; only a calibrated, scoreable output earns the *forecast* label.

| `model_id` | Inputs | Output | Horizon | Data ready | Label |
|---|---|---|---|---|---|
| `momentum.v1` | `candles_1m/1h`, log-odds returns, EWMA fast/slow | trend score in [−1, 1] | nowcast | MVP data | signal |
| `rvol.v1` | return-variance cagg; annualized on **logit scale** (prices bounded [0,1] make raw returns misleading near 0/1) | realized volatility, 24 h / 7 d | nowcast | MVP data | signal (descriptive) |
| `volume-z.v1` | trade-rate cagg baseline, trailing 24 h / 7 d | z-score of trades/min — same math as the alert `volume.burst` predicate | nowcast | MVP data | signal |
| `book-imbalance.v1` | `book_snapshots` top-N | depth-weighted bid/ask imbalance in [−1, 1]; microprice − mid gap | nowcast | MVP data | signal |
| `divergence.v1` | `market_state` across sibling markets | mispricing gap: event outcome-sum deviation from 1 beyond the fee band; entity-scoped spread between same-proposition markets | nowcast | event scope MVP data; entity scope v1 | signal |
| `calib-prob.v1` | implied price + `calibration_bins` (longshot-bias correction binned by price × time-to-resolution × liquidity tier) | adjusted probability with Wilson interval | to_resolution | resolved-market history + evaluation harness → **v2** | **forecast** |

Out of v0, named v2+ candidates: on-chain flow / holder-concentration signals (Goldsky data exists at v1 but earns its keep only with evaluation infra); any NLP or sentiment input stays out per [roadmap divergence 9](../product/roadmap.md#divergences-from-the-research-report). **No LLM inputs or outputs in this module, ever** (ADR-0012).

## Aggregated-odds materialization (v1)

The odds algorithm is specified once in [topic-lookup](topic-lookup.md#aggregated-odds-entityodds); v1 moves its execution here. Refresh is event-driven: a member market's `canon.v1.market.{id}.state` message marks the owning entities dirty (via the `entity_links` reference cache); a debounced task (≥ 2 s per entity) recomputes the document, upserts `entity_odds`, and publishes `canon.v1.forecast.{entity_id}.odds`. The MVP read-time path in `owt-api` is deleted when this lands — one implementation at a time.

## API surface (additive `/v1`)

| Endpoint | Returns |
|---|---|
| `GET /v1/markets/{slug}/forecasts` | latest `forecast_state` per model + horizon; `?model=&from=&to=` for history from `forecasts` |
| `GET /v1/entities/{id}/forecasts` | entity-scoped model outputs (e.g. entity-scope `divergence.v1`) |
| `GET /v1/entities/{id}/odds` | the `EntityOdds` document (also available as `include=odds` on the composite) |
| `GET /v1/models` | model registry: id, version, `kind`, methodology summary, status |

WS topics: `market:{slug}:forecasts`, `entity:{id}:odds` — conflatable, coalesce-to-latest like state/book ([query-api § WebSocket](query-api.md#websocket-protocol)).

**Response contract:** every forecast object carries `model_id`, `model_version`, `kind`, `value`, nullable `interval`, `ts`, `horizon`; every forecasts/odds response carries a top-level `disclosure: "Statistical model output for research purposes; not financial advice."`. The TUI renders the model label + version wherever a value is shown ([tui-client](tui-client.md), [security-and-privacy](../ops/security-and-privacy.md)).

## Evaluation

A `kind: forecast` model that cannot be scored does not ship:

- Nightly job joins forecast history to market resolutions; tracks **Brier score** and a calibration curve per `(model_id, model_version)`.
- Published as metrics (`forecast_brier_score{model}`) and via `GET /v1/models`; a regression beyond a configured threshold flags the model `degraded` in the registry (surfaced in TUI labeling).
- Backtests run through `owtd forecast rebuild` against historical facts — same code path as production, no separate research pipeline to drift.

## Metrics & tests

`forecast_lag_seconds{model}` (input ts → row visible), `forecast_rebuild_rows_total`, `forecast_brier_score{model}`, `entity_odds_refresh_seconds`, plus standard consumer lag via `bus_consumer_lag{consumer="forecast-engine"}`.

Test gate (v1): golden-value model tests on fixture windows; rebuild-determinism test (recompute a processed range ⇒ zero row changes); property tests on odds invariants shared with [topic-lookup](topic-lookup.md#test-plan).

## Related

- [../adr/0012-forecast-derived-data-module.md](../adr/0012-forecast-derived-data-module.md)
- [storage.md](storage.md)
- [realtime-bus.md](realtime-bus.md)
- [query-api.md](query-api.md)
- [topic-lookup.md](topic-lookup.md)
