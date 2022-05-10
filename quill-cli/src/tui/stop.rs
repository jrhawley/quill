//! Gracefully close down the terminal user interface.

use std::io::Stdout;

use crossterm::terminal::disable_raw_mode;
use tui::{backend::CrosstermBackend, Terminal};

/// Disable terminal raw mode, clear the screen, and restore the cursor.
pub fn stop_tui(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    term.clear()?;
    term.show_cursor()?;
    Ok(())
}
