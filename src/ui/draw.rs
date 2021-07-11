use anyhow::Result;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem};

use super::helper::terminal::Terminal;
use super::App;

pub fn draw_ui(terminal: &mut Terminal, app: &mut App) -> Result<()> {
    terminal.draw(|f| {
        // Create two chunks with equal horizontal screen space
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        // Iterate through all elements in the `items` app and append some debug text to it.
        let items: Vec<ListItem> = app
            .items
            .items
            .iter()
            .map(|name| ListItem::new(name.clone()))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // We can now render the item list
        f.render_stateful_widget(items, chunks[0], &mut app.items.state);
    })?;

    Ok(())
}
