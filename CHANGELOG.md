# Changelog

All notable changes to owt are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). owt is
pre-1.0: minor releases may include breaking changes, patch releases never do (see
[docs/ops/ci-cd-and-release.md](docs/ops/ci-cd-and-release.md)). There are no tagged
releases yet — everything below is unreleased and the workspace is at `0.1.0`.

## [Unreleased]

### Added

- `owtd`: `serve --roles <ingest,normalize,index,api,alerts|all>` now runs a real
  supervised runtime (ADR-0003) — one gracefully-shutdown tokio worker per selected
  role, drained on SIGINT/SIGTERM (double-Ctrl-C force-quits) within a bounded 30s
  window. Role workers are placeholders until each module's ADR lands. `check-config`
  validates and prints the merged configuration.
- `owt-runtime`: the shared server chassis behind `owtd` — signal-driven graceful
  `shutdown`, a restart-with-backoff task `supervisor` (panics surface as structured
  logs, not silent aborts), JSON `telemetry` init (`OWT_LOG`/`RUST_LOG`), and the
  layered `config` loader (defaults → TOML → `OWT__*` env) with a minimal
  `RuntimeConfig`.
- `owt-client`: the typed REST client now performs real requests against the `/v1` API —
  `search`, market detail / book / trades / news, event, entity, and watchlist —
  replacing the previous not-yet-implemented stubs.
- `owt-client`: a live WebSocket client (`WsConnection`) plus a `WsSession` reconnect and
  resubscribe driver built on the existing `Backoff`/`ConnState` state machine.
- `owt-client`: structured error variants `ClientError::Api(ProblemDetails)` and
  `ClientError::Status { status, body }` for non-2xx responses.
- `owt` (TUI): screens load from the live client (market detail fans out to four
  endpoints concurrently), the status bar reflects real WebSocket connection health, and
  opening a market subscribes to its live topics.
- Tests: REST contract tests against `wiremock` and WebSocket connect/subscribe/update
  plus reconnect against an in-test server.
- Docs: a developer/operator bootstrap
  ([docs/ops/getting-started.md](docs/ops/getting-started.md)) and this changelog.

### Notes

- The `owtd` server now runs as a modular-monolith chassis (ADR-0003): `serve`
  supervises one placeholder worker per role and `check-config` works, but the roles do
  no real work yet (ingest/normalize/store/search/API and the NATS bus land in later
  ADRs), and `migrate`, `backfill`, `reindex`, and `replay` still print "not yet
  implemented". See [docs/ops/getting-started.md](docs/ops/getting-started.md) for what
  runs today.

<!--
Release process: at each release, `release.yml` cuts this [Unreleased] section into a
dated `## [X.Y.Z] - YYYY-MM-DD` heading and starts a fresh, empty [Unreleased]. The cut
entries become the source material for the GitHub Release notes rendered from the
template in docs/ops/ci-cd-and-release.md. Keep entries grouped under
Added / Changed / Deprecated / Removed / Fixed / Security.
-->

[Unreleased]: https://github.com/pinakin/owt/commits/main
