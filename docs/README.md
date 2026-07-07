---
type: index
status: draft
owner: pinakin
summary: "Vault index: the map of all owt documentation, reading paths, and authoring conventions."
tags: [governance]
related:
  - product/glossary.md
  - adr/README.md
---

# owt documentation vault

**owt** is an open-source research terminal for prediction markets: a Rust backend that fuses Polymarket's official APIs, on-chain data, and news/RSS feeds into queryable timelines, search, watchlists, and alerts — surfaced through a native terminal (ratatui) client. This vault is the project's source of truth: product specs, architecture, low-level designs, decision records, and ops docs, written as atomic interlinked notes.

Open `docs/` as an Obsidian vault to browse the relation graph; every link is a standard relative Markdown link, so everything also renders on GitHub.

## How to read this vault

- **Newcomer:** [vision-and-scope](product/vision-and-scope.md) → [system-overview](architecture/system-overview.md) → [prd-mvp](product/prd-mvp.md) → [roadmap](product/roadmap.md)
- **Implementer:** [glossary](product/glossary.md) → [data-model](architecture/data-model.md) → [data-sources](architecture/data-sources.md) → [cargo-workspace](architecture/cargo-workspace.md) → the [design/](design/) LLD for your area, plus its linked ADRs
- **Operator:** [deployment](ops/deployment.md) → [observability](ops/observability.md) → [runbook](ops/runbook.md)
- **AI agents:** read this file, then [glossary](product/glossary.md). From the map below, load only the note(s) whose `summary` matches your task; follow each note's `related` links one hop at a time.

## The map

### Product

| Note | Status | Summary |
|---|---|---|
| [product/vision-and-scope.md](product/vision-and-scope.md) | draft | Why owt exists: the jobs it does, who it serves, its boundaries per release, and what it will never be. |
| [product/prd-mvp.md](product/prd-mvp.md) | draft | Testable product requirements for the owt MVP: features, screens, commands, data needs, NFRs, and acceptance criteria. |
| [product/roadmap.md](product/roadmap.md) | draft | Release trains MVP through v3 with goals, deliverables, and gates, re-plotted for the Rust/ratatui TUI-first stack. |
| [product/glossary.md](product/glossary.md) | draft | Canonical vocabulary for the Polymarket domain, the owt domain, and infrastructure terms used across the vault. |

### Architecture (HLD)

| Note | Status | Summary |
|---|---|---|
| [architecture/system-overview.md](architecture/system-overview.md) | draft | High-level design: modular monolith with async workers, component responsibilities, end-to-end data flows, and scale-up triggers. |
| [architecture/cargo-workspace.md](architecture/cargo-workspace.md) | draft | Cargo workspace decomposition: crate responsibilities, strict dependency layering, and MVP-versus-later phasing. |
| [architecture/data-sources.md](architecture/data-sources.md) | draft | Catalog of every upstream source with authority rank, endpoints, rate limits, failure modes, and compliance caveats. |
| [architecture/data-model.md](architecture/data-model.md) | draft | Canonical entities, identifier grammar, the normalized ingest envelope, and schema versioning policy. |

### Design (LLD)

| Note | Status | Summary |
|---|---|---|
| [design/ingestion.md](design/ingestion.md) | draft | LLD for backfill and realtime ingestion: scheduling, rate-limit budgets, checkpoints, idempotency, DLQ, and connection state machines. |
| [design/normalization.md](design/normalization.md) | draft | LLD for source-to-canonical mapping, cross-source dedupe and precedence, entity resolution, and timeline construction. |
| [design/storage.md](design/storage.md) | draft | LLD for PostgreSQL/Timescale: logical schema, hypertables, indexes, migrations, retention, and capacity math. |
| [design/search.md](design/search.md) | draft | LLD for Typesense: collection schemas, relevance, query DSL compilation, index maintenance, and blue/green rebuilds. |
| [design/query-api.md](design/query-api.md) | draft | LLD for the REST plus WebSocket API: endpoint inventory, subscription protocol, pagination, errors, and auth posture. |
| [design/realtime-bus.md](design/realtime-bus.md) | draft | LLD for NATS JetStream: subject taxonomy, stream and consumer configs, replay and rehydration, and fanout. |
| [design/tui-client.md](design/tui-client.md) | draft | LLD for the owt ratatui client: TEA event loop, pane system, command grammar, performance budgets, reconnect behavior, and packaging. |
| [design/alerts.md](design/alerts.md) | draft | LLD for the v1 alert engine: rule model, predicate DSL, streaming evaluation, and TUI-first delivery. |

### Decisions (ADR)

The authoritative index with process lives at [adr/README.md](adr/README.md).

