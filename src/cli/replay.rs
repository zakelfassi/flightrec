use anyhow::Result;

use crate::storage;

pub fn run(path: Option<String>, since: Option<String>) -> Result<()> {
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
    Ok(())
}
