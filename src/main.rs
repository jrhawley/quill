use cli::cli_extract_cfg;
use log::error;
use simple_logger;
use std::collections::HashMap;
use std::process;

mod cli;
mod config;
mod models;
mod tui;

use crate::config::Config;
use crate::models::{Date, Statement};
use crate::tui::start_tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initiate the log level
    simple_logger::init()?;
    
    // parse and validate the CLI arguments
    let conf = cli_extract_cfg()?;

    // get a sorted list of account keys
    let acct_order = conf.accounts_sorted().0;
    // create a HashMap of all accounts and their statements
    let mut acct_stmts: HashMap<&str, Vec<(Date, Option<Statement>)>> = HashMap::new();
    for (key, acct) in conf.accounts() {
        let matched_stmts = match acct.match_statements() {
            Ok(stmts) => stmts,
            Err(e) => {
                error!(
                    "{}: {}. {}:\n\t{}\n{}",
                    "Could not match statements from account",
                    key,
                    "The following error occurred",
                    e,
                    "Please check the directories in your configuration file are correct."
                );
                process::exit(1);
            }
        };
        acct_stmts.insert(key.as_str(), matched_stmts);
    }

    // 2. Set up TUI
    start_tui(&conf, &acct_stmts, &acct_order)
}
