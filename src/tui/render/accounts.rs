//! Functions for rendering the "Accounts" page.

use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
};

use crate::cfg::Config;

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
                    .fg(Color::Yellow)
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
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default().bg(Color::Blue));
    acct_table
}
