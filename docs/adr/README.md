---
type: index
status: draft
owner: pinakin
summary: "ADR index and process: numbering, lifecycle, template, and when an ADR is mandatory."
tags: [governance]
related:
  - ../README.md
  - ../governance/open-decisions.md
---

# Architecture Decision Records

An ADR captures one significant decision: the context that forced it, the decision itself, and its consequences. ADRs are append-only history — a reversed decision gets a *new* ADR that supersedes the old one; the old file is never rewritten.

## Index

| ADR | Title | Status | Decision in one line |
|---|---|---|---|
| [0001](0001-rust-backend.md) | Rust backend | approved | Rust for all backend services, superseding the research report's recommended stack. |
| [0002](0002-ratatui-tui-first-client.md) | ratatui TUI-first client | approved | A native ratatui TUI is the first client; browser terminal and Node CLI rejected for MVP. |
| [0003](0003-modular-monolith.md) | Modular monolith | approved | Modular monolith with async workers instead of microservices. |
| [0004](0004-postgres-timescale-primary-store.md) | Postgres + Timescale | approved | PostgreSQL with TimescaleDB as the primary store; ClickHouse deferred until a named bottleneck. |
| [0005](0005-nats-jetstream-bus.md) | NATS JetStream bus | approved | NATS JetStream as the replayable ingest bus; Kafka-class systems deferred. |
| [0006](0006-typesense-search.md) | Typesense search | approved | Typesense for instant search; OpenSearch deferred as the scale path. |
| [0007](0007-goldsky-onchain-truth.md) | Goldsky on-chain truth | approved | Goldsky V2 datasets as canonical on-chain truth; legacy subgraphs prohibited. |
| [0008](0008-read-only-through-v1.md) | Read-only through v1 | approved | Read-only product through v1: no wallets, accounts, or trading until v2/v3. |
| [0009](0009-docs-obsidian-vault.md) | Docs as Obsidian vault | approved | Documentation lives in an Obsidian-compatible vault of atomic notes with standard Markdown links. |
| [0010](0010-rest-ws-api-protocol.md) | REST + WS API protocol | approved | REST/JSON plus one multiplexed WebSocket as the client API; gRPC and GraphQL rejected. |

## When an ADR is mandatory

Any change to:

- the canonical schema or identifier grammar ([data-model](../architecture/data-model.md))
- the auth or key-handling model
- storage engines or the bus ([storage](../design/storage.md), [realtime-bus](../design/realtime-bus.md))
- deployment topology ([deployment](../ops/deployment.md))
- the public API contract ([query-api](../design/query-api.md))

Smaller decisions start as rows in [open-decisions](../governance/open-decisions.md) and get promoted to ADRs when confirmed or contested.

## Process

1. Copy the template below to `NNNN-kebab-title.md` using the next free 4-digit number (numbers are never reused).
2. Open a PR with frontmatter `status: draft`; discussion happens on the PR.
3. On merge approval the status flips to `approved`. A later reversal adds a new ADR with a "Supersedes" link both ways and flips the old one to `superseded`.
4. Add the new row to the table above and to the [vault index](../README.md).

## Template

```markdown
---
type: adr
status: draft
owner: <github-handle>
summary: "<the decision in one sentence>"
tags: [area/<area>]
related:
  - <affected docs>
---

# ADR-NNNN: <title>

## Context
<the forces and constraints that make this decision necessary>

## Decision
<what we chose, stated as a fact>

## Consequences
<what becomes easier, harder, or newly required>

## Alternatives considered
<each rejected option and the reason it lost>

## Rollout notes
<migration, sequencing, or "none">

## Related
<links repeated from frontmatter>
```

## Related

- [../README.md](../README.md)
- [../governance/open-decisions.md](../governance/open-decisions.md)
