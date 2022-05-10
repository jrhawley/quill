//! Display the upcoming statements for each account.

use chrono::NaiveDate;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::cfg::Config;

/// Create a block to render the "Upcoming" page for account statements.
pub fn upcoming<'a>(conf: &'a Config<'a>) -> List<'a> {
    // get the next statment date for each account
    let mut next_statements: Vec<(&str, NaiveDate)> = conf
        .accounts()
        .iter()
        .map(|(_, acct)| (acct.name(), acct.next_statement()))
        .collect();

    // sort them by date so that the next closest dates are at the beginning
    next_statements.sort_by(|a, b| a.1.cmp(&b.1));

    // convert items into `ListItem`s for display
    let next_stmt_items: Vec<ListItem> = next_statements
        .iter()
        .map(|(name, date)| ListItem::new(format!("{}  {}", date.format("%Y-%m-%d"), name)))
        .collect();

    // create the `List` that will be rendered by the TUI
    let accts_list = List::new(next_stmt_items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default());

    accts_list
}
