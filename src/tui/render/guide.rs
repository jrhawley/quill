//! Render the guide keys on the screen.

use super::colours::FOREGROUND_DIMMED;
use tui::{
    style::Style,
    symbols::line::VERTICAL,
    text::Spans,
    widgets::{Block, Tabs},
};

const GUIDE_KEYS: [&str; 4] = [
    "Next Tab [\u{21e5}]",
    "Prev Tab [\u{21e4}]",
    "Navigate [\u{2190}\u{2193}\u{2191}\u{2192}/hjkl]",
    "Quit [q]",
];

/// Render the key guide.
pub fn guide() -> Tabs<'static> {
    let guide_spans: Vec<Spans> = GUIDE_KEYS.iter().cloned().map(Spans::from).collect();
    Tabs::new(guide_spans)
        .block(Block::default())
        .style(Style::default().fg(FOREGROUND_DIMMED))
        .divider(VERTICAL)
}
