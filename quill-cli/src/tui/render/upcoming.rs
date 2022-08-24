//! Display the upcoming statements for each account.

use std::io::Stdout;

use chrono::NaiveDate;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{cfg::Config, tui::state::TuiState};

/// Create a block to render the "Upcoming" page for account statements.
fn upcoming_widget<'a>(conf: &'a Config<'a>) -> List<'a> {
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

/// Render the body for the "Upcoming" tab
pub fn upcoming_body(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    conf: &Config,
    state: &mut TuiState,
    area: &Rect,
) {
    let widget = upcoming_widget(conf);
    let widget_state = state.mut_missing().mut_state();

    f.render_stateful_widget(widget, *area, widget_state);
}
