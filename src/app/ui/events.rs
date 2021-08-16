use std::time::Duration;

use anyhow::{bail, Result};
use chrono::Local;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use super::state::{AppState, Input, InputType, PromptType, UiState};
use crate::app::{
    helper::{
        list::Navigate,
        terminal::{restore_terminal, Terminal},
    },
    saves::{delete_save, manually_save_game, rename_save, restore_save},
};

/// This enum signals the parent function, which actions should be taken.
pub enum EventResult {
    /// The event has been handled and we should redraw the window
    Redraw,
    /// The user wants to quit. Exit the program
    Quit,
    /// The event has been handled, but it should be ignored it.
    Ignore,
    /// The event hasn't been handled by a handler, we can check with the next one.
    NotHandled,
}

/// Handle all events.
///
/// Returns true, if we should exit the program
pub fn handle_events(terminal: &mut Terminal, state: &mut AppState) -> Result<EventResult> {
    // Check if there are any new events.
    // Return earyl if there aren't.
    if !poll(Duration::from_millis(100))? {
        return Ok(EventResult::NotHandled);
    }

    match read()? {
        Event::Key(event) => handle_key(&event, terminal, state),
        Event::Resize(_, _) => Ok(EventResult::Redraw),
        _ => Ok(EventResult::NotHandled),
    }
}

/// Handle all kinds of key events
fn handle_key(
    event: &KeyEvent,
    terminal: &mut Terminal,
    state: &mut AppState,
) -> Result<EventResult> {
    let current_ui_state = state.get_state();

    // Run through strictly state-specific handlers.
    let mut result = match current_ui_state {
        UiState::Input(input) => return handle_input(event, state, input),
        UiState::Prompt(prompt_type) => return handle_prompt(event, state, prompt_type),
        UiState::Games => handle_game_list(event, state)?,
        UiState::Autosave => handle_autosave_list(event, state)?,
        UiState::ManualSave => handle_manual_save_list(event, state)?,
    };

    // Return the result, if it has been handled by one of the specific handlers
    if !matches!(result, EventResult::NotHandled) {
        return Ok(result);
    }

    if matches!(
        current_ui_state,
        UiState::Games | UiState::Autosave | UiState::ManualSave
    ) {
        result = handle_main_view(event, terminal, state)?;
    }

    // Return the result, if it has been handled by the main view handler
    if !matches!(result, EventResult::NotHandled) {
        return Ok(result);
    }

    handle_exits(event, terminal)
}

