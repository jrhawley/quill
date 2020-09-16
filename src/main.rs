use std::path::Path;

mod models;
mod config;
use config::parse;
use clap::{App, Arg, crate_authors, crate_description, crate_version, crate_name};

fn main() {
    // CLI interface for binary
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("The statement configuration file")
            .takes_value(true)
            .default_value("config.toml")
        )
        .get_matches();

    // 1. read account configuration
    // parse CLI args for config file
    let conf_path = matches.value_of("config").unwrap();
    let conf = parse(Path::new(conf_path));
    let mut none_missing: bool = true;

    // 2. check each account
    for (_, acct) in conf.accounts() {
        let missing = acct.missing_statements();
        // see if there are any missing statements
        if missing.len() > 0 {
            none_missing = false;
            println!("{}: {:?}", acct.name, missing);
        }
    }
    if none_missing {
        eprintln!("No missing statements.")
    }
}
