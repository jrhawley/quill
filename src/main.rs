use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use prettytable::{cell, format, row, Table};
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use tui::widgets::{Block, Borders, ListItem, Tabs, Widget};
use tui::Terminal;
use tui::{backend::CrosstermBackend, style::Style, symbols::DOT, text::Spans};
use tui::{
    layout::{Constraint, Direction, Layout},
    widgets::List,
};

mod config;
mod models;
mod paging;
use config::Config;
use models::account::Account;
use models::date::Date;
use paging::log_account_dates;

fn main() -> Result<(), io::Error> {
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
        .get_matches();

    // 1. read account configuration
    // parse CLI args for config file

    let conf_path = matches.value_of("config").unwrap();
    let conf = Config::new(Path::new(conf_path));

    // get the accounts and sort them by their key name
    let accts = conf.accounts();
    // get missing statements for each account
    let missing_stmts: HashMap<&str, Vec<Date>> = accts
        .values()
        .map(|a| (a.name(), a.missing_statements()))
        .filter(|(k, v)| v.len() > 0)
        .collect();

    // 2. Set up TUI
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        // get terminal window dimensions
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    // tab row
                    Constraint::Length(3),
                    // body
                    Constraint::Percentage(90),
                    // footer and keyboard options
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(f.size());

        // render tabs
        let titles = ["Missing", "Log", "Accounts"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().title("Tabs").borders(Borders::ALL))
            .style(Style::default())
            .highlight_style(Style::default())
            .divider(DOT);
        f.render_widget(tabs, chunks[0]);

        // render list of accounts with missing statements
        let accts_with_missing: Vec<ListItem> = missing_stmts
            .iter()
            .map(|(&a, _)| {
                ListItem::new(a.to_string())
                // let missing_dates = v
                //     .iter()
                //     .map(|d| ListItem::new(d.to_string()).collect::<Vec<String>>());
                // combined_v.append(missing_dates)
            })
            .collect();
        let accts_list = List::new(accts_with_missing)
            .block(Block::default().title("Accounts").borders(Borders::ALL))
            .style(Style::default())
            .highlight_style(Style::default());
        f.render_widget(accts_list, chunks[1]);
    })
}

// fn main() {
//
//         .subcommand(
//             SubCommand::with_name("list")
//                 .visible_alias("ls")
//                 .about("List accounts and institutions from the config file"),
//         )
//         .subcommand(
//             SubCommand::with_name("next")
//                 .visible_alias("n")
//                 .about("List upcoming bills from all accounts"),
//         )
//         .subcommand(
//             SubCommand::with_name("prev")
//                 .visible_alias("p")
//                 .about("List most recent bills from all accounts"),
//         )
//         .subcommand(
//             SubCommand::with_name("log")
//                 .visible_alias("l")
//                 .about("Show a history log from a given account")
//                 .arg(
//                     Arg::with_name("account")
//                         .value_name("ACCT")
//                         .takes_value(true)
//                         .required(true)
//                         .help("The account whose history to show"),
//                 ),
//         )
//         .get_matches();

//     let mut none_missing: bool = true;

//     // 2. Match subcommands, if available
//     match matches.subcommand_name() {
//         Some("list") => {
//             println!("Configuration file:\n\t{}\n", conf_path);

//             // create a table using prettytable
//             let mut display_table = Table::new();
//             // hide extra lines in the table
//             display_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
//             // add headers to the columns
//             display_table.set_titles(row!["Key", "Institution"]);

//             // add info as a row to the display table
//             for k in &conf.institutions_sorted() {
//                 let inst = insts.get(k).unwrap();
//                 display_table.add_row(row![k, inst.name()]);
//             }
//             // print the table to STDOUT
//             display_table.printstd();
//             println!("\n");

//             // create a table using prettytable
//             let mut display_table = Table::new();
//             // hide extra lines in the table
//             display_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
//             // add headers to the columns
//             display_table.set_titles(row!["Key", "Account", "Institution"]);

//             // get the accounts and sort them by their key name
//             let accts = conf.accounts();
//             let mut acct_keys: Vec<String> = conf.accounts().keys().cloned().collect();
//             acct_keys.sort();
//             // add info as a row to the display table
//             for k in &acct_keys {
//                 let acct = accts.get(k).unwrap();
//                 display_table.add_row(row![k, acct.name(), acct.institution()]);
//             }
//             // print the table to STDOUT
//             display_table.printstd();
//         }
//         Some("next") => {
//             // create a table using prettytable
//             let mut display_table = Table::new();
//             // hide extra lines in the table
//             display_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
//             // add headers to the columns
//             display_table.set_titles(row!["Account", "Institution", "Next Bill"]);

//             // get the accounts and sort them by the next statement date
//             // (this involves caluclating next_statement() twice, but I'm not too concerned about that)
//             let mut accts: Vec<&Account> = conf.accounts().iter().map(|(_, acct)| acct).collect();
//             accts.sort_by_key(|a| a.next_statement());
//             // add each triple as a row to the display table
//             for acct in accts {
//                 display_table.add_row(row![acct.name(), acct.institution(), acct.next_statement()]);
//             }
//             // print the table to STDOUT
//             display_table.printstd();
//         }
//         Some("prev") => {
//             // create a table using prettytable
//             let mut display_table = Table::new();
//             // hide extra lines in the table
//             display_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
//             // add headers to the columns
//             display_table.set_titles(row!["Account", "Institution", "Most Recent Bill"]);

//             // get the accounts and sort them by the next statement date
//             // (this involves caluclating next_statement() twice, but I'm not too concerned about that)
//             let mut accts: Vec<&Account> = conf.accounts().iter().map(|(_, acct)| acct).collect();
//             accts.sort_by_key(|a| a.prev_statement());
//             // add each triple as a row to the display table
//             for acct in accts {
//                 display_table.add_row(row![acct.name(), acct.institution(), acct.prev_statement()]);
//             }
//             // print the table to STDOUT
//             display_table.printstd();
//         }
//         Some("log") => {
//             let submatches = matches.subcommand_matches("log").unwrap();
//             let selected_acct = submatches.value_of("account").unwrap();
//             // search for the account by name/key
//             let acct = conf.query_account(selected_acct);
//             // print a log of the account dates if found, or an error if not
//             match acct {
//                 Some(a) => log_account_dates(a),
//                 None => eprintln!("The account '{}' is not listed in the configuration. Please specify a different config file or add a new account.", selected_acct)
//             }
//         }
//         // clap handles this case with suggestions for typos
//         // leaving this branch in the match statement for completeness
//         Some(_) => {}
//         None => {
//             // default to showing missing statements if no subcommand given
//             for (_, acct) in conf.accounts() {
//                 let missing = acct.missing_statements();
//                 // see if there are any missing statements
//                 if missing.len() > 0 {
//                     none_missing = false;
//                     println!("{}: {:?}", acct.name(), missing);
//                 }
//             }
//             if none_missing {
//                 eprintln!("No missing statements.")
//             }
//         }
//     }
// }
