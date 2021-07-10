use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

/// The config for one game
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameConfig {
    /// The folder where the save files are located.
    pub savegame_location: String,
    /// Determines, whether there are multiple saves located in the folder or whether the specified
    /// folder itself is a singular save of the game.
    pub singular_save: bool,
    /// The amount of autosaves to keep, as soon as changes are detected.
    /// Set to 0, if you want to disable autosaves.
    pub autosave: usize,
    /// A list of glob patterns that should be ignored.
    /// The paths should be relative to `savegame_location/`.
    ///
    /// `.ignore` Files will also be respected.
    pub ignored_files: Vec<String>,
}

impl GameConfig {
    pub fn savegame_location(&self) -> PathBuf {
        PathBuf::from(tilde(&self.savegame_location).into_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub backup_directory: String,
    pub games: HashMap<String, GameConfig>,
}

impl Config {
    pub fn backup_directory(&self) -> PathBuf {
        PathBuf::from(tilde(&self.backup_directory).into_owned())
    }
}

impl Config {
    /// Either get the config from an existing configuration file or
    /// create a new one from scratch
    pub fn new(path: &Option<PathBuf>) -> Result<Self> {
        let path = if let Some(path) = path {
            path.clone()
        } else {
            Config::get_config_path()?
        };

        // The config file exists. Try to parse it
        if path.exists() {
            let mut file = File::open(path)?;
            let mut config = String::new();
            file.read_to_string(&mut config)?;

            let config: Config = toml::from_str(&config)?;
            return Ok(config);
        }

        // No config exists yet. Create a default config and persist it onto disk.
        let default_config = Config {
            backup_directory: "~/.local/share/game_saver/".into(),
            games: HashMap::new(),
        };
        default_config.write()?;

        Ok(default_config)
    }

    /// Write the current config to disk.
    pub fn write(&self) -> Result<()> {
        let path = Config::get_config_path()?;

        // The config file exists. Try to parse it
        let mut file = if path.exists() {
            File::open(path)?
        } else {
            File::create(path)?
        };

        let config = toml::to_string(&self)?;
        file.write_all(&config.as_bytes())?;

        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Couldn't find config dir")?;
        Ok(config_dir.join("game_saver.toml"))
    }
}
