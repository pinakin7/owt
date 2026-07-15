//! The SDK error type. One taxonomy over REST and WS failures; the TUI maps these to
//! toasts and `ConnState` transitions.

use owt_api_types::error::ProblemDetails;
use thiserror::Error;

/// Everything the SDK can fail with.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ClientError {
    /// The method is defined against the contract but not yet wired to a live server
    /// (ADR-0002 skeleton pass). Carries the method name for diagnostics.
    #[error("owt-client feature not yet implemented: {0}")]
    Unimplemented(&'static str),

    /// A REST request failed at the transport layer or its body could not be decoded.
    #[error("http error: {0}")]
    Http(String),

    /// A non-2xx response carrying an RFC 7807 `application/problem+json` body.
    #[error("api error {}: {}", .0.status, .0.title)]
    Api(ProblemDetails),

    /// A non-2xx response without a parseable problem body.
    #[error("http status {status}: {body}")]
    Status {
        /// The HTTP status code.
        status: u16,
        /// The raw response body (may be empty).
        body: String,
    },

    /// A WebSocket frame could not be sent, received, or (de)serialized.
    #[error("websocket error: {0}")]
    Ws(String),

    /// The configured server URL could not be parsed.
    #[error("invalid server url: {0}")]
    InvalidUrl(String),
}
