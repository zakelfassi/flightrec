pub mod app;
mod event;
mod ui;

pub use app::App;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::panic;

/// Minimal RAII guard that disables raw mode on drop.
/// Used during `TerminalGuard` construction to ensure raw mode is restored
/// if any later setup step (alternate screen, terminal init) fails.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// RAII guard that calls LeaveAlternateScreen on drop.
/// Created immediately after EnterAlternateScreen succeeds so that any
/// subsequent setup failure (e.g. Terminal::new) leaves the alternate screen
/// rather than leaving the terminal stuck there.
struct AltScreenGuard;

impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// RAII guard that restores the terminal on drop (including panics).
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        // Guard raw mode immediately; dropped (restoring raw mode) on any failure below.
        let raw_guard = RawModeGuard;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        // Guard alternate screen now that it has been entered; dropped (leaving alternate
        // screen) if Terminal::new or any later step fails.
        let alt_guard = AltScreenGuard;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        // All setup succeeded — TerminalGuard takes over cleanup responsibility.
        std::mem::forget(raw_guard);
        std::mem::forget(alt_guard);
        Ok(TerminalGuard { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort: ignore errors during cleanup.
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Main TUI entry point.
pub fn run() -> Result<()> {
    // Install a panic hook that restores the terminal before printing the panic
    // message, so the user's shell is not left in raw mode.
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let mut guard = TerminalGuard::new()?;
    let mut app = App::new();
    app.load_timeline()?;

    loop {
        guard.terminal.draw(|f| ui::render(f, &app))?;

        event::handle_events(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    /// Helper that appends a label to a shared log on drop, letting tests
    /// verify cleanup ordering without involving real terminal state.
    struct RecordingGuard {
        log: Arc<Mutex<Vec<&'static str>>>,
        label: &'static str,
    }

    impl Drop for RecordingGuard {
        fn drop(&mut self) {
            self.log.lock().unwrap().push(self.label);
        }
    }

    /// Verifies the two-guard setup pattern: when a simulated setup failure
    /// occurs after both guards are active, they are cleaned up in reverse
    /// declaration order (alt-screen first, then raw-mode).
    #[test]
    fn setup_guards_clean_up_in_reverse_order_on_failure() {
        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

        let result: Result<(), &str> = {
            let _raw = RecordingGuard {
                log: log.clone(),
                label: "raw",
            };
            let _alt = RecordingGuard {
                log: log.clone(),
                label: "alt",
            };
            // Simulate a setup failure (e.g. Terminal::new returns Err).
            Err("setup failed")
        };

        assert!(result.is_err());
        // Rust drops locals in reverse declaration order: _alt then _raw.
        assert_eq!(*log.lock().unwrap(), vec!["alt", "raw"]);
    }

    /// Verifies that when setup succeeds (both guards forgotten), no cleanup
    /// entries are recorded — responsibility transfers to TerminalGuard.
    #[test]
    fn setup_guards_skipped_when_forgotten() {
        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

        {
            let raw = RecordingGuard {
                log: log.clone(),
                label: "raw",
            };
            let alt = RecordingGuard {
                log: log.clone(),
                label: "alt",
            };
            std::mem::forget(raw);
            std::mem::forget(alt);
        }

        assert!(log.lock().unwrap().is_empty());
    }
}
