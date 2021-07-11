use std::io::Stdout;

use anyhow::Result;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Frame as TuiFrame;

use crate::app::state::UiState;

use super::helper::list::StatefulList;
use super::helper::terminal::Terminal;
use super::AppState;

type Frame<'backend> = TuiFrame<'backend, CrosstermBackend<Stdout>>;

/// Draw the terminal ui.
/// This function doesn't change any state. Its sole purpose is to take the current state and
/// render the terminal ui epending on the app state.
pub fn draw_ui(terminal: &mut Terminal, state: &mut AppState) -> Result<()> {
    terminal.draw(|frame| {
        // Create two horizontally split chunks with 1/3 to 2/3
        // The left chunk will be the list of games
        // The right chunk will be used to display save games
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
            .split(frame.size());

        // Draw the list of games
        draw_list(frame, main_chunks[0], &mut state.games, "Games", true);

        // Split the right side into three chunks
        // - Autosave list
        // - Normal save list
        // - Block that's used as input field.
        let right_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Min(4),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(main_chunks[1]);

        // Draw the save lists
        draw_list(
            frame,
            right_chunks[0],
            &mut state.autosaves,
            "Autosaves",
            matches!(state.state, UiState::Autosave),
        );
        draw_list(
            frame,
            right_chunks[1],
            &mut state.manual_saves,
            "Saves",
            matches!(state.state, UiState::ManualSave),
        );

        // Draw the input field
        let paragraph = Paragraph::new(Text::from(state.input.clone()))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        frame.render_widget(paragraph, right_chunks[2]);
    })?;

    Ok(())
}

fn draw_list(
    frame: &mut Frame,
    chunk: Rect,
    stateful_list: &mut StatefulList,
    title: &str,
    highlight: bool,
) {
    // Create the game selection.
    let items: Vec<ListItem> = stateful_list
        .items
        .iter()
        .map(|name| ListItem::new(name.clone()))
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let mut list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
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

    // Render the list
    frame.render_stateful_widget(list, chunk, &mut stateful_list.state);
}
