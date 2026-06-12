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

/// RAII guard that restores the terminal on drop (including panics).
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        // Immediately guard raw mode so any subsequent failure restores it.
        let raw_guard = RawModeGuard;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        // All setup succeeded — TerminalGuard takes over cleanup responsibility.
        std::mem::forget(raw_guard);
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
