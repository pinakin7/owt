//! Request/response DTOs. These are the wire projections of `owt-domain` entities —
//! deliberately separate types so the storage model can evolve without breaking the
//! contract. The field set is the minimal-but-sufficient surface the TUI panes render
//! (`docs/product/prd-mvp.md` § Data requirements per pane); it grows alongside
//! `docs/design/query-api.md`.

use owt_domain::entities::{EntityKind, TimelineKind};
use owt_domain::ids::{EntityId, EventId, MarketId, NewsId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Summary projection of a market for list endpoints and event membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    /// Market identifier.
    pub market_id: MarketId,
    /// Human-readable question.
    pub title: String,
}

/// The kind of thing a [`SearchHit`] points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum HitKind {
    /// A market.
    Market,
    /// An event.
    Event,
    /// A news item.
    News,
    /// A dictionary entity (topic lookup).
    Entity,
}

/// A single result row from `GET /v1/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// What kind of entity this hit is.
    pub kind: HitKind,
    /// Opaque slug/id used to open the hit.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Optional highlighted snippet.
    pub snippet: Option<String>,
}

/// A single tradable outcome, projected for the market header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeDto {
    /// Outcome label, e.g. `Yes`.
    pub name: String,
    /// Last known price in `[0, 1]`.
    pub price: f64,
}

/// Market-detail state header (`GET /v1/markets/{slug}`): the top strip of the
/// market screen plus the outcomes and event context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDetail {
    /// Market identifier.
    pub market_id: MarketId,
    /// Grouping event.
    pub event_id: EventId,
    /// Human-readable question.
    pub title: String,
    /// Structured outcomes.
    pub outcomes: Vec<OutcomeDto>,
    /// Last trade price in `[0, 1]`.
    pub last: f64,
    /// Best bid in `[0, 1]`.
    pub bid: f64,
    /// Best ask in `[0, 1]`.
    pub ask: f64,
    /// Bid/ask spread in `[0, 1]`.
    pub spread: f64,
    /// Trailing 24h volume, quote currency.
    pub vol_24h: f64,
    /// Human-readable liquidity band (e.g. `high`).
    pub liquidity: String,
    /// Free-form tags.
    pub tags: Vec<String>,
}

/// A single price level in a [`BookDto`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    /// Level price in `[0, 1]`.
    pub price: f64,
    /// Resting size.
    pub size: f64,
}

/// Top-of-book snapshot (`…/book` + WS `market:{slug}:book`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDto {
    /// Bids, best first.
    pub bids: Vec<BookLevel>,
    /// Asks, best first.
    pub asks: Vec<BookLevel>,
    /// Book midpoint in `[0, 1]`.
    pub mid: f64,
}

/// Aggressor side of a [`TradeDto`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Side {
    /// Buy-side aggressor.
    Buy,
    /// Sell-side aggressor.
    Sell,
}

/// A trade-tape row (`…/trades` + WS `market:{slug}:trades`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDto {
    /// Execution time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Aggressor side.
    pub side: Side,
    /// Trade price in `[0, 1]`.
    pub price: f64,
    /// Trade size.
    pub size: f64,
}

/// A linked-news row (`…/news`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsDto {
    /// Stable news id.
    pub news_id: NewsId,
    /// Publisher name.
    pub publisher: String,
    /// Headline.
    pub headline: String,
    /// Canonical URL.
    pub url: String,
    /// Publication time.
    #[serde(with = "time::serde::rfc3339")]
    pub published_at: OffsetDateTime,
}

/// Price context attached to a [`TimelineEntry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericDeltaDto {
    /// Delta in basis points.
    pub delta_bp: i64,
    /// Window width in seconds.
    pub window_s: u64,
}

/// One row of an event timeline (`/v1/events/{slug}/timeline`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Item time.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Item kind (mirrors `owt_domain::entities::TimelineKind`).
    pub kind: TimelineKind,
    /// Short title.
    pub title: String,
    /// Optional price context.
    pub numeric_delta: Option<NumericDeltaDto>,
    /// Correlation confidence in `[0, 1]`.
    pub confidence: f64,
}

/// Event workspace projection (`GET /v1/events/{slug}`): member markets plus the
/// merged timeline strip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetail {
    /// Event identifier.
    pub event_id: EventId,
    /// Human-readable title.
    pub title: String,
    /// Member markets.
    pub markets: Vec<MarketSummary>,
    /// Merged timeline, newest first.
    pub timeline: Vec<TimelineEntry>,
}

/// A single aggregated-odds row on the topic page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OddsRow {
    /// Facet label (e.g. a candidate or outcome).
    pub label: String,
    /// Implied probability in `[0, 1]`.
    pub probability: f64,
    /// Backing market to open.
    pub market_id: MarketId,
}

/// A related-entity pointer on the topic page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    /// Related entity id.
    pub entity_id: EntityId,
    /// Display name.
    pub name: String,
}

/// Topic-lookup composite (`GET /v1/entities/{id}?include=…`): entity context,
/// aggregated odds, and linked news in one read (ADR-0011).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityView {
    /// Entity identifier.
    pub entity_id: EntityId,
    /// Entity kind (mirrors `owt_domain::entities::EntityKind`).
    pub kind: EntityKind,
    /// Display name.
    pub name: String,
    /// Human-curated context blurb (never generated; ADR-0011).
    pub description: Option<String>,
    /// Aggregated odds board.
    pub odds: Vec<OddsRow>,
    /// Linked news.
    pub news: Vec<NewsDto>,
    /// Related entities for navigation.
    pub related: Vec<RelatedEntity>,
}

/// One compact live row in a watchlist (`GET /v1/watchlists/{id}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistRow {
    /// Backing market.
    pub market_id: MarketId,
    /// Display title.
    pub title: String,
    /// Last price in `[0, 1]`.
    pub price: f64,
    /// 24h change in `[0, 1]` price terms (signed).
    pub chg_24h: f64,
    /// Bid/ask spread in `[0, 1]`.
    pub spread: f64,
    /// Trailing 24h volume, quote currency.
    pub volume: f64,
}

/// A named watchlist with its live rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistView {
    /// Watchlist name.
    pub name: String,
    /// Compact rows.
    pub rows: Vec<WatchlistRow>,
}
