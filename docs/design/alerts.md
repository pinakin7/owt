---
type: lld
status: draft
owner: pinakin
summary: "LLD for the v1 alert engine: rule model, predicate DSL, streaming evaluation, and TUI-first delivery."
tags: [area/alerts, release/v1]
related:
  - realtime-bus.md
  - query-api.md
  - tui-client.md
  - storage.md
---

# Alert engine (v1)

Designed now, built in v1 ([roadmap](../product/roadmap.md#v1--realtime-everywhere--alerts)): a streaming evaluator over canonical subjects that turns rules into TUI-inbox notifications and webhooks. MVP ships only the schema slots and a palette stub.

## Rule model

```json
{
  "id": "…",
  "workspace_id": "default",
  "name": "fed-market big move",
  "scope": { "kind": "market", "ref": "will-fed-cut-rates-in-september" },
  "predicate": "price.delta:>0.05 window:30m",
  "throttle": "15m",
  "severity": "warn",
  "channels": ["inbox", "webhook:ops"],
  "enabled": true
}
```

- **Scope kinds:** `market`, `event`, `watchlist`, `tag`, `global` — resolved to a market set at evaluation time (watchlist/tag scopes re-resolve on membership change).
- **Severity:** `info` | `warn` | `crit` — drives TUI styling and terminal bell policy.
- **Throttle:** per (rule, market) cooldown; a rule can't fire for the same market more often than this.

## Predicate types

| Predicate | Semantics | State needed |
|---|---|---|
| `price.delta:>X window:W` | \|price now − price W ago\| > X | ring buffer of ticks per market (W ≤ 1 h) |
| `price.cross:0.5 dir:up` | last-trade crosses a level | last price |
| `spread:>X` | current spread exceeds X | market_state |
| `volume.burst z:>3 window:10m` | trades/min z-score vs trailing 24 h | rolling stats (seeded from candles) |
| `news.burst count:>N window:1h` | linked news items in window | counter per market/event |
| `comment.spike z:>3 window:10m` | comment rate z-score | counter |
| `liquidity.drop pct:>50 window:1h` | liquidity_score drop | snapshot history |
| `resolution` | market resolved | event subscription |

Grammar is the shared DSL parser in `owt-domain` (same family as palette filters — [tui-client § grammar](tui-client.md#command-palette--grammar)); typed AST, compile errors carry caret positions.

## Evaluation

- One durable JetStream consumer (`alert-engine`) over `canon.v1.>` ([realtime-bus](realtime-bus.md#durable-consumers)).
- **Windowed state in memory:** per-market ring buffers for ≤ 1 h windows; anything longer evaluates on schedule against candles/Postgres instead of streaming state. Restart recovery: rebuild ring buffers from `candles_1m` + recent ticks — the engine is stateless-on-disk by design. The rolling-stat primitives (EWMA, z-score, ring buffers) live in `owt-domain`, shared with the forecast engine ([forecasts](forecasts.md)) — the `volume.burst` predicate and the `volume-z.v1` signal are the same math.
- **Firing:** predicate true ∧ not throttled ⇒ AlertEvent `{rule_id, market_id, ts, severity, snapshot-of-trigger-values}`, deduped by `(rule_id, market_id, window_id)`.
- **Replay/staleness guard:** events older than `max(2× window, 5 min)` relative to wall clock (bus catch-up, replays) are **suppressed from delivery** but still recorded with `stale: true` — alerts must never re-fire history after a restart.

## Delivery

1. Persist to `alert_events` (hypertable, 90 d retention — [storage](storage.md)).
2. Publish `canon.v1.alert.{rule_id}` → API fanout → WS topic `alerts`.
3. **TUI:** inbox screen (unacked count in status bar, `severity` styling), toast flash, terminal bell + OSC 9 desktop notification for `warn`+ (configurable). Inbox acks are per-workspace and survive reconnects (server-side `acked_at`).
4. **Webhook:** per-channel config `webhook:{name}` → POST JSON with HMAC signature header, 3 retries exponential; ntfy-compatible payload shape so self-hosters get phone pushes for free. Email deferred.

## Provisioning

Two sources, merged:

- **File-provisioned** (`alerts.toml` in server config) — GitOps-friendly for self-hosts; read-only from the API (`"source": "file"`).
- **API-managed** — CRUD under `/v1/alerts/rules` ([query-api](query-api.md)); persisted per workspace.

Name collisions: file wins, API rule disabled with a surfaced conflict warning. Both compile through the same parser at load; invalid file rules fail startup loudly (`owtd check-config` catches them pre-deploy).

## Failure semantics

- Evaluator lag > 30 s ⇒ ops alert (meta!) via `bus_consumer_lag{consumer="alert-engine"}` — product alerts and ops alerts stay separate systems ([observability](../ops/observability.md)).
- Duplicate-fire prevention is idempotent storage (`alert_events` natural key) + the dedupe window — a crashed engine that re-processes acked bus messages emits zero duplicate deliveries.
- Clock discipline: all windows computed on event `ts` (upstream time), not arrival time; skew beyond 5 min flags the source, not the rule.

## Metrics & tests

`alert_eval_lag_seconds`, `alert_fired_total{rule,severity}`, `alert_suppressed_total{reason}` (throttle/stale/dedupe), `alert_delivery_latency_seconds` (event ts → WS frame; SLO p95 ≤ 2 s), `webhook_failures_total`.

Test gate (v1): synthetic-feed scenarios in `owt-testkit` — a scripted tick/news sequence with known expected firings, run against the full pipeline; plus property tests on the DSL parser and staleness suppression.

## Related

- [realtime-bus.md](realtime-bus.md)
- [query-api.md](query-api.md)
- [tui-client.md](tui-client.md)
- [storage.md](storage.md)
