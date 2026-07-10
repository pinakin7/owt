//! Messages: the single input alphabet of the event loop. Everything that can change
//! state arrives as a `Msg` on one channel (`docs/design/tui-client.md` § Architecture).

use owt_api_types::dto::{
    BookDto, EntityView, EventDetail, MarketDetail, NewsDto, SearchHit, TradeDto, WatchlistView,
};
use owt_client::reconnect::ConnState;

/// A message driving `update`.
#[derive(Debug, Clone)]
pub enum Msg {
    /// A key was pressed.
    Key(crossterm::event::KeyEvent),
    /// The terminal was resized to `(cols, rows)`.
    Resize(u16, u16),
    /// The periodic render/refresh tick.
    Tick,
    /// Data for a route finished loading.
    Loaded(Loaded),
    /// A load failed; surfaced as a toast.
    Failed {
        /// What was being loaded.
        what: String,
        /// Human-readable error.
        error: String,
    },
    /// The connection state changed (status bar).
    Conn(ConnState),
}

/// A completed data load, tagged by the screen it feeds.
#[derive(Debug, Clone)]
pub enum Loaded {
    /// Search results.
    Search {
        /// Matching hits.
        hits: Vec<SearchHit>,
    },
    /// A market detail bundle (header + book + tape + linked news).
    Market {
        /// State header.
        detail: Box<MarketDetail>,
        /// Order book.
        book: BookDto,
        /// Trade tape, newest first.
        trades: Vec<TradeDto>,
        /// Linked news.
        news: Vec<NewsDto>,
    },
    /// An event workspace.
    Event(Box<EventDetail>),
    /// A topic (entity) page.
    Entity(Box<EntityView>),
    /// The watchlists.
    Watchlists(Vec<WatchlistView>),
}
