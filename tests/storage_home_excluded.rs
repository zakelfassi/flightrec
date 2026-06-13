use assert_cmd::Command;
use tempfile::TempDir;

/// The active storage home must never be snapshotted as user changes, even when
/// `FLIGHTREC_HOME` is relocated to a directory that does not match the default
/// `**/.flightrec/**` exclude glob and lives *inside* a watched root.
///
/// Without the guard, the second `watch --once` would record the first run's
/// `config.toml` and `snapshots/*.json` as freshly "added" files — self-generated
/// timeline noise.
#[test]
fn storage_home_artifacts_are_not_captured() {
    let tree = TempDir::new().unwrap();

    // Storage home lives inside the watched tree, under a non-".flightrec" name
    // so only the robust storage-root guard (not the default glob) can exclude it.
    let home = tree.path().join("frhome");
    std::fs::create_dir_all(&home).unwrap();

    let flightrec = |args: &[&str]| {
        let mut c = Command::cargo_bin("flightrec").unwrap();
        c.env("FLIGHTREC_HOME", &home);
        c.current_dir(tree.path());
        c.args(args);
        c
    };

    // Plant a real user file so the tree is non-empty.
    std::fs::write(tree.path().join("notes.txt"), "initial\n").unwrap();

    // Configure flightrec to watch the temp tree (root = canonicalized CWD).
    flightrec(&["init"]).assert().success();

    // Baseline snapshot — writes config.toml + snapshots/*.json under `home`.
    flightrec(&["watch", "--once", "--no-llm"])
        .assert()
        .success();
    // Second snapshot — would diff in the storage artifacts if not excluded.
    flightrec(&["watch", "--once", "--no-llm"])
        .assert()
        .success();

    // Inspect every produced diff: no change path may live under the storage home.
    let home_canon = std::fs::canonicalize(&home).unwrap();
    let home_str = home_canon.to_string_lossy().to_string();

    let diffs_dir = home.join("diffs");
    if let Ok(entries) = std::fs::read_dir(&diffs_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().extension().is_none_or(|x| x != "json") {
                continue;
            }
            let json: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap();
            if let Some(changes) = json["changes"].as_array() {
                for c in changes {
                    let path = c["path"].as_str().unwrap_or_default();
                    assert!(
                        !path.starts_with(&home_str),
                        "storage-home artifact captured as a change: {path}"
                    );
                }
            }
        }
    }

    // Confirm the snapshots themselves never list a file under the storage home.
    let snaps_dir = home.join("snapshots");
    for entry in std::fs::read_dir(&snaps_dir)
        .expect("snapshots/ must exist")
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().is_none_or(|x| x != "json") {
            continue;
        }
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap();
        if let Some(files) = json["files"].as_array() {
            for f in files {
                let path = f["path"].as_str().unwrap_or_default();
                assert!(
                    !path.starts_with(&home_str),
                    "storage-home file captured in snapshot: {path}"
                );
            }
        }
    }
}
