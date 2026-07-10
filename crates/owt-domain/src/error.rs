//! Error taxonomy. Library crates surface typed errors via `thiserror`; binaries use
//! `anyhow` at the composition root (ADR-0001).

use thiserror::Error;

/// Crate result alias defaulting to [`DomainError`].
pub type Result<T, E = DomainError> = std::result::Result<T, E>;

/// Errors originating in the pure domain layer. `#[non_exhaustive]` so new variants
/// are additive within a schema version (`data-model.md` § Versioning).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DomainError {
    /// An identifier did not match its grammar.
    #[error("invalid identifier: {0}")]
    InvalidId(String),
    /// The envelope `schema_version` is not supported by this build.
    #[error("unsupported envelope schema version {found} (expected {expected})")]
    UnsupportedSchemaVersion {
        /// The version seen on the wire.
        found: u32,
        /// The version this build understands.
        expected: u32,
    },
    /// (De)serialization of a canonical record or payload failed.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
