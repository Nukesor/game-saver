use std::time::Duration;

use anyhow::{Context, Result};
use crossbeam_channel::Receiver;
use crossterm::event::{poll, read, Event, KeyCode};
use log::debug;

mod draw;
mod helper;
mod state;

use self::draw::draw_ui;
use self::helper::files::init_directories;
use self::state::AppState;
use crate::config::Config;
use crate::watcher::Update;

/// Run the app.
///
/// - Initialize directories
/// - Initialize terminal
/// - Enter the Event->Update->Draw loop
pub fn run(config: Config, receiver: Receiver<Update>) -> Result<()> {
    debug!("Initializing directories");
    init_directories(&config).context("Failed while initializing directories")?;
    // Create a new app with some example state
    let mut state = AppState::new(&config)?;

    debug!("Initializing terminal");
    let mut terminal = helper::terminal::init_terminal()?;

    // One initial clear and draw
    // From now on, we only redraw, if there're actual changes.
    terminal.clear()?;
    draw_ui(&mut terminal, &mut state)?;

    loop {
        let mut draw_scheduled = false;
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
                        state.games.next();
                        draw_scheduled = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.games.previous();
                        draw_scheduled = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Go through all updates for changed files.
        while let Ok(update) = receiver.try_recv() {}

        // Draw at the end of the loop after everything has been processed.
        // Only refresh the screen, if we have to.
        if draw_scheduled {
            draw_ui(&mut terminal, &mut state)?;
        }
    }

    Ok(())
}
