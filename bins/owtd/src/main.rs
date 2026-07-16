//! `owtd` — the owt server monolith and composition root
//! (`docs/architecture/cargo-workspace.md`, `docs/architecture/system-overview.md`,
//! `docs/adr/0003-modular-monolith.md`).
//!
//! Roles (source adapters, normalizer, store writer, search indexer, timeline
//! builder, query API + WS fanout) are runtime flags, not features — they compile
//! unconditionally and are selected via `serve --roles`. Workers are supervised tokio
//! tasks ([`owt_runtime::supervisor`]); a signal-driven [`owt_runtime::shutdown`]
//! drains them on SIGINT/SIGTERM.
//!
//! Per ADR-0003 each role's real logic arrives in a later ADR (0004 store, 0005 bus,
//! 0006 search, query-api). Today each role is a placeholder that logs, then idles
//! until shutdown — the supervision chassis is real, the work is not yet.

use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use owt_runtime::shutdown::Shutdown;
use owt_runtime::supervisor::{Supervisor, Worker};

#[derive(Debug, Parser)]
#[command(name = "owtd", version, about = "owt server monolith")]
struct Cli {
    /// Path to a TOML config file (overrides `OWT_CONFIG` and the default `owt.toml`).
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the server with the given roles (e.g. `api`, `ingest,normalize`, `all`).
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

/// A backend role selectable at runtime via `serve --roles`.
///
/// Ordering (used to dedupe/sort a role selection) follows declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Role {
    Ingest,
    Normalize,
    Index,
    Api,
    Alerts,
}

impl Role {
    /// Every role, in canonical order — the expansion of `all`.
    const ALL: [Role; 5] = [
        Role::Ingest,
        Role::Normalize,
        Role::Index,
        Role::Api,
        Role::Alerts,
    ];

    /// The role's stable CLI/log name.
    fn as_str(self) -> &'static str {
        match self {
            Role::Ingest => "ingest",
            Role::Normalize => "normalize",
            Role::Index => "index",
            Role::Api => "api",
            Role::Alerts => "alerts",
        }
    }
}

/// A failure to parse the `--roles` selection.
#[derive(Debug)]
enum RoleError {
    Unknown(String),
    Empty,
}

impl std::fmt::Display for RoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoleError::Unknown(role) => write!(
                f,
                "unknown role `{role}` (expected one of: ingest, normalize, index, api, alerts, all)"
            ),
            RoleError::Empty => write!(f, "no roles selected"),
        }
    }
}

impl std::error::Error for RoleError {}

impl FromStr for Role {
    type Err = RoleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ingest" => Ok(Role::Ingest),
            "normalize" => Ok(Role::Normalize),
            "index" => Ok(Role::Index),
            "api" => Ok(Role::Api),
            "alerts" => Ok(Role::Alerts),
            other => Err(RoleError::Unknown(other.to_owned())),
        }
    }
}

/// Parse the `--roles` tokens: split on `,`, expand `all`, dedupe, and sort into
/// canonical order. An unknown role errors; an empty selection errors.
fn parse_roles(tokens: &[String]) -> Result<Vec<Role>, RoleError> {
    use std::collections::BTreeSet;

    let mut set: BTreeSet<Role> = BTreeSet::new();
    for token in tokens {
        for part in token.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if part.eq_ignore_ascii_case("all") {
                set.extend(Role::ALL);
            } else {
                set.insert(Role::from_str(part)?);
            }
        }
    }
    if set.is_empty() {
        return Err(RoleError::Empty);
    }
    Ok(set.into_iter().collect())
}

/// Build the supervised placeholder worker for a role. Real logic lands in later ADRs;
/// today the worker logs it started, then idles until shutdown.
fn role_worker(role: Role, shutdown: Shutdown) -> Worker {
    Worker::new(role.as_str(), move || {
        let mut shutdown = shutdown.clone();
        async move {
            let name = role.as_str();
            if matches!(role, Role::Alerts) {
                // `owt-alerts` is a v1 crate, not yet scaffolded (cargo-workspace.md
                // § Phasing); the role is recognized but does no work yet.
                tracing::warn!(
                    role = name,
                    "role is a v1 stub (owt-alerts not yet scaffolded); awaiting shutdown only"
                );
            } else {
                tracing::info!(role = name, "role started");
            }
            shutdown.cancelled().await;
            tracing::info!(role = name, "role stopping");
            Ok::<(), anyhow::Error>(())
        }
    })
}

