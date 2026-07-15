//! Terminal lifecycle and the panic hook (`docs/design/tui-client.md` § Terminal
//! hygiene; `docs/ops/testing-strategy.md` scenario 8).
//!
//! A panic must leave the terminal usable — otherwise a crash reads as an
//! instant-uninstall bug. The panic hook restores the terminal (leave alt-screen,
//! show cursor, disable raw mode) *before* the default hook prints the trace.

use std::io::{self, Stdout};

use anyhow::Result;
use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

/// The concrete terminal type the app renders into.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Enter raw mode + the alternate screen and install the restoring panic hook.
pub fn enter() -> Result<Tui> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    install_panic_hook();
    let terminal = Terminal::new(CrosstermBackend::new(out))?;
    Ok(terminal)
}

/// Restore the terminal to its pre-launch state. Safe to call more than once.
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    Ok(())
}

/// Wrap the current panic hook so the terminal is restored before the trace prints.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort restore; ignore errors — we're already panicking.
        let _ = restore();
        previous(info);
    }));
}
