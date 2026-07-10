---
type: governance
status: draft
owner: pinakin
summary: "Living register of open decisions with the defaults currently in force and their promotion path to ADRs."
tags: [area/product, area/architecture]
related:
  - ../adr/README.md
  - ../product/roadmap.md
---

# Open decisions

How this register works: every unresolved question gets a row with a **default in force** — the assumption all docs and code build against until the row is resolved. Resolving a row means confirming or overturning the default; either way the outcome becomes an ADR (or a doc edit for small items) and the row moves to the resolved table. Never resolve a row silently.

## Open

| # | Priority | Question | Default in force | Resolve by |
|---|---|---|---|---|
| D-01 | High | Target scale: concurrent TUI clients, indexed docs/day | ≤ 50 concurrent clients per `owtd`; laptop-class self-host | MVP feature-freeze |
| D-02 | High | Confirm proposed SLO numbers ([prd-mvp](../product/prd-mvp.md), [observability](../ops/observability.md)) | Proposed tables stand as written | MVP feature-freeze |
| D-03 | High | Hosted public instance, or self-host only? | Self-host only through MVP; no hosted infra to secure | v1 kickoff |
| D-04 | High | News-API budget beyond free tiers (NewsAPI, Event Registry) and X API tier | RSS + GDELT only (free); X deferred to v2 | v1 kickoff |
| D-05 | High | Goldsky access tier, pricing, and dataset-name verification | Plan for standard tier; fallback = direct Polygon indexer ([data-sources](../architecture/data-sources.md)) | v1 kickoff |
| D-06 | Medium | Web client timing (reuses REST/WS contract unchanged) | Not before v2; revisit with SIWE work | v2 kickoff |
| D-07 | Medium | Plugin mechanism evolution | MVP: compile-time trait adapters; v1+: out-of-process adapters publishing envelopes to `raw.*` (the bus contract is the plugin API); WASM explored ≥ v2 | v1 retro |
| D-08 | Medium | Multi-tenant team workspaces (beyond per-user) | Single `default` workspace pre-auth; schema carries `workspace_id` from day one | v2 design |
| D-09 | Medium | TUI local cache for offline/instant cold start | None in MVP (in-memory only) | v1 retro |
| D-10 | Medium | SIWE-from-terminal UX specifics (device-flow analog, keychain storage) | OAuth-style device flow: `owt login` → browser/QR SIWE → polled session token → OS keychain | v2 design (auth ADR) |
| D-11 | Low | Brand/name beyond the working name `owt` | Keep `owt` (binary/crate/config naming already assumes it) | before first public release |
| D-12 | Low | Monorepo confirmation as code lands | Single repo: workspace + vault | first code phase |
| D-13 | Low | Notes/annotations feature (per-market research notes) | Out of MVP; v2 candidate | v1 retro |
| D-14 | Medium | Aggregated-odds weighting formula and thresholds ([topic-lookup](../design/topic-lookup.md)) | `liquidity_score` weights (fallback `volume_24h`), staleness TTL 24 h, low-liquidity badge not hidden | MVP feature-freeze |
| D-15 | Medium | Forecast evaluation bar: Brier/calibration thresholds that gate a model shipping or getting flagged `degraded` ([forecasts](../design/forecasts.md#evaluation)) | Brier tracked per (model, version) from first v1 deploy; ship-gate numbers set with real data | v2 kickoff (calibration model) |

## Resolved by founder (2026-07-07)

| Decision | Outcome | Record |
|---|---|---|
| Backend language | Rust | [ADR-0001](../adr/0001-rust-backend.md) |
| First client | Native ratatui TUI | [ADR-0002](../adr/0002-ratatui-tui-first-client.md) |
| Product boundary | Read-only through v1 | [ADR-0008](../adr/0008-read-only-through-v1.md) |
| Docs format | Obsidian-compatible vault in-repo | [ADR-0009](../adr/0009-docs-obsidian-vault.md) |
| License | Apache-2.0 (code), CC-BY-4.0 (docs) | `LICENSE`, [../README.md](../README.md) |
| Architecture style | Modular monolith + async workers | [ADR-0003](../adr/0003-modular-monolith.md) |
| Primary store / bus / search | Postgres+Timescale / NATS JetStream / Typesense | [ADR-0004](../adr/0004-postgres-timescale-primary-store.md) · [ADR-0005](../adr/0005-nats-jetstream-bus.md) · [ADR-0006](../adr/0006-typesense-search.md) |
| On-chain truth | Goldsky V2 datasets; legacy subgraphs prohibited | [ADR-0007](../adr/0007-goldsky-onchain-truth.md) |
| Client API protocol | REST/JSON + one multiplexed WebSocket | [ADR-0010](../adr/0010-rest-ws-api-protocol.md) |

## Related

- [../adr/README.md](../adr/README.md)
- [../product/roadmap.md](../product/roadmap.md)
