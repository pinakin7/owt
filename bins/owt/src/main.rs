//! `owt` — the ratatui TUI client (ADR-0002, `docs/design/tui-client.md`).
//!
//! An Elm-style `Model`/`Msg`/`update`/`view` loop over a single WebSocket, talking to
//! the server exclusively through `owt-client` + `owt-api-types`. This scaffold parses
//! arguments and prints a banner; the render loop and connection lifecycle land in the
//! TUI implementation phase.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "owt {} — TUI client scaffold (not yet implemented)",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}
