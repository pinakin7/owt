//! Navigation routes. The `route_stack` in [`crate::app::AppState`] is a
//! push/pop history; `Esc` pops (back), opening a screen pushes.

/// A screen the client can navigate to. Help is an overlay, not a route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// Search / browse.
    Search,
    /// A single market's detail screen.
    MarketDetail {
        /// Market slug.
        slug: String,
    },
    /// An event workspace.
    EventWorkspace {
        /// Event slug.
        slug: String,
    },
    /// A topic (entity) page.
    TopicView {
        /// Entity id / slug.
        id: String,
    },
    /// The watchlists screen.
    Watchlists,
}

impl Route {
    /// A short label for the status bar / breadcrumbs.
    pub fn label(&self) -> String {
        match self {
            Route::Search => "search".to_owned(),
            Route::MarketDetail { slug } => format!("market: {slug}"),
            Route::EventWorkspace { slug } => format!("workspace: {slug}"),
            Route::TopicView { id } => format!("topic: {id}"),
            Route::Watchlists => "watchlists".to_owned(),
        }
    }
}
