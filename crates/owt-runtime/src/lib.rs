//! `owt-runtime` — shared bootstrap for every binary.
//!
//! Layered config (figment: defaults → TOML → `OWT__*` env → CLI), tracing/OTel/
//! Prometheus initialization, graceful shutdown, and task supervision
//! (`docs/ops/observability.md`, `docs/ops/deployment.md`).

/// Layered configuration loading (figment): defaults → TOML file → `OWT__*`
/// environment → CLI overrides, merged in that order of increasing precedence.
/// Concrete config structs live with their owning modules; this provides the merge
/// machinery.
pub mod config {}

/// Telemetry initialization: structured logs, OTel traces, Prometheus metrics. Wires
/// `tracing-subscriber` with an env-filter and JSON output and registers the
/// Prometheus recorder. Called once at process start.
pub mod telemetry {}

/// Graceful shutdown coordination — a broadcast signal driven by SIGINT/SIGTERM that
/// supervised tasks await to drain in-flight work before exit.
pub mod shutdown {}

/// Async task supervision — spawns and restarts long-lived worker tasks with backoff,
/// surfacing panics as structured errors rather than silent aborts.
pub mod supervisor {}
