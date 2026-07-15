//! `owt-client` — the typed Rust SDK over the public API (`docs/design/query-api.md`).
//!
//! A `reqwest` REST client plus a `tokio-tungstenite` WebSocket client with the
//! reconnect/resubscribe state machine. Depends only on `owt-api-types`; the TUI and
//! any scripts get a fully typed client for free.
//!
//! The REST methods (`http`) and the WS client (`ws`) are wired to a live server over
//! the contract; the reconnect/resubscribe loop lives in `session`. The TUI drives its
//! screens through these (ADR-0002).

pub mod config;
pub mod error;
pub mod http;
pub mod reconnect;
pub mod session;
pub mod ws;

pub use config::{EffectiveConfig, KeysConfig, ServerConfig, Theme, UiConfig};
pub use error::ClientError;
pub use http::RestClient;
pub use reconnect::{Backoff, ConnState};
pub use session::WsSession;
pub use ws::{WsClient, WsConnection};
