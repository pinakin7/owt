# Contributing to owt

Thanks for considering it. owt is docs-first right now — the founding design vault in [`docs/`](docs/README.md) is the source of truth, and contributions that sharpen it are as valuable as code.

## Ground rules

- **Read the vault index first:** [docs/README.md](docs/README.md) explains the conventions (frontmatter, link style, note size). The [glossary](docs/product/glossary.md) defines every term; use them exactly.
- **Decisions are ADRs.** Changes to the canonical schema, auth model, storage engines, deployment topology, or public API contract require an ADR — process in [docs/adr/README.md](docs/adr/README.md). Smaller open questions go through [open-decisions](docs/governance/open-decisions.md).
- **The vault wins over the original research report.** The report has been removed from the vault; [ADR-0001](docs/adr/0001-rust-backend.md) and [ADR-0002](docs/adr/0002-ratatui-tui-first-client.md) are the sole surviving record of the stack it recommended and why it was rejected — never cite that stack as authority.
- **Stack of record:** Rust backend + ratatui TUI ([ADR-0001](docs/adr/0001-rust-backend.md), [ADR-0002](docs/adr/0002-ratatui-tui-first-client.md)).

## How to contribute

1. **Docs review (now):** typos and clarity fixes as direct PRs; substantive design disagreements as issues referencing the specific note and section.
2. **Source adapters (best first code):** a new upstream = one adapter crate implementing the ingest traits, publishing envelopes — no core changes. The path is documented in [data-sources § adding a source](docs/architecture/data-sources.md#adding-a-source-contributor-path).
3. **Core code:** follow the [cargo-workspace layering rules](docs/architecture/cargo-workspace.md) — dependency direction violations are design bugs by definition.

## Pull requests

- Keep PRs single-topic; link the doc/ADR they implement.
- Ingestion, storage, and (later) auth changes need a maintainer architecture sign-off before merge.
- New external dependencies need a one-line justification in the PR description.
- CI must pass, including the docs job (link check, frontmatter lint, banned-term grep) for vault changes.
- Update fixtures deliberately: upstream drift is meant to be *visible* in diffs.

## Labels

`area:*` (product, ingestion, storage, search, api, tui, ops) · `source:*` (polymarket, goldsky, news, rss, x) · `adr-needed` · `good first issue`.

## Licensing of contributions

By contributing you agree your contributions are licensed under the repository licenses: Apache-2.0 for code, CC-BY-4.0 for documentation. Sign-off (DCO) may be introduced when code lands.
