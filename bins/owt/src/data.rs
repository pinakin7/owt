//! The data seam.
//!
//! [`fetch`] is the live path: it maps a [`Route`] to the `owt-client` REST calls that
//! back it and returns a [`Loaded`] bundle (the runtime spawns it and feeds the result
//! back as `Msg::Loaded`/`Msg::Failed`). The fixture functions below (`load`, `search`,
//! `market`, …) feed the golden-frame and `update` test harnesses the same shapes
//! without a server, so snapshots stay deterministic.

use owt_api_types::TimelineKind;
use owt_api_types::dto::{
    BookDto, BookLevel, EntityView, EventDetail, HitKind, MarketDetail, MarketSummary, NewsDto,
    NumericDeltaDto, OddsRow, OutcomeDto, RelatedEntity, SearchHit, Side, TimelineEntry, TradeDto,
    WatchlistRow, WatchlistView,
};
use owt_api_types::{EntityKind, ids};
use owt_client::{ClientError, RestClient};
use time::OffsetDateTime;
use time::macros::datetime;

use crate::app::Loaded;
use crate::app::route::Route;

/// Fetch the data backing `route` from a live server. `query` is the current search
/// input (used only by [`Route::Search`]). Market detail fans out to four endpoints
/// concurrently; the failure of any one fails the load.
pub async fn fetch(client: &RestClient, route: &Route, query: &str) -> Result<Loaded, ClientError> {
    match route {
        Route::Search => {
            let page = client.search(query).await?;
            Ok(Loaded::Search { hits: page.items })
        }
        Route::MarketDetail { slug } => {
            let (detail, book, trades, news) = tokio::try_join!(
                client.market(slug),
                client.book(slug),
                client.trades(slug),
                client.market_news(slug),
            )?;
            Ok(Loaded::Market {
                detail: Box::new(detail),
                book,
                trades,
                news,
            })
        }
        Route::EventWorkspace { slug } => Ok(Loaded::Event(Box::new(client.event(slug).await?))),
        Route::TopicView { id } => Ok(Loaded::Entity(Box::new(client.entity(id).await?))),
        // No list route exists yet (ADR-0002 scope); show the `default` workspace.
        Route::Watchlists => Ok(Loaded::Watchlists(vec![client.watchlist("default").await?])),
    }
}

/// A fixed "now" so fixtures and golden frames are deterministic.
fn now() -> OffsetDateTime {
    datetime!(2026-07-10 10:14:05 UTC)
}

/// Load the data backing a route. Infallible for fixtures; the live version returns a
/// `Result` and the runtime maps failures to `Msg::Failed`.
pub fn load(route: &Route) -> Loaded {
    match route {
        Route::Search => Loaded::Search {
            hits: search("fed"),
        },
        Route::MarketDetail { slug } => market(slug),
        Route::EventWorkspace { slug } => Loaded::Event(Box::new(event(slug))),
        Route::TopicView { id } => Loaded::Entity(Box::new(entity(id))),
        Route::Watchlists => Loaded::Watchlists(watchlists()),
    }
}

/// Fixture search results.
pub fn search(_query: &str) -> Vec<SearchHit> {
    vec![
        SearchHit {
            kind: HitKind::Market,
            slug: "will-fed-cut-rates-in-september".to_owned(),
            title: "Will the Fed cut rates in September?".to_owned(),
            snippet: Some("YES 42.0¢ · $3.2M 24h vol".to_owned()),
        },
        SearchHit {
            kind: HitKind::Event,
            slug: "federal-reserve-september-meeting".to_owned(),
            title: "Federal Reserve September Meeting".to_owned(),
            snippet: Some("4 markets".to_owned()),
        },
        SearchHit {
            kind: HitKind::Entity,
            slug: "ent-jerome-powell".to_owned(),
            title: "Jerome Powell".to_owned(),
            snippet: Some("Person · Fed Chair".to_owned()),
        },
        SearchHit {
            kind: HitKind::News,
            slug: "reuters-fed-official-signals".to_owned(),
            title: "Reuters: Fed official signals openness to a cut".to_owned(),
            snippet: None,
        },
    ]
}

/// Fixture market-detail bundle.
pub fn market(slug: &str) -> Loaded {
    let detail = MarketDetail {
        market_id: ids::MarketId::from(slug),
        event_id: ids::EventId::from("federal-reserve-september-meeting"),
        title: "Will the Fed cut rates in September?".to_owned(),
        outcomes: vec![
            OutcomeDto {
                name: "Yes".to_owned(),
                price: 0.42,
            },
            OutcomeDto {
                name: "No".to_owned(),
                price: 0.58,
            },
        ],
        last: 0.42,
        bid: 0.41,
        ask: 0.43,
        spread: 0.02,
        vol_24h: 3_200_000.0,
        liquidity: "high".to_owned(),
        tags: vec!["economics".to_owned(), "fed".to_owned(), "macro".to_owned()],
    };
    let book = BookDto {
        asks: vec![
            BookLevel {
                price: 0.44,
                size: 800.0,
            },
            BookLevel {
                price: 0.43,
                size: 1_200.0,
            },
        ],
        bids: vec![
            BookLevel {
                price: 0.41,
                size: 1_050.0,
            },
            BookLevel {
                price: 0.40,
                size: 2_100.0,
            },
        ],
        mid: 0.42,
    };
    let trades = vec![
        TradeDto {
            ts: datetime!(2026-07-10 10:14:02 UTC),
            side: Side::Buy,
            price: 0.42,
            size: 90.0,
        },
        TradeDto {
            ts: datetime!(2026-07-10 10:13:58 UTC),
            side: Side::Sell,
            price: 0.41,
            size: 120.0,
        },
        TradeDto {
            ts: datetime!(2026-07-10 10:13:44 UTC),
            side: Side::Buy,
            price: 0.42,
            size: 50.0,
        },
    ];
    let news = vec![
        NewsDto {
            news_id: ids::NewsId::from("n1"),
            publisher: "Reuters".to_owned(),
            headline: "Fed official signals openness to a September cut".to_owned(),
            url: "https://example.com/reuters/fed".to_owned(),
            published_at: datetime!(2026-07-10 09:58:00 UTC),
        },
        NewsDto {
            news_id: ids::NewsId::from("n2"),
            publisher: "FT".to_owned(),
            headline: "Treasury yields slip as rate-cut bets firm".to_owned(),
            url: "https://example.com/ft/yields".to_owned(),
            published_at: datetime!(2026-07-10 09:31:00 UTC),
        },
        NewsDto {
            news_id: ids::NewsId::from("n3"),
            publisher: "AP".to_owned(),
            headline: "Powell remarks parsed for policy hints".to_owned(),
            url: "https://example.com/ap/powell".to_owned(),
            published_at: datetime!(2026-07-10 08:47:00 UTC),
        },
    ];
    Loaded::Market {
        detail: Box::new(detail),
        book,
        trades,
        news,
    }
}

