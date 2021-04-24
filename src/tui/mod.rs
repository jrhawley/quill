use crossterm::terminal::enable_raw_mode;
use mpsc;
use std::io;
use std::time::Duration;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols::DOT,
    text::Spans,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Tabs},
};

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

pub fn start_tui() -> Result<(), io::Error> {
    // 1. Configure TUI
    // -------------------------------------------
    // enable raw mode to avoid waiting for ENTER to respond to keystrokes
    enable_raw_mode().expect("can run in raw mode");
    // set up a multi-producer single consumer channel to communicate between the input handler and
    // rendering loop
    let (tx, rx) = mpsc::channel();
    // 200 ms delay between refreshes
    let tick_rate = Duration::from_millis(200);
    // start the threading
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        // get terminal window dimensions
        let size = f.size();
        // draw a full black rectangle to hide everything
        f.render_widget(Block::default().style(Style::default().bg(Color::Black)), size);

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

        // render tabs
        let titles = ["Missing", "Log", "Accounts"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().title("Tabs").borders(Borders::ALL))
            .style(Style::default().bg(Color::Black))
            .highlight_style(Style::default())
            .divider(DOT);
        f.render_widget(tabs, chunks[0]);

        // // render list of accounts with missing statements
        // let accts_with_missing: Vec<ListItem> = missing_stmts
        //     .iter()
        //     .map(|(&a, _)| {
        //         ListItem::new(a.to_string())
        //         // let missing_dates = v
        //         //     .iter()
        //         //     .map(|d| ListItem::new(d.to_string()).collect::<Vec<String>>());
        //         // combined_v.append(missing_dates)
        //     })
        //     .collect();
        // let accts_list = List::new(accts_with_missing)
        //     .block(Block::default()
        //         .title("Accounts").borders(Borders::ALL))
        //     .style(Style::default().bg(Color::Black))
        //     .highlight_style(Style::default());
        // f.render_widget(accts_list, chunks[1]);
    })
}
