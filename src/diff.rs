use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::snapshot::SnapshotManifest;
use crate::utils::now_iso;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Renamed,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChangeRecord {
    pub path: String,
    pub change_type: ChangeType,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
    pub diff_text: Option<String>,
    pub renamed_from: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffEvent {
    pub diff_id: String,
    pub from_snapshot_id: String,
    pub to_snapshot_id: String,
    pub created_at: String,
    pub changes: Vec<ChangeRecord>,
}

pub fn compute_diff(old: &SnapshotManifest, new: &SnapshotManifest) -> DiffEvent {
    let old_map: HashMap<&str, &crate::snapshot::FileEntry> =
        old.files.iter().map(|f| (f.path.as_str(), f)).collect();
    let new_map: HashMap<&str, &crate::snapshot::FileEntry> =
        new.files.iter().map(|f| (f.path.as_str(), f)).collect();

    // Reverse map: blob_hash -> new path (for rename detection)
    let new_hash_to_path: HashMap<&str, &str> = new
        .files
        .iter()
        .map(|f| (f.blob_hash.as_str(), f.path.as_str()))
        .collect();

    let mut changes: Vec<ChangeRecord> = Vec::new();
    let mut rename_targets: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Removed or renamed
    for (path, old_entry) in &old_map {
        if new_map.contains_key(path) {
            continue; // still present — check for modify below
        }
        if let Some(&new_path) = new_hash_to_path.get(old_entry.blob_hash.as_str()) {
            if !old_map.contains_key(new_path) {
                // Rename detected
                rename_targets.insert(new_path.to_string());
                changes.push(ChangeRecord {
                    path: new_path.to_string(),
                    change_type: ChangeType::Renamed,
                    old_hash: Some(old_entry.blob_hash.clone()),
                    new_hash: Some(old_entry.blob_hash.clone()),
                    old_size: Some(old_entry.size),
                    new_size: new_map
                        .get(new_path)
                        .map(|e| e.size),
                    diff_text: None,
                    renamed_from: Some(path.to_string()),
                });
                continue;
            }
        }
        changes.push(ChangeRecord {
            path: path.to_string(),
            change_type: ChangeType::Removed,
            old_hash: Some(old_entry.blob_hash.clone()),
            new_hash: None,
            old_size: Some(old_entry.size),
            new_size: None,
            diff_text: None,
            renamed_from: None,
        });
    }

    // Added
    for (path, new_entry) in &new_map {
        if !old_map.contains_key(path) && !rename_targets.contains(*path) {
            changes.push(ChangeRecord {
                path: path.to_string(),
                change_type: ChangeType::Added,
                old_hash: None,
                new_hash: Some(new_entry.blob_hash.clone()),
                old_size: None,
                new_size: Some(new_entry.size),
                diff_text: None,
                renamed_from: None,
            });
        }
    }

    // Modified
    for (path, old_entry) in &old_map {
        if let Some(new_entry) = new_map.get(path) {
            if old_entry.blob_hash != new_entry.blob_hash {
                let diff_text = if old_entry.is_text && new_entry.is_text {
                    Some(format!(
                        "size: {} -> {} bytes",
                        old_entry.size, new_entry.size
                    ))
                } else {
                    None
                };
                changes.push(ChangeRecord {
                    path: path.to_string(),
                    change_type: ChangeType::Modified,
                    old_hash: Some(old_entry.blob_hash.clone()),
                    new_hash: Some(new_entry.blob_hash.clone()),
                    old_size: Some(old_entry.size),
                    new_size: Some(new_entry.size),
                    diff_text,
                    renamed_from: None,
                });
            }
        }
    }

    let now = chrono::Utc::now();
    DiffEvent {
        diff_id: format!(
            "diff-{}-{:03}",
            now.format("%Y%m%dT%H%M%S"),
            now.timestamp_subsec_millis()
        ),
        from_snapshot_id: old.snapshot_id.clone(),
        to_snapshot_id: new.snapshot_id.clone(),
        created_at: now_iso(),
        changes,
    }
}

pub fn text_diff(old: &str, new: &str) -> String {
    use similar::{ChangeTag, TextDiff};
    let diff = TextDiff::from_lines(old, new);
    let mut out = String::new();
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => out.push_str(&format!("-{}", change)),
            ChangeTag::Insert => out.push_str(&format!("+{}", change)),
            ChangeTag::Equal => {}
        }
    }
    out
}
