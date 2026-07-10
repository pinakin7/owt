//! `owt-source-news` — news upstream adapters (`docs/architecture/data-sources.md`).
//!
//! RSS polling over a registered feed set, plus the GDELT client (news APIs land in
//! v1+). Emits envelopes through `owt-ingest-core`; owns its source-native payload
//! types.

/// RSS poller — fetches and diffs registered feeds on an interval.
pub mod rss {}

/// The feed registry — the curated set of feeds and their poll cadence.
pub mod feed_registry {}

/// GDELT client (v1+).
pub mod gdelt {}
