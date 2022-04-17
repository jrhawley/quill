//! Functions for rendering the "Missing" page.

use quill_statement::{StatementCollection, StatementStatus};
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::cfg::Config;

/// Create a block to render the "Missing" page for account statements.
pub fn missing<'a>(conf: &'a Config<'a>, acct_stmts: &StatementCollection) -> List<'a> {
    // render list of accounts with missing statements
    let mut accts_with_missing: Vec<ListItem> = vec![];
    for acct_key in conf.keys() {
        let this_acct = conf.accounts().get(acct_key.as_str()).unwrap();
        let missing_stmts: Vec<ListItem> = acct_stmts
            .get(acct_key.as_str())
            .unwrap()
            .iter()
            .filter(|&obs_stmt| obs_stmt.status() == StatementStatus::Missing)
            .map(|obs_stmt| ListItem::new(format!("  {}", obs_stmt.statement().date())))
            .collect();
        if !missing_stmts.is_empty() {
            accts_with_missing.push(ListItem::new(this_acct.name()));
            for li in missing_stmts {
                accts_with_missing.push(li);
            }
        }
    }
    let accts_list = List::new(accts_with_missing)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default());
    accts_list
}
