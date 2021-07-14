use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local};

use super::helper::files::{get_archive_files, SaveFile};
use super::helper::list::{SaveList, StatefulList};
use crate::config::{Config, GameConfig};

/// This indicates the current focused part of the UI.
#[derive(Clone)]
pub enum UiState {
    Games(String),
    Autosave(String),
    ManualSave(String),
    /// The user is in the middle of writing something into the input field.
    Input(Input),
    /// The user is in the middle of writing something into the input field.
    Prompt(PromptType),
}

#[derive(Clone)]
pub struct Input {
    pub game: String,
    pub input: String,
    pub input_type: InputType,
}

#[derive(Clone)]
pub enum InputType {
    /// Create a new save for a specific game
    Create,
    /// Rename an existing save file.
    Rename(SaveFile),
}

#[derive(Clone)]
pub enum PromptType {
    Rename {
        save: SaveFile,
        new_name: String,
    },
    RenameOverwrite {
        save: SaveFile,
        new_name: String,
    },
    CreateOverwrite {
        input: String,
        game: String,
        config: GameConfig,
    },
}

/// This struct holds the state for the tui-rs interface.
/// This includes, lists, selected items as well as temporary input elements.
pub struct AppState {
    /// A local clone of the config for convenience purposes.
    pub config: Config,

    // All lists that are displayed in the app
    pub games: StatefulList,
    pub autosaves: SaveList,
    pub manual_saves: SaveList,
    /// This is a non-persisted event log, which is used to show the user performed actions.
    pub event_log: Vec<String>,

    // As we have an interactive UI, we have to do a lot of state management
    /// This represents the current active state.
    pub state: UiState,
    /// Remember the previous state. That way we can return to the correct position after prompts.
    pub previous_state: UiState,

    /// This map is used to store games that recently changed on disk.
    /// We perform changes once there haven't been any changes for some time.
    /// That's why we have to cache this state for a little while.
    pub changes_detected: HashMap<String, DateTime<Local>>,
    /// This map is used to temporarily ignore changes on the filesystem.
    /// This is needed so we don't trigger saves when restoring saves.
    /// (As the restore is a change in the filesystem that get's detected).
    pub ignore_changes: HashMap<String, DateTime<Local>>,
}

impl AppState {
    /// Create a new state with all games from the configuration file.
    pub fn new(config: &Config) -> Result<AppState> {
        // Get the very first game of the config. This will be auto-selected.
        // If no game exists, we just abort with an error message.
        let game = if let Some(name) = config.games.keys().next() {
            name.clone()
        } else {
            bail!("There must be at least one game in your config.");
        };

        // Get a list of all games in the config
        let mut items = vec![];
        for name in config.games.keys() {
            items.push(name.clone());
        }
        items.sort();

        let mut state = AppState {
            games: StatefulList::with_items(items),
            autosaves: SaveList::with_items(Vec::new()),
            manual_saves: SaveList::with_items(Vec::new()),
            event_log: Vec::new(),
            state: UiState::Games(game.clone()),
            previous_state: UiState::Games(game),
            config: config.clone(),
            changes_detected: HashMap::new(),
            ignore_changes: HashMap::new(),
        };
        // Load the list of saves if we selected a game.
        state.update_saves()?;
        state.update_autosaves()?;
        state.autosaves.autoselect_first();
        state.manual_saves.autoselect_first();

        Ok(state)
    }

    pub fn set_state(&mut self, state: UiState) {
        self.previous_state = self.state.clone();
        self.state = state;
    }

    pub fn get_state(&mut self) -> UiState {
        self.state.clone()
    }

    /// Return the name of the currently selected game.
    pub fn get_selected_game(&self) -> String {
        self.games
            .get_selected()
            .expect("We make sure there's at least one game, before creating the state.")
    }

    /// Return whether we have to handle autosave or not.
    pub fn selected_game_has_autosave(&self) -> bool {
        let game_name = self.get_selected_game();
        let game_config = self.config.games.get(&game_name).unwrap();
        if game_config.autosaves == 0 {
            return false;
        }

        return true;
    }

    pub fn log(&mut self, message: &str) {
        let prefix = Local::now().format("%H:%M:%S").to_string();
        self.event_log.push(format!("{} - {}", prefix, message));
    }

    /// Convenience wrapper, which calls [self.update_saves] and [self.update_autosaves].
    pub fn update_saves(&mut self) -> Result<()> {
        self.update_autosaves()
            .context("Failed while updating autosaves")?;
        self.update_manual_saves()
            .context("Failed while updating manual")
    }

    /// Update the list of saves that're currently in the autosave folder of the selected game.
    pub fn update_autosaves(&mut self) -> Result<()> {
        let name = self.get_selected_game();

        // Return early, if autosaves are disabled for the currently selected game.
        if !self.selected_game_has_autosave() {
            return Ok(());
        }

        let autosave_dir = self.config.autosave_dir(&name);
        let saves = get_archive_files(&autosave_dir)?;

        self.autosaves.items = saves;
        Ok(())
    }

    /// Update the list of saves that're currently in the savegame folder of the selected game.
    pub fn update_manual_saves(&mut self) -> Result<()> {
        let name = self.get_selected_game();

        // Return early, if autosaves are disabled for the currently selected game.
        let game_config = self.config.games.get(&name).unwrap();
        if game_config.autosaves == 0 {
            return Ok(());
        }

        let save_dir = self.config.backup_directory().join(name);
        let saves = get_archive_files(&save_dir)?;

        self.manual_saves.items = saves;
        Ok(())
    }
}
