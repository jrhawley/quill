//! Handle tab navigation within the TUI.

use super::{colours::BACKGROUND, step, PRIMARY};
use tui::{
    style::{Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Borders, Tabs},
};

/// The page selected from the tab menu.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum MenuItem {
    Missing,
    Upcoming,
    Log,
    Accounts,
}

const N_MENU_ITEMS: usize = 4;

impl MenuItem {
    /// Switch from one MenuItem to an adjacent one by a given step size
    fn step(&self, n: usize, positive: bool) -> Self {
        MenuItem::from(step(N_MENU_ITEMS, *self as usize, n, positive))
    }

    /// Set the MenuItem as its immediately next neighbour
    pub(crate) fn next(&mut self) {
        *self = self.step(1, true);
    }

    /// Set the MenuItem as its immediately next neighbour
    pub(crate) fn prev(&mut self) {
        *self = self.step(1, false);
    }
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Missing => 0,
            MenuItem::Upcoming => 1,
            MenuItem::Log => 2,
            MenuItem::Accounts => 3,
        }
    }
}

impl From<usize> for MenuItem {
    fn from(input: usize) -> MenuItem {
        match input {
            0 => MenuItem::Missing,
            1 => MenuItem::Upcoming,
            2 => MenuItem::Log,
            3 => MenuItem::Accounts,
            _ => MenuItem::Missing,
        }
    }
}

impl Default for MenuItem {
    fn default() -> Self {
        MenuItem::Missing
    }
}

/// Create a stylized Span for a selected MenuItem.
pub fn tabs(selected: MenuItem) -> Tabs<'static> {
    let menu_titles = vec!["[1] Missing", "[2] Upcoming", "[3] Log", "[4] Accounts"];
    let menu_title_spans: Vec<Spans> = menu_titles.iter().cloned().map(Spans::from).collect();

    // convert tab menu items into spans to be rendered
    Tabs::new(menu_title_spans)
        .select(selected.into())
        .block(Block::default().title("Tabs").borders(Borders::ALL))
        .style(Style::default().bg(BACKGROUND))
        .highlight_style(Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD))
        .divider(DOT)
}
