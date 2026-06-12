use flightrec::diff::{compute_diff, ChangeType};
use flightrec::snapshot::{FileEntry, SnapshotManifest};

fn snap(id: &str, files: Vec<(&str, &str, u64)>) -> SnapshotManifest {
    SnapshotManifest {
        snapshot_id: id.to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        roots: vec!["/test".to_string()],
        files: files
            .into_iter()
            .map(|(path, hash, size)| FileEntry {
                path: path.to_string(),
                size,
                blob_hash: hash.to_string(),
                is_text: true,
            })
            .collect(),
    }
}

#[test]
fn test_added() {
    let old = snap("snap-a", vec![]);
    let new = snap("snap-b", vec![("/test/new.rs", "abc123", 100)]);
    let diff = compute_diff(&old, &new);
    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].change_type, ChangeType::Added);
    assert_eq!(diff.changes[0].path, "/test/new.rs");
}

#[test]
fn test_removed() {
    let old = snap("snap-a", vec![("/test/old.rs", "abc123", 100)]);
    let new = snap("snap-b", vec![]);
    let diff = compute_diff(&old, &new);
    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].change_type, ChangeType::Removed);
    assert_eq!(diff.changes[0].path, "/test/old.rs");
}

#[test]
fn test_modified() {
    let old = snap("snap-a", vec![("/test/file.rs", "hash1", 50)]);
    let new = snap("snap-b", vec![("/test/file.rs", "hash2", 80)]);
    let diff = compute_diff(&old, &new);
    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].change_type, ChangeType::Modified);
    assert_eq!(diff.changes[0].old_hash.as_deref(), Some("hash1"));
    assert_eq!(diff.changes[0].new_hash.as_deref(), Some("hash2"));
}

#[test]
fn test_renamed() {
    let old = snap("snap-a", vec![("/test/old_name.rs", "samehash", 100)]);
    let new = snap("snap-b", vec![("/test/new_name.rs", "samehash", 100)]);
    let diff = compute_diff(&old, &new);
    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].change_type, ChangeType::Renamed);
    assert_eq!(diff.changes[0].path, "/test/new_name.rs");
    assert_eq!(
        diff.changes[0].renamed_from.as_deref(),
        Some("/test/old_name.rs")
    );
}

#[test]
fn test_no_changes() {
    let old = snap("snap-a", vec![("/test/file.rs", "hash1", 100)]);
    let new = snap("snap-b", vec![("/test/file.rs", "hash1", 100)]);
    let diff = compute_diff(&old, &new);
    assert!(diff.changes.is_empty());
}
