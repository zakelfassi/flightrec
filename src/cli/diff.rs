use anyhow::Result;

use crate::{diff, storage};

pub fn run(snap_a: String, snap_b: String, json: bool) -> Result<()> {
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
    Ok(())
}
