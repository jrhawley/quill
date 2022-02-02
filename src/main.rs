//! Query all your bills and accounts to check on your financial statements.

use cli::CliOpts;
use quill_statement::StatementCollection;
use structopt::StructOpt;

mod cli;
mod config;
mod tui;

use crate::config::Config;
use crate::tui::{start_tui, stop_tui};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse and validate the CLI arguments
    let opts = CliOpts::from_args_safe()?;

    let conf = Config::try_from(opts)?;

    // create a HashMap of all accounts and their statements
    let sc = StatementCollection::try_from(&conf)?;

    // start the TUI and run it
    let mut terminal = start_tui(&conf, &sc)?;

    // close everything down
    stop_tui(&mut terminal)
    // Ok(())
}
