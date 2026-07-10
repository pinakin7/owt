//! `owt-ingest-core` — the contract source adapters implement.
//!
//! `BackfillSource`/`StreamSource` traits, per-(source, endpoint-class) token buckets
//! (`governor`), retry/backoff policies, a checkpoint trait, the raw-archive hook, and
//! the DLQ publisher (`docs/design/ingestion.md`). Source adapters depend on this
//! crate and never on `owt-store` — everything flows through the bus (layering rule 3).

use async_trait::async_trait;

// The ingest envelope is defined once in owt-domain; re-exported here so adapters
// depend only on owt-ingest-core (cargo-workspace.md dependency diagram).
pub use owt_domain::envelope::{Envelope, Source};

/// Errors surfaced by ingest adapters and the core machinery.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IngestError {
    /// An upstream request failed after exhausting the retry policy.
    #[error("upstream request failed: {0}")]
    Upstream(String),
    /// A checkpoint could not be loaded or persisted.
    #[error("checkpoint error: {0}")]
    Checkpoint(String),
}

/// A bounded, replayable historical backfill of an upstream surface.
#[async_trait]
pub trait BackfillSource: Send + Sync {
    /// Stable identity of the upstream this adapter serves.
    fn source(&self) -> Source;

    /// Run one backfill pass, publishing envelopes via `sink`.
    async fn backfill(&self, sink: &dyn EnvelopeSink) -> Result<(), IngestError>;
}

/// A long-lived streaming connection to an upstream surface.
#[async_trait]
pub trait StreamSource: Send + Sync {
    /// Stable identity of the upstream this adapter serves.
    fn source(&self) -> Source;

    /// Hold the connection, publishing envelopes via `sink` until cancelled.
    async fn stream(&self, sink: &dyn EnvelopeSink) -> Result<(), IngestError>;
}

/// Where an adapter hands finished envelopes — normally a bus publisher, but abstract
/// so the testkit can capture them.
#[async_trait]
pub trait EnvelopeSink: Send + Sync {
    /// Publish one envelope.
    async fn publish(&self, envelope: Envelope) -> Result<(), IngestError>;
}

/// Durable per-source ingest position for resumable backfills and streams.
#[async_trait]
pub trait Checkpoint: Send + Sync {
    /// Load the last persisted position for a logical stream key.
    async fn load(&self, key: &str) -> Result<Option<String>, IngestError>;
    /// Persist a new position for a logical stream key.
    async fn store(&self, key: &str, position: &str) -> Result<(), IngestError>;
}

/// Rate limiting: per-(source, endpoint-class) token buckets built on `governor` —
/// token-bucket construction and the endpoint-class taxonomy each adapter maps its
/// requests onto.
pub mod rate {}

/// Retry and backoff policies shared across adapters — exponential backoff with jitter
/// and per-error-class retry classification.
pub mod retry {}
