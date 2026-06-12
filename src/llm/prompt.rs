use crate::diff::DiffEvent;

/// Maximum number of diff-text lines included per change in the prompt.
const MAX_DIFF_LINES: usize = 40;

/// Render a deterministic (system, user) prompt pair for `event`.
///
/// - Changes are sorted by path before rendering (guarantees byte-identical output
///   for the same logical event regardless of HashMap iteration order).
/// - `max_changes` caps how many changes are included in the prompt.
/// - `diff_text` is truncated to `MAX_DIFF_LINES` lines if present.
pub fn render(event: &DiffEvent, max_changes: usize) -> (String, String) {
    let system =
        "You are an observability analyst. Summarize filesystem changes clearly and tersely. \
                  Output ONLY valid JSON with keys: short (string), actions (array of strings), \
                  intent_guess (string or null). No markdown fences, no extra fields."
            .to_string();

    let mut sorted_changes = event.changes.clone();
    sorted_changes.sort_by(|a, b| a.path.cmp(&b.path));
    let capped = sorted_changes
        .into_iter()
        .take(max_changes)
        .collect::<Vec<_>>();

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "Changes from snapshot {} → {}:",
        event.from_snapshot_id, event.to_snapshot_id
    ));
    for c in &capped {
        lines.push(format!("- path: {}", c.path));
        lines.push(format!("  change: {:?}", c.change_type).to_lowercase());
        if let Some(renamed_from) = &c.renamed_from {
            lines.push(format!("  renamed_from: {}", renamed_from));
        }
        if let Some(dt) = &c.diff_text {
            let truncated: String = dt
                .lines()
                .take(MAX_DIFF_LINES)
                .collect::<Vec<_>>()
                .join("\n");
            lines.push("  diff: |".to_string());
            for diff_line in truncated.lines() {
                lines.push(format!("    {}", diff_line));
            }
        }
    }
    lines.push(String::new());
    lines.push("Output JSON with keys: short, actions, intent_guess.".to_string());

    (system, lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::{ChangeRecord, ChangeType, DiffEvent};

    fn make_event(paths: &[&str]) -> DiffEvent {
        DiffEvent {
            diff_id: "diff-test-001".to_string(),
            from_snapshot_id: "snap-a".to_string(),
            to_snapshot_id: "snap-b".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            changes: paths
                .iter()
                .map(|p| ChangeRecord {
                    path: p.to_string(),
                    change_type: ChangeType::Modified,
                    old_hash: Some("aaa".to_string()),
                    new_hash: Some("bbb".to_string()),
                    old_size: Some(100),
                    new_size: Some(200),
                    diff_text: None,
                    renamed_from: None,
                })
                .collect(),
            summary: None,
        }
    }

    #[test]
    fn render_is_deterministic() {
        // Same event → byte-identical render twice
        let event = make_event(&["z/file.rs", "a/file.rs", "m/file.rs"]);
        let (sys1, user1) = render(&event, 30);
        let (sys2, user2) = render(&event, 30);
        assert_eq!(sys1, sys2);
        assert_eq!(user1, user2);
    }

    #[test]
    fn render_changes_sorted_by_path() {
        let event = make_event(&["z/z.rs", "a/a.rs", "m/m.rs"]);
        let (_, user) = render(&event, 30);
        let positions = ["a/a.rs", "m/m.rs", "z/z.rs"].map(|p| user.find(p).unwrap());
        assert!(positions[0] < positions[1]);
        assert!(positions[1] < positions[2]);
    }

    #[test]
    fn render_truncates_at_max_changes() {
        let paths: Vec<String> = (0..50).map(|i| format!("file_{:02}.rs", i)).collect();
        let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let event = make_event(&path_refs);
        let (_, user) = render(&event, 10);
        // Only 10 changes should appear — count "- path:" occurrences
        let count = user.matches("- path:").count();
        assert_eq!(count, 10);
    }
}
