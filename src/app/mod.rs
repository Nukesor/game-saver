use std::fs::create_dir;
use std::time::Duration;

use anyhow::{Context, Result};
use crossbeam_channel::Receiver;
use crossterm::event::{poll, read, Event, KeyCode};

mod draw;
mod helper;

use crate::config::Config;
use crate::unwrap_or_ok;
use crate::watcher::Update;

use draw::draw_ui;
use helper::files::get_archive_files;
use helper::list::StatefulList;

/// This struct holds the state for the tui-rs interface.
/// This includes, lists, selected items as well as temporary input elements.
pub struct AppState {
    games: StatefulList,
    autosaves: StatefulList,
}

impl AppState {
    /// Create a new state with all games from the configuration file.
    fn new(config: &Config) -> Result<AppState> {
        let mut items = vec![];
        for (name, _) in &config.games {
            items.push(name.clone());
        }
        let mut state = AppState {
            games: StatefulList::with_items(items),
            autosaves: StatefulList::with_items::<String>(Vec::new()),
        };
        // Load the list of autosaves if we selected a game.
        state.update_autosaves(config)?;

        Ok(state)
    }

    /// Return the name of the currently selected game.
    fn get_selected_game(&mut self) -> Option<String> {
        let selected = self.games.state.selected()?;
        let game = self
            .games
            .items
            .get(selected)
            .expect("The game should exist as it's selected")
            .clone();

        Some(game)
    }

    fn update_autosaves(&mut self, config: &Config) -> Result<()> {
        let name = unwrap_or_ok!(self.get_selected_game());

        // Return early, if autosaves are disabled for the currently selected game.
        let game_config = config.games.get(&name).unwrap();
        if game_config.autosave == 0 {
            return Ok(());
        }

        let autosave_dir = config.backup_directory().join(name).join("autosaves");
        let saves = get_archive_files(&autosave_dir)?;

        self.autosaves.items = saves;
        self.autosaves.autoselect_first();
        Ok(())
    }
}

pub fn run(config: Config, receiver: Receiver<Update>) -> Result<()> {
    // Create a new app with some example state
    let mut state = AppState::new(&config)?;
    init_directories(&config)?;

    let mut terminal = helper::terminal::init_terminal()?;

    // One initial clear and draw
    // From now on, we only redraw, if there're actual changes.
    terminal.clear()?;
    draw_ui(&mut terminal, &mut state)?;

    loop {
        // This is a simple example on how to handle events
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        helper::terminal::restore_terminal(terminal)?;
                        break;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {}
                    KeyCode::Right | KeyCode::Char('l') => {}
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.games.next();
                        draw_ui(&mut terminal, &mut state)?;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.games.previous();
                        draw_ui(&mut terminal, &mut state)?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let updates = receiver.recv();
        for update in updates {}
    }

    Ok(())
}

/// Create all directories that are needed for normal operation.
fn init_directories(config: &Config) -> Result<()> {
    let backup_dir = config.backup_directory();
    // Create the backup directory
    create_dir(&backup_dir).context("Failed to create backup directory")?;

    // Create subfolders for each game
    for (name, config) in &config.games {
        let game_backup_dir = backup_dir.join(name);
        if !game_backup_dir.exists() {
            create_dir(&game_backup_dir).context(format!(
                "Failed to create backup directory for game {}",
                name
            ))?;
        }

        // Create an autosave directory, if autosave is enabled for this game.
        if config.autosave > 0 {
            let autosave_dir = game_backup_dir.join("autosaves");
            create_dir(autosave_dir).context(format!(
                "Failed to create autosave directory for game {}",
                name
            ))?;
        }
    }

    Ok(())
}
