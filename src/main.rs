use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use prettytable::{cell, format, row, Table};
use std::env;
use std::path::Path;

mod config;
mod models;
use config::Config;
use models::account::Account;

fn main() {
    // get QUILL_CONFIG environment variable to find location of the default config file
    let conf_env_path = match env::var("QUILL_CONFIG") {
        Ok(p) => p,
        Err(_) => String::from("config.toml"),
    };

    // CLI interface for binary
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONF")
                .help("The statement configuration file")
                .takes_value(true)
                .default_value(&conf_env_path),
        )
        .subcommand(
            SubCommand::with_name("list")
                .visible_alias("ls")
                .about("List accounts and institutions from the config file"),
        )
        .subcommand(
            SubCommand::with_name("next")
                .visible_alias("n")
                .about("List upcoming bills from all accounts"),
        )
        .subcommand(
            SubCommand::with_name("prev")
                .visible_alias("p")
                .about("List most recent bills from all accounts"),
        )
        .subcommand(
            SubCommand::with_name("log")
                .visible_alias("l")
                .about("Show a history log from a given account")
                .arg(
                    Arg::with_name("account")
                        .value_name("ACCT")
                        .takes_value(true)
                        .required(true)
                        .help("The account whose history to show"),
                ),
        )
        .get_matches();

    // 1. read account configuration
    // parse CLI args for config file
    let conf_path = matches.value_of("config").unwrap();
    let conf = Config::new(Path::new(conf_path));
    let mut none_missing: bool = true;

    // 2. Match subcommands, if available
    match matches.subcommand_name() {
        Some("list") => {
            println!("Configuration file:\n\t{}", conf_path);
            println!("\nInstitutions:");
            // get institution names, sorted
            let inst_names = conf.institutions_sorted();
            // print them one-by-one
            for inst in inst_names {
                println!("\t{}", inst);
            }
            // repeat the above with all accounts
            println!("\nAccounts:");
            let acct_names = conf.accounts_sorted();
            for acct in acct_names {
                println!("\t{}", acct);
            }
        }
        Some("next") => {
            // create a table using prettytable
            let mut display_table = Table::new();
            // hide extra lines in the table
            display_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            // add headers to the columns
            display_table.set_titles(row!["Account", "Institution", "Next Bill"]);

            // get the accounts and sort them by the next statement date
            // (this involves caluclating next_statement() twice, but I'm not too concerned about that)
            let mut accts: Vec<&Account> = conf.accounts().iter().map(|(_, acct)| acct).collect();
            accts.sort_by_key(|a| a.next_statement());
            // add each triple as a row to the display table
            for acct in accts {
                display_table.add_row(row![acct.name(), acct.institution(), acct.next_statement()]);
            }
            // print the table to STDOUT
            display_table.printstd();
        }
        Some("prev") => {
            // create a table using prettytable
            let mut display_table = Table::new();
            // hide extra lines in the table
            display_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            // add headers to the columns
            display_table.set_titles(row!["Account", "Institution", "Most Recent Bill"]);

            // get the accounts and sort them by the next statement date
            // (this involves caluclating next_statement() twice, but I'm not too concerned about that)
            let mut accts: Vec<&Account> = conf.accounts().iter().map(|(_, acct)| acct).collect();
            accts.sort_by_key(|a| a.prev_statement());
            // add each triple as a row to the display table
            for acct in accts {
                display_table.add_row(row![acct.name(), acct.institution(), acct.prev_statement()]);
            }
            // print the table to STDOUT
            display_table.printstd();
        }
        Some("log") => {
            let submatches = matches.subcommand_matches("log").unwrap();
            let selected_acct = submatches.value_of("account").unwrap();
            match conf.accounts().get(selected_acct) {
                Some(acct) => {
                    for stmt in acct.statement_dates() {
                        println!("{}", stmt);
                    }
                }
                None => eprintln!("The account '{}' is not listed in the configuration. Please specify a different config file or add a new account.", selected_acct)
            };
        }
        // clap handles this case with suggestions for typos
        // leaving this branch in the match statement for completeness
        Some(_) => {}
        None => {
            // default to showing missing statements if no subcommand given
            for (_, acct) in conf.accounts() {
                let missing = acct.missing_statements();
                // see if there are any missing statements
                if missing.len() > 0 {
                    none_missing = false;
                    println!("{}: {:?}", acct.name(), missing);
                }
            }
            if none_missing {
                eprintln!("No missing statements.")
            }
        }
    }
}
