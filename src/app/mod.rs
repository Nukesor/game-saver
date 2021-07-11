use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::Receiver;
use crossterm::event::{poll, read, Event, KeyCode};

mod draw;
mod helper;

use crate::{config::Config, watcher::Update};
use draw::draw_ui;
use helper::list::StatefulList;

/// This struct holds the state for the tui-rs interface.
/// This includes, lists, selected items as well as temporary input elements.
pub struct AppState {
    items: StatefulList,
}

impl AppState {
    fn new(config: &Config) -> AppState {
        let mut items = vec![];
        for (name, _) in &config.games {
            items.push(name.clone());
        }
        AppState {
            items: StatefulList::with_items(items),
        }
    }
}

pub fn run(config: Config, receiver: Receiver<Update>) -> Result<()> {
    // Create a new app with some example state
    let mut state = AppState::new(&config);

    let mut terminal = helper::terminal::init_terminal()?;

    // One initial clear and draw
    // From now on, we only redraw, if there're actual changes.
    terminal.clear()?;
    draw_ui(&mut terminal, &mut state)?;

    loop {
        // This is a simple example on how to handle events
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        helper::terminal::restore_terminal(terminal)?;
                        break;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {}
                    KeyCode::Right | KeyCode::Char('l') => {}
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.items.next();
                        draw_ui(&mut terminal, &mut state)?;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.items.previous();
                        draw_ui(&mut terminal, &mut state)?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let updates = receiver.recv();
        for update in updates {}
    }

    Ok(())
}
