---
type: lld
status: draft
owner: pinakin
summary: "LLD for backfill and realtime ingestion: scheduling, rate-limit budgets, checkpoints, idempotency, DLQ, and connection state machines."
tags: [area/ingestion, source/polymarket, source/news, release/mvp]
related:
  - ../architecture/data-sources.md
  - ../architecture/data-model.md
  - realtime-bus.md
  - normalization.md
---

# Ingestion & backfill

Two planes over the same machinery: **backfill** (paginated REST sweeps building history) and **realtime** (long-lived stream consumers). Both emit [envelopes](../architecture/data-model.md#the-ingest-envelope-normative) to `raw.*` subjects and share the invariants:

1. **Idempotent** — re-running any job produces no duplicates (envelope dedupe + natural-key upserts).
2. **Checkpointed** — every job can crash at any instant and resume without loss or repeats.
3. **Replayable** — raw envelopes are preserved (bus window + object archive) so normalization can be re-run without touching vendors.

## Adapter traits (`owt-ingest-core`)

```rust
#[async_trait]
pub trait BackfillSource {
    fn source_id(&self) -> SourceId;
    /// Enumerate work partitions (e.g. one per endpoint, per market page range).
    async fn plan(&self, ck: &CheckpointStore) -> Result<Vec<Partition>, IngestError>;
    /// Fetch one page for a partition; return envelopes + the next cursor (None = done).
    async fn fetch(&self, p: &Partition, cursor: Option<Cursor>)
        -> Result<(Vec<Envelope>, Option<Cursor>), IngestError>;
}

#[async_trait]
pub trait StreamSource {
    fn source_id(&self) -> SourceId;
    /// Run until cancelled; push envelopes into `tx`; internal reconnect loop.
    async fn run(&self, tx: EnvelopeSink, cancel: CancellationToken) -> Result<(), IngestError>;
}
```

`IngestError` classification drives the failure-mode matrix below.

## Endpoint inventory (MVP)

| Source | Endpoint / channel | Pagination | Used for | Freshness need |
|---|---|---|---|---|
| Gamma | `GET /events` | offset/limit | event universe | sweep ≤ 60 s (active) |
| Gamma | `GET /markets` | offset/limit | market universe + metadata | sweep ≤ 60 s (active) |
| Gamma | `GET /tags`, `/public-search` | offset/limit | tag dictionary; completion | daily / on-demand |
| CLOB | `GET /book?token_id=` | — | book snapshot (seed + resync) | on-demand |
| CLOB | `GET /prices-history?market=…&interval=…` | range params | price backfill | backfill only |
| CLOB | `GET /midpoint`, `/price` | — | spot checks, reconciliation | on-demand |
| Data | `GET /trades` | cursor/offset | trade tape (historical + sweep) | sweep ≤ 60 s (watched) |
| PM WS | `market` channel | subscribe by token_id | live book/price/last-trade | streaming |
| RSS | feed registry | conditional GET (ETag/Last-Modified) | news items | poll 1–5 min |
| GDELT | doc API sweeps | time-windowed | news breadth | poll 15 min |
| RTDS *(v1)* | `comments` topic | subscribe | comments | streaming |
| Goldsky *(v1)* | V2 datasets | vendor cursor | fills/positions | ≤ 60 s |

Exact payload shapes are fixture-recorded per endpoint ([testing-strategy](../ops/testing-strategy.md)); ⚠-status details in [data-sources](../architecture/data-sources.md) get verified as fixtures are captured.

## Rate-limit budgets

Documented limits (verified 2026-07-07) live in [data-sources](../architecture/data-sources.md). owt's **configured budgets default to ≤ 50 % of documented**, are per-(source, endpoint-class) token buckets (`governor`), and are **config values, never constants** — the platform rewrote its backend in April 2026 and limits will drift.

| Bucket | Documented /10 s | Default budget /10 s |
|---|---|---|
| `gamma.events` | 500 | 250 |
| `gamma.markets` | 300 | 150 |
| `gamma.search` | 350 | 100 |
| `data.trades` | 200 | 100 |
| `data.positions` | 150 | 75 |
| `clob.book_price` | 1,500 | 750 |
| `clob.prices_history` | 1,000 | 500 |
| global per-host concurrency | — | 8 in-flight |

**Throttle detection:** Polymarket throttling is Cloudflare *queueing* — requests get delayed, not 429-rejected. Adapters therefore track a rolling latency baseline per bucket; sustained latency > 3× baseline triggers a **pause window** (30 s, doubling to 5 min) exactly as a 429 would. Plain 429s (seen under bursts) use `Retry-After` when present, else exponential backoff with jitter.

## Scheduler

Job taxonomy (all tokio tasks under `owt-runtime` supervision, intervals jittered ±20 %):

| Job | Interval | Priority | Notes |
|---|---|---|---|
| `sweep.active_markets` | 60 s | high | Gamma incremental (changed-since ordering) |
| `sweep.long_tail` | 10 min | low | full-universe pagination, resumable |
| `sweep.trades.watched` | 30 s | high | Data `/trades` for watchlisted/open markets |
| `poll.rss.{feed}` | per-registry (1–5 min) | med | conditional GET; back off on 304 streaks |
| `poll.gdelt` | 15 min | med | time-windowed sweeps |
| `backfill.*` | manual (`owtd backfill`) | low | five-phase plan below |

Priorities matter only under budget contention: high-priority jobs take tokens first; backfill consumes leftovers.

## Backfill plan (five phases)

`owtd backfill --top N --days D` runs phases sequentially, each resumable:

1. **Gamma universe** — all events + markets → reference tables. *Illustrative math:* ~25k markets at 100/page = 250 calls ÷ 150/10 s budget ≈ **17 s**.
2. **CLOB price history** — for the top-N active tokens (by liquidity/volume): ~4 range calls per token × 500 tokens = 2,000 calls ÷ 500/10 s ≈ **40 s**; long tail proceeds in the background for hours — fine.
3. **Data trades** — last D days for top-N markets; the tightest budget (100/10 s) makes this the pacing item: ~10k calls ≈ **17 min**.
4. **Goldsky replay** *(v1)* — historical fills via vendor cursor.
5. **News history** — RSS archives where available + GDELT windows keyed to the entity dictionary.

Every phase logs `{calls_made, budget, eta}`; the PRD gate is universe + 30 d × top-200 ≤ 4 h on a laptop, which the math clears with wide margin even at half budgets — re-verify with real payload sizes when fixtures land.

## Checkpoints

Table `ingest_checkpoints` ([storage](storage.md)): key `(source, endpoint, partition)`, value `{cursor, ts, ingest_version}`.

- Cursor formats are source-native: offset for Gamma, time-range watermark for prices-history, trade-id watermark for Data, vendor cursor for Goldsky, per-feed `(etag, last_modified, newest_item_hash)` for RSS.
- **Atomicity rule:** the checkpoint row advances in the *same Postgres transaction* as the batch's canonical upserts. Crash between bus-publish and store-write is safe: envelope dedupe absorbs re-publishes, upserts absorb re-writes.

## Idempotency

- **Bus:** publish with `Nats-Msg-Id = envelope_id`; JetStream's dedupe window (2 min) absorbs immediate retries. Deterministic envelope IDs for RESTfetched objects (UUIDv7 derived at first receipt, stored with the checkpoint) keep re-fetches from double-publishing.
- **Store:** every canonical table has a natural key and `ON CONFLICT` upsert ([storage](storage.md)); facts conflict-ignore, snapshots conflict-update with `updated_at` guard.

## Dead-letter queue

- Subjects `dlq.{stage}.{source}` (stages: `ingest`, `normalize`, `write`) carry `{envelope, error, attempt, first_seen}`.
- A consumer exceeding `max_deliver` (5) on a message NAKs it to the DLQ; poison messages (non-deserializable, oversized) go straight there.
- DLQ consumer persists to the `dlq_quarantine` table for inspection; `owtd replay --from-quarantine <ids>` re-injects after a fix. Runbook: [runbook](../ops/runbook.md).
- **SLO: DLQ steady-state ≈ 0**; any sustained growth pages ([observability](../ops/observability.md)).

## Raw archive

- Object store (MinIO/S3) keys: `raw/{source}/{yyyy}/{mm}/{dd}/{envelope_id}.json.zst`; filesystem fallback (`data/raw/…`) when MinIO isn't deployed.
- Written by a bus consumer off `raw.>` (not by adapters — one write path).
- Retention: raw HTTP 30 d, raw WS frames 14 d (config; classes in [storage](storage.md)). The archive is what makes replay-beyond-bus-window possible ([realtime-bus § rehydration](realtime-bus.md)).

## WebSocket / RTDS connection management

State machine per connection: `Connecting → Subscribing → Live → Degraded → Backoff → Connecting…`

- **Heartbeat (verified 2026-07-07):** answer server pings within 10 s; send client `PING` every 10 s and expect `PONG`. A missed pong or 30 s silence ⇒ `Degraded` ⇒ reconnect.
- **Subscribe batching:** token subscriptions batched (≤ 100 per frame ⚠ verify); subscription registry re-plays the full set after reconnect.
- **Backoff:** 1 s → 2 → 4 → … cap 60 s, jittered; alert after 5 consecutive failures.
- **Resync on reconnect:** for each subscribed token, fetch a REST `book` snapshot, then apply WS deltas; sequence gaps (where `seq` exists) force another snapshot. Books are versioned so stale deltas drop.
- **Gap accounting:** the outage window is marked; watched markets get a targeted `trades` sweep to fill the tape hole.
- Mandatory reconnect tests at 15 s / 60 s / 5 min outages ([testing-strategy](../ops/testing-strategy.md)).

## Failure-mode matrix

| Class | Examples | Action |
|---|---|---|
| `Transient` | timeouts, 5xx, disconnects | retry w/ backoff+jitter, then DLQ |
| `RateLimited` | 429, latency-throttle detection | pause window per bucket; no retry burn |
| `AuthExpired` | 401/403 (v1+ sources) | pause source, alert operator |
| `SchemaDrift` | deserialization fails post-tolerance | envelope to DLQ, drift counter++, alert; fixtures updated |
| `Permanent` | 404 on known object, malformed feed | quarantine, don't retry |

## Metrics (first-class)

`ingest_lag_seconds{source}`, `budget_utilization{bucket}`, `throttle_pauses_total{bucket}`, `ws_reconnects_total{channel}`, `ws_gap_seconds`, `checkpoint_age_seconds{job}`, `dlq_depth{stage}`, `schema_drift_total{source}`, `backfill_progress{phase}`. Dashboards: [observability](../ops/observability.md).

## Related

- [../architecture/data-sources.md](../architecture/data-sources.md)
- [../architecture/data-model.md](../architecture/data-model.md)
- [realtime-bus.md](realtime-bus.md)
- [normalization.md](normalization.md)
