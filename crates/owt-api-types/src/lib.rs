//! `owt-api-types` — the public API contract (ADR-0010, `docs/design/query-api.md`).
//!
//! Request/response DTOs, cursor pagination, RFC 7807 error bodies with a stable
//! error-code enum, and the WebSocket frame protocol. Depends only on `owt-domain`;
//! shared by the server (`owt-api`) and the SDK (`owt-client`) so there is exactly
//! one definition of the wire format.

pub mod dto;
pub mod error;
pub mod pagination;
pub mod ws;

/// The API version prefix all routes live under (`/v1`).
pub const API_VERSION: &str = "v1";
