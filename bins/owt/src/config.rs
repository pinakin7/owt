//! Layered configuration loading (`docs/design/tui-client.md` § Configuration).
//!
//! Precedence, lowest to highest: built-in defaults → `$XDG_CONFIG_HOME/owt/config.toml`
//! → `OWT__*` environment → CLI flags. `--profile <name>` overlays the file's
//! `[profiles.<name>]` table (only the keys it sets) between the file and env layers.
//! The schema itself lives in `owt-client` so scripts share it.

use std::path::PathBuf;

use anyhow::{Context, Result};
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use owt_client::config::EffectiveConfig;

use crate::cli::Args;

/// Resolve the effective config from all layers.
pub fn load(args: &Args) -> Result<EffectiveConfig> {
    let path = config_path();

    let mut fig = Figment::from(Serialized::defaults(EffectiveConfig::default()));

    if let Some(path) = &path {
        if path.exists() {
            let file = Figment::from(Toml::file(path));
            // Default profile: the top-level [server]/[ui]/[keys] tables.
            fig = fig.merge(Toml::file(path));
            // Optional profile overlay: only the keys present under [profiles.<name>].
            if let Some(name) = &args.profile {
                match file.find_value(&format!("profiles.{name}")) {
                    Ok(value) => fig = fig.merge(Serialized::defaults(value)),
                    Err(_) => anyhow::bail!("profile `{name}` not found in {}", path.display()),
                }
            }
        } else if let Some(name) = &args.profile {
            anyhow::bail!(
                "--profile {name} given but no config file at {}",
                path.display()
            );
        }
    }

    fig = fig.merge(Env::prefixed("OWT__").split("__"));

    let mut cfg: EffectiveConfig = fig
        .extract()
        .context("failed to parse the merged configuration")?;

    // Highest-precedence flag overrides.
    if let Some(server) = &args.server {
        cfg.server.url = server.clone();
    }

    Ok(cfg)
}

/// The config file path: `$XDG_CONFIG_HOME/owt/config.toml`, else
/// `$HOME/.config/owt/config.toml`. `None` if neither base is set.
pub fn config_path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("owt").join("config.toml"));
    }
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("owt")
            .join("config.toml")
    })
}
