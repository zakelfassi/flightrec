use assert_cmd::Command;
use tempfile::TempDir;

fn flightrec(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin("flightrec").unwrap();
    c.env("FLIGHTREC_HOME", tmp.path());
    c
}

#[test]
fn init_creates_config_with_cwd_root() {
    let tmp = TempDir::new().unwrap();
    flightrec(&tmp).arg("init").assert().success();

    let config_path = tmp.path().join("config.toml");
    assert!(config_path.exists(), "config.toml should exist after init");

    let content = std::fs::read_to_string(&config_path).unwrap();
    let cwd = std::fs::canonicalize(".").unwrap();
    assert!(
        content.contains(cwd.to_str().unwrap()),
        "config.toml should contain canonicalized CWD: {cwd:?}\ncontent:\n{content}"
    );
}

#[test]
fn init_no_personal_paths_in_config() {
    let tmp = TempDir::new().unwrap();
    flightrec(&tmp).arg("init").assert().success();

    let content = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(
        !content.contains("openclaw"),
        "config should not contain personal 'openclaw' path"
    );
    assert!(
        !content.contains("tac-monorepo"),
        "config should not contain personal 'tac-monorepo' path"
    );
    assert!(
        !content.contains("clawd"),
        "config should not contain personal 'clawd' path"
    );
}

#[test]
fn init_refuses_second_run_without_force() {
    let tmp = TempDir::new().unwrap();
    flightrec(&tmp).arg("init").assert().success();
    flightrec(&tmp)
        .arg("init")
        .assert()
        .failure()
        .code(predicates::ord::gt(0));
}

#[test]
fn init_force_succeeds_on_existing_config() {
    let tmp = TempDir::new().unwrap();
    flightrec(&tmp).arg("init").assert().success();
    flightrec(&tmp)
        .arg("init")
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn init_with_root_override_uses_custom_path() {
    let tmp = TempDir::new().unwrap();
    let root_dir = TempDir::new().unwrap();
    let canonical_root = std::fs::canonicalize(root_dir.path()).unwrap();

    flightrec(&tmp)
        .arg("init")
        .arg("--root")
        .arg(root_dir.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert!(
        content.contains(canonical_root.to_str().unwrap()),
        "config.toml should contain canonical custom root: {canonical_root:?}\ncontent:\n{content}"
    );
}

#[test]
fn init_prints_path_and_next_steps() {
    let tmp = TempDir::new().unwrap();
    flightrec(&tmp)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("config.toml"))
        .stdout(predicates::str::contains("flightrec watch --once"))
        .stdout(predicates::str::contains("flightrec tui"));
}
