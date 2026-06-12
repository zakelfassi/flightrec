use anyhow::Result;

use crate::{blobstore::BlobStore, config::Config, diff, llm, snapshot, storage};

pub fn run(once: bool, interval: Option<u64>, no_llm: bool, cfg: &Config) -> Result<()> {
    let secs = interval.unwrap_or(cfg.daemon.interval_seconds);
    let paths = storage::StoragePaths::new();
    let blob_store = BlobStore::new(&paths.objects);
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
                    let sym = match c.change_type {
                        diff::ChangeType::Added => "+",
                        diff::ChangeType::Removed => "-",
                        diff::ChangeType::Modified => "~",
                        diff::ChangeType::Renamed => "→",
                    };
                    println!("    {} {}", sym, c.path);
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
