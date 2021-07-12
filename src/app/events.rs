use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::saves::restore_save;

use super::helper::terminal::{restore_terminal, Terminal};
use super::state::{AppState, UiState};

/// This enum signals the parent function, which actions should be taken.
pub enum EventResult {
    Redraw,
    Quit,
    Ignore,
}

/// Handle all events.
///
/// Returns true, if we should exit the program
pub fn handle_events(terminal: &mut Terminal, state: &mut AppState) -> Result<EventResult> {
    // Check if there are any new events.
    // Return earyl if there aren't.
    if !poll(Duration::from_millis(100))? {
        return Ok(EventResult::Ignore);
    }

    match read()? {
        Event::Key(event) => handle_key(&event, terminal, state),
        _ => Ok(EventResult::Ignore),
    }
}

/// Handle all kinds of key events
fn handle_key(
    event: &KeyEvent,
    terminal: &mut Terminal,
    state: &mut AppState,
) -> Result<EventResult> {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return handle_control(event, terminal, state);
    }
    match event.code {
        KeyCode::Char('q') => {
            // 'q' instantly exits the program.
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // Navigate to the next item of the focused list
            match state.state {
                UiState::Games => {
                    state.games.next();
                    state.update_saves()?;
                }
                UiState::Autosave => state.autosaves.next(),
                UiState::ManualSave => state.manual_saves.next(),
                _ => (),
            }
            return Ok(EventResult::Redraw);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            // Navigate to the previous item of the focused list
            match state.state {
                UiState::Games => {
                    state.games.previous();
                    state.update_saves()?;
                }
                UiState::Autosave => state.autosaves.previous(),
                UiState::ManualSave => state.manual_saves.previous(),
                _ => (),
            }
            return Ok(EventResult::Redraw);
        }
        KeyCode::Enter => {
            // Restore a savegame
            if matches!(state.state, UiState::Autosave) {
                if let Some(save) = state.autosaves.get_selected() {
                    if let Some(game_name) = state.get_selected_game() {
                        restore_save(&state.config, &game_name, save)?;
                        state.ignore_changes.insert(game_name, Local::now());
                    }
                }
            }

            if matches!(state.state, UiState::ManualSave) {
                if let Some(save) = state.manual_saves.get_selected() {
                    if let Some(game_name) = state.get_selected_game() {
                        restore_save(&state.config, &game_name, save)?;
                        state.ignore_changes.insert(game_name, Local::now());
                    }
                }
            }
        }
        _ => {}
    }

    Ok(EventResult::Ignore)
}

/// Handle all key combinations with the CTRL modifier
fn handle_control(
    event: &KeyEvent,
    terminal: &mut Terminal,
    state: &mut AppState,
) -> Result<EventResult> {
    match event.code {
        KeyCode::Char('c') => {
            // Classict CTRL+C should kill the program
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Moving to the left moves focus to the game list.
            if matches!(state.state, UiState::Autosave | UiState::ManualSave) {
                state.state = UiState::Games;
                return Ok(EventResult::Redraw);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            // Moving to the left right moves focus to the saves.
            // If autosaves are enabled we focus it, otherwise we fallback to manual saves.
            if state.selected_game_has_autosave() {
                state.state = UiState::Autosave;
                state.autosaves.focus();
            } else {
                state.state = UiState::ManualSave;
                state.manual_saves.focus();
            }
            return Ok(EventResult::Redraw);
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Up | KeyCode::Char('k') => {
            // Moving up down while focus is on the save lists should switch to the other
            // non-focused list.
            if state.selected_game_has_autosave() && matches!(state.state, UiState::ManualSave) {
                state.state = UiState::Autosave;
                state.autosaves.focus();
                return Ok(EventResult::Redraw);
            } else if matches!(state.state, UiState::Autosave) {
                state.state = UiState::ManualSave;
                state.manual_saves.focus();
                return Ok(EventResult::Redraw);
            }
        }

        _ => return Ok(EventResult::Ignore),
    }

    Ok(EventResult::Ignore)
}
