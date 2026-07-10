//! RFC 7807 problem-detail error bodies with a stable, versioned error-code enum.

use serde::{Deserialize, Serialize};

/// Stable machine-readable error code. Additive-only; clients switch on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorCode {
    /// The requested resource does not exist.
    NotFound,
    /// The request was malformed.
    BadRequest,
    /// A pagination cursor was invalid or expired.
    InvalidCursor,
    /// The client exceeded its rate budget.
    RateLimited,
    /// An unexpected server-side error.
    Internal,
}

/// RFC 7807 `application/problem+json` body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// Stable error code.
    pub code: ErrorCode,
    /// Short, human-readable summary.
    pub title: String,
    /// HTTP status code mirrored into the body.
    pub status: u16,
    /// Optional human-readable detail.
    pub detail: Option<String>,
}
