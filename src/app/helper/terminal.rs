use std::io::{self, Stdout};

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal as TuiTerminal, backend::CrosstermBackend};

pub type Terminal = TuiTerminal<CrosstermBackend<Stdout>>;

/// Initialize and return a crossterm terminal instance
pub fn init_terminal() -> Result<Terminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).context("Failed to initialize terminal")
}

/// Restore the terminal to its previous "normal" state.
pub fn restore_terminal(terminal: &mut Terminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
