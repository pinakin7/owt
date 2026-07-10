//! `owt-client` — the typed Rust SDK over the public API (`docs/design/query-api.md`).
//!
//! A `reqwest` REST client plus a `tokio-tungstenite` WebSocket client with the
//! reconnect/resubscribe state machine. Depends only on `owt-api-types`; the TUI and
//! any scripts get a fully typed client for free.

/// REST client over the `/v1` endpoints.
pub mod http {}

/// WebSocket client speaking the multiplexed frame protocol.
pub mod ws {}

/// The reconnect + resubscribe state machine with backoff.
pub mod reconnect {}
