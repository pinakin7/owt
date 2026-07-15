//! File logging (`docs/design/tui-client.md` § Terminal hygiene).
//!
//! Logs go to `$XDG_STATE_HOME/owt/owt.log` (falling back to `~/.local/state` then the
//! temp dir) and **never** to stdout/stderr — writing to the terminal while the UI owns
//! it would corrupt the display. Level comes from `--log`, else `RUST_LOG`, else
//! `owt=info`.

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;

use crate::cli::Args;

/// Initialize the file logger. Idempotent-safe: a second call (e.g. in tests) is a
/// no-op rather than a panic.
pub fn init(args: &Args) -> Result<()> {
    let path = log_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("creating log dir {}", dir.display()))?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening log file {}", path.display()))?;

    let filter = match &args.log {
        Some(directive) => EnvFilter::new(directive.clone()),
        None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("owt=info")),
    };

    // `try_init` returns Err if a subscriber is already set (harmless for our purposes).
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(Mutex::new(file))
        .with_ansi(false)
        .with_target(true)
        .try_init();

    Ok(())
}

/// The log file path with XDG fallbacks.
pub fn log_path() -> PathBuf {
    if let Some(state) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(state).join("owt").join("owt.log");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("owt")
            .join("owt.log");
    }
    std::env::temp_dir().join("owt").join("owt.log")
}
