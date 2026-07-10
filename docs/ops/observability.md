---
type: ops
status: draft
owner: pinakin
summary: "Telemetry standards: OpenTelemetry pipeline, first-class metrics catalog, SLO dashboards, and logging conventions."
tags: [area/ops, release/mvp]
related:
  - ../architecture/system-overview.md
  - ../design/ingestion.md
  - runbook.md
---

# Observability

owt's hardest production problems will be **upstream drift, throttling, ingest gaps, and index lag** — not request latency. Telemetry is designed around proving, at a glance, that the data can be trusted.

## Pipeline

- **Server (`owtd`):** `tracing` + `tracing-opentelemetry` → OTLP → OpenTelemetry Collector → Prometheus (metrics), Loki (logs), Tempo-or-nothing (traces optional at self-host). Grafana dashboards ship in-repo (`ops/dashboards/` when code lands).
- **Trace continuity across the bus:** envelope carries `traceparent` in headers; consumers continue the trace — one trace spans adapter → normalizer → writer → fanout.
- **TUI:** file logs only, no OTel, **no telemetry by default, ever** — a research terminal must not phone home. A `--debug` overlay shows client-side health locally ([tui-client](../design/tui-client.md)).
- Correlation: every REST response carries `x-owt-request-id`; WS frames carry it on errors; logs are JSON with `request_id`/`envelope_id`/`trace_id` fields.

## First-class metrics

The trust dashboard, one row per question an operator asks:

| Question | Metric(s) |
|---|---|
| Is ingest fresh? | `ingest_lag_seconds{source}` — upstream event ts → canon publish |
| Are we being throttled? | `throttle_pauses_total{bucket}`, `budget_utilization{bucket}` |
| Are streams healthy? | `ws_reconnects_total{channel}`, `ws_gap_seconds`, RTDS heartbeat misses |
| Is normalization clean? | `normalize_failures_total{source}`, `schema_drift_total{source}`, `dedupe_suppressed_total{kind}` |
| Is anything stuck? | `dlq_depth{stage}` (SLO ≈ 0), `bus_consumer_lag{consumer}`, `checkpoint_age_seconds{job}` |
| Is search current? | `search_index_lag_seconds` (≤ 5 s p95 live) |
| Is the API fast? | `http_request_duration_seconds{route}` p50/95/99 vs [query-api budgets](../design/query-api.md#latency-budgets-server-side-warm-k6-enforced-from-v1) |
| Is fanout keeping up? | `fanout_dropped_frames_total`, `fanout_conflation_ratio`, WS client count |
| Are alerts timely? *(v1)* | `alert_delivery_latency_seconds` p95 ≤ 2 s, `alert_eval_lag_seconds` |
| Is storage healthy? | writer batch latency, rows/s, Postgres connections/queue depth |

## SLOs (dashboard-encoded; from [prd-mvp](../product/prd-mvp.md) — confirm via D-02)

| SLO | Target |
|---|---|
| WS tick → TUI-deliverable frame | p95 ≤ 750 ms |
| News published → searchable | p95 ≤ 2 min |
| Search API | p95 ≤ 100 ms |
| Market-detail composite | p95 ≤ 150 ms |
| Search index lag (live) | p95 ≤ 5 s |
| DLQ depth | ≈ 0 sustained |
| Alert delivery *(v1)* | p95 ≤ 2 s |

Each SLO panel pairs the measurement with its burn alert (below).

## Ops alerts (distinct from product alerts)

Prometheus alert rules, delivered to the operator (not the TUI inbox — [alerts](../design/alerts.md) is the *product* engine):

- `ingest_lag_seconds{source} > 300` for 5 min — per active source
- `dlq_depth > 0` sustained 10 min
- `checkpoint_age_seconds{job} > 3× interval`
- `ws_reconnects_total` rate > 5/10 min per channel
- `search_index_lag_seconds > 60` for 5 min
- API p99 > 2× budget for 10 min
- Postgres disk > 80 %; NATS stream bytes > 80 % of max

Every ops alert names its [runbook](runbook.md) entry in the annotation.

## Logging conventions

- JSON lines; levels: `error` (actionable), `warn` (degradation), `info` (state transitions: connect, checkpoint, phase done), `debug` (per-item, off in prod).
- One log per decision, not per row: batch writers log batch summaries.
- No payload bodies at info+ (PII/licensing hygiene); envelope IDs make payloads findable in the archive instead.
- Retention: 7–30 d hot (Loki), then drop — canonical data is the history, not logs.

## Related

- [../architecture/system-overview.md](../architecture/system-overview.md)
- [../design/ingestion.md](../design/ingestion.md)
- [runbook.md](runbook.md)
