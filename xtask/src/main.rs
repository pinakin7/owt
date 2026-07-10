//! `xtask` — repository dev automation, invoked as `cargo xtask <cmd>`
//! (`docs/ops/ci-cd-and-release.md`, `docs/ops/testing-strategy.md`).
//!
//! Scaffold: subcommands are wired but not yet implemented.

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "owt dev automation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Capture or refresh recorded upstream fixtures.
    Fixtures,
    /// Orchestrate the local Docker Compose stack.
    Compose,
    /// Build distributable binaries (cargo-dist).
    Dist,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Fixtures => eprintln!("xtask fixtures — not yet implemented"),
        Command::Compose => eprintln!("xtask compose — not yet implemented"),
        Command::Dist => eprintln!("xtask dist — not yet implemented"),
    }
    Ok(())
}
