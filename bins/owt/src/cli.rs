//! Command-line arguments (`docs/design/tui-client.md` § Configuration). Flags are the
//! highest-precedence config layer; `--profile` selects a `[profiles.<name>]` overlay.

use clap::Parser;

/// The `owt` terminal research client.
#[derive(Debug, Parser)]
#[command(
    name = "owt",
    version,
    about = "owt — terminal research client for prediction markets"
)]
pub struct Args {
    /// Config profile to activate (overlays `[profiles.<name>]`), e.g. `staging`.
    #[arg(long)]
    pub profile: Option<String>,

    /// Override the server URL for this run.
    #[arg(long)]
    pub server: Option<String>,

    /// Log filter directive written to the log file, e.g. `info` or `owt=debug`.
    #[arg(long)]
    pub log: Option<String>,

    /// Show the debug overlay (FPS, frame time, WS lag).
    #[arg(long)]
    pub debug: bool,
}

impl Args {
    /// Parse from the process arguments.
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
