---
type: ops
status: draft
owner: pinakin
summary: "Developer bootstrap for the current tree: prerequisites, build/test/lint commands, running the owt TUI, the CI gates, and what works today versus scaffold."
tags: [area/ops, release/mvp]
related:
  - deployment.md
  - runbook.md
  - ci-cd-and-release.md
  - testing-strategy.md
  - ../architecture/cargo-workspace.md
  - ../design/tui-client.md
---

# Getting started

What actually runs in the repository today, and how to build, test, and drive it. The
deployment guide ([deployment](deployment.md)) and runbook ([runbook](runbook.md))
describe the *target* operation of the full stack; this note is the ground truth for the
current tree, where the server runs as a supervised chassis but its roles do no real work
yet.

> **State of the tree.** The `owt` TUI client and the `owt-client` SDK are implemented.
> The `owtd` server runs as a modular-monolith chassis (ADR-0003): `serve --roles`
> supervises one placeholder worker per role with graceful shutdown, and `check-config`
> loads and prints the merged configuration — but the roles do no real work yet, and
> `migrate`/`backfill`/`reindex`/`replay` still print "not yet implemented". There is no
> Docker Compose stack, database, or bus yet. You can build, test, lint, run the TUI, and
> start the server, but it exposes no API for the TUI to talk to.

## Prerequisites

- **Rust toolchain**, pinned by `rust-toolchain.toml` (stable channel; workspace MSRV is
  1.95). `rustup` installs the pinned toolchain automatically on first `cargo` invocation.
- **git**.
- Optional but recommended (they mirror CI): [`cargo-nextest`](https://nexte.st) as the
  test runner, `cargo-audit` and `cargo-deny` for the security job, and `python3` for the
  docs linter (`scripts/verify-docs.py`, standard library only).

No infrastructure services (Postgres/Timescale, NATS, Typesense) are needed yet — nothing
in the tree connects to them. They arrive with the server implementation
([cargo-workspace](../architecture/cargo-workspace.md)).

## Build

```bash
cargo build --workspace
```

## Test and lint (the CI gates)

`ci.yml` runs these on every PR and on `main`; run them locally before pushing. The build
uses `--locked`, so commit `Cargo.lock` changes when dependencies move.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets --locked
cargo nextest run --workspace        # or: cargo test --workspace
cargo test --workspace --doc         # doctests
python3 scripts/verify-docs.py       # docs vault linter (the docs job)
```

A separate `msrv` job runs `cargo +1.95.0 check`. The `security.yml` workflow runs
`cargo audit` and `cargo deny check bans licenses sources advisories` — run those two
locally if you touched dependencies.

Test layout worth knowing: `owt-client` has REST contract tests against a `wiremock`
mock and WebSocket tests against an in-test server; the TUI has golden-frame snapshots
over a `TestBackend` plus pure `update()` tests. The broader plan is in
[testing-strategy](testing-strategy.md).

## Run the TUI

```bash
cargo run -p owt                     # or the built binary: ./target/debug/owt
```

Flags (the highest-precedence config layer): `--server <url>` overrides the server URL
for the run, `--profile <name>` overlays a `[profiles.<name>]` config table,
`--log <directive>` sets the file-log filter (e.g. `owt=debug`), and `--debug` shows the
FPS / frame-time / WS-lag overlay. `--version` and `--help` work.

Config is loaded (figment) as defaults -> `~/.config/owt/config.toml`
(`$XDG_CONFIG_HOME/owt/config.toml` if set) -> `OWT__*` env -> CLI flags. The default
server URL is `http://127.0.0.1:8080`. Full schema and precedence live in
[tui-client](../design/tui-client.md).

**With no server running** (the normal case today) the TUI still starts: the status bar
shows `reconnecting`/`offline`, and each screen load surfaces a `Failed` toast. This
exercises the SDK's live REST/WS error and connection-health paths end to end.

## The owtd server

```bash
cargo run -p owtd -- --help
cargo run -p owtd -- serve --roles api    # supervised runtime; Ctrl-C to drain
cargo run -p owtd -- check-config         # print the merged configuration
```

`serve` and `check-config` run; the remaining subcommands parse but print "not yet
implemented" and exit. Current shape (narrower than the target CLI the design docs
describe):

| Subcommand | Current args | Status |
|---|---|---|
| `serve` | `--roles <csv>` (default `all`), `--config <path>` | works (chassis) |
| `check-config` | `--config <path>` | works |
| `migrate` | none | scaffold |
| `backfill` | `<source>` (positional) | scaffold |
| `reindex` | none | scaffold |
| `replay` | `<subject>` (positional) | scaffold |

`serve --roles <ingest,normalize,index,api,alerts|all>` starts one supervised tokio
worker per role and drains them on SIGINT/SIGTERM (a second Ctrl-C force-quits). The
workers are **placeholders** — they log that they started and idle until shutdown; each
role's real work (store, bus, search, API) lands in a later ADR. Config is the layered
figment merge (defaults → `owt.toml`/`OWT_CONFIG` → `OWT__*` env), overridable with
`--config`.

Where [deployment](deployment.md) and [runbook](runbook.md) show richer invocations
(e.g. `owtd backfill --top 200 --days 30`, `owtd replay --subjects ... --from ...`), those
are the **target** CLI, not what the binary accepts today.

## Developer automation (xtask)

`cargo xtask <compose|dist|fixtures>` is wired (via the `.cargo` alias) but each
subcommand is a scaffold that prints "not yet implemented".

## What works today versus target

| Area | State |
|---|---|
| `owt` TUI (event loop, screens, config, keymap) | works |
| `owt-client` SDK (REST client, WS client, reconnect session) | works |
| Domain types / API DTOs (`owt-domain`, `owt-api-types`) | works |
| CI (`ci.yml`, `security.yml`) | works |
| `owtd` runtime chassis (`serve --roles`, supervision, graceful shutdown, `check-config`) | works |
| `owtd` role work + `migrate` / `backfill` / `reindex` / `replay` | scaffold |
| Persistence / bus / search wiring (Postgres, NATS, Typesense) | scaffold |
| HTTP/WS API, `/v1/*`, health, admin surface | scaffold |
| Docker Compose stack, `.env.example`, migrations | design only ([deployment](deployment.md)) |
| `release.yml` / `deploy.yml`, cargo-dist packaging | design only ([ci-cd-and-release](ci-cd-and-release.md)) |

## Changelog

Notable changes are tracked in the root [CHANGELOG.md](../../CHANGELOG.md) under a running
`[Unreleased]` heading — update it in the same change that adds a user-facing behavior.
The policy is in [ci-cd-and-release § Changelog](ci-cd-and-release.md#changelog).

## Related

- [deployment.md](deployment.md)
- [runbook.md](runbook.md)
- [ci-cd-and-release.md](ci-cd-and-release.md)
- [testing-strategy.md](testing-strategy.md)
- [../architecture/cargo-workspace.md](../architecture/cargo-workspace.md)
- [../design/tui-client.md](../design/tui-client.md)
