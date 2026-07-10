//! `owt-api` — the axum HTTP + WebSocket server (ADR-0010, `docs/design/query-api.md`).
//!
//! REST `/v1` query services backed by `owt-store` and `owt-search`, one multiplexed
//! WebSocket per client, a bus fanout subscriber with server-side conflation
//! (≤10 Hz), and OpenAPI generation via utoipa. There is no separate realtime
//! gateway — fanout is a module of this service (`docs/architecture/system-overview.md`).

/// REST route handlers and the router assembly.
pub mod routes {}

/// The per-client WebSocket session: subscription state and frame handling.
pub mod ws {}

/// Bus fanout subscriber with server-side conflation.
pub mod fanout {}

/// OpenAPI document generation (utoipa).
pub mod openapi {}
