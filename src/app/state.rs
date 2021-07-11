use anyhow::Result;

use super::helper::files::get_archive_files;
use super::helper::list::StatefulList;
use crate::config::Config;
use crate::unwrap_or_ok;

/// This struct holds the state for the tui-rs interface.
/// This includes, lists, selected items as well as temporary input elements.
pub struct AppState {
    pub games: StatefulList,
    pub autosaves: StatefulList,
}

impl AppState {
    /// Create a new state with all games from the configuration file.
    pub fn new(config: &Config) -> Result<AppState> {
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
