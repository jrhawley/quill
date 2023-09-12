//! Render the guide keys on the screen.

use super::colours::FOREGROUND_DIMMED;
use ratatui::{
    style::Style,
    symbols::line::VERTICAL,
    text::Line,
    widgets::{Block, Tabs},
};

const GUIDE_KEYS: [&str; 5] = [
    "Next Tab [\u{21e5}]",
    "Prev Tab [\u{21e4}]",
    "Navigate [\u{2190}\u{2193}\u{2191}\u{2192}/hjkl]",
    "Refresh [r]",
    "Quit [q]",
];

/// Render the key guide.
pub fn guide() -> Tabs<'static> {
    let guide_lines: Vec<Line> = GUIDE_KEYS.iter().cloned().map(Line::from).collect();
    Tabs::new(guide_lines)
        .block(Block::default())
        .style(Style::default().fg(FOREGROUND_DIMMED))
        .divider(VERTICAL)
}
