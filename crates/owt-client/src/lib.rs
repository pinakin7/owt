//! `owt-client` — the typed Rust SDK over the public API (`docs/design/query-api.md`).
//!
//! A `reqwest` REST client plus a `tokio-tungstenite` WebSocket client with the
//! reconnect/resubscribe state machine. Depends only on `owt-api-types`; the TUI and
//! any scripts get a fully typed client for free.
//!
//! Status: the config schema and the reconnect state machine are implemented; the
//! network methods (`http`, `ws`) are typed against the contract but return
//! [`ClientError::Unimplemented`] until the server ships. The TUI drives its screens
//! from a fixture seam in the meantime (ADR-0002 skeleton pass).

pub mod config;
pub mod error;
pub mod http;
pub mod reconnect;
pub mod ws;

pub use config::{EffectiveConfig, KeysConfig, ServerConfig, Theme, UiConfig};
pub use error::ClientError;
pub use http::RestClient;
pub use reconnect::{Backoff, ConnState};
pub use ws::WsClient;
