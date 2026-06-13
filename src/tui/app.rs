use anyhow::Result;

use crate::diff::{ChangeRecord, ChangeType};
use crate::storage;

/// A summarised entry shown in the Timeline screen.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub diff_id: String,
    pub created_at: String,
    pub change_count: usize,
    pub short_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Timeline,
    DiffDetail,
    FileDiff,
}

pub struct App {
    pub screen: Screen,

    // Timeline state
    pub timeline: Vec<DiffEntry>,
    pub timeline_selected: usize,

    // DiffDetail state
    pub detail_changes: Vec<ChangeRecord>,
    pub detail_selected: usize,

    // FileDiff state
    pub file_lines: Vec<String>,
    pub file_scroll: usize,

    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::Timeline,
            timeline: Vec::new(),
            timeline_selected: 0,
            detail_changes: Vec::new(),
            detail_selected: 0,
            file_lines: Vec::new(),
            file_scroll: 0,
            should_quit: false,
        }
    }

    /// Load (or reload) the timeline from storage.
    pub fn load_timeline(&mut self) -> Result<()> {
        let ids = storage::list_diffs()?;
        self.timeline = ids
            .iter()
            .filter_map(|id| {
                storage::load_diff(id).ok().map(|ev| DiffEntry {
                    diff_id: id.clone(),
                    created_at: ev.created_at.clone(),
                    change_count: ev.changes.len(),
                    short_summary: ev.summary.as_ref().map(|s| s.short.clone()),
                })
            })
            .collect();
        // Clamp selection
        if self.timeline.is_empty() {
            self.timeline_selected = 0;
        } else if self.timeline_selected >= self.timeline.len() {
            self.timeline_selected = self.timeline.len() - 1;
        }
        Ok(())
    }

    /// Navigate into the next screen level.
    pub fn enter(&mut self) -> Result<()> {
        match self.screen {
            Screen::Timeline => {
                if self.timeline.is_empty() {
                    return Ok(());
                }
                let id = &self.timeline[self.timeline_selected].diff_id;
                let ev = storage::load_diff(id)?;
                self.detail_changes = ev.changes;
                self.detail_selected = 0;
                self.screen = Screen::DiffDetail;
            }
            Screen::DiffDetail => {
                if self.detail_changes.is_empty() {
                    return Ok(());
                }
                let change = &self.detail_changes[self.detail_selected];
                let raw = change.diff_text.clone().unwrap_or_default();
                self.file_lines = raw.lines().map(|l| l.to_string()).collect();
                self.file_scroll = 0;
                self.screen = Screen::FileDiff;
            }
            Screen::FileDiff => {} // already deepest level
        }
        Ok(())
    }

    /// Navigate back to the parent screen.
    pub fn back(&mut self) {
        match self.screen {
            Screen::DiffDetail => self.screen = Screen::Timeline,
            Screen::FileDiff => self.screen = Screen::DiffDetail,
            Screen::Timeline => {} // nowhere to go back
        }
    }

    /// Move selection / scroll down by one.
    pub fn next(&mut self) {
        match self.screen {
            Screen::Timeline => {
                if !self.timeline.is_empty() {
                    self.timeline_selected =
                        (self.timeline_selected + 1).min(self.timeline.len() - 1);
                }
            }
            Screen::DiffDetail => {
                if !self.detail_changes.is_empty() {
                    self.detail_selected =
                        (self.detail_selected + 1).min(self.detail_changes.len() - 1);
                }
            }
            Screen::FileDiff => {
                if !self.file_lines.is_empty() {
                    self.file_scroll = (self.file_scroll + 1).min(self.file_lines.len() - 1);
                }
            }
        }
    }

    /// Move selection / scroll up by one.
    pub fn prev(&mut self) {
        match self.screen {
            Screen::Timeline => {
                self.timeline_selected = self.timeline_selected.saturating_sub(1);
            }
            Screen::DiffDetail => {
                self.detail_selected = self.detail_selected.saturating_sub(1);
            }
            Screen::FileDiff => {
                self.file_scroll = self.file_scroll.saturating_sub(1);
            }
        }
    }

    /// Jump to the first item.
    pub fn go_top(&mut self) {
        match self.screen {
            Screen::Timeline => self.timeline_selected = 0,
            Screen::DiffDetail => self.detail_selected = 0,
            Screen::FileDiff => self.file_scroll = 0,
        }
    }

    /// Jump to the last item.
    pub fn go_bottom(&mut self) {
        match self.screen {
            Screen::Timeline => {
                if !self.timeline.is_empty() {
                    self.timeline_selected = self.timeline.len() - 1;
                }
            }
            Screen::DiffDetail => {
                if !self.detail_changes.is_empty() {
                    self.detail_selected = self.detail_changes.len() - 1;
                }
            }
            Screen::FileDiff => {
                if !self.file_lines.is_empty() {
                    self.file_scroll = self.file_lines.len() - 1;
                }
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Reload timeline data (r key).
    pub fn refresh(&mut self) -> Result<()> {
        self.load_timeline()
    }

    /// Symbol + label for a change type.
    pub fn change_symbol(ct: &ChangeType) -> &'static str {
        match ct {
            ChangeType::Added => "+",
            ChangeType::Removed => "-",
            ChangeType::Modified => "~",
            ChangeType::Renamed => "→",
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diff_entry(id: &str) -> DiffEntry {
        DiffEntry {
            diff_id: id.to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            change_count: 1,
            short_summary: None,
        }
    }

    fn make_change(ct: ChangeType) -> ChangeRecord {
        ChangeRecord {
            path: "/foo/bar.rs".to_string(),
            change_type: ct,
            old_hash: None,
            new_hash: None,
            old_size: None,
            new_size: None,
            diff_text: Some("@@ -1,1 +1,2 @@\n context\n+added\n".to_string()),
            renamed_from: None,
        }
    }

    // ── empty state ──────────────────────────────────────────────────────────

    #[test]
    fn empty_timeline_no_panic() {
        let mut app = App::new();
        assert!(app.timeline.is_empty());
        assert_eq!(app.screen, Screen::Timeline);

        // navigation on empty state must not panic
        app.next();
        app.prev();
        app.go_top();
        app.go_bottom();

        // enter on empty timeline is a no-op
        app.enter().unwrap();
        assert_eq!(app.screen, Screen::Timeline);
    }

    #[test]
    fn back_from_timeline_is_noop() {
        let mut app = App::new();
        app.back();
        assert_eq!(app.screen, Screen::Timeline);
    }

    // ── Timeline navigation ──────────────────────────────────────────────────

    #[test]
    fn timeline_next_prev_bounds() {
        let mut app = App::new();
        app.timeline = vec![
            make_diff_entry("a"),
            make_diff_entry("b"),
            make_diff_entry("c"),
        ];

        // starts at 0
        assert_eq!(app.timeline_selected, 0);

        app.next();
        assert_eq!(app.timeline_selected, 1);

        app.next();
        assert_eq!(app.timeline_selected, 2);

        // clamps at last
        app.next();
        assert_eq!(app.timeline_selected, 2);

        app.prev();
        assert_eq!(app.timeline_selected, 1);

        app.go_top();
        assert_eq!(app.timeline_selected, 0);

        app.go_bottom();
        assert_eq!(app.timeline_selected, 2);
    }

    #[test]
    fn prev_from_zero_saturates() {
        let mut app = App::new();
        app.timeline = vec![make_diff_entry("a")];
        app.timeline_selected = 0;
        app.prev();
        assert_eq!(app.timeline_selected, 0);
    }

    // ── Screen transitions ───────────────────────────────────────────────────

    #[test]
    fn detail_screen_transition_with_changes() {
        let mut app = App::new();
        app.timeline = vec![make_diff_entry("x")];
        app.detail_changes = vec![make_change(ChangeType::Added)];
        app.screen = Screen::DiffDetail;

        // Enter on DiffDetail → FileDiff
        app.enter().unwrap();
        assert_eq!(app.screen, Screen::FileDiff);
        assert!(!app.file_lines.is_empty(), "file_lines should be populated");

        // Esc back to DiffDetail
        app.back();
        assert_eq!(app.screen, Screen::DiffDetail);

        // Esc again back to Timeline
        app.back();
        assert_eq!(app.screen, Screen::Timeline);
    }

    #[test]
    fn enter_on_file_diff_is_noop() {
        let mut app = App::new();
        app.screen = Screen::FileDiff;
        app.enter().unwrap();
        assert_eq!(app.screen, Screen::FileDiff);
    }

    #[test]
    fn empty_detail_enter_is_noop() {
        let mut app = App::new();
        app.screen = Screen::DiffDetail;
        assert!(app.detail_changes.is_empty());
        app.enter().unwrap();
        assert_eq!(app.screen, Screen::DiffDetail);
    }

    // ── FileDiff scroll bounds ───────────────────────────────────────────────

    #[test]
    fn file_scroll_bounds() {
        let mut app = App::new();
        app.screen = Screen::FileDiff;
        app.file_lines = vec!["line1".into(), "line2".into(), "line3".into()];
        app.file_scroll = 0;

        app.next();
        assert_eq!(app.file_scroll, 1);
        app.next();
        assert_eq!(app.file_scroll, 2);
        // clamped
        app.next();
        assert_eq!(app.file_scroll, 2);

        app.prev();
        assert_eq!(app.file_scroll, 1);

        app.go_top();
        assert_eq!(app.file_scroll, 0);
        app.go_bottom();
        assert_eq!(app.file_scroll, 2);
    }

    // ── Quit ─────────────────────────────────────────────────────────────────

    #[test]
    fn quit_sets_flag() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    // ── Symbol helpers ───────────────────────────────────────────────────────

    #[test]
    fn change_symbols_correct() {
        assert_eq!(App::change_symbol(&ChangeType::Added), "+");
        assert_eq!(App::change_symbol(&ChangeType::Removed), "-");
        assert_eq!(App::change_symbol(&ChangeType::Modified), "~");
        assert_eq!(App::change_symbol(&ChangeType::Renamed), "→");
    }
}
