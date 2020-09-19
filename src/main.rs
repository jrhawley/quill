use std::path::Path;

mod models;
mod config;
use config::parse;
use models::account::Account;
use clap::{App, Arg, SubCommand, crate_authors, crate_description, crate_version, crate_name};
use prettytable::{Table, row, cell, format};

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
        .subcommand(SubCommand::with_name("list")
            .about("List accounts and institutions from the config file")
        )
        .subcommand(SubCommand::with_name("next")
            .about("List upcoming bills from all accounts")
        )
        .get_matches();

    // 1. read account configuration
    // parse CLI args for config file
    let conf_path = matches.value_of("config").unwrap();
    let conf = parse(Path::new(conf_path));
    let mut none_missing: bool = true;

    // 2. Match subcommands, if available
    if let Some(_) = matches.subcommand_matches("list") {
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
    } else if let Some(_) = matches.subcommand_matches("next") {
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
    } else {
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
