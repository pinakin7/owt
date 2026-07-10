//! `owt-testkit` — shared test infrastructure (`docs/ops/testing-strategy.md`).
//!
//! The recorded-fixture harness, wiremock helpers for upstream HTTP, a synthetic feed
//! generator, and golden-file utilities. Consumed only as a dev-dependency; no
//! production crate depends on it. Upstream drift is meant to be *visible* in fixture
//! diffs.

/// Recorded-fixture loading and replay.
pub mod fixtures {}

/// wiremock server builders preloaded with recorded upstream responses.
pub mod mock {}

/// Synthetic feed generator for deterministic pipeline tests.
pub mod synthetic {}

/// Golden-file assertion helpers.
pub mod golden {}
