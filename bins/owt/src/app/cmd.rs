//! Commands: the side effects `update` requests but never performs. The runtime layer
//! executes each `Cmd` (fetch, subscribe, export, quit) and feeds results back as
//! `Msg`s — this is what keeps `update` pure and testable without I/O.

use crate::app::route::Route;

/// A side effect for the runtime to perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    /// Load the data backing a route (runtime → data seam → `Msg::Loaded`).
    Load(Route),
    /// Subscribe to a live WS topic.
    Subscribe(String),
    /// Unsubscribe from a live WS topic.
    Unsubscribe(String),
    /// Export the named pane's data to a path.
    Export {
        /// Export format (`json` | `csv`).
        format: String,
        /// Destination path (empty = default).
        path: String,
    },
    /// Tear down and exit.
    Quit,
}
