//! Functions for rendering the "Accounts" page.

use super::{colours::BACKGROUND, PRIMARY};
use crate::cfg::Config;
use tui::{
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table},
};

/// Block for rendering "Accounts" page
pub fn accounts<'a>(conf: &'a Config) -> Table<'a> {
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
        .highlight_style(Style::default().bg(PRIMARY));
    acct_table
}
