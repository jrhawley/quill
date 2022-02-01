//! Query all your bills and accounts to check on your financial statements.

use quill_statement::StatementCollection;

mod cli;
mod config;
mod models;
mod tui;
mod utils;

use crate::cli::cli_extract_cfg;
use crate::config::Config;
use crate::tui::{start_tui, stop_tui};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse and validate the CLI arguments
    let conf = cli_extract_cfg()?;

    // create a HashMap of all accounts and their statements
    let sc = StatementCollection::try_from(&conf)?;

    // start the TUI and run it
    let mut terminal = start_tui(&conf, &sc)?;

    // close everything down
    stop_tui(&mut terminal)
    // Ok(())
}
