use std::path::Path;

mod models;
mod config;
use config::parse;
use clap::{App, Arg, SubCommand, crate_authors, crate_description, crate_version, crate_name};

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
        // collect institution names
        let mut inst_names = conf.institutions()
            .iter()
            .map(|(_, inst)| inst.name())
            .collect::<Vec<&str>>();
        // sort the vector for better display
        inst_names.sort();
        // print them one-by-one
        for inst in inst_names {
            println!("\t{}", inst);
        }
        // repeat the above with all accounts
        println!("\nAccounts:");
        // collect institution names
        let mut acct_names = conf.accounts()
            .iter()
            .map(|(_, acct)| acct.name())
            .collect::<Vec<&str>>();
        // sort the vector for better display
        acct_names.sort();
        // print them one-by-one
        for acct in acct_names {
            println!("\t{}", acct);
        }
    } else {
        // default to showing missing statements if no subcommand given
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
}
