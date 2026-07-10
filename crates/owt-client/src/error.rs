//! The SDK error type. One taxonomy over REST and WS failures; the TUI maps these to
//! toasts and `ConnState` transitions.

use thiserror::Error;

/// Everything the SDK can fail with.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ClientError {
    /// The method is defined against the contract but not yet wired to a live server
    /// (ADR-0002 skeleton pass). Carries the method name for diagnostics.
    #[error("owt-client feature not yet implemented: {0}")]
    Unimplemented(&'static str),

    /// A REST request failed (transport, status, or decode).
    #[error("http error: {0}")]
    Http(String),

    /// A WebSocket frame could not be sent, received, or (de)serialized.
    #[error("websocket error: {0}")]
    Ws(String),

    /// The configured server URL could not be parsed.
    #[error("invalid server url: {0}")]
    InvalidUrl(String),
}
