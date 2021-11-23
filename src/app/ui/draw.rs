use std::io::Stdout;

use anyhow::Result;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame as TuiFrame,
};

use super::state::{AppState, PromptType, UiState};
use crate::app::helper::terminal::Terminal;

type Frame<'backend> = TuiFrame<'backend, CrosstermBackend<Stdout>>;

/// Draw the terminal ui.
/// This function doesn't change any state. Its sole purpose is to take the current state and
/// render the terminal ui epending on the app state.
pub fn draw_ui(terminal: &mut Terminal, state: &mut AppState) -> Result<()> {
    terminal.draw(|mut frame| {
        // Create two horizontally split chunks with 1/3 to 2/3
        // The left chunk will be the list of games
        // The right chunk will be used to display save games
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
            .split(frame.size());

        // Draw the list of games
        let game_list = build_list(state.games.items.clone(), "Games", true);
        frame.render_stateful_widget(game_list, main_chunks[0], &mut state.games.state);

        let game_config = state.config.games.get(&state.get_selected_game()).unwrap();

        // Split the right side into either two or three chunks
        // - Autosave list -> Dependant on whether the selected game has autosaves enabled
        // - Normal save list
        // - Block that's used as input field.
        let (autosave_chunk, manual_chunk, event_log_chunk) = if game_config.autosaves != 0 {
            let chunks = Layout::default()
                .constraints(
                    [
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                    ]
                    .as_ref(),
                )
                .split(main_chunks[1]);

            (Some(chunks[0]), chunks[1], chunks[2])
        } else {
            let chunks = Layout::default()
                .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
                .split(main_chunks[1]);
            (None, chunks[0], chunks[1])
        };

        if let Some(chunk) = autosave_chunk {
            // Draw autosave list
            let autosave_list = build_list(
                state
                    .autosaves
                    .items
                    .iter()
                    .map(|save| save.file_name.clone())
                    .collect(),
                "Autosaves",
                matches!(state.state, UiState::Autosave),
            );
            frame.render_stateful_widget(autosave_list, chunk, &mut state.autosaves.state);
        }

        // Draw manual save list
        let manual_list = build_list(
            state
                .manual_saves
                .items
                .iter()
                .map(|save| save.file_name.clone())
                .collect(),
            "Saves",
            matches!(state.state, UiState::ManualSave),
        );
        frame.render_stateful_widget(manual_list, manual_chunk, &mut state.manual_saves.state);

        // Draw event log
        let event_log = build_list(state.event_logs.items.clone(), "Event log", false);
        frame.render_stateful_widget(event_log, event_log_chunk, &mut state.event_logs.state);

        // Draw the input field in the middle of the screen, if we're expecting input
        if let UiState::Input(input) = &state.state {
            let modal = get_modal(&mut frame);

            let paragraph = Paragraph::new(Text::from(input.input.clone())).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Savefile Name"),
            );
            frame.render_widget(paragraph, modal);
        }

        if let UiState::Prompt(prompt_type) = &state.state {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Are you sure?");

            let text = get_prompt_text(prompt_type, state.get_selected_game());
            let paragraph = Paragraph::new(text).block(block);

            let modal = get_modal(&mut frame);
            frame.render_widget(paragraph, modal);
        }
    })?;

    Ok(())
}

fn build_list(items: Vec<String>, title: &str, highlight: bool) -> List {
    // Create the game selection.
    let items: Vec<ListItem> = items.into_iter().map(ListItem::new).collect();

    // Create a List from all list items and highlight the currently selected one
    let mut list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title.clone()))
        .highlight_symbol(">> ");

    // Only do highlight styling, if it's the focused window.
    // The selected item can still be identified by the highlight_symbol.
    if highlight {
        list = list.highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    }

    list
}

/// Create a block with 3 height and 3/4 of the screen's width.
/// The block is positioned in the middle of the screen and is used as an modal.
/// We clear that block before returning it, that way you can directly write onto it.
fn get_modal(frame: &mut Frame) -> Rect {
    // Get the vertical middle of the screen.
    let overlay_vertical = Layout::default()
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Length(3),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(frame.size());

    // Get the horizontal middle of the screen.
    let overlay_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 8),
                Constraint::Ratio(3, 4),
                Constraint::Ratio(1, 8),
            ]
            .as_ref(),
        )
        .split(overlay_vertical[1]);

    // Clear the input block, so we can simply draw on it in the parent function.
    frame.render_widget(Clear, overlay_horizontal[1]);

    overlay_horizontal[1]
}

fn get_prompt_text(prompt_type: &PromptType, game: String) -> Text {
    let message = match prompt_type {
        PromptType::Delete { save } => {
            format!(
                "Delete the savefile '{}' for game {}",
                &save.file_name, game
            )
        }
        PromptType::Rename { save, new_name } => {
            format!("Rename the save '{}' to '{}'", &save.file_name, &new_name)
        }
        PromptType::RenameOverwrite { save, new_name } => {
            format!(
                "Do you realy want to overwrite save '{}' with '{}'",
                &new_name, &save.file_name
            )
        }
        PromptType::CreateOverwrite { new_name, .. } => {
            format!("Do you really want to overwrite save '{}'", &new_name)
        }
    };

    Text::from(format!("{} (Y/n)", message))
}
