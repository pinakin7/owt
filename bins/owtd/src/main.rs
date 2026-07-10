//! `owtd` — the owt server monolith and composition root
//! (`docs/architecture/cargo-workspace.md`, `docs/architecture/system-overview.md`).
//!
//! Roles (source adapters, normalizer, store writer, search indexer, timeline
//! builder, query API + WS fanout) are runtime flags, not features — they compile
//! unconditionally and are selected via `serve --roles`.
//!
//! Scaffold: subcommands are wired to the CLI but not yet implemented.

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "owtd", version, about = "owt server monolith")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the server with the given roles (e.g. `api`, `workers`, `all`).
    Serve {
        /// Comma-separated role list.
        #[arg(long, value_delimiter = ',', default_value = "all")]
        roles: Vec<String>,
    },
    /// Apply embedded database migrations and exit.
    Migrate,
    /// Run a bounded backfill pass for a named source.
    Backfill {
        /// Source identity (e.g. `gamma`, `data_api`, `rss`).
        source: String,
    },
    /// Rebuild the search index via alias swap.
    Reindex,
    /// Replay raw envelopes matching a subject range.
    Replay {
        /// Subject filter, e.g. `raw.pm_ws.market.>`.
        subject: String,
    },
    /// Validate the merged configuration and exit.
    CheckConfig,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve { roles } => {
            eprintln!(
                "owtd serve --roles {} — not yet implemented",
                roles.join(",")
            );
        }
        Command::Migrate => eprintln!("owtd migrate — not yet implemented"),
        Command::Backfill { source } => {
            eprintln!("owtd backfill {source} — not yet implemented");
        }
        Command::Reindex => eprintln!("owtd reindex — not yet implemented"),
        Command::Replay { subject } => {
            eprintln!("owtd replay {subject} — not yet implemented");
        }
        Command::CheckConfig => eprintln!("owtd check-config — not yet implemented"),
    }
    Ok(())
}
