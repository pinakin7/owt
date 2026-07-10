---
type: adr
status: draft
owner: pinakin
summary: "Forecasts are derived data computed by a new owt-forecast module from statistical models only; rebuildable from Postgres, never LLM-generated, always disclosed."
tags: [area/forecasts, area/data-model]
related:
  - ../design/forecasts.md
  - ../design/storage.md
  - ../design/realtime-bus.md
  - ../design/query-api.md
  - 0011-topic-lookup-entity-composite.md
---

# ADR-0012: Forecast engine as a derived-data module

## Context

The topic-lookup flagship ([ADR-0011](0011-topic-lookup-entity-composite.md)) promises predictions: aggregated market odds plus **owt's own model outputs** — trend, volatility, anomaly, mispricing, and calibration-adjusted probability signals computed from ingested facts. Nothing in the vault covers where such values are computed, stored, published, or labeled. This decision adds canonical-schema tables, bus subjects, and public API surface — all mandatory-ADR territory ([adr process](README.md)).

Constraints: modules communicate only via the bus ([ADR-0003](0003-modular-monolith.md)); derived rollups live in Timescale where they are pure windowed aggregations ([ADR-0004](0004-postgres-timescale-primary-store.md)); replay determinism is a product guarantee ([ADR-0005](0005-nats-jetstream-bus.md)); the product is self-hostable with no external service dependencies beyond its data sources.

## Decision

1. **Forecasts are layer-3 derived documents** ([data-model](../architecture/data-model.md)) — never canonical facts. They are excluded from the raw-envelope replay guarantee and get a **rebuild guarantee** instead, same class as search: `owtd forecast rebuild --model <id> --from <ts>` recomputes any range from Postgres facts, and same `model_version` + same input rows ⇒ identical output rows (property-tested, fixed evaluation order).
2. **A new module crate, `owt-forecast` (v1),** computes them: stream-triggered nowcast signals from a durable consumer over `canon.v1.market.>` (debounced per market) plus scheduled windowed models reading candles/aggregates. MVP ships schema slots, reserved subjects, and the crate row only — the `owt-alerts` precedent.
3. **The in-DB / module split follows ADR-0004:** pure windowed aggregations over one hypertable (return-variance and trade-rate buckets) are Timescale continuous aggregates; anything combining inputs, applying a model transform, or carrying a `model_version` is module-computed.
4. **Storage:** `forecasts` hypertable keyed `(model_id, scope_kind, scope_id, horizon, ts)` where `ts` is the as-of time of the input window (rebuilds are idempotent upserts); hot snapshots `forecast_state` and `entity_odds`; ops tables `model_registry` and `calibration_bins`. Schema: [forecasts](../design/forecasts.md), [storage](../design/storage.md).
5. **Bus:** module output publishes on `canon.v1.forecast.{market_id}.{model_id}` and `canon.v1.forecast.{entity_id}.odds`; the store-writer persists them like any canonical subject ([realtime-bus](../design/realtime-bus.md)).
6. **Statistical models only — no LLM, ever, in this module.** Model outputs must be deterministic, explainable from named inputs, and scoreable. Every `kind: forecast` model is tracked against market resolutions (Brier score, calibration curve); a model that cannot be scored does not ship.
7. **Disclosure is contractual:** every forecast/odds API response carries `model_id`, `model_version`, `kind` (`signal` | `forecast`), and a research-purposes disclosure string; the TUI must render the label wherever a value appears ([query-api](../design/query-api.md), [security-and-privacy](../ops/security-and-privacy.md)).

## Consequences

- Topic pages and market detail gain a predictions pane in v1 with zero new MVP scope risk.
- A model bug is recoverable by rebuild, exactly like a search-index bug — the fact tables are never contaminated.
- `model_version` bump semantics mirror `ingest_version`: minor = enriched output, major = semantics changed → rebuild of that model's rows, recorded in `model_registry`.
- Evaluation infrastructure (resolution outcomes joined to forecast history) becomes a v2 prerequisite for the calibration model — the first true forecast waits for it.
- Rolling-stat primitives (EWMA, z-score, ring buffers) move to `owt-domain` and are shared with the alert engine rather than duplicated.

## Alternatives considered

- **Query-time computation in `owt-api`** — breaks the WS push path (fanout needs a materialized value), recomputes per request, and couples model code into the API service. Rejected for v1; read-time is retained only for MVP odds where no push exists yet (ADR-0011).
- **Separate forecasting service** — violates the modular monolith ([ADR-0003](0003-modular-monolith.md)) with no scale evidence.
- **LLM-generated outlooks** — non-deterministic, unrebuildable, unscoreable, and adds an external dependency a self-hoster cannot audit. Rejected outright; any future NL enrichment is a different feature behind its own ADR.
- **Everything as continuous aggregates** — SQL cannot express calibration curves, cross-market aggregation, or versioned model transforms maintainably; caggs stay limited to pure windowed inputs.

## Rollout notes

MVP: tables, subjects, and crate row reserved (no code). v1: `owt-forecast` ships with the signal catalog v0 and the `/forecasts` + odds API surface. v2: `calib-prob.v1` calibration forecasts gated on the evaluation harness. Greenfield — no migration.

## Related

- [../design/forecasts.md](../design/forecasts.md)
- [../design/storage.md](../design/storage.md)
- [../design/realtime-bus.md](../design/realtime-bus.md)
- [../design/query-api.md](../design/query-api.md)
- [0011-topic-lookup-entity-composite.md](0011-topic-lookup-entity-composite.md)
