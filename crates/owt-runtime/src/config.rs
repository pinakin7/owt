//! Layered configuration loading (`docs/architecture/cargo-workspace.md`
//! § Workspace policy; `docs/ops/deployment.md` § `OWT__*` environment).
//!
//! The merge order, in increasing precedence, is: built-in defaults → a TOML file →
//! `OWT__*` environment variables (`__` nests keys) → CLI overrides. This crate owns
//! only the merge *machinery* plus a minimal shared [`RuntimeConfig`]; concrete
//! per-module config structs live with their owning modules and are layered in the
//! same [`figment`] the same way.

use std::path::Path;

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

/// The minimal configuration shared across every `owtd` role.
///
/// `nats_url` and `api_bind` are **reserved placeholders**: they are carried here so
/// the shape and precedence are exercised now, but they migrate into `owt-bus` /
/// `owt-api` config structs when those modules gain real config (ADR-0005 / query-api).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RuntimeConfig {
    /// Logging configuration.
    pub log: LogConfig,
    /// NATS server URL (reserved; consumed by `owt-bus` in ADR-0005).
    pub nats_url: String,
    /// API listen address (reserved; consumed by `owt-api`).
    pub api_bind: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            log: LogConfig::default(),
            nats_url: "nats://127.0.0.1:4222".to_owned(),
            api_bind: "127.0.0.1:8080".to_owned(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LogConfig {
    /// Reserved filter directive; telemetry currently reads `OWT_LOG`/`RUST_LOG`
    /// directly at process start (before config load), so this is not yet consumed.
    pub level: String,
    /// Output format.
    pub format: LogFormat,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_owned(),
            format: LogFormat::Json,
        }
    }
}

/// Log output format.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// One JSON object per line (default; machine-parseable).
    #[default]
    Json,
    /// Human-readable text.
    Text,
}

/// A configuration load or merge error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The merged configuration was invalid or a source failed to parse.
    /// Boxed because `figment::Error` is large (`clippy::result_large_err`).
    #[error("configuration error: {0}")]
    Figment(#[source] Box<figment::Error>),
}

/// Build the layered figment: defaults → TOML file → `OWT__*` env.
///
/// The TOML path is taken from `config_path`, else the `OWT_CONFIG` env var, else
/// `owt.toml` in the working directory. A missing file is not an error — it simply
/// contributes nothing. CLI overrides, where a subcommand has them, are appended by
/// the caller as a final highest-precedence `.merge(Serialized::defaults(cli))` layer.
pub fn figment(config_path: Option<&Path>) -> Figment {
    let path = config_path
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("OWT_CONFIG").map(Into::into))
        .unwrap_or_else(|| "owt.toml".into());

    Figment::from(Serialized::defaults(RuntimeConfig::default()))
        .merge(Toml::file(path))
        .merge(Env::prefixed("OWT__").split("__"))
}

/// Merge the layered sources and extract the shared [`RuntimeConfig`].
pub fn load(config_path: Option<&Path>) -> Result<RuntimeConfig, ConfigError> {
    figment(config_path)
        .extract()
        .map_err(|e| ConfigError::Figment(Box::new(e)))
}
