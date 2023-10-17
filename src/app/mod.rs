use anyhow::{Context, Result};
use crossbeam_channel::Receiver;
use log::info;

mod helper;
mod saves;
mod ui;
mod update;

use self::{
    helper::{
        files::init_directories,
        terminal::{restore_terminal, Terminal},
    },
    ui::{
        draw::draw_ui,
        events::{handle_events, EventResult},
        state::AppState,
    },
    update::handle_updates,
};
use crate::{config::Config, watcher::Update};

/// Run the app.
///
/// - Initialize directories
/// - Initialize terminal
/// - Enter the Event->Update->Draw loop
pub fn run(config: Config, receiver: Receiver<Update>) -> Result<()> {
    info!("Initializing directories");
    init_directories(&config).context("Failed while initializing directories")?;
    // Create a new app with some example state
    let mut state = AppState::new(&config)?;

    info!("Initializing terminal");
    let mut terminal = helper::terminal::init_terminal()?;

    // One initial clear and draw
    // From now on, we only redraw, if there're actual changes.
    terminal.clear()?;
    draw_ui(&mut terminal, &mut state)?;

    // Restore the terminal in case any errors happen.
    // Otherwise the terminal won't be usable as it's still in AlternateScreen mode.
    if let Err(error) = main_loop(&mut state, &mut terminal, receiver) {
        restore_terminal(&mut terminal)?;
        return Err(error);
    }

    Ok(())
}

/// A simple encapsulation of the main loop.
///
/// This way, we can catch all errors from the app and restore the terminal before exiting the
/// program. Otherwise we would have a broken terminal.
pub fn main_loop(
    state: &mut AppState,
    terminal: &mut Terminal,
    receiver: Receiver<Update>,
) -> Result<()> {
    loop {
        let mut draw_scheduled = false;

        match handle_events(terminal, state)? {
            EventResult::Redraw => draw_scheduled = true,
            EventResult::Quit => break,
            _ => (),
        }
        if handle_updates(state, &receiver)? {
            draw_scheduled = true;
        }

        // Draw at the end of the loop after everything has been processed.
        // Only refresh the screen, if we have to.
        if draw_scheduled {
            draw_ui(terminal, state)?;
        }
    }

    Ok(())
}
