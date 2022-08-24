//! Functions for rendering the "Log" page.

use std::io::Stdout;

use super::{
    colours::{ERROR, FOREGROUND_DIMMED},
    PRIMARY,
};
use crate::{
    cfg::Config,
    tui::state::{LogState, TuiState},
};
use quill_statement::{ObservedStatement, StatementCollection, StatementStatus};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Create a block to render the "Log" page.
fn log_widget<'a>(
    conf: &'a Config<'a>,
    acct_stmts: &'a StatementCollection,
    state: &LogState,
) -> (List<'a>, List<'a>) {
    let acct_names_ordered: Vec<ListItem> = conf
        .keys()
        .iter()
        .map(|a| ListItem::new(conf.accounts().get(a.as_str()).unwrap().name()))
        .collect();

    let mut accts = List::new(acct_names_ordered)
        .block(Block::default().title("Accounts").borders(Borders::ALL))
        .highlight_style(Style::default().bg(PRIMARY));

    // get the log of statements for the selected account
    let rows: Vec<ListItem> = match state.selected_account() {
        Some(acct_idx) => {
            // get the HashMap key of the account that's highlighted
            let acct_key = conf.keys()[acct_idx].as_str();
            // convert the statements into formatted Rows
            acct_stmts
                .get(acct_key)
                .unwrap()
                .iter()
                // go through in reverse chronological order so latest is at the top
                .rev()
                .map(stylize_obs_stmt)
                .collect()
        }
        // return the template table if no Account is selected
        // this should never happen
        None => vec![ListItem::new("There are no accounts")],
    };
    let mut log = List::new(rows)
        .block(Block::default().title("Statements").borders(Borders::ALL))
        .highlight_style(Style::default().bg(PRIMARY));

    // dim the side that is not selected
    if state.selected_log().is_some() {
        accts = accts.style(Style::default().add_modifier(Modifier::DIM));
        log = log.style(Style::default());
    } else {
        accts = accts.style(Style::default());
        log = log.style(Style::default().add_modifier(Modifier::DIM));
    }

    (accts, log)
}

/// Stylize the statement date strings in the log pane
fn stylize_obs_stmt(obs_stmt: &ObservedStatement) -> ListItem {
    // format the string to be printed
    let li_str = format!(
        "{} {}",
        obs_stmt.statement().date(),
        String::from(obs_stmt.status())
    );

    let mut li = ListItem::new(li_str);
    // style the string based on the statement's status
    match obs_stmt.status() {
        StatementStatus::Ignored => li = li.style(Style::default().fg(FOREGROUND_DIMMED)),
        StatementStatus::Missing => li = li.style(Style::default().fg(ERROR)),
        _ => {}
    };

    li
}

/// Render the body for the "Log" tab
pub fn log_body(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    conf: &Config,
    acct_stmts: &StatementCollection,
    state: &mut TuiState,
    area: &Rect,
) {
    // define side-by-side layout
    let log_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                // accounts column
                Constraint::Percentage(50),
                // log for the selected account
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(*area);

    let (left, right) = log_widget(conf, acct_stmts, state.log());

    f.render_stateful_widget(left, log_chunks[0], state.mut_log().mut_accounts());
    f.render_stateful_widget(right, log_chunks[1], state.mut_log().mut_log());
}
