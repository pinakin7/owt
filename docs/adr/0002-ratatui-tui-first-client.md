---
type: adr
status: approved
owner: pinakin
summary: "A native ratatui TUI is the first client; browser terminal and Node CLI rejected for MVP."
tags: [area/tui]
related:
  - 0001-rust-backend.md
  - ../design/tui-client.md
  - ../product/vision-and-scope.md
---

# ADR-0002: Native ratatui TUI as the first client

## Context

The research report recommended a browser terminal (a managed-runtime web toolchain rendering into xterm.js) plus an optional Node/Ink CLI. Two things changed: [ADR-0001](0001-rust-backend.md) removed the stack that made that browser-compiled web client attractive, and the founder explicitly chose a native TUI as the first surface. Polymarket also already ships an official experimental Rust CLI for basic market browsing and order placement — so owt's client must differentiate on research workflow, not command coverage.

## Decision

The first and only v1 client is **`owt`**, a native terminal application built on **ratatui + crossterm**, speaking exclusively to the owt server's REST + WebSocket API ([query-api](../design/query-api.md)) via the shared `owt-client` SDK crate. It never touches the database, search engine, or bus directly.

A web client is a **deferred option, not a rejection** — its timing is tracked in [open-decisions](../governance/open-decisions.md) and it would reuse the same API surface unchanged.

## Consequences

- Truest "terminal" experience with the lowest input-to-render latency; works over ssh/tmux; single static binary per platform.
- No browser reach at MVP — mitigated with asciinema demos in the README and prebuilt binaries (cargo-dist) so trying owt is one download.
- E2E testing has no browser to drive: the gate becomes ratatui `TestBackend` golden-frame snapshots plus PTY-driven flows ([testing-strategy](../ops/testing-strategy.md)); Playwright is out.
- The deployment story loses the CDN/static-bundle leg entirely ([deployment](../ops/deployment.md)); client shipping is a release-asset concern ([ci-cd-and-release](../ops/ci-cd-and-release.md)).
- Design detail for panes, command grammar, and budgets lives in [tui-client](../design/tui-client.md).

## Alternatives considered

- **Managed-runtime web toolchain + xterm.js browser terminal** (the report's recommendation) — rejected for MVP: depends on the stack voided per ADR-0001, and requires web hosting before the product has proven itself.
- **Node + Ink CLI** — rejected: introduces a second language and duplicates what the TUI does, worse.
- **TypeScript/React web app first** — broadest reach; deferred, not rejected — it becomes attractive at v2 when auth and sharing arrive.
- **Native GUI (Tauri/egui)** — rejected: heavier than a TUI, without the terminal-native workflow the product is named for.

## Rollout notes

Greenfield. The API contract (`owt-api-types`) is designed so a later web client is purely additive.

## Related

- [0001-rust-backend.md](0001-rust-backend.md)
- [../design/tui-client.md](../design/tui-client.md)
- [../product/vision-and-scope.md](../product/vision-and-scope.md)
