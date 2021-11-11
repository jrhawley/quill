use crossterm::{
	event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
	terminal::enable_raw_mode,
};
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
	layout::{Constraint, Direction, Layout},
	style::{Color, Style},
	widgets::Block,
	Frame, Terminal,
};

use crate::{config::Config, models::StatementCollection};

use super::{
	open_account_external, open_stmt_external,
	render::{self, MenuItem},
	state::TuiState,
};

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
		terminal.draw(|f| draw_tui(f, &conf, &mut state, acct_stmts))?;
		if let Err(_) = process_user_events(&rx, conf, &mut state, acct_stmts) {
			break;
		}
	}
	Ok(terminal)
}

/// Initiate the TUI with a basic configuration
fn initiate_tui(tx: Sender<UserEvent<KeyEvent>>) -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
	// enable raw mode to avoid waiting for ENTER to respond to keystrokes
	enable_raw_mode()?;

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

	// render the main block depending on what tab is selected
	match state.active_tab() {
		MenuItem::Missing => {
			f.render_stateful_widget(
				render::missing(conf, acct_stmts),
				chunks[1],
				state.mut_missing().mut_state(),
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
			let (left, right) = render::log(conf, acct_stmts, state.log());
			f.render_stateful_widget(left, log_chunks[0], state.mut_log().mut_accounts());
			f.render_stateful_widget(right, log_chunks[1], state.mut_log().mut_log());
		}
		MenuItem::Accounts => {
			f.render_stateful_widget(
				render::accounts(conf),
				chunks[1],
				state.mut_accounts().mut_state(),
			);
		}
	};

	let guide = render::guide();
	f.render_widget(guide, chunks[2]);
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
			(KeyCode::Char('h'), _) | (KeyCode::Left, _) => match state.active_tab() {
				MenuItem::Log => {
					state.mut_log().select_log(None);
				}
				_ => {}
			},
			(KeyCode::Char('j'), _) | (KeyCode::Down, _) => match state.active_tab() {
				MenuItem::Accounts => {
					if let Some(_) = state.accounts().selected() {
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
			(KeyCode::Char('l'), _) | (KeyCode::Right, _) => match state.active_tab() {
				MenuItem::Log => {
					state.mut_log().select_log(Some(0));
				}
				_ => {}
			},
			(KeyCode::Enter, _) => match state.active_tab() {
				MenuItem::Log => match state.log().selected() {
					(Some(selected_acct), None) => {
						// open the file explorer for this account in its specified directory
						open_account_external(conf, selected_acct);
					}
					(Some(selected_acct), Some(selected_stmt)) => {
						// open the statement PDF
						open_stmt_external(conf, acct_stmts, selected_acct, selected_stmt);
					}
					(_, _) => {}
				},
				_ => {}
			},
			// if the KeyCode alone doesn't match, look for modifiers
			_ => {}
		},
		UserEvent::Tick => {}
	}
	Ok(())
}
