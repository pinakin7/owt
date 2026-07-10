//! `owt-bus` — the NATS JetStream wrapper (ADR-0005, `docs/design/realtime-bus.md`).
//!
//! Subject constants, stream/consumer configs expressed as code, `Nats-Msg-Id`
//! dedupe publishing (keyed on `envelope_id`), and replay + rehydration helpers. The
//! only path source adapters have to downstream consumers (layering rule 3).

/// Subject name constants and builders — the `raw.*` and `canon.v1.*` hierarchies.
/// The subject grammar is `raw.{source}.{kind}.{token}` for ingest and
/// `canon.v1.{entity}.{id}.{facet}` for canonical output.
pub mod subjects {
    /// Root of the raw ingest subject space.
    pub const RAW_ROOT: &str = "raw";
    /// Root of the canonical subject space (versioned).
    pub const CANON_ROOT: &str = "canon.v1";
}

/// Stream and consumer configuration, declared as code and reconciled on boot.
pub mod streams {}

/// Dedupe-aware publishing (`Nats-Msg-Id` = `envelope_id`).
pub mod publish {}

/// Replay and rehydration helpers for reprocessing `raw.*` ranges.
pub mod replay {}
