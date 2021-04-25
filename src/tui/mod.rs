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
    symbols::DOT,
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState, Row, Table, TableState, Tabs},
    Terminal,
};

use crate::{models::date::Date, Config};

enum UserEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
enum MenuItem {
    Missing,
    Log,
    Accounts,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Missing => 0,
            MenuItem::Log => 1,
            MenuItem::Accounts => 2,
        }
    }
}

impl From<usize> for MenuItem {
    fn from(input: usize) -> MenuItem {
        match input {
            1 => MenuItem::Log,
            2 => MenuItem::Accounts,
            _ => MenuItem::Missing,
        }
    }
}

pub fn start_tui(conf: &Config) -> Result<(), Box<dyn std::error::Error>> {
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

    // Menu tabs
    let menu_titles = vec!["Missing", "Log", "Accounts"];
    let mut active_menu_item = MenuItem::Accounts;

    // persistent states for each tab
    let mut state_missing = ListState::default();
    let mut state_log = ListState::default();
    let mut state_accounts = TableState::default();
    state_missing.select(Some(0));
    state_log.select(Some(0));
    state_accounts.select(Some(0));

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
                        Constraint::Percentage(100),
                        // footer
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let titles: Vec<Spans> = menu_titles.iter().cloned().map(Spans::from).collect();
            let tabs = Tabs::new(titles)
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
                    f.render_stateful_widget(render_missing(conf), chunks[1], &mut state_missing)
                }
                MenuItem::Log => {
                    f.render_stateful_widget(render_log(conf), chunks[1], &mut state_log)
                }
                MenuItem::Accounts => f.render_stateful_widget(
                    render_accounts(conf, &mut state_accounts),
                    chunks[1],
                    &mut state_accounts,
                ),
            }
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
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => match active_menu_item {
                    MenuItem::Accounts => {
                        if let Some(selected) = state_accounts.selected() {
                            let modulo = conf.accounts().len();
                            let row_val = (selected + 1) % modulo;
                            state_accounts.select(Some(row_val));
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

/// Block for rendering "Missing" page
fn render_missing<'a>(conf: &'a Config) -> List<'a> {
    // get the accounts and sort them by their key name
    let accts = conf.accounts();
    // get missing statements for each account
    let missing_stmts: HashMap<&str, Vec<Date>> = accts
        .values()
        .map(|a| (a.name(), a.missing_statements()))
        .filter(|(_, v)| v.len() > 0)
        .collect();

    // render list of accounts with missing statements
    let accts_with_missing: Vec<ListItem> = missing_stmts
        .iter()
        .map(|(&a, _)| {
            ListItem::new(a)
            // let missing_dates = v
            //     .iter()
            //     .map(|d| ListItem::new(d.to_string()).collect::<Vec<String>>());
            // combined_v.append(missing_dates)
        })
        .collect();
    let accts_list = List::new(accts_with_missing)
        .block(Block::default().title("Accounts").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default());
    accts_list
}

/// Block for rendering "Log" page
fn render_log<'a>(conf: &'a Config) -> List<'a> {
    let log = List::new(vec![])
        .block(Block::default().title("Log").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black));
    log
}

/// Block for rendering "Accounts" page
fn render_accounts<'a>(conf: &'a Config, state: &mut TableState) -> Table<'a> {
    let accts: Vec<Row> = conf
        .accounts_sorted()
        .0
        .iter()
        .map(|&k| {
            Row::new(vec![
                k,
                conf.accounts().get(k).unwrap().name(),
                conf.accounts().get(k).unwrap().institution(),
            ])
        })
        .collect();
    let acct_table = Table::new(accts)
        .header(
            Row::new(vec!["Key", "Account Name", "Institution"]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        )
        .block(Block::default().borders(Borders::ALL))
        .widths(&[
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Min(20),
        ])
        .column_spacing(2)
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default().bg(Color::Blue));
    acct_table
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
