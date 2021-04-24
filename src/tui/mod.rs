use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Borders, List, ListItem, Tabs},
    Terminal,
};

enum UserEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
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

pub fn start_tui() -> Result<(), Box<dyn std::error::Error>> {
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
    let starting_time = Instant::now();

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
        })?;
        if starting_time.elapsed() >= Duration::from_secs(5) {
            break;
        }

        // receive input from the user about what to do next
        match rx.recv()? {
            UserEvent::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                }
                // Tab to move forward one tab
                KeyCode::Tab => {
                    let modulo = menu_titles.len();
                    let mut tab_val = active_menu_item as usize;
                    tab_val = (tab_val + 1) % modulo;
                    active_menu_item = MenuItem::from(tab_val)
                }
                // Shift + Tab to move backward one tab
                KeyCode::BackTab => {
                    let modulo = menu_titles.len();
                    let mut tab_val = active_menu_item as usize;
                    // this modular arithmetic has to be a bit tricker to deal with -1
                    tab_val = ((tab_val - 1) % modulo + modulo) % modulo;
                    active_menu_item = MenuItem::from(tab_val)
                }
                KeyCode::Char('1') => active_menu_item = MenuItem::Missing,
                KeyCode::Char('2') => active_menu_item = MenuItem::Log,
                KeyCode::Char('3') => active_menu_item = MenuItem::Accounts,
                _ => {}
            },
            UserEvent::Tick => {}
        }
    }

    //     // // render list of accounts with missing statements
    //     // let accts_with_missing: Vec<ListItem> = missing_stmts
    //     //     .iter()
    //     //     .map(|(&a, _)| {
    //     //         ListItem::new(a.to_string())
    //     //         // let missing_dates = v
    //     //         //     .iter()
    //     //         //     .map(|d| ListItem::new(d.to_string()).collect::<Vec<String>>());
    //     //         // combined_v.append(missing_dates)
    //     //     })
    //     //     .collect();
    //     // let accts_list = List::new(accts_with_missing)
    //     //     .block(Block::default()
    //     //         .title("Accounts").borders(Borders::ALL))
    //     //     .style(Style::default().bg(Color::Black))
    //     //     .highlight_style(Style::default());
    //     // f.render_widget(accts_list, chunks[1]);
    // })
    Ok(())
}
