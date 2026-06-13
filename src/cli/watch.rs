use std::io::IsTerminal;

use anyhow::Result;

use crate::{blobstore::BlobStore, config::Config, diff, llm, snapshot, storage};

/// Returns whether ANSI color output should be used for stdout.
/// Respects `NO_COLOR` env var and non-tty detection.
fn use_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}

/// Wraps `text` in the given ANSI color sequence when `colored` is true.
fn colorize(text: &str, ansi_code: &str, colored: bool) -> String {
    if colored {
        format!("\x1b[{ansi_code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn run(once: bool, interval: Option<u64>, no_llm: bool, cfg: &Config) -> Result<()> {
    let secs = interval.unwrap_or(cfg.daemon.interval_seconds);
    let paths = storage::StoragePaths::new();
    let blob_store = BlobStore::new(&paths.objects);
    let colored = use_color();
    loop {
        let snap = snapshot::take_snapshot(
            &cfg.watch.roots,
            &cfg.filter.include,
            &cfg.filter.exclude,
            Some(&blob_store),
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
            let mut event = diff::compute_diff(&prev, &snap);
            diff::enrich_with_diffs(&mut event, &blob_store);
            if !event.changes.is_empty() {
                storage::save_diff(&event)?;
                println!("  {} changes:", event.changes.len());
                for c in &event.changes {
                    // ANSI escape codes are kept here for broad terminal
                    // compatibility (watch output streams to non-TUI stdout).
                    // The semantic mapping mirrors the design-system signal
                    // colors in src/tui/theme.rs: added=#1A9E55 (green/32),
                    // removed=#D43B3B (red/31), modified=#C7860A (amber/33),
                    // renamed=neutral (dim/2).
                    let (sym, ansi) = match c.change_type {
                        diff::ChangeType::Added => ("+", "32"),
                        diff::ChangeType::Removed => ("-", "31"),
                        diff::ChangeType::Modified => ("~", "33"),
                        diff::ChangeType::Renamed => ("→", "2"),
                    };
                    println!("    {} {}", colorize(sym, ansi, colored), c.path);
                }

                // LLM summarization — failure is non-fatal (warn + continue).
                if cfg.llm.enabled && !no_llm {
                    match llm::summarize_diff(&event, &cfg.llm) {
                        Ok(summary) => {
                            event.summary = Some(summary);
                            if let Err(e) = storage::save_diff(&event) {
                                eprintln!("warning: could not re-save diff with summary: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "warning: LLM summarization failed (diff persisted without summary): {e}"
                            );
                        }
                    }
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
    Ok(())
}
