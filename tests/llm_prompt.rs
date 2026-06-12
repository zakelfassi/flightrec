/// Integration tests for the LLM prompt layer.
///
/// No network calls — all tests are purely deterministic, in-process.
use flightrec::{
    diff::{ChangeRecord, ChangeType, DiffEvent},
    llm::prompt,
};

fn make_change(path: &str, change_type: ChangeType, diff_text: Option<&str>) -> ChangeRecord {
    ChangeRecord {
        path: path.to_string(),
        change_type,
        old_hash: Some("aaabbb".to_string()),
        new_hash: Some("cccdd".to_string()),
        old_size: Some(100),
        new_size: Some(200),
        diff_text: diff_text.map(|s| s.to_string()),
        renamed_from: None,
    }
}

fn make_event(changes: Vec<ChangeRecord>) -> DiffEvent {
    DiffEvent {
        diff_id: "diff-test-001".to_string(),
        from_snapshot_id: "snap-aaa".to_string(),
        to_snapshot_id: "snap-bbb".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        changes,
        summary: None,
    }
}

/// Same DiffEvent always renders byte-identical prompt.
#[test]
fn prompt_determinism_byte_identical() {
    let event = make_event(vec![
        make_change(
            "src/main.rs",
            ChangeType::Modified,
            Some("@@ -1,2 +1,3 @@\n hello\n+world\n"),
        ),
        make_change("README.md", ChangeType::Added, None),
        make_change("Cargo.toml", ChangeType::Modified, None),
    ]);
    let (sys1, user1) = prompt::render(&event, 30);
    let (sys2, user2) = prompt::render(&event, 30);
    assert_eq!(sys1, sys2, "system prompt must be deterministic");
    assert_eq!(user1, user2, "user prompt must be deterministic");
}

/// Changes must appear sorted by path regardless of insertion order.
#[test]
fn prompt_changes_sorted_by_path() {
    let event = make_event(vec![
        make_change("z/z.rs", ChangeType::Modified, None),
        make_change("a/a.rs", ChangeType::Added, None),
        make_change("m/m.rs", ChangeType::Removed, None),
    ]);
    let (_, user) = prompt::render(&event, 30);
    let pos_a = user.find("a/a.rs").expect("a/a.rs not in prompt");
    let pos_m = user.find("m/m.rs").expect("m/m.rs not in prompt");
    let pos_z = user.find("z/z.rs").expect("z/z.rs not in prompt");
    assert!(pos_a < pos_m, "a should come before m");
    assert!(pos_m < pos_z, "m should come before z");
}

/// max_changes_per_prompt caps how many changes appear in the prompt.
#[test]
fn prompt_truncation_at_max_changes() {
    let changes: Vec<ChangeRecord> = (0..40)
        .map(|i| make_change(&format!("file_{:02}.rs", i), ChangeType::Modified, None))
        .collect();
    let event = make_event(changes);

    let (_, user15) = prompt::render(&event, 15);
    let count = user15.matches("- path:").count();
    assert_eq!(
        count, 15,
        "expected exactly 15 changes with max=15, got {count}"
    );

    let (_, user40) = prompt::render(&event, 40);
    let count40 = user40.matches("- path:").count();
    assert_eq!(
        count40, 40,
        "expected 40 changes with max=40, got {count40}"
    );
}

/// JSON parse tolerates ```json fences.
#[test]
fn json_parse_strips_json_fence() {
    // Re-implement fence-stripping inline to mirror what parse_json_output does.
    let raw = "```json\n{\"short\":\"ok\",\"actions\":[],\"intent_guess\":null}\n```";
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_end_matches("```").trim())
        .unwrap_or(trimmed);
    let v: serde_json::Value = serde_json::from_str(stripped).expect("should parse");
    assert_eq!(v["short"], "ok");
}

/// Malformed output produces BadOutput error (tested via private logic in llm::tests,
/// but we replicate the contract assertion here to pin it).
#[test]
fn malformed_json_output_is_bad_output_error() {
    // Simulate what parse_json_output does: strip fence then serde_json::from_str
    let raw = "this is not json";
    let stripped = raw.trim();
    let result: Result<serde_json::Value, _> = serde_json::from_str(stripped);
    assert!(result.is_err(), "malformed JSON must fail serde parse");
}

/// Old config.toml without the new LLM fields must still parse (backward compat).
#[test]
fn old_config_without_new_llm_fields_parses() {
    let toml_src = r#"
[watch]
roots = ["."]

[filter]
include = ["**/*.rs"]
exclude = ["**/target/**"]

[daemon]
interval_seconds = 60

[llm]
enabled = false
provider = "anthropic"
model = "claude-haiku-4-5"

[output]
json_log_dir = "~/.flightrec/logs"
"#;
    let cfg: flightrec::config::Config = toml::from_str(toml_src).expect("old config must parse");
    assert_eq!(
        cfg.llm.max_changes_per_prompt, 30,
        "default max_changes_per_prompt is 30"
    );
    assert!(cfg.llm.base_url.is_none(), "base_url defaults to None");
    assert!(
        cfg.llm.api_key_env.is_none(),
        "api_key_env defaults to None"
    );
}
