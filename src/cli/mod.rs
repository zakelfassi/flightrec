mod diff;
mod replay;
mod report;
mod watch;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{config, storage};

#[derive(Parser)]
#[command(name = "flightrec")]
#[command(about = "Git-like filesystem observability for AI agents")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch filesystem and continuously record snapshots + diffs
    Watch {
        /// Run once and exit (instead of looping)
        #[arg(long)]
        once: bool,
        /// Override poll interval in seconds
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Compute and display the diff between two snapshots
    Diff {
        snap_a: String,
        snap_b: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Replay diff history in chronological order
    Replay {
        /// Filter changes by path substring
        #[arg(long)]
        path: Option<String>,
        /// Only show diffs at or after this ISO timestamp
        #[arg(long)]
        since: Option<String>,
    },
    /// Generate a human-readable report for a diff
    Report {
        diff_id: String,
        #[arg(long, default_value = "md")]
        format: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load_config()?;
    storage::init_storage()?;

    match cli.command {
        Commands::Watch { once, interval } => watch::run(once, interval, &cfg)?,
        Commands::Diff {
            snap_a,
            snap_b,
            json,
        } => diff::run(snap_a, snap_b, json)?,
        Commands::Replay { path, since } => replay::run(path, since)?,
        Commands::Report { diff_id, format } => report::run(diff_id, format)?,
    }

    Ok(())
}
