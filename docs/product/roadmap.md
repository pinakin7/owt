---
type: product
status: draft
owner: pinakin
summary: "Release trains MVP through v3 with goals, deliverables, and gates, re-plotted for the Rust/ratatui TUI-first stack."
tags: [area/product, area/release]
related:
  - vision-and-scope.md
  - prd-mvp.md
  - ../ops/ci-cd-and-release.md
  - ../adr/0008-read-only-through-v1.md
---

# Roadmap

owt ships on **release trains** — each train has a goal, a fixed deliverable set, and explicit testing/CI/docs gates. Scope that misses a train waits for the next one; the roadmap is this file, not a backlog. PRDs are written just-in-time: [prd-mvp](prd-mvp.md) exists now; the v1 PRD is authored at MVP feature-freeze.

## MVP — read-only research terminal

**Goal:** a self-hostable terminal that answers "what is this market and why is it moving" from historical + live market data and news.

| Aspect | Content |
|---|---|
| Milestones | Ingest core (Gamma/CLOB/Data backfill, RSS/GDELT news); canonical store + search; query API; `owt` TUI; live `market` WS channel |
| Deliverables | `owtd` + `owt` binaries (cargo-dist), Docker Compose stack (4 services), search / market detail / event timeline / topic page with aggregated odds ([prd-topic-lookup](prd-topic-lookup.md)) / watchlists / saved views, historical price backfill, market–news linkage, `.env.example` |
| Testing gate | Unit + property tests on normalizers; contract tests on recorded fixtures; `TestBackend` golden frames; PTY smoke flow; index-rebuild-from-Postgres test |
| CI/CD gate | fmt/clippy/build/nextest green; docs lint (links, frontmatter, banned terms); container build; draft GitHub release with binaries |
| Docs gate | This vault complete (statuses ≥ review); README quick start verified on a clean machine |

**Explicitly in MVP:** the Polymarket `market` WebSocket channel (live book, price changes, last trade). A terminal without a live tape is a market browser — this resolves the research report's internal contradiction between its MVP boundary and its release table.

## v1 — realtime everywhere + alerts

**Goal:** the operational terminal: everything live, alerting, and correlation.

| Aspect | Content |
|---|---|
| Milestones | RTDS (comments + side-streams); sports/user WS channels; Goldsky on-chain fills; alert engine; price/news correlation; forecast engine ([forecasts](../design/forecasts.md)) |
| Deliverables | Live comment feeds, on-chain fill tape, in-TUI alert inbox + webhook delivery, correlation confidence on timelines, model-signal catalog v0 + forecast/odds endpoints + materialized entity odds (F14, [prd-topic-lookup](prd-topic-lookup.md)), replayable event log demos |
| Testing gate | Synthetic-feed alert tests; WS reconnect suite (15s/60s/5m); replay determinism; forecast golden-value + rebuild-determinism tests; chaos pass (Toxiproxy) |
| CI/CD gate | Perf workflow (k6) enforcing API SLOs; staging deploy + rollback drill |
| Docs gate | [runbook](../ops/runbook.md) and [observability](../ops/observability.md) flip to approved; v1 PRD published |

## v2 — identity and workspaces

**Goal:** persistent multi-device research identity — still no custody.

| Aspect | Content |
|---|---|
| Milestones | SIWE auth via terminal device-flow (browser/QR handoff, session in OS keychain); multi-workspace model; private Polymarket account *views* |
| Deliverables | Login flow, shared/private workspaces, saved-search sync, account position views (read-only), calibration-adjusted probability forecasts + backtest/evaluation harness ([forecasts](../design/forecasts.md#evaluation)), optional hosted-instance hardening |
| Testing gate | Auth/session expiry suites; workspace isolation tests; deletion/export (privacy rights) path |
| CI/CD gate | Secret scanning + security review gates; canary deploy |
| Docs gate | Auth ADR; [security-and-privacy](../ops/security-and-privacy.md) deepened (threat model, retention, rights handling) |

The **web client track may open here** — it reuses the REST/WS contract unchanged; timing is an open decision ([open-decisions](../governance/open-decisions.md)).

## v3 — execution-adjacent (gated)

**Goal:** optional, guarded order workflows without becoming a custodian.

| Aspect | Content |
|---|---|
| Milestones | Signer-isolation ADR; local signer sidecar; guarded order preview + dry-run; immutable audit trail |
| Deliverables | Trade preview in TUI, local signing path, execution receipts, kill-switch config |
| Testing gate | Signer isolation tests; fault-injection on order paths; rate-limit stress |
| CI/CD gate | Signed release artifacts, SBOM/provenance, production promotion checks |
| Docs gate | Execution guide, incident playbooks, compliance notes |

## Divergences from the research report

Deliberate changes from the original research report (since removed from the vault; see [ADR-0009](../adr/0009-docs-obsidian-vault.md)), each anchored in an ADR or LLD:

1. **Native ratatui TUI replaces the browser terminal and Node CLI** ([ADR-0002](../adr/0002-ratatui-tui-first-client.md)); web client deferred to ~v2.
2. **Rust replaces the report's originally proposed managed-runtime backend** ([ADR-0001](../adr/0001-rust-backend.md)).
3. **Live `market` WS channel moved into MVP** (report deferred all WS to v1 while claiming live books in MVP scope).
4. **Redis cut from MVP** — in-process cache suffices single-node; reintroduction trigger documented ([deployment](../ops/deployment.md)).
5. **No separate realtime gateway** — WS fanout lives in the API service ([system-overview](../architecture/system-overview.md)).
6. **API style settled as REST + WebSocket** ([ADR-0010](../adr/0010-rest-ws-api-protocol.md)); the report left "GraphQL-or-REST" open.
7. **Goldsky deferred to v1**; Data API supplies the MVP tape. Legacy subgraphs prohibited ([ADR-0007](../adr/0007-goldsky-onchain-truth.md)).
8. **X/social ingestion deferred to v2** (paid access, policy sensitivity); schema slots reserved.
9. **NLP fields (`sentiment`, `importance`) nullable at MVP**; entity resolution is dictionary-based first ([normalization](../design/normalization.md)).
10. **Playwright replaced by TestBackend + PTY harness** ([testing-strategy](../ops/testing-strategy.md)).
11. **Fill records use `amount_collateral` + `collateral_token`** (pUSD era), not `amount_usdc` ([data-model](../architecture/data-model.md)).

## Beyond v3

Multi-venue support (other prediction markets under the same canonical model) is the natural horizon — deliberately unplanned until the Polymarket-first product proves the model.

## Related

- [vision-and-scope.md](vision-and-scope.md)
- [prd-mvp.md](prd-mvp.md)
- [../ops/ci-cd-and-release.md](../ops/ci-cd-and-release.md)
- [../adr/0008-read-only-through-v1.md](../adr/0008-read-only-through-v1.md)
