//! The ingest envelope — normative wrapper for every payload entering the bus
//! (`docs/architecture/data-model.md` § The ingest envelope).
//!
//! The shared contract between source adapters, the normalizer, replay, and the raw
//! archive. The `payload` is vendor JSON kept **verbatim** — no cleanup before the
//! bus, ever.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::EnvelopeId;

/// Adapter identity — the upstream surface that produced a raw payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Source {
    /// Polymarket Gamma REST.
    Gamma,
    /// Polymarket CLOB REST.
    Clob,
    /// Polymarket Data API.
    DataApi,
    /// Polymarket market/user WebSocket.
    PmWs,
    /// Polymarket real-time data service.
    PmRtds,
    /// Goldsky V2 datasets (v1).
    Goldsky,
    /// RSS feeds.
    Rss,
    /// GDELT (v1+).
    Gdelt,
    /// X API (v2).
    X,
}

/// The normalized ingest envelope. `schema_version` is the envelope schema major
/// (currently [`SCHEMA_VERSION`]); `ingest_version` is the semver of the
/// adapter+normalizer pipeline that produced (or will produce) canonical output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// UUIDv7, minted once at receipt; JetStream dedupe key and archive object name.
    pub envelope_id: EnvelopeId,
    /// Adapter identity.
    pub source: Source,
    /// Source-scoped event kind, dotted (e.g. `market.price_change`, `news.item`).
    pub kind: String,
    /// Upstream event time (best available).
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Our clock at receipt.
    #[serde(with = "time::serde::rfc3339")]
    pub received_at: OffsetDateTime,
    /// Canonical keys known at receipt; may be partial — the normalizer completes them.
    pub entity_keys: BTreeMap<String, String>,
    /// Ordering key (usually `market_id`); maps to the subject token.
    pub partition_key: String,
    /// Semver of the adapter+normalizer pipeline.
    pub ingest_version: String,
    /// Envelope schema major.
    pub schema_version: u32,
    /// Vendor JSON, verbatim.
    pub payload: serde_json::Value,
}

/// Current envelope schema major (`data-model.md` § Versioning).
pub const SCHEMA_VERSION: u32 = 1;
