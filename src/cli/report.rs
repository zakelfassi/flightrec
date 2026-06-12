use anyhow::Result;

use crate::{diff, storage};

pub fn run(diff_id: String, format: String) -> Result<()> {
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
    Ok(())
}