/// Fixture event workspace.
pub fn event(slug: &str) -> EventDetail {
    EventDetail {
        event_id: ids::EventId::from(slug),
        title: "Federal Reserve September Meeting".to_owned(),
        markets: vec![
            MarketSummary {
                market_id: ids::MarketId::from("will-fed-cut-rates-in-september"),
                title: "Rate cut in September?".to_owned(),
            },
            MarketSummary {
                market_id: ids::MarketId::from("fed-cut-size-september"),
                title: "Cut size: 25bp vs 50bp".to_owned(),
            },
            MarketSummary {
                market_id: ids::MarketId::from("fed-dissent-september"),
                title: "Any dissenting votes?".to_owned(),
            },
        ],
        timeline: vec![
            TimelineEntry {
                ts: datetime!(2026-07-10 10:02:00 UTC),
                kind: TimelineKind::News,
                title: "Powell speech scheduled".to_owned(),
                numeric_delta: None,
                confidence: 0.9,
            },
            TimelineEntry {
                ts: datetime!(2026-07-10 10:20:00 UTC),
                kind: TimelineKind::PriceMove,
                title: "YES +3.0¢ in 18m".to_owned(),
                numeric_delta: Some(NumericDeltaDto {
                    delta_bp: 300,
                    window_s: 1_080,
                }),
                confidence: 0.82,
            },
            TimelineEntry {
                ts: datetime!(2026-07-10 10:24:00 UTC),
                kind: TimelineKind::News,
                title: "Reuters: 12 linked articles".to_owned(),
                numeric_delta: None,
                confidence: 0.71,
            },
        ],
    }
}

/// Fixture topic (entity) page.
pub fn entity(id: &str) -> EntityView {
    EntityView {
        entity_id: ids::EntityId::from(id),
        kind: EntityKind::Person,
        name: "Jerome Powell".to_owned(),
        description: Some(
            "Chair of the Federal Reserve. Context is human-curated (ADR-0011).".to_owned(),
        ),
        odds: vec![
            OddsRow {
                label: "Cut in September".to_owned(),
                probability: 0.42,
                market_id: ids::MarketId::from("will-fed-cut-rates-in-september"),
            },
            OddsRow {
                label: "Hold in September".to_owned(),
                probability: 0.58,
                market_id: ids::MarketId::from("will-fed-cut-rates-in-september"),
            },
        ],
        news: vec![NewsDto {
            news_id: ids::NewsId::from("n1"),
            publisher: "Reuters".to_owned(),
            headline: "Fed official signals openness to a September cut".to_owned(),
            url: "https://example.com/reuters/fed".to_owned(),
            published_at: datetime!(2026-07-10 09:58:00 UTC),
        }],
        related: vec![
            RelatedEntity {
                entity_id: ids::EntityId::from("ent-federal-reserve"),
                name: "Federal Reserve".to_owned(),
            },
            RelatedEntity {
                entity_id: ids::EntityId::from("ent-inflation"),
                name: "Inflation".to_owned(),
            },
        ],
    }
}

/// Fixture watchlists (the `default` workspace).
pub fn watchlists() -> Vec<WatchlistView> {
    let _ = now(); // keep a single "now" source for future relative rendering
    vec![
        WatchlistView {
            name: "macro".to_owned(),
            rows: vec![
                WatchlistRow {
                    market_id: ids::MarketId::from("will-fed-cut-rates-in-september"),
                    title: "Fed cut in September?".to_owned(),
                    price: 0.42,
                    chg_24h: 0.03,
                    spread: 0.02,
                    volume: 3_200_000.0,
                },
                WatchlistRow {
                    market_id: ids::MarketId::from("us-recession-2026"),
                    title: "US recession in 2026?".to_owned(),
                    price: 0.31,
                    chg_24h: -0.01,
                    spread: 0.03,
                    volume: 1_450_000.0,
                },
            ],
        },
        WatchlistView {
            name: "elections".to_owned(),
            rows: vec![WatchlistRow {
                market_id: ids::MarketId::from("senate-control-2026"),
                title: "Senate control 2026".to_owned(),
                price: 0.55,
                chg_24h: 0.02,
                spread: 0.02,
                volume: 5_900_000.0,
            }],
        },
    ]
}
