use anyhow::Result;
use std::path::PathBuf;

use crate::diff::DiffEvent;
use crate::snapshot::SnapshotManifest;
use crate::utils::expand_tilde;

pub struct StoragePaths {
    pub base: PathBuf,
    pub snapshots: PathBuf,
    pub diffs: PathBuf,
    pub objects: PathBuf,
}

impl StoragePaths {
    pub fn new() -> Self {
        let base = expand_tilde("~/.agentscope");
        StoragePaths {
            snapshots: base.join("snapshots"),
            diffs: base.join("diffs"),
            objects: base.join("objects"),
            base,
        }
    }
}

impl Default for StoragePaths {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init_storage() -> Result<StoragePaths> {
    let paths = StoragePaths::new();
    std::fs::create_dir_all(&paths.snapshots)?;
    std::fs::create_dir_all(&paths.diffs)?;
    std::fs::create_dir_all(&paths.objects)?;
    Ok(paths)
}

pub fn save_snapshot(snapshot: &SnapshotManifest) -> Result<PathBuf> {
    let paths = StoragePaths::new();
    let file = paths
        .snapshots
        .join(format!("{}.json", snapshot.snapshot_id));
    std::fs::write(&file, serde_json::to_string_pretty(snapshot)?)?;
    Ok(file)
}

pub fn load_snapshot(id: &str) -> Result<SnapshotManifest> {
    let paths = StoragePaths::new();
    let file = paths.snapshots.join(format!("{}.json", id));
    let content = std::fs::read_to_string(&file)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn list_snapshots() -> Result<Vec<String>> {
    let paths = StoragePaths::new();
    let mut ids = Vec::new();
    if paths.snapshots.exists() {
        for entry in std::fs::read_dir(&paths.snapshots)? {
            let name = entry?.file_name();
            let s = name.to_string_lossy().to_string();
            if s.ends_with(".json") {
                ids.push(s.trim_end_matches(".json").to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

pub fn load_latest_snapshot() -> Result<Option<SnapshotManifest>> {
    let ids = list_snapshots()?;
    match ids.last() {
        Some(id) => Ok(Some(load_snapshot(id)?)),
        None => Ok(None),
    }
}

pub fn save_diff(diff: &DiffEvent) -> Result<PathBuf> {
    let paths = StoragePaths::new();
    let file = paths.diffs.join(format!("{}.json", diff.diff_id));
    std::fs::write(&file, serde_json::to_string_pretty(diff)?)?;
    Ok(file)
}

pub fn load_diff(id: &str) -> Result<DiffEvent> {
    let paths = StoragePaths::new();
    let file = paths.diffs.join(format!("{}.json", id));
    let content = std::fs::read_to_string(&file)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn list_diffs() -> Result<Vec<String>> {
    let paths = StoragePaths::new();
    let mut ids = Vec::new();
    if paths.diffs.exists() {
        for entry in std::fs::read_dir(&paths.diffs)? {
            let name = entry?.file_name();
            let s = name.to_string_lossy().to_string();
            if s.ends_with(".json") {
                ids.push(s.trim_end_matches(".json").to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}
