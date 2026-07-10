---
type: adr
status: approved
owner: pinakin
summary: "NATS JetStream as the replayable ingest bus; Kafka-class systems deferred."
tags: [area/realtime, area/ingestion]
related:
  - ../design/realtime-bus.md
  - ../design/ingestion.md
---

# ADR-0005: NATS JetStream as the replayable ingest bus

## Context

Deterministic replay is a product guarantee, not an implementation detail: when the normalizer changes, owt re-derives canonical facts from stored raw envelopes instead of re-fetching from rate-limited vendors. That requires a persistent bus with per-consumer cursors, subject filtering, and light enough operations that a self-hoster runs it without noticing.

## Decision

**NATS JetStream** is the only transport between backend modules:

- Streams `RAW`, `CANON`, `DLQ` over the subject taxonomy in [realtime-bus](../design/realtime-bus.md).
- Durable consumers per module (store-writer, search-indexer, timeline-builder, alert-engine, API fanout).
- Publish-side dedupe via `Nats-Msg-Id` = `envelope_id`.
- Client: `async-nats` (official).

The bus is a **bounded replay window**, not the archive: Postgres holds durable truth, object storage holds long-horizon raw envelopes, and an archive→bus **rehydration** path covers replay beyond the window.

## Consequences

- Replay and consumer isolation from day one; adding a consumer never disturbs existing ones.
- NATS is a single small binary in compose — ops cost near zero at self-host scale.
- Retention windows must be sized consciously (defaults in [realtime-bus](../design/realtime-bus.md)); the rehydration path is mandatory design, not an afterthought.

## Alternatives considered

- **Redis Streams** — fine small-scale, but weaker replay/retention semantics and would reintroduce Redis, which MVP otherwise doesn't need.
- **Kafka / Redpanda** — the scale path, deferred behind a named trigger (sustained >10k msgs/s or external consumer teams needing Kafka compatibility).
- **Postgres-as-queue** — couples pipeline throughput to the primary store and makes fanout polling-based.

## Rollout notes

None (greenfield). Stream/consumer configs are code (`owt-bus`), applied idempotently at startup.

## Related

- [../design/realtime-bus.md](../design/realtime-bus.md)
- [../design/ingestion.md](../design/ingestion.md)
