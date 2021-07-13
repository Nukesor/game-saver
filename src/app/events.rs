use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::saves::restore_save;

use super::helper::terminal::{restore_terminal, Terminal};
use super::saves::manually_save_game;
use super::state::{AppState, Input, InputType, UiState};

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
    let current_ui_state = state.get_state();

    if let UiState::Input(input) = current_ui_state {
        return handle_input(event, state, input);
    }

    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return handle_control(event, terminal, state, current_ui_state);
    }
    match event.code {
        KeyCode::Char('q') => {
            // 'q' instantly exits the program.
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        KeyCode::Char('a') => {
            let game_name = state.get_selected_game();
            // Create a new savegame for the current game.
            state.set_state(UiState::Input(Input {
                game: game_name,
                input: String::new(),
                input_type: InputType::Create,
            }));
            return Ok(EventResult::Redraw);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // Navigate to the next item of the focused list.
            match current_ui_state {
                UiState::Games(_) => {
                    state.games.next();
                    state.set_state(UiState::Games(state.get_selected_game()));
                    state.update_saves()?;
                }
                UiState::Autosave(_) => state.autosaves.next(),
                UiState::ManualSave(_) => state.manual_saves.next(),
                _ => (),
            }
            return Ok(EventResult::Redraw);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            // Navigate to the previous item of the focused list.
            match current_ui_state {
                UiState::Games(_) => {
                    state.games.previous();
                    state.set_state(UiState::Games(state.get_selected_game()));
                    state.update_saves()?;
                }
                UiState::Autosave(_) => state.autosaves.previous(),
                UiState::ManualSave(_) => state.manual_saves.previous(),
                _ => (),
            }
            return Ok(EventResult::Redraw);
        }
        KeyCode::Enter => {
            // Restore a savegame.
            match current_ui_state {
                UiState::Autosave(game_name) => {
                    if let Some(save) = state.autosaves.get_selected() {
                        restore_save(&state.config, &game_name, &save)?;
                        state.ignore_changes.insert(game_name.clone(), Local::now());
                        state.log(&format!(
                            "Restored savefile {} for {}",
                            save.file_name, game_name
                        ));
                        return Ok(EventResult::Redraw);
                    }
                }
                UiState::ManualSave(game_name) => {
                    if let Some(save) = state.manual_saves.get_selected() {
                        restore_save(&state.config, &game_name, &save)?;
                        state.ignore_changes.insert(game_name.clone(), Local::now());
                        state.log(&format!(
                            "Restored savefile '{}' for {}",
                            save.file_name, game_name
                        ));
                        return Ok(EventResult::Redraw);
                    }
                }
                _ => (),
            }
        }
        _ => {}
    }

    Ok(EventResult::Ignore)
}

/// Handle input during
fn handle_input(event: &KeyEvent, state: &mut AppState, mut input: Input) -> Result<EventResult> {
    match event.code {
        KeyCode::Esc => {
            // Abort the savegame cration process
            state.state = state.previous_state.clone();
            return Ok(EventResult::Redraw);
        }
        KeyCode::Enter => {
            // Create a new save.
            manually_save_game(&state.config, &input.game, &input.input)?;
            state.log(&format!(
                "New manual save for {} with name '{}'",
                &input.game, &input.input
            ));
            state.state = state.previous_state.clone();
            state.update_manual_saves()?;
            return Ok(EventResult::Redraw);
        }
        KeyCode::Backspace => {
            // Remove a character from the name
            input.input.pop();
            state.state = UiState::Input(input);
            return Ok(EventResult::Redraw);
        }
        KeyCode::Char(character) => {
            // Add the character to the name
            input.input.push(character);
            state.state = UiState::Input(input);
            return Ok(EventResult::Redraw);
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
    current_ui_state: UiState,
) -> Result<EventResult> {
    match event.code {
        KeyCode::Char('c') => {
            // Classict CTRL+C should kill the program
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Moving to the left moves focus to the game list.
            match current_ui_state {
                UiState::Autosave(game) | UiState::ManualSave(game) => {
                    state.set_state(UiState::Games(game));
                    return Ok(EventResult::Redraw);
                }
                _ => {}
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            // Moving to the left right moves focus to the saves.
            // If autosaves are enabled we focus it, otherwise we fallback to manual saves.
            if state.selected_game_has_autosave() {
                state.state = UiState::Autosave(state.get_selected_game());
                state.autosaves.focus();
            } else {
                state.state = UiState::ManualSave(state.get_selected_game());
                state.manual_saves.focus();
            }
            return Ok(EventResult::Redraw);
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Up | KeyCode::Char('k') => {
            // Moving up down while focus is on the save lists should switch to the other
            // non-focused list.
            match current_ui_state {
                UiState::ManualSave(game) => {
                    if state.selected_game_has_autosave() {
                        state.set_state(UiState::Autosave(game));
                        state.autosaves.focus();
                        return Ok(EventResult::Redraw);
                    }
                }
                UiState::Autosave(game) => {
                    state.state = UiState::ManualSave(game);
                    state.manual_saves.focus();
                    return Ok(EventResult::Redraw);
                }
                _ => (),
            }
        }

        _ => return Ok(EventResult::Ignore),
    }

    Ok(EventResult::Ignore)
}
