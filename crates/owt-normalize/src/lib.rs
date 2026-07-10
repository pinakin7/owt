//! `owt-normalize` — raw payloads to canonical records (`docs/design/normalization.md`).
//!
//! Source→canonical mapping, cross-source dedupe, entity resolution, and the timeline
//! builder with price/news correlation. Versioned and deterministic: same envelope +
//! same `ingest_version` ⇒ byte-identical canonical record (`data-model.md`
//! § Versioning). Imports source *payload types* only, never source *clients*
//! (layering rule 4).

/// Source→canonical field mapping, per surface.
pub mod mapping {}

/// Cross-source deduplication (URL canonicalization, content hashing).
pub mod dedupe {}

/// Entity resolution — links records to dictionary entities via the alias index.
pub mod entity_resolution {}

/// Timeline builder — correlates price moves with news/comments into timeline items.
pub mod timeline {}