| ADR | Decision | Status |
|---|---|---|
| [0001](adr/0001-rust-backend.md) | Rust for all backend services | approved |
| [0002](adr/0002-ratatui-tui-first-client.md) | Native ratatui TUI is the first client | approved |
| [0003](adr/0003-modular-monolith.md) | Modular monolith with async workers | approved |
| [0004](adr/0004-postgres-timescale-primary-store.md) | PostgreSQL + TimescaleDB as primary store | approved |
| [0005](adr/0005-nats-jetstream-bus.md) | NATS JetStream as the replayable ingest bus | approved |
| [0006](adr/0006-typesense-search.md) | Typesense for instant search | approved |
| [0007](adr/0007-goldsky-onchain-truth.md) | Goldsky V2 datasets as canonical on-chain truth | approved |
| [0008](adr/0008-read-only-through-v1.md) | Read-only product through v1 | approved |
| [0009](adr/0009-docs-obsidian-vault.md) | Docs as an Obsidian-compatible vault | approved |
| [0010](adr/0010-rest-ws-api-protocol.md) | REST/JSON + one multiplexed WebSocket API | approved |

### Ops

| Note | Status | Summary |
|---|---|---|
| [ops/deployment.md](ops/deployment.md) | draft | Dev and production topology: four-service Docker Compose stack, single-region production, migrations flow, and scale triggers. |
| [ops/observability.md](ops/observability.md) | draft | Telemetry standards: OpenTelemetry pipeline, first-class metrics catalog, SLO dashboards, and logging conventions. |
| [ops/runbook.md](ops/runbook.md) | draft | Incident runbook entries for the failure classes known at design time. |
| [ops/security-and-privacy.md](ops/security-and-privacy.md) | draft | Security model and privacy stance for the read-only v1, with forward constraints for the auth and execution phases. |
| [ops/testing-strategy.md](ops/testing-strategy.md) | draft | Test pyramid mapped to Rust tooling, the mandatory scenario list, and CI gates. |
| [ops/ci-cd-and-release.md](ops/ci-cd-and-release.md) | draft | CI/CD workflow set, release artifacts and distribution, versioning and skew policy, and the supply-chain ramp. |

### Governance & research

| Note | Status | Summary |
|---|---|---|
| [governance/open-decisions.md](governance/open-decisions.md) | draft | Living register of open decisions with the defaults currently in force and their promotion path to ADRs. |

## Conventions

Every vault note begins with YAML frontmatter:

```yaml
---
type: product        # product | hld | lld | adr | ops | governance | index | research
status: draft        # draft | review | approved | superseded
owner: pinakin       # GitHub handle
summary: "One sentence stating what this note specifies."
tags: [area/product, release/mvp]
related:
  - ../architecture/data-model.md
---
```

- **`summary` is mandatory** and must match this index verbatim — it is how humans and agents pick a note without opening it.
- **Links** are standard relative Markdown links; `[[wikilinks]]` are banned (they break GitHub rendering — see [ADR-0009](adr/0009-docs-obsidian-vault.md)). Obsidian is configured for this via the committed `.obsidian/app.json`. Each note ends with a `## Related` section repeating its `related` frontmatter as real links, which feeds the Obsidian graph.
- **Statuses:** `draft` → `review` → `approved`; `superseded` when replaced. For ADRs, `approved` means the decision is accepted; a superseding ADR must link back.
- **Size:** target 100–300 lines per note, soft cap 500 — split only along independently reviewable boundaries.
- **Tags** are controlled: `area/*` (product, architecture, ingestion, data-model, storage, search, api, realtime, tui, alerts, ops, security, testing, release), `source/*` (polymarket, goldsky, news, rss, x, polygon), `release/*` (mvp, v1, v2, v3).
- **Diagrams:** Mermaid only — renders on GitHub and in Obsidian.
- **Naming:** kebab-case filenames, stable once created (renames break links); the H1 matches the filename's human title.
- **Terminology** must match the [glossary](product/glossary.md); fix the doc or the glossary, never diverge silently.

## Precedence

The original deep-research report that seeded this vault has been removed; where its recommendations are still relevant, **the vault wins**. The stack of record is a Rust backend with a native ratatui client, per [ADR-0001](adr/0001-rust-backend.md) and [ADR-0002](adr/0002-ratatui-tui-first-client.md). Deliberate deviations from the research report are listed in [roadmap § Divergences](product/roadmap.md#divergences-from-the-research-report).

## How decisions are made

Open questions live in [governance/open-decisions.md](governance/open-decisions.md) with a default-in-force. When a default is confirmed or overturned, it becomes an ADR (process in [adr/README.md](adr/README.md)) and the register row is closed. ADRs are mandatory for changes to the canonical schema, auth model, storage engines, deployment topology, or public API contract.

## Related

- [product/glossary.md](product/glossary.md)
- [adr/README.md](adr/README.md)
