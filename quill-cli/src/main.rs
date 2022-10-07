//! Query all your bills and accounts to check on your financial statements.

use clap::Parser;
use cli::CliOpts;

mod cfg;
mod cli;
mod tui;

use crate::cfg::Config;
use crate::tui::{start_tui, stop_tui};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse and validate the CLI arguments
    let opts = CliOpts::parse();

    let conf = Config::try_from(opts)?;

    // start the TUI and run it
    let mut terminal = start_tui(&conf, &conf.statements())?;

    // close everything down
    stop_tui(&mut terminal)
}
