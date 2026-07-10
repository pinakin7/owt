//! Canonical entities — `docs/architecture/data-model.md` § Canonical entities.
//!
//! Facts (ticks, trades, fills, news, comments, timeline items) are append-only.
//! Snapshots (`Market`, `Event`, `market_state`) are mutable with `updated_at`.
//!
//! Skeleton note: fields here are a representative subset. The full field set,
//! validation, and `From<source payload>` conversions land in the data-model
//! implementation phase; these types fix the names and the crate's public surface.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{
    CommentId, ConditionId, EntityId, EventId, FillId, MarketId, NewsId, QuestionId, TimelineId,
    TokenId, TradeId,
};

/// Provenance string of the form `{source}:{kind}:{native_id}`.
pub type SourceRef = String;

/// A single tradable outcome (name + token + price together — the normalizer zips
/// Gamma's parallel `outcomes`/`outcomePrices` arrays).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Outcome label, e.g. `Yes`.
    pub name: String,
    /// The CLOB token id backing this outcome.
    pub token_id: TokenId,
    /// Last known price in `[0, 1]`.
    pub price: f64,
}

/// Market snapshot. Live top-of-book state lives in `market_state`, not here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Polymarket-native market id.
    pub market_id: MarketId,
    /// Grouping event.
    pub event_id: EventId,
    /// On-chain condition id.
    pub condition_id: Option<ConditionId>,
    /// On-chain question id.
    pub question_id: Option<QuestionId>,
    /// Human-readable question.
    pub title: String,
    /// Structured outcomes.
    pub outcomes: Vec<Outcome>,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
    /// Last mutation time.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Event snapshot. One event groups one or more markets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Polymarket-native event id.
    pub event_id: EventId,
    /// Human-readable title.
    pub title: String,
    /// Markets grouped under this event.
    pub market_ids: Vec<MarketId>,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
    /// Last mutation time.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// The kind of a [`PricePoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PriceKind {
    /// Executed trade price.
    Trade,
    /// Book midpoint.
    Midpoint,
    /// Best bid.
    BestBid,
    /// Best ask.
    BestAsk,
    /// Backfilled historical point.
    HistoryBackfill,
}

/// A single price observation (fact). Feeds `price_ticks` and the candle aggregates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Token observed.
    pub token_id: TokenId,
    /// Owning market.
    pub market_id: MarketId,
    /// Observation time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Price in `[0, 1]`.
    pub price: f64,
    /// Optional size.
    pub size: Option<f64>,
    /// What produced this point.
    pub kind: PriceKind,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
}

/// A single price level in a [`BookSnapshot`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Level price.
    pub price: f64,
    /// Level size.
    pub size: f64,
}

/// Top-N order-book snapshot (fact, short retention). Full-depth history is
/// deliberately not kept (`docs/design/storage.md`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSnapshot {
    /// Token observed.
    pub token_id: TokenId,
    /// Owning market.
    pub market_id: MarketId,
    /// Snapshot time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Top-N bids.
    pub bids: Vec<PriceLevel>,
    /// Top-N asks.
    pub asks: Vec<PriceLevel>,
    /// Optional sequence number.
    pub seq: Option<u64>,
}

/// Data-API trade tape entry (fact).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Source-native trade id.
    pub trade_id: TradeId,
    /// Owning market.
    pub market_id: MarketId,
    /// Token traded.
    pub token_id: TokenId,
    /// Execution time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Trade price.
    pub price: f64,
    /// Trade size.
    pub size: f64,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
}

/// On-chain order fill from Goldsky (fact, v1). Note `amount_collateral` +
/// `collateral_token` (pUSD era) — not the report's `amount_usdc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFill {
    /// `{tx_hash}:{log_index}`.
    pub fill_id: FillId,
    /// Token filled.
    pub token_id: TokenId,
    /// Owning market.
    pub market_id: MarketId,
    /// Shares transacted.
    pub amount_shares: f64,
    /// Collateral transacted.
    pub amount_collateral: f64,
    /// Collateral token symbol (e.g. `pUSD`).
    pub collateral_token: String,
    /// Block timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub block_timestamp: OffsetDateTime,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
}

/// News item (fact). `sentiment`/`importance` are nullable and null at MVP —
/// dictionary-based linking first, model-assisted enrichment later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    /// `sha256(canonical_url)` hex.
    pub news_id: NewsId,
    /// Publisher name.
    pub publisher: String,
    /// Headline.
    pub headline: String,
    /// Canonical URL.
    pub canonical_url: String,
    /// Publication time.
    #[serde(with = "time::serde::rfc3339")]
    pub published_at: OffsetDateTime,
    /// Linked entities.
    pub entities: Vec<EntityId>,
    /// Linked events.
    pub event_links: Vec<EventId>,
    /// Linked markets.
    pub market_links: Vec<MarketId>,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
}

/// User comment (fact, v1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Source-native comment id.
    pub comment_id: CommentId,
    /// Owning market (if any).
    pub market_id: Option<MarketId>,
    /// Owning event (if any).
    pub event_id: Option<EventId>,
    /// Comment time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Author handle.
    pub author_handle: String,
    /// Comment body.
    pub body: String,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
}

/// The kind of dictionary [`Entity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EntityKind {
    /// A person.
    Person,
    /// An organization.
    Org,
    /// A topic.
    Topic,
    /// A place.
    Place,
}

/// Dictionary entity. `description` is human-curated, never generated (ADR-0011).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// `ent_{kebab-name}`.
    pub entity_id: EntityId,
    /// Entity kind.
    pub kind: EntityKind,
    /// Display name.
    pub name: String,
    /// Human-curated context blurb (nullable).
    pub description: Option<String>,
    /// Alias rows.
    pub aliases: Vec<EntityAlias>,
}

/// A single alias for an [`Entity`]. Every row records provenance for auditability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAlias {
    /// The alias text.
    pub alias: String,
    /// Match confidence in `[0, 1]`.
    pub confidence: f64,
    /// Who or what added this alias.
    pub added_by: String,
}

/// The kind of a [`TimelineItem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TimelineKind {
    /// A significant price move.
    PriceMove,
    /// A news item.
    News,
    /// A burst of trades.
    TradeBurst,
    /// A spike in comments (v1).
    CommentSpike,
    /// Market resolution.
    Resolution,
}

/// Price context attached to a [`TimelineItem`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericDelta {
    /// Price before the correlation window.
    pub price_before: f64,
    /// Price after the correlation window.
    pub price_after: f64,
    /// Delta in basis points.
    pub delta_bp: i64,
    /// Window width in seconds.
    pub window_s: u64,
}

/// Timeline item (fact) — correlates price moves with news/comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItem {
    /// `event:{event_id}:{ts}:{kind}:{hash8}`.
    pub timeline_id: TimelineId,
    /// Owning event.
    pub event_id: EventId,
    /// Item time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Item kind.
    pub kind: TimelineKind,
    /// Short title.
    pub title: String,
    /// Optional price context.
    pub numeric_delta: Option<NumericDelta>,
    /// Correlation confidence in `[0, 1]`.
    pub confidence: f64,
    /// Provenance.
    pub source_refs: Vec<SourceRef>,
}
