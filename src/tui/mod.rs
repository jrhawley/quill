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

use crate::{
    models::{date::Date, statement::Statement},
    Config,
};

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

    // Menu tabs
    let menu_titles = vec!["Missing", "Log", "Accounts"];
    let mut active_menu_item = MenuItem::Missing;

    // persistent states for each tab
    let mut state_missing = ListState::default();
    let mut state_log_accounts = ListState::default();
    let mut state_log_log = ListState::default();
    let mut state_accounts = TableState::default();
    state_missing.select(Some(0));
    state_log_accounts.select(Some(0));
    state_log_log.select(None);
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

/// Block for rendering "Missing" page
fn render_missing<'a>(
    conf: &'a Config,
    acct_stmts: &'a HashMap<&str, Vec<(Date, Option<Statement>)>>,
    acct_order: &'a Vec<&str>,
) -> List<'a> {
    // render list of accounts with missing statements
    let mut accts_with_missing: Vec<ListItem> = vec![];
    for &acct_key in acct_order {
        let this_acct = conf.accounts().get(acct_key).unwrap();
        let missing_stmts: Vec<ListItem> = acct_stmts
            .get(acct_key)
            .unwrap()
            .iter()
            .filter(|(_, stmt)| stmt.is_none())
            .map(|(d, _)| ListItem::new(format!("  {}", d)))
            .collect();
        if missing_stmts.len() > 0 {
            accts_with_missing.push(ListItem::new(this_acct.name()));
            for li in missing_stmts {
                accts_with_missing.push(li);
            }
        }
    }
    let accts_list = List::new(accts_with_missing)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default());
    accts_list
}

/// Block for rendering "Log" page
fn render_log<'a>(
    conf: &'a Config,
    acct_stmts: &'a HashMap<&str, Vec<(Date, Option<Statement>)>>,
    acct_order: &'a Vec<&str>,
    acct_state: &ListState,
    log_state: &ListState,
) -> (List<'a>, List<'a>) {
    let acct_names_ordered: Vec<ListItem> = acct_order
        .iter()
        .map(|&a| ListItem::new(conf.accounts().get(a).unwrap().name()))
        .collect();

    let mut accts = List::new(acct_names_ordered)
        .block(Block::default().title("Accounts").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue));

    // get the log of statements for the selected account
    let rows: Vec<ListItem> = match acct_state.selected() {
        Some(acct_idx) => {
            // get the HashMap key of the account that's highlighted
            let acct_key = acct_order[acct_idx];
            // convert the statements into formatted Rows
            acct_stmts
                .get(acct_key)
                .unwrap()
                .iter()
                // go through in reverse chronological order so latest is at the top
                .rev()
                .map(|(d, s)| {
                    ListItem::new(format!(
                        "{} {}",
                        d,
                        match s {
                            Some(_) => String::from("✔"),
                            None => String::from("❌"),
                        }
                    ))
                })
                .collect()
        }
        // return the template table if no Account is selected
        // this should never happen
        None => vec![ListItem::new(
            "This shouldn't occur, unless there are no accounts",
        )],
    };
    let mut log = List::new(rows)
        .block(Block::default().title("Statements").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue));

    // dim the side that is not selected
    if let Some(_) = log_state.selected() {
        accts = accts.style(Style::default().add_modifier(Modifier::DIM));
        log = log.style(Style::default());
    } else {
        accts = accts.style(Style::default());
        log = log.style(Style::default().add_modifier(Modifier::DIM));
    }

    (accts, log)
}

/// Block for rendering "Accounts" page
fn render_accounts<'a>(conf: &'a Config, acct_order: &'a Vec<&str>) -> Table<'a> {
    let accts: Vec<Row> = acct_order
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

/// Open a PDF statement with the operating system as a separate process
fn open_stmt_external<'a>(
    acct_stmts: &'a HashMap<&str, Vec<(Date, Option<Statement>)>>,
    acct_order: &'a Vec<&str>,
    selected_acct: usize,
    selected_stmt: usize,
) {
    // get the key for the selected account
    let acct_name = acct_order[selected_acct];
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
fn open_account_external<'a>(conf: &'a Config, acct_order: &'a Vec<&str>, selected_acct: usize) {
    let acct_name = acct_order[selected_acct];
    if let Some(acct) = conf.accounts().get(acct_name) {
        // open the directory for the account
        open::that_in_background(acct.directory());
    }
}
