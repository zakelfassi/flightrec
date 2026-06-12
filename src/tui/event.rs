use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use super::app::App;

/// Block up to `timeout` waiting for a key event. Returns `true` if an event
/// was handled (caller should re-render), `false` on timeout or unhandled key.
pub fn handle_events(app: &mut App) -> Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            // Ignore key-release events (crossterm sometimes emits them).
            use crossterm::event::KeyEventKind;
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            match key.code {
                // Quit anywhere
                KeyCode::Char('q') => app.quit(),
                // Ctrl-C safety valve
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),

                // Navigation down
                KeyCode::Char('j') | KeyCode::Down => app.next(),

                // Navigation up
                KeyCode::Char('k') | KeyCode::Up => app.prev(),

                // Jump top / bottom
                KeyCode::Char('g') => app.go_top(),
                KeyCode::Char('G') => app.go_bottom(),

                // Drill in
                KeyCode::Enter => app.enter()?,

                // Back
                KeyCode::Esc | KeyCode::Backspace => app.back(),

                // Refresh timeline
                KeyCode::Char('r') => app.refresh()?,

                _ => return Ok(false),
            }
            return Ok(true);
        }
    }
    Ok(false)
}
