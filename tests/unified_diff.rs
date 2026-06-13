use flightrec::diff::unified_diff;

const OLD: &str = include_str!("fixtures/old_file.txt");
const NEW: &str = include_str!("fixtures/new_file.txt");
const EXPECTED: &str = include_str!("fixtures/expected_unified.diff");

#[test]
fn golden_unified_diff_matches_fixture() {
    let result = unified_diff(OLD, NEW);
    assert_eq!(
        result, EXPECTED,
        "unified diff output does not match fixture"
    );
}

#[test]
fn unified_diff_contains_hunk_header() {
    let result = unified_diff(OLD, NEW);
    assert!(result.contains("@@"), "missing @@ hunk header");
}

#[test]
fn unified_diff_contains_removed_line() {
    let result = unified_diff(OLD, NEW);
    assert!(result.contains("-charlie"), "missing removed line");
}

#[test]
fn unified_diff_contains_added_line() {
    let result = unified_diff(OLD, NEW);
    assert!(result.contains("+CHARLIE"), "missing added line");
}

#[test]
fn unified_diff_contains_context_lines() {
    let result = unified_diff(OLD, NEW);
    // Context lines are prefixed with a space
    assert!(result.contains(" alpha"), "missing context line 'alpha'");
    assert!(result.contains(" bravo"), "missing context line 'bravo'");
    assert!(result.contains(" delta"), "missing context line 'delta'");
}

#[test]
fn unified_diff_identical_produces_empty() {
    let text = "same\nlines\n";
    let result = unified_diff(text, text);
    assert!(
        result.is_empty(),
        "identical inputs should produce empty diff"
    );
}
