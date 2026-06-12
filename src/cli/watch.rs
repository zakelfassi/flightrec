use anyhow::Result;

use crate::{config::Config, diff, snapshot, storage};

pub fn run(once: bool, interval: Option<u64>, cfg: &Config) -> Result<()> {
    let secs = interval.unwrap_or(cfg.daemon.interval_seconds);
    loop {
        let snap =
            snapshot::take_snapshot(&cfg.watch.roots, &cfg.filter.include, &cfg.filter.exclude)?;
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
    Ok(())
}
