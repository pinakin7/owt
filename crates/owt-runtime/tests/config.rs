//! Contract tests for the layered config loader (`docs/architecture/cargo-workspace.md`
//! § Workspace policy: defaults → TOML → `OWT__*` env → CLI). Uses `figment::Jail` for
//! isolated per-test env vars and working directory.

// `figment::Jail::expect_with` requires a closure returning `Result<(), figment::Error>`,
// and `figment::Error` is large — this is figment's API surface, not ours to box.
#![allow(clippy::result_large_err)]

use figment::Jail;
use owt_runtime::config::{LogFormat, RuntimeConfig, load};

#[test]
fn defaults_load_without_file() {
    Jail::expect_with(|_jail| {
        let cfg = load(None).expect("defaults load");
        let expected = RuntimeConfig::default();
        assert_eq!(cfg.nats_url, expected.nats_url);
        assert_eq!(cfg.api_bind, expected.api_bind);
        assert_eq!(cfg.log.level, expected.log.level);
        assert_eq!(cfg.log.format, LogFormat::Json);
        Ok(())
    });
}

#[test]
fn toml_file_overrides_defaults() {
    Jail::expect_with(|jail| {
        jail.create_file("owt.toml", "nats_url = \"nats://from-toml:4222\"\n")?;
        let cfg = load(None).expect("toml loads");
        assert_eq!(cfg.nats_url, "nats://from-toml:4222");
        // Untouched keys keep their defaults.
        assert_eq!(cfg.api_bind, "127.0.0.1:8080");
        Ok(())
    });
}

#[test]
fn env_overrides_toml() {
    Jail::expect_with(|jail| {
        jail.create_file("owt.toml", "nats_url = \"nats://from-toml:4222\"\n")?;
        jail.set_env("OWT__NATS_URL", "nats://from-env:4222");
        jail.set_env("OWT__LOG__LEVEL", "debug");

        let cfg = load(None).expect("env loads");
        // env beats TOML beats defaults.
        assert_eq!(cfg.nats_url, "nats://from-env:4222");
        // Nested key via the `__` separator.
        assert_eq!(cfg.log.level, "debug");
        Ok(())
    });
}

#[test]
fn unknown_key_is_ignored() {
    Jail::expect_with(|jail| {
        jail.set_env("OWT__DOES_NOT_EXIST", "whatever");
        let cfg = load(None).expect("unknown keys are ignored, not fatal");
        assert_eq!(cfg.nats_url, RuntimeConfig::default().nats_url);
        Ok(())
    });
}
