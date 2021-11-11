//! The TUI for quill.

use crate::{Config, models::{StatementCollection, StatementStatus}};

mod render;
mod start;
mod state;
mod stop;

pub use start::start_tui;
pub use stop::stop_tui;

/// Open a PDF statement with the operating system as a separate process
fn open_stmt_external(
    conf: &Config,
    acct_stmts: &StatementCollection,
    selected_acct: usize,
    selected_stmt: usize,
) {
    // get the key for the selected account
    let acct_name = conf.keys()[selected_acct].as_str();
    // construct the path to the statement file
    let obs_stmt = acct_stmts
        .get(acct_name)
        .unwrap()
        .iter()
        .rev()
        .nth(selected_stmt)
        .unwrap();

    if obs_stmt.status() == StatementStatus::Available {
        // open the statement with an external program
        open::that_in_background(obs_stmt.statement().path());
    }
}

/// Open a PDF statement with the operating system as a separate process
fn open_account_external<'a>(conf: &'a Config, selected_acct: usize) {
    let acct_name = conf.keys()[selected_acct].as_str();
    if let Some(acct) = conf.accounts().get(acct_name) {
        // open the directory for the account
        open::that_in_background(acct.directory());
    }
}
