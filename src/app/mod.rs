use anyhow::{Context, Result};
use chrono::{Duration, Local};
use crossbeam_channel::Receiver;
use log::debug;

mod draw;
mod events;
mod helper;
mod saves;
mod state;

use self::draw::draw_ui;
use self::helper::files::init_directories;
use self::helper::terminal::Terminal;
use self::state::AppState;
use crate::app::helper::terminal::restore_terminal;
use crate::app::saves::autosave_game;
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

    // Restore the terminal in case any errors happen.
    // Otherwise the terminal won't be usable as it's still in AlternateScreen mode.
    if let Err(error) = main_loop(&mut state, &mut terminal, receiver) {
        restore_terminal(&mut terminal)?;
        return Err(error);
    }

    Ok(())
}

/// A simple encapsulation the main loop.
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

        match events::handle_events(terminal, state)? {
            events::EventResult::Redraw => draw_scheduled = true,
            events::EventResult::Quit => break,
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

/// Process updates (filesystem changes) according to the current app state.
///
/// If enabled, filesystem changes will trigger autosaves.
/// Updates will be ignored during save restoration.
fn handle_updates(state: &mut AppState, receiver: &Receiver<Update>) -> Result<bool> {
    let mut draw_scheduled = false;
    // Go through all updates for changed files and set the update time to now.
    while let Ok(update) = receiver.try_recv() {
        let game_config = state.config.games.get(&update.game_name).unwrap();
        if game_config.autosave == 0 {
            continue;
        }

        // Don't schedule a autosave, if we just restored a save for that game.
        if state.ignore_changes.contains_key(&update.game_name) {
            continue;
        }

        state
            .changes_detected
            .insert(update.game_name.clone(), update.time);
    }

    // Save all games whose save directory hasn't been touched for a few seconds, .
    let watched_changes: Vec<String> = state
        .changes_detected
        .keys()
        .map(|key| key.clone())
        .collect();
    for game_name in watched_changes.iter() {
        let time = state.changes_detected.get(game_name).unwrap();
        if (Local::now() - Duration::seconds(5)).gt(time) {
            // We can create the autosave.
            // Schedule a redraw and remove that update from our watchlist.
            autosave_game(&state.config, &game_name)?;
            state.log(&format!("Autosave created for {}", game_name));
            state.update_autosaves()?;
            draw_scheduled = true;
            state.changes_detected.remove(game_name);
        }
    }

    // Remove the ignore rule for file changes after a few seconds.
    // We only have to lock this for a short amount of time, after the restore.
    let ignore_changes: Vec<String> = state.ignore_changes.keys().map(|key| key.clone()).collect();
    for game_name in ignore_changes.iter() {
        let time = state.ignore_changes.get(game_name).unwrap();
        if (Local::now() - Duration::seconds(1)).gt(time) {
            state.ignore_changes.remove(game_name);
        }
    }

    Ok(draw_scheduled)
}
