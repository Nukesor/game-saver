use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

static DEFAULT_CONFIG: &'static str = include_str!("../example_game_saver.toml");

/// The config for one game
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameConfig {
    /// The folder where the save files are located.
    pub savegame_location: String,
    /// The amount of autosave slots you want to keep.
    /// Once this limit is reached, the oldest autosave files will be deleted.
    ///
    /// Set to 0, if you want to disable.
    pub autosaves: usize,
    /// By default, game-saver saves the game everytime something changes on disk.
    /// As this can be quite often, you can specify a timeout up to which all changes on disk will
    /// be simply ignored.
    ///
    /// The timeout is specified in seconds.
    /// Set to 0, to disable the timeout.
    pub autosave_timeout: usize,
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
    /// The directory where Game-saver will store the backups of your games' save files.
    pub backup_directory: String,
    pub games: HashMap<String, GameConfig>,
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
        let mut file = File::create(&path)?;
        file.write_all(&DEFAULT_CONFIG.as_bytes())?;

        // Recursively load config, now that we made sure it exists.
        let config = Config::new(&Some(path))?;
        Ok(config)
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Couldn't find config dir")?;
        Ok(config_dir.join("game_saver.toml"))
    }

    pub fn backup_directory(&self) -> PathBuf {
        PathBuf::from(tilde(&self.backup_directory).into_owned())
    }

    /// Get the backup directory for a specific game.
    pub fn save_dir(&self, name: &str) -> PathBuf {
        PathBuf::from(tilde(&self.backup_directory).into_owned()).join(name)
    }

    /// Get the autosave directory for a specific game.
    pub fn autosave_dir(&self, name: &str) -> PathBuf {
        self.save_dir(name).join("autosaves")
    }
}
