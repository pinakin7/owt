# Changelog

All notable changes to owt are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). owt is
pre-1.0: minor releases may include breaking changes, patch releases never do (see
[docs/ops/ci-cd-and-release.md](docs/ops/ci-cd-and-release.md)). There are no tagged
releases yet — everything below is unreleased and the workspace is at `0.1.0`.

## [Unreleased]

### Added

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

- The `owtd` server is still a scaffold: its subcommands parse but print "not yet
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
