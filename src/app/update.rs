use std::convert::TryInto;

use anyhow::Result;
use chrono::{Duration, Local};
use crossbeam_channel::Receiver;

use super::{saves::autosave_game, ui::state::AppState};
use crate::watcher::Update;

/// Process updates (filesystem changes) according to the current app state.
///
/// If enabled, filesystem changes will trigger autosaves.
/// Updates will be ignored during save restoration.
pub fn handle_updates(state: &mut AppState, receiver: &Receiver<Update>) -> Result<bool> {
    let mut draw_scheduled = false;

    receive_updates(state, receiver);

    if save_games(state)? {
        draw_scheduled = true;
    }

    remove_ignored_changes(state);
    remove_autosave_timeouts(state);

    Ok(draw_scheduled)
}

/// Go through all updates for changed files.
/// If autosaves are enabled and no autosave-timeout is active schedule a save for the given game.
pub fn receive_updates(state: &mut AppState, receiver: &Receiver<Update>) {
    while let Ok(update) = receiver.try_recv() {
        let game_config = state.config.games.get(&update.game_name).unwrap();
        if !game_config.has_autosaves() {
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
}

/// Save all games whose save directory hasn't been touched for a few seconds.
pub fn save_games(state: &mut AppState) -> Result<bool> {
    let mut draw_scheduled = false;
    let watched_changes: Vec<String> = state
        .changes_detected
        .keys()
        .map(|key| key.clone())
        .collect();

    for game in watched_changes.iter() {
        // Make sure there weren't any changes for a few seconds.
        // Otherwise we might create a backup, while the game is still writing files.
        let time = state.changes_detected.get(game).unwrap();
        if (Local::now() - Duration::seconds(5)).lt(time) {
            continue;
        }

        // Don't save, if the autosave timeout didn't finish yet.
        if state.autosave_timeouts.contains_key(game) {
            continue;
        }

        // We can create the autosave.
        autosave_game(&state.config, &game)?;
        state.log(&format!("Autosave created for {}", game));
        state.update_autosaves()?;

        // Set a autosave timeout, if it is specified for the current game.
        let game_config = state.config.games.get(game).unwrap();
        if game_config.autosave_timeout > 0 {
            state.autosave_timeouts.insert(game.clone(), Local::now());
        }

        // Schedule a redraw and remove that update from our watchlist.
        state.changes_detected.remove(game);
        draw_scheduled = true;
    }

    Ok(draw_scheduled)
}

/// Changes will be ignored for a short time after restoring a save file.
/// Remove the ignore rule for file changes after a few seconds.
/// We only have to lock this for a short amount of time, after the restore.
pub fn remove_ignored_changes(state: &mut AppState) {
    let ignored_duration = Duration::seconds(5);

    let games: Vec<String> = state.ignore_changes.keys().map(|key| key.clone()).collect();

    for game in games.iter() {
        let time = state.ignore_changes.get(game).unwrap();
        if (Local::now() - ignored_duration).gt(time) {
            state.ignore_changes.remove(game);
        }
    }
}

/// If an autosave timeout is specified, we won't save the game until the specified timeout
/// finished.
/// Remove the ignore rule for file changes after a few seconds.
/// We only have to lock this for a short amount of time, after the restore.
pub fn remove_autosave_timeouts(state: &mut AppState) {
    let games: Vec<String> = state
        .autosave_timeouts
        .keys()
        .map(|key| key.clone())
        .collect();

    for game in games.iter() {
        let game_config = state.config.games.get(game).unwrap();
        let timeout = game_config.autosave_timeout;
        let timeout_duration = Duration::seconds(timeout.try_into().unwrap_or(i64::MAX));

        let last_save = state.autosave_timeouts.get(game).unwrap();
        if (Local::now() - timeout_duration).gt(last_save) {
            state.autosave_timeouts.remove(game);
        }
    }
}
