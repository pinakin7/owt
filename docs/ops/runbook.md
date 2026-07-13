---
type: ops
status: draft
owner: pinakin
summary: "Incident runbook entries for the failure classes known at design time."
tags: [area/ops, release/v1]
related:
  - observability.md
  - ../design/ingestion.md
  - ../design/realtime-bus.md
  - getting-started.md
---

# Runbook

Entries for the failure classes we can name *before* production teaches us new ones. Stays `draft` until v1 hardens it with real incidents. Every [ops alert](observability.md#ops-alerts-distinct-from-product-alerts) annotation links to an entry here.

> **Status — target design.** These procedures assume the running server, workers, bus,
> store, and metrics that the v1 stack will have. The `owtd` server is currently a
> scaffold, so the commands and signals below are not yet operable. For what runs today,
> see [getting-started](getting-started.md).

Entry format: **Symptom → Detection → Immediate → Diagnosis → Rollback/Recovery → Post-incident.**

## R1 — Upstream schema drift

- **Symptom:** `schema_drift_total{source}` climbing; normalize failures; drift-canary issue opened.
- **Immediate:** nothing breaks by design (tolerant parsing); confirm DLQ isn't filling (R3 if it is).
- **Diagnosis:** diff live payload vs fixture (`xtask capture-fixture`); identify added/renamed/retyped fields.
- **Recovery:** update mapping + fixtures + goldens; bump `ingest_version` (minor if additive, major if semantics changed); deploy; `owtd replay` the affected window ([realtime-bus § replay](../design/realtime-bus.md#replay--rehydration)).
- **Post:** note the drift in the data-sources verification column.

## R2 — Rate-limit exhaustion / queue-throttling

- **Symptom:** `throttle_pauses_total` spiking; ingest lag rising; upstream latency 3× baseline.
- **Immediate:** verify pause windows engaged (they self-heal); check no runaway job (a bad backfill flag is the usual cause) — `owtd` admin `/ingest/status`.
- **Diagnosis:** which bucket, which job; compare `budget_utilization` vs configured budget.
- **Recovery:** kill offending job; lower its budget in config; resume — checkpoints make this loss-free.
- **Post:** if documented limits changed, update [data-sources](../architecture/data-sources.md#rate-limits--from-the-official-rate-limits-page-2026-07-07) with a new verification date.

## R3 — DLQ growth

- **Symptom:** `dlq_depth > 0` sustained.
- **Immediate:** inspect `dlq_quarantine` newest rows — error class and stage tell you which entry you're actually in (R1 drift is the common cause).
- **Recovery:** fix root cause, then `owtd replay --from-quarantine <ids>`; verify rows drain.
- **Post:** every DLQ incident adds a fixture reproducing it.

## R4 — WS disconnect storm

- **Symptom:** `ws_reconnects_total` rate alert; `ws_gap_seconds` accruing; TUI users see Degraded.
- **Immediate:** confirm it's upstream (adapter logs show close codes) vs local (NATS/CPU pressure).
- **Diagnosis:** single channel vs all; correlate with Polymarket status/announcements.
- **Recovery:** self-heals via backoff+resync; verify gap accounting triggered targeted trade sweeps; if subscriptions exceed batch limits after a market surge, raise shard count (config).
- **Post:** check tape holes on watched markets: `owtd backfill --phase trades --window <outage>`.

## R5 — RTDS heartbeat loss *(v1)*

- **Symptom:** RTDS reconnect loop; comments stale.
- **Diagnosis:** confirm ping cadence (client 10 s; answer server pings ≤ 10 s) — a busy runtime starving the ping task is the classic local cause.
- **Recovery:** restart worker role if starved; verify comment flow resumes.

## R6 — Search index lag / corruption

- **Symptom:** `search_index_lag_seconds` alert, or wrong/missing results with healthy lag.
- **Immediate:** API degrades to Postgres search automatically (`degraded: true`) — user impact is reduced ranking, not outage.
- **Recovery:** lag → check `search-indexer` consumer (`bus_consumer_lag`); corruption/schema change → `owtd reindex` (blue/green, from Postgres only) then alias-swap ([search § rebuild](../design/search.md#rebuild-bluegreen)).

## R7 — Bus consumer lag / stream pressure

- **Symptom:** `bus_consumer_lag{consumer}` alert; `stream_bytes` near max.
- **Diagnosis:** slow consumer (writer batch latency? Postgres pressure → R8) vs burst (election night).
- **Recovery:** consumers are idempotent — safe to scale the worker role or raise `max_ack_pending`; stream discard=old means overflow costs replay window, not truth.

## R8 — Postgres pressure

- **Symptom:** writer batch latency up; API p99 breaching; connection queue depth.
- **Immediate:** check long-running queries (`pg_stat_activity`); kill runaways (statement_timeout should have).
- **Diagnosis:** missing index (new query shape?), chunk exclusion failing (query without time bound), compression job contention.
- **Recovery:** add time bounds/index via migration; retune batch sizes; scale instance as last resort.

## R9 — Goldsky outage *(v1)*

- **Symptom:** `ingest_lag_seconds{source="goldsky"}` alert.
- **Immediate:** none urgent — Data API tape continues; on-chain facts backfill later (checkpointed cursor).
- **Recovery:** resume from cursor when vendor recovers; extended outage → escalate D-05 fallback discussion (`owt-source-polygon`).

## R10 — Replay procedure (planned operation)

1. Announce; note current `ingest_version` and target.
2. `owtd replay --subjects '<filter>' --from <ts> --ingest-version <new>` (bus window) or `--from-archive` (rehydration).
3. Watch determinism guard: re-replay of an already-current range must show zero row changes.
4. `owtd reindex` if search-visible fields changed.
5. Record run in `normalizer_versions`; update goldens.

## R11 — Restore drill (backup recovery)

Postgres restore (PITR/dump) → `owtd migrate` (no-op check) → `owtd reindex` → verify counts vs `ingest_runs` audit → resume ingest (checkpoints self-locate). Streams start empty by design ([storage § backup](../design/storage.md#backup--restore)).

## Related

- [observability.md](observability.md)
- [../design/ingestion.md](../design/ingestion.md)
- [../design/realtime-bus.md](../design/realtime-bus.md)
- [getting-started.md](getting-started.md)
