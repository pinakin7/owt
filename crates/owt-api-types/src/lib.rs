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

// The id newtypes and domain enums that appear in DTO signatures are part of the
// public contract, so the contract crate re-exports them. Clients (e.g. the TUI, which
// by ADR-0010 may only depend on this crate + owt-client) construct and match on them
// without taking a direct dependency on owt-domain.
pub use owt_domain::entities::{EntityKind, PriceKind, TimelineKind};
pub use owt_domain::ids;

/// The API version prefix all routes live under (`/v1`).
pub const API_VERSION: &str = "v1";
