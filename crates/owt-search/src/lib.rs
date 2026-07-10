//! `owt-search` — the Typesense integration (ADR-0006, `docs/design/search.md`).
//!
//! Collection schemas expressed as code, a thin REST client, the indexer consumer
//! that projects `canon.v1.*` into search documents, and the alias-swap rebuild job
//! for zero-downtime reindexing.

/// Typesense collection schemas, declared as code.
pub mod collections {}

/// Thin Typesense REST client.
pub mod client {}

/// Bus consumer that denormalizes canonical records into search documents.
pub mod indexer {}

/// Alias-swap rebuild job for zero-downtime full reindexes.
pub mod rebuild {}
