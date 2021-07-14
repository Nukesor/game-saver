use std::io::Stdout;

use anyhow::Result;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Text;
use tui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use tui::Frame as TuiFrame;

use crate::app::state::UiState;

use super::helper::list::{SaveList, StatefulList};
use super::helper::terminal::Terminal;
use super::AppState;

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

        // Split the right side into three chunks
        // - Autosave list
        // - Normal save list
        // - Block that's used as input field.
        let right_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(main_chunks[1]);

        // Draw autosave list
        let autosave_list = build_list(
            state
                .autosaves
                .items
                .iter()
                .map(|save| save.file_name.clone())
                .collect(),
            "Autosaves",
            matches!(state.state, UiState::Autosave(_)),
        );
        frame.render_stateful_widget(autosave_list, right_chunks[0], &mut state.autosaves.state);

        // Draw manual save list
        let manual_list = build_list(
            state
                .manual_saves
                .items
                .iter()
                .map(|save| save.file_name.clone())
                .collect(),
            "Saves",
            matches!(state.state, UiState::ManualSave(_)),
        );
        frame.render_stateful_widget(manual_list, right_chunks[1], &mut state.manual_saves.state);

        // Draw event log
        let event_log = build_list(state.event_log.clone(), "Event log", false);
        frame.render_widget(event_log, right_chunks[2]);

        // Draw the input field in the middle of the screen, if we're expecting input
        if let UiState::Input(input) = &state.state {
            let modal = get_modal(&mut frame);

            let paragraph = Paragraph::new(Text::from(input.input.clone()))
                .block(Block::default().borders(Borders::ALL).title("Name"));

            // Clear the input block and draw the input field
            frame.render_widget(paragraph, modal);
        }
    })?;

    Ok(())
}

fn build_list(items: Vec<String>, title: &str, highlight: bool) -> List {
    // Create the game selection.
    let items: Vec<ListItem> = items.into_iter().map(|name| ListItem::new(name)).collect();

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
