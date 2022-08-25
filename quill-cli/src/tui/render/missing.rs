//! Functions for rendering the "Missing" page.

use super::colours::FOREGROUND_DIMMED;
use crate::{cfg::Config, tui::state::TuiState};
use quill_statement::{ObservedStatement, StatementCollection, StatementStatus};
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Create a block to render the "Missing" page for account statements.
fn missing_widget<'a>(conf: &'a Config<'a>, acct_stmts: &'a StatementCollection) -> List<'a> {
    // render list of accounts with missing statements
    let mut accts_with_missing: Vec<ListItem> = vec![];
    for acct_key in conf.keys() {
        let this_acct = conf.accounts().get(acct_key.as_str()).unwrap();
        let missing_stmts: Vec<ListItem> = acct_stmts
            .get(acct_key.as_str())
            .unwrap()
            .iter()
            .filter(|&obs_stmt| obs_stmt.status() == StatementStatus::Missing)
            .map(stylize_missing_stmt)
            .collect();

        if !missing_stmts.is_empty() {
            accts_with_missing.push(ListItem::new(this_acct.name()));
            for li in missing_stmts {
                accts_with_missing.push(li);
            }
        }
    }

    // tell the user that there are no missing statements
    if accts_with_missing.is_empty() {
        accts_with_missing.push(
            // dim the colour so it displays differently than when accounts have missing statements
            ListItem::new("No missing statements").style(Style::default().fg(FOREGROUND_DIMMED)),
        );
    }

    let accts_list = List::new(accts_with_missing)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default());

    accts_list
}

/// Stylize the observed statement
fn stylize_missing_stmt(obs_stmt: &ObservedStatement) -> ListItem {
    ListItem::new(format!("  {}", obs_stmt.statement().date()))
}

/// Render the body for the "Missing" tab
pub fn missing_body(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    conf: &Config,
    acct_stmts: &StatementCollection,
    state: &mut TuiState,
    area: &Rect,
) {
    let widget = missing_widget(conf, acct_stmts);
    let widget_state = state.mut_missing().mut_state();
    f.render_stateful_widget(widget, *area, widget_state);
}
