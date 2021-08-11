use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use std::collections::HashMap;
use std::env;
use std::path::Path;

mod config;
mod models;
mod tui;
use crate::config::{get_config_path, Config};
use crate::models::{date::Date, statement::Statement};
use crate::tui::start_tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get QUILL_CONFIG environment variable to find location of the default config file
    let conf_env_path = get_config_path();
    // CLI interface for binary
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONF")
                .help("The statement configuration file")
                .takes_value(true)
                .default_value(conf_env_path.to_str().unwrap()),
        )
        .get_matches();

    // 1. read account configuration
    // parse CLI args for config file
    let conf_path = matches.value_of("config").unwrap();
    let conf = Config::new(Path::new(conf_path));

    // get a sorted list of account keys
    let acct_order = conf.accounts_sorted().0;
    // create a HashMap of all accounts and their statements
    let mut acct_stmts: HashMap<&str, Vec<(Date, Option<Statement>)>> = HashMap::new();
    for (key, acct) in conf.accounts() {
        acct_stmts.insert(key.as_str(), acct.match_statements());
    }

    // 2. Set up TUI
    start_tui(&conf, &acct_stmts, &acct_order)
}
