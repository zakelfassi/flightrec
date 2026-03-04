use agentscope::{config, diff, snapshot, storage};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scope")]
#[command(about = "Git-like filesystem observability for AI agents")]
#[command(version = "0.1.0")]
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load_config()?;
    storage::init_storage()?;

    match cli.command {
        Commands::Watch { once, interval } => {
            let secs = interval.unwrap_or(cfg.daemon.interval_seconds);
            loop {
                let snap = snapshot::take_snapshot(
                    &cfg.watch.roots,
                    &cfg.filter.include,
                    &cfg.filter.exclude,
                )?;
                let count = snap.files.len();
                let saved = storage::save_snapshot(&snap)?;
                println!(
                    "[{}] snapshot {} — {} files → {}",
                    snap.created_at,
                    snap.snapshot_id,
                    count,
                    saved.display()
                );

                let all = storage::list_snapshots()?;
                if all.len() >= 2 {
                    let prev = storage::load_snapshot(&all[all.len() - 2])?;
                    let event = diff::compute_diff(&prev, &snap);
                    if !event.changes.is_empty() {
                        storage::save_diff(&event)?;
                        println!("  {} changes:", event.changes.len());
                        for c in &event.changes {
                            let sym = match c.change_type {
                                diff::ChangeType::Added => "+",
                                diff::ChangeType::Removed => "-",
                                diff::ChangeType::Modified => "~",
                                diff::ChangeType::Renamed => "→",
                            };
                            println!("    {} {}", sym, c.path);
                        }
                    } else {
                        println!("  no changes.");
                    }
                }

                if once {
                    break;
                }
                println!("sleeping {}s…", secs);
                std::thread::sleep(std::time::Duration::from_secs(secs));
            }
        }

        Commands::Diff {
            snap_a,
            snap_b,
            json,
        } => {
            let old = storage::load_snapshot(&snap_a)?;
            let new = storage::load_snapshot(&snap_b)?;
            let event = diff::compute_diff(&old, &new);
            if json {
                println!("{}", serde_json::to_string_pretty(&event)?);
            } else {
                println!("diff {} → {}", snap_a, snap_b);
                println!("{} changes:", event.changes.len());
                for c in &event.changes {
                    println!("  [{:?}] {}", c.change_type, c.path);
                    if let Some(ref rf) = c.renamed_from {
                        println!("      (from {})", rf);
                    }
                }
            }
        }

        Commands::Replay { path, since } => {
            let diffs = storage::list_diffs()?;
            for id in &diffs {
                let event = storage::load_diff(id)?;
                if let Some(ref s) = since {
                    if &event.created_at < s {
                        continue;
                    }
                }
                let filtered: Vec<_> = event
                    .changes
                    .iter()
                    .filter(|c| {
                        path.as_ref()
                            .map(|p| c.path.contains(p.as_str()))
                            .unwrap_or(true)
                    })
                    .collect();
                if !filtered.is_empty() {
                    println!("[{}] {} — {} changes", event.created_at, id, filtered.len());
                    for c in &filtered {
                        println!("  [{:?}] {}", c.change_type, c.path);
                    }
                }
            }
        }

        Commands::Report { diff_id, format } => {
            let event = storage::load_diff(&diff_id)?;
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&event)?),
                _ => {
                    println!("# Diff Report: {}", diff_id);
                    println!("**From:** `{}`", event.from_snapshot_id);
                    println!("**To:** `{}`", event.to_snapshot_id);
                    println!("**At:** {}", event.created_at);
                    println!("**Changes:** {}\n", event.changes.len());
                    for c in &event.changes {
                        let icon = match c.change_type {
                            diff::ChangeType::Added => "➕",
                            diff::ChangeType::Removed => "➖",
                            diff::ChangeType::Modified => "✏️ ",
                            diff::ChangeType::Renamed => "🔀",
                        };
                        print!("{} `{}`", icon, c.path);
                        if let Some(ref rf) = c.renamed_from {
                            print!(" ← `{}`", rf);
                        }
                        if let Some(ref dt) = c.diff_text {
                            print!(" ({})", dt);
                        }
                        println!();
                    }
                }
            }
        }
    }

    Ok(())
}
