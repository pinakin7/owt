//! Per-screen state and rendering. Each screen owns a small state struct (loaded DTOs
//! with selection/focus) and a `render` function; the top-level [`crate::view`]
//! dispatches to the active screen. Layouts follow the PRD wireframes in
//! `docs/product/prd-mvp.md` (Screen inventory).

pub mod event_workspace;
pub mod market_detail;
pub mod search;
pub mod topic_view;
pub mod watchlists;

pub use event_workspace::EventWorkspaceState;
pub use market_detail::MarketDetailState;
pub use search::SearchState;
pub use topic_view::TopicViewState;
pub use watchlists::WatchlistsState;

/// All per-screen state, one field per screen. Screens keep their state across
/// navigation so `Esc` back returns to where you were.
#[derive(Debug, Default)]
pub struct ScreenStates {
    /// Search / browse.
    pub search: SearchState,
    /// Market detail.
    pub market_detail: MarketDetailState,
    /// Event workspace.
    pub event_workspace: EventWorkspaceState,
    /// Topic view.
    pub topic_view: TopicViewState,
    /// Watchlists.
    pub watchlists: WatchlistsState,
}

use crate::app::route::Route;

/// The outcome of opening (`Enter`) the current selection on a screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenResult {
    /// Navigate to a route (push + load).
    Route(Route),
    /// Nothing to navigate to; show an informational toast instead.
    Toast(String),
    /// No selection / nothing happens.
    None,
}

/// Move a selection index up by one, saturating at 0.
pub fn select_up(index: usize) -> usize {
    index.saturating_sub(1)
}

/// Move a selection index down by one, clamped to `len - 1` (no-op when empty).
pub fn select_down(index: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (index + 1).min(len - 1)
    }
}
