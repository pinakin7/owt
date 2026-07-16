//! `owt-runtime` — shared bootstrap for every binary.
//!
//! Layered config (figment: defaults → TOML → `OWT__*` env → CLI), tracing/OTel/
//! Prometheus initialization, graceful shutdown, and task supervision
//! (`docs/ops/observability.md`, `docs/ops/deployment.md`).

pub mod config;
pub mod shutdown;
pub mod supervisor;
pub mod telemetry;
