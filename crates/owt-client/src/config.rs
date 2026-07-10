//! The client configuration **schema** (`docs/design/tui-client.md` § Configuration).
//!
//! The schema lives here — not in the TUI binary — so any script or alternative
//! client built on the SDK shares one definition. The *loading* machinery (figment:
//! defaults → TOML → `OWT__*` env → CLI flags, plus `--profile` selection) lives in
//! the consuming binary; these structs are just the merge target, with `Default`
//! providing the built-in defaults layer.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The fully-merged effective configuration the client runs with.
///
/// Unknown fields are tolerated (not `deny_unknown_fields`) so a config file may also
/// carry a `[profiles.<name>]` table that the loader overlays on `--profile`; the
/// `profiles` key is not a field here and is simply ignored during extraction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectiveConfig {
    /// Server connection settings.
    pub server: ServerConfig,
    /// UI preferences.
    pub ui: UiConfig,
    /// Keybinding overrides (action → key spec); empty means "use the built-in map".
    pub keys: KeysConfig,
}

/// How to reach the `owtd` server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfig {
    /// Base URL of the server's REST/WS API.
    pub url: String,
    /// Optional bearer token for a remote self-host.
    pub bearer_token: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8080".to_owned(),
            bearer_token: None,
        }
    }
}

/// Terminal color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Dark background (default).
    #[default]
    Dark,
    /// Light background.
    Light,
}

/// UI preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiConfig {
    /// Color theme.
    pub theme: Theme,
    /// Render/refresh tick period in milliseconds.
    pub tick_ms: u64,
    /// Ring-buffer cap for the trade tape pane.
    pub tape_rows: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            tick_ms: 250,
            tape_rows: 1_000,
        }
    }
}

/// Keybinding overrides: action name → key spec (e.g. `palette = ":"`). Serialized
/// transparently so the `[keys]` TOML table maps straight in. Empty by default; the
/// TUI merges these over its built-in default map.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeysConfig(pub BTreeMap<String, String>);

impl KeysConfig {
    /// Look up an override for an action name.
    pub fn get(&self, action: &str) -> Option<&str> {
        self.0.get(action).map(String::as_str)
    }

    /// Whether any overrides are present.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_lld() {
        let c = EffectiveConfig::default();
        assert_eq!(c.server.url, "http://127.0.0.1:8080");
        assert_eq!(c.server.bearer_token, None);
        assert_eq!(c.ui.theme, Theme::Dark);
        assert_eq!(c.ui.tick_ms, 250);
        assert_eq!(c.ui.tape_rows, 1_000);
        assert!(c.keys.is_empty());
    }

    #[test]
    fn keys_lookup() {
        let mut map = BTreeMap::new();
        map.insert("palette".to_owned(), ":".to_owned());
        let keys = KeysConfig(map);
        assert_eq!(keys.get("palette"), Some(":"));
        assert_eq!(keys.get("missing"), None);
        assert!(!keys.is_empty());
    }

    #[test]
    fn partial_document_fills_defaults() {
        // serde(default) means an override document only needs the keys it changes;
        // everything else keeps the built-in default. (The binary feeds figment TOML
        // through this same serde contract.)
        let cfg: EffectiveConfig = serde_json::from_value(serde_json::json!({
            "server": { "url": "https://owt.example.dev" },
            "ui": { "theme": "light" }
        }))
        .expect("valid config");
        assert_eq!(cfg.server.url, "https://owt.example.dev");
        assert_eq!(cfg.ui.theme, Theme::Light);
        assert_eq!(cfg.ui.tick_ms, 250, "unset fields keep their default");
    }

    #[test]
    fn unknown_top_level_keys_are_tolerated() {
        // A config file may carry a [profiles.*] table; extraction must ignore it.
        let cfg: EffectiveConfig = serde_json::from_value(serde_json::json!({
            "server": { "url": "http://localhost:9000" },
            "profiles": { "staging": { "server": { "url": "https://staging" } } }
        }))
        .expect("profiles table is ignored, not rejected");
        assert_eq!(cfg.server.url, "http://localhost:9000");
    }
}
