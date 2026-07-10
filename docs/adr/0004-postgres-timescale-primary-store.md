---
type: adr
status: approved
owner: pinakin
summary: "PostgreSQL with TimescaleDB as the primary store; ClickHouse deferred until a named bottleneck."
tags: [area/storage]
related:
  - ../design/storage.md
  - ../architecture/data-model.md
---

# ADR-0004: PostgreSQL + TimescaleDB as the primary store

## Context

owt's durable data is two-shaped: relational reference data (events, markets, tokens, entities, workspaces) needing integrity and transactions, and append-only time-series facts (ticks, trades, news, timeline items) needing retention, compression, and rollups. Checkpointed ingestion also requires committing data and cursor atomically — a real transaction.

## Decision

One PostgreSQL (16+) instance with the TimescaleDB extension:

- Fact tables are **hypertables** with per-class chunking, compression, and retention policies.
- Candles are **continuous aggregates** (1m base, cascaded 5m/1h/1d) — computed in the database, never in application code.
- Access via **sqlx** with compile-time-checked SQL; migrations are embedded SQL run by explicit `owtd migrate` ([storage](../design/storage.md)).

## Consequences

- One database to operate, back up, and reason about; search and bus state are rebuildable from it.
- **Licensing note:** hypertable compression and continuous aggregates are Timescale-licensed (TSL) — free to self-host, but may not be resold as a managed database service. Fine for owt's model; anyone building a DBaaS on top must know.
- ClickHouse is deferred behind a named trigger: sustained analytical scans over billions of fact rows, or ingest beyond what a tuned single Postgres sustains ([system-overview](../architecture/system-overview.md)).

## Alternatives considered

- **Plain PostgreSQL** — viable start, but retention/compression/rollups become hand-rolled cron jobs; Timescale automates exactly the maintenance owt needs.
- **ClickHouse-first** — superb ingest/scan rates, but weak transactional semantics for checkpoint-atomic writes and heavier ops; wrong first store for a self-hosted tool.
- **SQLite (embedded mode)** — rejected for the server: concurrent writers, bus consumers, and search rebuilds outgrow it immediately.

## Rollout notes

None (greenfield). The compose stack pins a Timescale-enabled Postgres image ([deployment](../ops/deployment.md)).

## Related

- [../design/storage.md](../design/storage.md)
- [../architecture/data-model.md](../architecture/data-model.md)
