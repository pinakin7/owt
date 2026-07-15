//! `owt` — the ratatui TUI client (ADR-0002, `docs/design/tui-client.md`).
//!
//! A thin entry point: all logic lives in the `owt_tui` library so it can be unit- and
//! golden-frame-tested without a terminal. This shell just spins the async runtime and
//! delegates to [`owt_tui::run`].

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    owt_tui::run().await
}
