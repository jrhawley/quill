//! Query all your bills and accounts to check on your financial statements.

use clap::Parser;
use cli::CliOpts;
use quill_statement::StatementCollection;

mod cfg;
mod cli;
mod tui;

use crate::cfg::Config;
use crate::tui::{start_tui, stop_tui};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse and validate the CLI arguments
    let opts = CliOpts::parse();

    let conf = Config::try_from(opts)?;

    // create a HashMap of all accounts and their statements
    let sc = StatementCollection::try_from(&conf)?;

    // start the TUI and run it
    let mut terminal = start_tui(&conf, &sc)?;

    // close everything down
    stop_tui(&mut terminal)
}
