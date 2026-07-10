//! `owt_tui` — the library core of the `owt` TUI client (ADR-0002,
//! `docs/design/tui-client.md`).
//!
//! The pure Elm-style core (`app::AppState` / `app::Msg` / `app::update` / `view`)
//! lives here so it can be driven by unit tests and golden-frame tests without a
//! terminal. The process shell (`runtime`, `terminal`, `cli`, `config`, `logging`)
//! wraps it with the event loop and I/O; `main.rs` is a thin entry point over
//! [`run`].

pub mod app;
pub mod cli;
pub mod config;
pub mod data;
pub mod fmt;
pub mod input;
pub mod logging;
pub mod palette;
pub mod runtime;
pub mod screens;
pub mod terminal;
pub mod theme;
pub mod view;
pub mod widgets;

pub use app::{AppState, Cmd, Loaded, Msg, Route, update};
pub use view::view;

use anyhow::Result;

/// Parse arguments, load config, initialize logging, and run the TUI event loop,
/// restoring the terminal on exit (including on panic).
pub async fn run() -> Result<()> {
    let args = cli::Args::parse_args();
    let cfg = config::load(&args)?;
    logging::init(&args)?;

    let mut state = AppState::new(cfg);
    state.debug = args.debug;

    runtime::run(state).await
}
