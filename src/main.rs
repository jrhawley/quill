use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use log::error;
use simple_logger;
use std::collections::HashMap;
use std::path::Path;
use std::{env, process};

mod config;
mod models;
mod tui;
use crate::config::{config::Config, utils::get_config_path};
use crate::models::{date::Date, statement::Statement};
use crate::tui::start_tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initiate the log level
    simple_logger::init().unwrap();
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
    let conf = match Config::new_from_path(Path::new(conf_path)) {
        Ok(cfg) => cfg,
        Err(_) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Error parsing configuration file.",
            )))
        }
    };
    println!("{:#?}", conf);

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