/// `owtd serve` — bring up the supervised runtime for the selected roles.
async fn serve(role_tokens: Vec<String>, config: Option<PathBuf>) -> anyhow::Result<()> {
    let _telemetry = owt_runtime::telemetry::init();
    let cfg = owt_runtime::config::load(config.as_deref())?;
    tracing::info!(?cfg, "configuration loaded");

    let roles = parse_roles(&role_tokens).map_err(|e| anyhow::anyhow!("{e}"))?;
    let role_names: Vec<&str> = roles.iter().map(|r| r.as_str()).collect();
    tracing::info!(roles = ?role_names, "starting owtd");

    let shutdown = owt_runtime::shutdown::install();
    let mut supervisor = Supervisor::new(shutdown.clone());
    for role in roles {
        supervisor.register(role_worker(role, shutdown.clone()));
    }
    let supervised = tokio::spawn(supervisor.run());

    // Wait for the shutdown signal, then bound the drain (deployment.md: ≤ 30s on
    // SIGTERM) so a stuck worker can't wedge the process forever.
    let mut drain = shutdown;
    drain.cancelled().await;
    tracing::info!("shutdown requested; draining workers (≤30s)");
    match tokio::time::timeout(std::time::Duration::from_secs(30), supervised).await {
        Ok(joined) => joined??,
        Err(_) => tracing::warn!("drain deadline exceeded; exiting without full drain"),
    }
    tracing::info!("owtd stopped");
    Ok(())
}

/// `owtd check-config` — validate the merged configuration and print it.
fn check_config(config: Option<PathBuf>) -> anyhow::Result<()> {
    let cfg = owt_runtime::config::load(config.as_deref())?;
    println!("{cfg:#?}");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve { roles } => serve(roles, cli.config).await?,
        Command::CheckConfig => check_config(cli.config)?,
        Command::Migrate => eprintln!("owtd migrate — not yet implemented"),
        Command::Backfill { source } => {
            eprintln!("owtd backfill {source} — not yet implemented");
        }
        Command::Reindex => eprintln!("owtd reindex — not yet implemented"),
        Command::Replay { subject } => {
            eprintln!("owtd replay {subject} — not yet implemented");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Role, RoleError, parse_roles};
    use std::str::FromStr;

    #[test]
    fn role_from_str_parses_each_variant() {
        assert_eq!(Role::from_str("ingest").unwrap(), Role::Ingest);
        assert_eq!(Role::from_str("normalize").unwrap(), Role::Normalize);
        assert_eq!(Role::from_str("index").unwrap(), Role::Index);
        assert_eq!(Role::from_str("api").unwrap(), Role::Api);
        assert_eq!(Role::from_str("alerts").unwrap(), Role::Alerts);
    }

    #[test]
    fn parse_roles_all_expands_to_every_role() {
        assert_eq!(
            parse_roles(&["all".to_owned()]).unwrap(),
            Role::ALL.to_vec()
        );
    }

    #[test]
    fn parse_roles_is_case_insensitive_and_trims() {
        assert_eq!(
            parse_roles(&[" Api , INDEX ".to_owned()]).unwrap(),
            vec![Role::Index, Role::Api]
        );
    }

    #[test]
    fn parse_roles_dedupes() {
        assert_eq!(
            parse_roles(&["api".to_owned(), "api".to_owned(), "index".to_owned()]).unwrap(),
            vec![Role::Index, Role::Api]
        );
    }

    #[test]
    fn parse_roles_rejects_unknown() {
        let err = parse_roles(&["workers".to_owned()]).unwrap_err();
        assert!(matches!(err, RoleError::Unknown(role) if role == "workers"));
    }

    #[test]
    fn parse_roles_empty_selection_is_error() {
        assert!(matches!(
            parse_roles(&[String::new()]).unwrap_err(),
            RoleError::Empty
        ));
        assert!(matches!(
            parse_roles(&[",".to_owned()]).unwrap_err(),
            RoleError::Empty
        ));
        assert!(matches!(parse_roles(&[]).unwrap_err(), RoleError::Empty));
    }

    #[test]
    fn parse_roles_mixed_all_and_named_still_all() {
        assert_eq!(
            parse_roles(&["api,all".to_owned()]).unwrap(),
            Role::ALL.to_vec()
        );
    }
}
