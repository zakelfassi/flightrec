use assert_cmd::Command;
use tempfile::TempDir;

fn flightrec(home: &TempDir) -> Command {
    let mut c = Command::cargo_bin("flightrec").unwrap();
    c.env("FLIGHTREC_HOME", home.path());
    c
}

/// Check whether a blob file exists in the git-style fan-out layout.
/// The hash stored in `DiffEvent` records is the full 64-char SHA-256 hex string.
fn blob_exists(objects_dir: &std::path::Path, hash: &str) -> bool {
    if hash.len() < 3 {
        return false;
    }
    let (prefix, rest) = hash.split_at(2);
    objects_dir
        .join(prefix)
        .join(format!("{rest}.blob"))
        .exists()
}

/// End-to-end dogfood test: flightrec observes a temp tree, a file is edited,
/// and the resulting diff JSON is asserted to contain exactly one modified change
/// with a proper unified diff and both blobs present in the object store.
/// Uses `watch --once` (synchronous), so no sleeps are needed.
#[test]
fn dogfood_watch_once_detects_modification() {
    let home = TempDir::new().unwrap();
    let tree = TempDir::new().unwrap();

    // Plant an initial text file in the watched tree.
    let watched_file = tree.path().join("notes.txt");
    std::fs::write(&watched_file, "initial content\n").unwrap();

    // Configure flightrec to watch the temp tree by running init from its CWD.
    flightrec(&home)
        .current_dir(tree.path())
        .arg("init")
        .assert()
        .success();

    // First watch --once: take the baseline snapshot.
    flightrec(&home)
        .args(["watch", "--once", "--no-llm"])
        .assert()
        .success();

    // Mutate the watched file by appending a line.
    let appended = "appended by dogfood test";
    std::fs::write(&watched_file, format!("initial content\n{appended}\n")).unwrap();

    // Second watch --once: take a new snapshot and produce a diff.
    flightrec(&home)
        .args(["watch", "--once", "--no-llm"])
        .assert()
        .success();

    // --- Assertions on the produced diff ---

    let diffs_dir = home.path().join("diffs");
    let entries: Vec<_> = std::fs::read_dir(&diffs_dir)
        .expect("diffs/ must exist after second watch")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected exactly one diff file; found {}",
        entries.len()
    );

    let diff_json = std::fs::read_to_string(entries[0].path()).unwrap();
    let event: serde_json::Value =
        serde_json::from_str(&diff_json).expect("diff file must be valid JSON");

    let changes = event["changes"]
        .as_array()
        .expect("changes must be an array");
    assert_eq!(
        changes.len(),
        1,
        "expected exactly one change; got: {changes:#?}"
    );
    let modified: Vec<_> = changes
        .iter()
        .filter(|c| c["change_type"] == "modified")
        .collect();

    assert_eq!(
        modified.len(),
        1,
        "expected exactly one 'modified' change; all changes: {changes:#?}"
    );

    let change = &modified[0];

    // diff_text must be a real unified diff with a hunk header and the appended line.
    let diff_text = change["diff_text"]
        .as_str()
        .expect("diff_text must be a string for a modified text file");

    assert!(
        diff_text.contains("@@"),
        "diff_text must contain a hunk header '@@'\ngot:\n{diff_text}"
    );
    assert!(
        diff_text.contains(&format!("+{appended}")),
        "diff_text must contain '+{appended}'\ngot:\n{diff_text}"
    );

    // Both old and new blobs must be present in objects/.
    let objects_dir = home.path().join("objects");
    let old_hash = change["old_hash"]
        .as_str()
        .expect("old_hash must be present");
    let new_hash = change["new_hash"]
        .as_str()
        .expect("new_hash must be present");

    assert!(
        blob_exists(&objects_dir, old_hash),
        "old blob {old_hash} must exist in objects/"
    );
    assert!(
        blob_exists(&objects_dir, new_hash),
        "new blob {new_hash} must exist in objects/"
    );
}
