//! Functions for rendering the "Accounts" page.

use std::io::Stdout;

use super::{colours::BACKGROUND, PRIMARY};
use crate::{cfg::Config, tui::state::TuiState};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

/// Block for rendering "Accounts" page
fn accounts_widget<'a>(conf: &'a Config) -> Table<'a> {
    let accts: Vec<Row> = conf
        .keys()
        .iter()
        .map(|k| {
            let acct = conf.accounts().get(k).unwrap();
            Row::new(vec![
                acct.name(),
                acct.institution(),
                acct.directory().to_str().unwrap_or(""),
            ])
        })
        .collect();
    let acct_table = Table::new(accts)
        .header(
            Row::new(vec!["Account Name", "Institution", "Directory"]).style(
                Style::default()
                    .fg(PRIMARY)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        )
        .block(Block::default().borders(Borders::ALL))
        .widths(&[
            Constraint::Min(20),
            Constraint::Min(30),
            Constraint::Min(20),
        ])
        .column_spacing(2)
        .style(Style::default().bg(BACKGROUND))
        .highlight_style(Style::default().fg(BACKGROUND).bg(PRIMARY));
    acct_table
}

/// Render the body for the "Accounts" tab
pub fn accounts_body(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    conf: &Config,
    state: &mut TuiState,
    area: &Rect,
) {
    let widget = accounts_widget(conf);
    let widget_state = state.mut_accounts().mut_state();

    f.render_stateful_widget(widget, *area, widget_state);
}
