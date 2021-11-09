//! The TUI for quill.
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, io};
use std::{io::Stdout, sync::mpsc::channel};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::{line::VERTICAL, DOT},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState, Row, Table, TableState, Tabs},
    Terminal,
};

use crate::{models::StatementCollection, Config};

mod render;
mod start;
mod state;
mod stop;

pub use start::start_tui;
pub use stop::stop_tui;

pub fn start_tui(
    conf: &Config,
    acct_stmts: &HashMap<&str, Vec<(Date, Option<Statement>)>>,
    acct_order: &Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure TUI
    // -------------------------------------------
    // enable raw mode to avoid waiting for ENTER to respond to keystrokes
    enable_raw_mode().expect("can run in raw mode");
    // set up a multi-producer single consumer channel to communicate between the input handler and the TUI rendering loop
    let (tx, rx) = channel();
    // 200 ms delay between refreshes
    let tick_rate = Duration::from_millis(200);
    // start the threading
    thread::spawn(move || {
        // record the time of the last Tick sent
        let mut last_tick = Instant::now();
        loop {
            // set a polling period to accept an input event from the user
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // poll the user for the given time, and if there is an input event, return it
            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    tx.send(UserEvent::Input(key)).expect("can send events");
                }
            }

            // if enough time has elapsed, return a Tick, since no Input has been triggered
            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(UserEvent::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    // Initialize the TUI to send to STDOUT
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear()?;

    // persistent states for each tab
    let mut state_missing = ListState::default();
    let mut state_log_accounts = ListState::default();
    let mut state_log_log = ListState::default();
    let mut state_accounts = TableState::default();
    state_missing.select(Some(0));
    if conf.accounts().len() > 0 {
        state_log_accounts.select(Some(0));
        state_accounts.select(Some(0));
    } else {
        state_accounts.select(None);
        state_log_accounts.select(None);
    }
    state_log_log.select(None);

    loop {
        terminal.draw(|f| {
            // get terminal window dimensions
            let size = f.size();
            // draw a full black rectangle to hide everything
            f.render_widget(
                Block::default().style(Style::default().bg(Color::Black)),
                size,
            );
            // define initial layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        // tab row
                        Constraint::Length(3),
                        // body
                        Constraint::Length(size.height - 6),
                        // footer
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // convert tab menu items into spans to be rendered
            let menu_title_spans: Vec<Spans> =
                menu_titles.iter().cloned().map(Spans::from).collect();
            let tabs = Tabs::new(menu_title_spans)
                .select(active_menu_item.into())
                .block(Block::default().title("Tabs").borders(Borders::ALL))
                .style(Style::default().bg(Color::Black))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .divider(DOT);
            f.render_widget(tabs, chunks[0]);

            // render the main block depending on what tab is selected
            match active_menu_item {
                MenuItem::Missing => {
                    f.render_stateful_widget(
                        render_missing(conf, &acct_stmts, &acct_order),
                        chunks[1],
                        &mut state_missing,
                    );
                }
                MenuItem::Log => {
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
                        .split(chunks[1]);
                    let (left, right) = render_log(
                        conf,
                        acct_stmts,
                        acct_order,
                        &state_log_accounts,
                        &state_log_log,
                    );
                    f.render_stateful_widget(left, log_chunks[0], &mut state_log_accounts);
                    f.render_stateful_widget(right, log_chunks[1], &mut state_log_log);
                }
                MenuItem::Accounts => {
                    f.render_stateful_widget(
                        render_accounts(conf, acct_order),
                        chunks[1],
                        &mut state_accounts,
                    );
                }
            };

            // render the key guide at the bottom
            let guide_keys = vec![
                "Next Tab [\u{21e5}]",
                "Prev Tab [\u{21e4}]",
                "Navigate [\u{2190}\u{2193}\u{2191}\u{2192}/hjkl]",
                "Quit [q]",
            ];
            let guide_tabs: Vec<Spans> =
                guide_keys.iter().cloned().map(|k| Spans::from(k)).collect();
            let guide = Tabs::new(guide_tabs)
                .block(Block::default())
                .style(Style::default())
                .divider(VERTICAL);
            f.render_widget(guide, chunks[2]);
        })?;

        // receive input from the user about what to do next
        match rx.recv()? {
            // destruct KeyCode and KeyModifiers for more legible match cases
            UserEvent::Input(KeyEvent { code, modifiers }) => match (code, modifiers) {
                // Quit
                (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    close_tui(&mut terminal)?;
                    break;
                }
                // Tab to move forward one tab
                (KeyCode::Tab, _) => {
                    let modulo = menu_titles.len();
                    let mut tab_val = active_menu_item as usize;
                    tab_val = (tab_val + 1) % modulo;
                    active_menu_item = MenuItem::from(tab_val)
                }
                // Shift + Tab to move backward one tab
                (KeyCode::BackTab, _) => {
                    let modulo = menu_titles.len();
                    let mut tab_val = active_menu_item as usize;
                    // this modular arithmetic has to be a bit tricker to deal with -1
                    tab_val = (tab_val + modulo - 1) % modulo;
                    active_menu_item = MenuItem::from(tab_val)
                }
                (KeyCode::Char('1'), _) => active_menu_item = MenuItem::Missing,
                (KeyCode::Char('2'), _) => active_menu_item = MenuItem::Log,
                (KeyCode::Char('3'), _) => active_menu_item = MenuItem::Accounts,
                (KeyCode::Char('h'), _) | (KeyCode::Left, _) => match active_menu_item {
                    MenuItem::Log => {
                        state_log_log.select(None);
                    }
                    _ => {}
                },
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => match active_menu_item {
                    MenuItem::Accounts => {
                        if let Some(selected) = state_accounts.selected() {
                            let modulo = conf.accounts().len();
                            let row_val = (selected + 1) % modulo;
                            state_accounts.select(Some(row_val));
                        }
                    }
                    MenuItem::Log => {
                        match (state_log_accounts.selected(), state_log_log.selected()) {
                            (Some(selected), None) => {
                                let modulo = conf.accounts().len();
                                let row_val = (selected + 1) % modulo;
                                state_log_accounts.select(Some(row_val));
                            }
                            (Some(acct_row_selected), Some(log_row_selected)) => {
                                let acct_key = acct_order[acct_row_selected];
                                // get the number of statements for this account
                                let modulo = acct_stmts.get(acct_key).unwrap().len();
                                let row_val = (log_row_selected + 1) % modulo;
                                state_log_log.select(Some(row_val));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => match active_menu_item {
                    MenuItem::Accounts => {
                        if let Some(selected) = state_accounts.selected() {
                            let modulo = conf.accounts().len();
                            let row_val = (selected + modulo - 1) % modulo;
                            state_accounts.select(Some(row_val));
                        }
                    }
                    MenuItem::Log => {
                        match (state_log_accounts.selected(), state_log_log.selected()) {
                            (Some(selected), None) => {
                                let modulo = conf.accounts().len();
                                let row_val = (selected + modulo - 1) % modulo;
                                state_log_accounts.select(Some(row_val));
                            }
                            (Some(acct_row_selected), Some(log_row_selected)) => {
                                let acct_key = acct_order[acct_row_selected];
                                // get the number of statements for this account
                                let modulo = acct_stmts.get(acct_key).unwrap().len();
                                let row_val = (log_row_selected + modulo - 1) % modulo;
                                state_log_log.select(Some(row_val));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                (KeyCode::Char('l'), _) | (KeyCode::Right, _) => match active_menu_item {
                    MenuItem::Log => {
                        state_log_log.select(Some(0));
                    }
                    _ => {}
                },
                (KeyCode::Enter, _) => match active_menu_item {
                    MenuItem::Log => {
                        match (state_log_accounts.selected(), state_log_log.selected()) {
                            (Some(selected_acct), Some(selected_stmt)) => {
                                // open the statement PDF
                                open_stmt_external(
                                    &acct_stmts,
                                    &acct_order,
                                    selected_acct,
                                    selected_stmt,
                                )
                            }
                            (Some(selected_acct), None) => {
                                // open the file explorer for this account in its specified directory
                                open_account_external(&conf, &acct_order, selected_acct)
                            }
                            (_, _) => {
                                // do nothing
                            }
                        }
                    }
                    _ => {}
                },
                // if the KeyCode alone doesn't match, look for modifiers
                _ => {}
            },
            UserEvent::Tick => {}
        }
    }
    Ok(())
}

/// Gracefully close down the TUI
fn close_tui(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    term.clear()?;
    term.show_cursor()?;
    Ok(())
}

/// Open a PDF statement with the operating system as a separate process
fn open_stmt_external(
    conf: &Config,
    acct_stmts: &StatementCollection,
    selected_acct: usize,
    selected_stmt: usize,
) {
    // get the key for the selected account
    let acct_name = conf.keys()[selected_acct].as_str();
    // construct the path to the statement file
    if let (_, Some(stmt)) = acct_stmts
        .get(acct_name)
        .unwrap()
        .iter()
        .rev()
        .nth(selected_stmt)
        .unwrap()
    {
        // open the statement with an external program
        open::that_in_background(stmt.path());
    }
}

/// Open a PDF statement with the operating system as a separate process
fn open_account_external<'a>(conf: &'a Config, selected_acct: usize) {
    let acct_name = conf.keys()[selected_acct].as_str();
    if let Some(acct) = conf.accounts().get(acct_name) {
        // open the directory for the account
        open::that_in_background(acct.directory());
    }
}
