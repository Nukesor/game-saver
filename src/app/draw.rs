use anyhow::Result;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem};

use super::helper::terminal::Terminal;
use super::AppState;

/// Draw the terminal ui.
/// This function doesn't change any state. Its sole purpose is to take the current state and
/// render the terminal ui epending on the app state.
pub fn draw_ui(terminal: &mut Terminal, state: &mut AppState) -> Result<()> {
    terminal.draw(|f| {
        // Create two chunks with equal horizontal screen space
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        // Create the game selection.
        let games: Vec<ListItem> = state
            .games
            .items
            .iter()
            .map(|name| ListItem::new(name.clone()))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let games = List::new(games)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // Render the game list
        f.render_stateful_widget(games, chunks[0], &mut state.games.state);
    })?;

    Ok(())
}
