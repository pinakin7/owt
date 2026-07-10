//! `owt-source-polymarket` — Polymarket upstream adapters (`docs/design/ingestion.md`,
//! `docs/architecture/data-sources.md`).
//!
//! One crate, a module per surface. Owns source-native payload types (which
//! `owt-normalize` may import — layering rule 4 — but never these clients). Emits
//! ingest envelopes through the `owt-ingest-core` traits; never touches `owt-store`.

/// Gamma REST client — markets, events, tags metadata.
pub mod gamma {}

/// CLOB REST client — order books, prices, token metadata.
pub mod clob {}

/// Data API client — the public trade tape.
pub mod data_api {}

/// `market`-channel WebSocket client (`sports`, `user` channels: v1).
pub mod ws {}

/// Real-time data service (RTDS) client (v1).
pub mod rtds {}
