//! The terminal user interface for quill.

use crate::Config;
use quill_statement::StatementStatus;

mod render;
mod start;
mod state;
mod stop;

pub use start::start_tui;
pub use stop::stop_tui;

/// Open a PDF statement with the operating system as a separate process.
fn open_stmt_external(conf: &Config, selected_acct: usize, selected_stmt: usize) {
    let (_, obs_stmt) = conf.get_account_statement(selected_acct, selected_stmt);

    if obs_stmt.status() == StatementStatus::Available {
        // open the statement with an external program
        open::that_in_background(obs_stmt.statement().path());
    }
}

/// Open a file explorer in the account's directory.
fn open_account_external(conf: &Config, selected_acct: usize) {
    let acct_key = conf.get_account_key(selected_acct);

    if let Some(acct) = conf.get_account(&acct_key) {
        // open the directory for the account
        open::that_in_background(acct.directory());
    }
}
