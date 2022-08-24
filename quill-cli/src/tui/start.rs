//! Start the terminal user interface, draw it, and manage keystrokes.

use super::{
    open_account_external, open_stmt_external,
    render::{self, MenuItem},
    state::TuiState,
};
use crate::cfg::Config;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::enable_raw_mode,
};
use quill_statement::StatementCollection;
use std::{
    io::{self, Stdout},
    sync::mpsc::{channel, Sender},
    thread,
};
use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
    Frame, Terminal,
};

/// Delay between TUI redraws
const TICK_RATE: Duration = Duration::from_millis(200);

/// An event specified by the user.
/// Is either a type of input (i.e. a keystroke), or an empty time frame
/// (nothing is pressed, so a "tick" is sent).
enum UserEvent<I> {
    Input(I),
    Tick,
}

pub fn start_tui(
    conf: &Config,
    acct_stmts: &StatementCollection,
) -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn std::error::Error>> {
    // set up a multi-producer single consumer channel to communicate between the input handler and the TUI rendering loop
    let (tx, rx): (Sender<UserEvent<KeyEvent>>, Receiver<UserEvent<KeyEvent>>) = channel();

    // construct the TUI from the user event sender channel
    let mut terminal = initiate_tui(tx)?;

    // persistent state of the entire TUI
    let mut state = TuiState::default();

    if conf.len() > 0 {
        state.mut_log().select_account(Some(0));
        state.mut_accounts().select(Some(0));
    }

    loop {
        terminal.draw(|f| draw_tui(f, conf, &mut state, acct_stmts))?;
        if process_user_events(&rx, conf, &mut state, acct_stmts).is_err() {
            break;
        }
    }
    Ok(terminal)
}

/// Construct the TUI from the user event sender channel
///
/// Creates the user event thread and determines where the output buffer is written
fn initiate_tui(tx: Sender<UserEvent<KeyEvent>>) -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    // enable raw mode to avoid waiting for ENTER to respond to keystrokes
    enable_raw_mode()?;

    // start the threading
    thread::spawn(move || {
        // record the time of the last Tick sent
        let mut last_tick = Instant::now();
        loop {
            // set a polling period to accept an input event from the user
            let timeout = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // poll the user for the given time, and if there is an input event, return it
            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    tx.send(UserEvent::Input(key)).expect("can send events");
                }
            }

            // if enough time has elapsed, return a Tick, since no Input has been triggered
            if (last_tick.elapsed() >= TICK_RATE) && (tx.send(UserEvent::Tick).is_ok()) {
                last_tick = Instant::now();
            }
        }
    });

    // Initialize the TUI to send to STDOUT
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // clear the screen before displaying anything
    terminal.clear()?;

    Ok(terminal)
}

/// Draw the TUI elements
fn draw_tui(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    conf: &Config,
    state: &mut TuiState,
    acct_stmts: &StatementCollection,
) {
    // get terminal window dimensions
    let size = f.size();

    // draw a full black rectangle to hide everything
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        size,
    );

    // create the chunks where the tab bar, main body, and footer are located
    let chunks = create_tab_body_footer(state, size, f);

    // render the main block depending on what tab is selected
    match state.active_tab() {
        MenuItem::Missing => render::missing_body(f, conf, acct_stmts, state, &chunks[1]),
        MenuItem::Log => render::log_body(f, conf, acct_stmts, state, &chunks[1]),
        MenuItem::Upcoming => render::upcoming_body(f, conf, state, &chunks[1]),
        MenuItem::Accounts => render::accounts_body(f, conf, state, &chunks[1]),
    };

    let guide = render::guide();
    f.render_widget(guide, chunks[2]);
}

/// Create chunks for the tab bar and the main body view
///
/// Takes the TUI state to determine which tab is active, the size of the window frame to render, and the frame that is rendering the chunks.
fn create_tab_body_footer(
    state: &mut TuiState,
    size: Rect,
    f: &mut Frame<CrosstermBackend<Stdout>>,
) -> Vec<Rect> {
    let tabs = render::tabs(state.active_tab());
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

    // render the tabs
    f.render_widget(tabs, chunks[0]);

    // return the chunks for use by other rendering functions
    chunks
}

/// Receive and process any keys pressed by the user.
/// Results in an Err() if the user quits or an error is reached internally.
fn process_user_events(
    rx: &Receiver<UserEvent<KeyEvent>>,
    conf: &Config,
    state: &mut TuiState,
    acct_stmts: &StatementCollection,
) -> Result<(), Box<dyn std::error::Error>> {
    // receive input from the user about what to do next
    match rx.recv()? {
        // destruct KeyCode and KeyModifiers for more legible match cases
        UserEvent::Input(KeyEvent { code, modifiers }) => match (code, modifiers) {
            // Quit
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Err(Box::new(io::Error::new(io::ErrorKind::Interrupted, "")));
            }
            // Tab to move forward one tab
            (KeyCode::Tab, _) => state.next_tab(),
            // Shift + Tab to move backward one tab
            (KeyCode::BackTab, _) => state.prev_tab(),
            (KeyCode::Char('1'), _) => state.set_active_tab(0.into()),
            (KeyCode::Char('2'), _) => state.set_active_tab(1.into()),
            (KeyCode::Char('3'), _) => state.set_active_tab(2.into()),
            (KeyCode::Char('4'), _) => state.set_active_tab(3.into()),
            (KeyCode::Char('h'), _) | (KeyCode::Left, _) => {
                if state.active_tab() == MenuItem::Log {
                    state.mut_log().select_log(None);
                }
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => match state.active_tab() {
                MenuItem::Accounts => {
                    if state.accounts().selected().is_some() {
                        state.mut_accounts().select_next(conf.len());
                    }
                }
                MenuItem::Log => match state.log().selected() {
                    (Some(_), None) => state.mut_log().select_next_account(conf.len()),
                    (Some(acct_row_selected), Some(_)) => {
                        // get the number of statements for this account
                        let acct_key = conf.keys()[acct_row_selected].as_str();
                        state
                            .mut_log()
                            .select_next_log(acct_stmts.get(acct_key).unwrap().len());
                    }
                    _ => {}
                },
                _ => {}
            },
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => match state.active_tab() {
                MenuItem::Accounts => state.mut_accounts().select_prev(conf.len()),
                MenuItem::Log => match state.log().selected() {
                    (Some(_), None) => {
                        state.mut_log().select_prev_account(conf.len());
                    }
                    (Some(acct_row_selected), Some(_)) => {
                        // get the number of statements for this account
                        let acct_key = conf.keys()[acct_row_selected].as_str();
                        state
                            .mut_log()
                            .select_prev_log(acct_stmts.get(acct_key).unwrap().len());
                    }
                    _ => {}
                },
                _ => {}
            },
            (KeyCode::Char('l'), _) | (KeyCode::Right, _) => {
                if state.active_tab() == MenuItem::Log {
                    state.mut_log().select_log(Some(0));
                }
            }
            (KeyCode::Enter, _) => {
                if state.active_tab() == MenuItem::Log {
                    match state.log().selected() {
                        (Some(selected_acct), None) => {
                            // open the file explorer for this account in its specified directory
                            open_account_external(conf, selected_acct);
                        }
                        (Some(selected_acct), Some(selected_stmt)) => {
                            // open the statement PDF
                            open_stmt_external(conf, acct_stmts, selected_acct, selected_stmt);
                        }
                        (_, _) => {}
                    }
                }
            }
            // if the KeyCode alone doesn't match, look for modifiers
            _ => {}
        },
        UserEvent::Tick => {}
    }
    Ok(())
}