/// Handle input during
fn handle_input(event: &KeyEvent, state: &mut AppState, mut input: Input) -> Result<EventResult> {
    match event.code {
        KeyCode::Esc => {
            // Abort the savegame cration process
            state.pop_state()?;
            return Ok(EventResult::Redraw);
        }
        KeyCode::Enter => {
            // Create a new save.
            match input.input_type {
                InputType::Create => {
                    manually_save_game(&state.config, &input.game, &input.input)?;
                    state.log(&format!(
                        "New manual save for {} with name '{}'",
                        &input.game, &input.input
                    ));
                    state.pop_state()?;
                    state.update_manual_saves()?;
                    return Ok(EventResult::Redraw);
                }
                InputType::Rename(save) => {
                    // Check if new filename already exists.
                    // If it does, whether the user wants to overwrite the existing file.
                    let parent_directory = save
                        .path
                        .parent()
                        .expect("Saves shouldn't be the root folder.");
                    if parent_directory
                        .join(format!("{}.tar.zst", &input.input))
                        .exists()
                    {
                        state.push_state(UiState::Prompt(PromptType::RenameOverwrite {
                            save,
                            new_name: input.input.clone(),
                        }));
                        return Ok(EventResult::Redraw);
                    }

                    // The destination doesn't exist yet. Simply ask if rename is correct.
                    state.push_state(UiState::Prompt(PromptType::Rename {
                        save,
                        new_name: input.input.clone(),
                    }));
                    return Ok(EventResult::Redraw);
                }
            }
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

/// Handle y/n prompts and do the appropriate action, depending on the prompt type.
fn handle_prompt(
    event: &KeyEvent,
    state: &mut AppState,
    prompt_type: PromptType,
) -> Result<EventResult> {
    match event.code {
        KeyCode::Char('n' | 'N') | KeyCode::Esc => {
            // Exit the prompt and enter the previous state.
            state.pop_state()?;
            return Ok(EventResult::Redraw);
        }
        KeyCode::Char('y' | 'Y') => match prompt_type {
            PromptType::RenameOverwrite { save, new_name }
            | PromptType::Rename { save, new_name } => {
                rename_save(&save, &new_name)?;
                state.update_saves()?;
                // Double pop the state, as we had to have an input beforehand.
                state.pop_state()?;
                state.pop_state()?;
                state.log(&format!("Renamed '{}' to '{}'", &save.file_name, &new_name));

                return Ok(EventResult::Redraw);
            }
            PromptType::CreateOverwrite { new_name, game } => {
                manually_save_game(&state.config, &game, &new_name)?;
                state.log(&format!(
                    "New manual save for {} with name '{}'",
                    &game, &new_name
                ));
                state.pop_state()?;
                state.pop_state()?;
                state.update_manual_saves()?;
                return Ok(EventResult::Redraw);
            }
            PromptType::Delete { save } => {
                delete_save(&save)?;
                state.log(&format!("Deleted save '{}'", &save.file_name));
                state.pop_state()?;
                match state.state {
                    UiState::Autosave => {
                        state.update_autosaves()?;
                        state.autosaves.focus();
                    }
                    UiState::ManualSave => {
                        state.update_manual_saves()?;
                        state.manual_saves.focus();
                    }
                    _ => bail!("Trying to delete when focus wasn't on a SaveList."),
                }
                return Ok(EventResult::Redraw);
            }
        },
        _ => {}
    }

    Ok(EventResult::Ignore)
}

/// Actions that are only possible when the game list is focused.
fn handle_game_list(event: &KeyEvent, state: &mut AppState) -> Result<EventResult> {
    match event.code {
        KeyCode::Down | KeyCode::Char('j') => {
            state.games.next();
            state.update_saves()?;
            return Ok(EventResult::Redraw);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.games.previous();
            state.update_saves()?;
            return Ok(EventResult::Redraw);
        }
        _ => {}
    }

    match event {
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('l'),
        } => {
            // Moving to the right moves focus to the save lists.
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
        _ => (),
    }

    Ok(EventResult::NotHandled)
}

/// Actions that are only possible when the autosave list is focused.
fn handle_autosave_list(event: &KeyEvent, state: &mut AppState) -> Result<EventResult> {
    match event {
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Down | KeyCode::Up | KeyCode::Char('j' | 'k'),
        } => {
            // Moving up down while focus is on the autosave list should switch focus
            // to the manual save list.
            state.state = UiState::ManualSave;
            state.manual_saves.focus();
            return Ok(EventResult::Redraw);
        }
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Left | KeyCode::Char('h'),
        } => {
            state.state = UiState::Games;
            return Ok(EventResult::Redraw);
        }
        _ => (),
    }

    match event.code {
        KeyCode::Down | KeyCode::Char('j') => {
            state.autosaves.next();
            return Ok(EventResult::Redraw);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.autosaves.previous();
            return Ok(EventResult::Redraw);
        }
        KeyCode::Delete | KeyCode::Char('d') => {
            // Delete a autosave
            if let Some(save) = state.autosaves.get_selected() {
                state.push_state(UiState::Prompt(PromptType::Delete { save }));
                return Ok(EventResult::Redraw);
            }
        }
        KeyCode::Char('r') => {
            // Rename a autosave
            if let Some(save) = state.autosaves.get_selected() {
                state.push_state(UiState::Input(Input {
                    game: state.get_selected_game(),
                    input: save.file_name.clone(),
                    input_type: InputType::Rename(save.clone()),
                }));
                return Ok(EventResult::Redraw);
            }
        }
        KeyCode::Enter => {
            // Restore a autosave game.
            if let Some(save) = state.autosaves.get_selected() {
                let game = state.get_selected_game();
                restore_save(&state.config, &game, &save)?;
                state.ignore_changes.insert(game.clone(), Local::now());
                state.log(&format!(
                    "Restored savefile {} for {}",
                    save.file_name, &game
                ));
                return Ok(EventResult::Redraw);
            }
        }
        _ => {}
    }
    Ok(EventResult::NotHandled)
}

/// Actions that are only possible when the manual save list is focused.
fn handle_manual_save_list(event: &KeyEvent, state: &mut AppState) -> Result<EventResult> {
    match event {
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Down | KeyCode::Up | KeyCode::Char('j' | 'k'),
        } => {
            // Moving up down while focus is on the manual save list should switch focus
            // to the autosave list. Only do this if autosaves are enabled.
            if state.selected_game_has_autosave() {
                state.state = UiState::Autosave;
                state.autosaves.focus();
                return Ok(EventResult::Redraw);
            }
        }
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Left | KeyCode::Char('h'),
        } => {
            state.state = UiState::Games;
            return Ok(EventResult::Redraw);
        }
        _ => (),
    }

    match event.code {
        KeyCode::Down | KeyCode::Char('j') => {
            state.manual_saves.next();
            return Ok(EventResult::Redraw);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.manual_saves.previous();
            return Ok(EventResult::Redraw);
        }
        KeyCode::Delete | KeyCode::Char('d') => {
            // Delete a autosave
            if let Some(save) = state.manual_saves.get_selected() {
                state.push_state(UiState::Prompt(PromptType::Delete { save }));
                return Ok(EventResult::Redraw);
            }
        }
        KeyCode::Char('r') => {
            // Rename a autosave
            if let Some(save) = state.manual_saves.get_selected() {
                state.push_state(UiState::Input(Input {
                    game: state.get_selected_game(),
                    input: save.file_name.clone(),
                    input_type: InputType::Rename(save.clone()),
                }));
                return Ok(EventResult::Redraw);
            }
        }
        KeyCode::Enter => {
            // Restore a autosave game.
            if let Some(save) = state.manual_saves.get_selected() {
                let game = state.get_selected_game();
                restore_save(&state.config, &game, &save)?;
                state.ignore_changes.insert(game.clone(), Local::now());
                state.log(&format!(
                    "Restored savefile '{}' for {}",
                    save.file_name, game
                ));
                return Ok(EventResult::Redraw);
            }
        }
        _ => {}
    }

    Ok(EventResult::NotHandled)
}

/// Actions that can be taken, when any component of the main user interface is focused.
/// -> No prompts are displayed.
/// -> No input is requested.
fn handle_main_view(
    event: &KeyEvent,
    terminal: &mut Terminal,
    state: &mut AppState,
) -> Result<EventResult> {
    //handle_global(event, terminal, state, current_ui_state);
    match event.code {
        KeyCode::Char('q') => {
            // 'q' instantly exits the program.
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        KeyCode::Char('a') => {
            let game = state.get_selected_game();
            // Create a new savegame for the current game.
            state.push_state(UiState::Input(Input {
                game,
                input: String::new(),
                input_type: InputType::Create,
            }));
            return Ok(EventResult::Redraw);
        }
        _ => {}
    }

    Ok(EventResult::NotHandled)
}

/// Handle all keys that exit the program.
fn handle_exits(event: &KeyEvent, terminal: &mut Terminal) -> Result<EventResult> {
    match event {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        } => {
            // Classict CTRL+C should kill the program
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
        } => {
            // 'q' instantly exits the program.
            restore_terminal(terminal)?;
            return Ok(EventResult::Quit);
        }
        _ => (),
    }

    Ok(EventResult::NotHandled)
}
