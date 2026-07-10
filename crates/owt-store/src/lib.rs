//! `owt-store` — the Postgres/Timescale persistence layer (ADR-0004,
//! `docs/design/storage.md`).
//!
//! sqlx repositories with compile-time-checked SQL, idempotent upserts, the checkpoint
//! store, embedded migrations, and continuous-aggregate (candle) management. Facts are
//! append-only; snapshots are mutable with `updated_at`.

/// Repository types — one per aggregate root (markets, events, ticks, news, …).
pub mod repositories {}

/// Durable ingest checkpoints backing the `owt-ingest-core::Checkpoint` trait.
pub mod checkpoints {}

/// Embedded, ordered schema migrations applied by `owtd migrate`.
pub mod migrations {}

/// Timescale continuous aggregates — candle rollups and their refresh policies.
pub mod aggregates {}
