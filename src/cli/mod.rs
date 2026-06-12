mod diff;
mod init;
mod replay;
mod report;
mod tui;
mod watch;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::{config, storage};

#[derive(Parser)]
#[command(name = "flightrec")]
#[command(about = "Git-like filesystem observability for AI agents")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        /// Disable LLM summarization even if enabled in config
        #[arg(long)]
        no_llm: bool,
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
    /// Write a starter config.toml in FLIGHTREC_HOME
    Init {
        /// Overwrite an existing config
        #[arg(long)]
        force: bool,
        /// Watch root path (repeatable; defaults to canonicalized CWD)
        #[arg(long, value_name = "PATH")]
        root: Vec<PathBuf>,
    },
    /// Open the interactive TUI (timeline → diff detail → file diff)
    Tui,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // No subcommand → launch the TUI (default experience).
    let cmd = match cli.command {
        Some(c) => c,
        None => Commands::Tui,
    };

    match cmd {
        // init handles its own config/storage setup
        Commands::Init { force, root } => init::run(force, root),
        // TUI does its own terminal setup; skip storage init.
        Commands::Tui => tui::run(),
        cmd => {
            let cfg = config::load_config()?;
            storage::init_storage()?;
            match cmd {
                Commands::Watch {
                    once,
                    interval,
                    no_llm,
                } => watch::run(once, interval, no_llm, &cfg)?,
                Commands::Diff {
                    snap_a,
                    snap_b,
                    json,
                } => diff::run(snap_a, snap_b, json)?,
                Commands::Replay { path, since } => replay::run(path, since)?,
                Commands::Report { diff_id, format } => report::run(diff_id, format)?,
                Commands::Init { .. } | Commands::Tui => unreachable!(),
            }
            Ok(())
        }
    }
}
