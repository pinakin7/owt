---
type: ops
status: draft
owner: pinakin
summary: "Test pyramid mapped to Rust tooling, the mandatory scenario list, and CI gates."
tags: [area/testing, release/mvp]
related:
  - ../design/normalization.md
  - ../design/tui-client.md
  - ci-cd-and-release.md
---

# Testing strategy

The system's riskiest surface is the boundary with upstreams that drift and disconnect; the pyramid is weighted accordingly — heavy on fixtures and determinism, light on mocks of our own code.

## Levels

| Level | Scope | Tooling | Runs |
|---|---|---|---|
| **Unit** | parsers, normalizer mappings, DSL, ranking math, `update()` in the TUI | `cargo nextest`, `proptest` | every PR, seconds |
| **Property** | normalization determinism, URL canonicalization idempotence, outcome zip integrity, cursor round-trips | proptest | every PR |
| **Contract** | recorded upstream fixtures → adapter/normalizer output asserted against goldens; detects Polymarket/GDELT/RSS drift | `owt-testkit` + wiremock, `insta` goldens | every PR + **scheduled daily against live APIs** (drift canary) |
| **Integration** | bus → store → search path; migrations; replay; checkpoint resume | testcontainers-rs (PG+NATS+Typesense) or compose in CI | every PR, minutes |
| **API contract** | `owt-client` against `owt-api` with seeded store; OpenAPI diff (additive-only); WS conformance script | in-process axum + testkit | every PR |
| **TUI E2E** | golden frames per screen/breakpoint; PTY-driven demo-script flow | `TestBackend`+insta; `expectrl` | frames every PR; PTY on merge queue |
| **Load** *(gate from v1)* | query/search/WS-fanout SLOs | k6 vs staging | pre-release |
| **Chaos** *(gate from v1)* | bus latency/partitions, DB latency, WS drops | Toxiproxy scenarios | pre-release |

Playwright appears nowhere: there is no browser ([ADR-0002](../adr/0002-ratatui-tui-first-client.md)).

## Fixture policy

- Fixtures live in git (`fixtures/{source}/…`), small and scrubbed; **raw archives never enter git**.
- Each mapping row in [normalization](../design/normalization.md#mapping-tables-normative-once-fixtures-land) requires: one pristine capture, plus edge cases (nulls, stringified-array oddities, drifted fields).
- `xtask capture-fixture <source> <endpoint>` records + scrubs live responses; fixture updates are reviewed diffs, so upstream drift is *visible* in PRs.
- The daily drift canary re-runs contract tests against live endpoints and opens an issue on failure — the automated version of the report's "upstream-compatibility audits."

## Mandatory scenarios

Automated, named, and kept green — the system's standing claims:

1. **Schema drift**: inject an unknown field + a retyped field into each source fixture → unknown accepted (counter increments), retyped → DLQ, never a crash.
2. **WS reconnect at 15 s / 60 s / 5 m**: gap detection, snapshot resync, no duplicate ticks, tape hole backfilled.
3. **Cross-source dedupe**: same story via RSS and GDELT → one cluster; same trade via Data API and Goldsky *(v1)* → linked, not duplicated.
4. **Replay determinism**: replay a captured raw window twice → zero row changes the second time; `ingest_version` bump replay → expected golden diffs only.
5. **Rate-limit exhaustion**: simulated queue-throttle latency + 429s → pause windows engage, budgets never exceeded, backfill resumes.
6. **Search rebuild from canonical only**: wipe Typesense → `owtd reindex` → golden queries pass; doc counts match ±0.1 %.
7. **Checkpoint crash-resume**: kill -9 mid-backfill-batch → restart resumes with no duplicates and no gaps (transaction atomicity proof).
8. **Terminal restore**: TUI panic in a PTY → terminal state restored (no raw-mode residue).
9. **Workspace deletion/export** *(lands v2, listed now)*: privacy-rights path.

## CI gates

Per PR (blocking): fmt + clippy `-D warnings` → unit/property/contract → integration → API contract → TUI golden frames → docs job (links, frontmatter, banned terms — [ci-cd-and-release](ci-cd-and-release.md)). Merge queue adds PTY smoke. Release adds load + chaos (from v1) and the full [prd-mvp demo script](../product/prd-mvp.md#acceptance-demo-script) run manually.

## Related

- [../design/normalization.md](../design/normalization.md)
- [../design/tui-client.md](../design/tui-client.md)
- [ci-cd-and-release.md](ci-cd-and-release.md)
