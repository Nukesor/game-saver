use std::{path::PathBuf, thread};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use crossbeam_channel::Sender;
use log::{error, info};
use watchexec::{
    config::{Config as WatchexecConfig, ConfigBuilder},
    error::Result as WatchexecResult,
    pathop::PathOp,
    run::{watch, Handler},
};

use crate::config::{Config, GameConfig};

pub struct Update {
    pub game_name: String,
    pub locations: Vec<PathBuf>,
    pub time: DateTime<Local>,
}

struct MyHandler {
    pub game_name: String,
    config: WatchexecConfig,
    sender: Sender<Update>,
}

impl Handler for MyHandler {
    fn args(&self) -> WatchexecConfig {
        self.config.clone()
    }

    /// This shouldn't be called, as we don't configure the handler to run at startup.
    fn on_manual(&self) -> WatchexecResult<bool> {
        Ok(true)
    }

    /// Send an update notificarion via mpsc
    fn on_update(&self, ops: &[PathOp]) -> WatchexecResult<bool> {
        self.sender
            .send(Update {
                game_name: self.game_name.clone(),
                locations: ops.iter().map(|op| op.path.clone()).collect(),
                time: Local::now(),
            })
            .expect("Failed to send update.");
        Ok(true)
    }
}

/// Convenience wrapper around `spawn_watcher` for multiple watchers.
pub fn spawn_watchers(config: &Config, sender: &Sender<Update>) -> Result<()> {
    for (name, game_config) in &config.games {
        info!("Building watcher for {}", name);
        spawn_watcher(name, game_config, sender)?;
    }

    Ok(())
}

/// Create a new watcher from a GameConfig and spin it of in its own thread.
/// As soon as files change, the handler sends notifications via the mpsc channel.
fn spawn_watcher(game_name: &str, game_config: &GameConfig, sender: &Sender<Update>) -> Result<()> {
    let config = ConfigBuilder::default()
        .paths(vec![game_config.savegame_location()])
        .ignores(game_config.ignored_files.clone())
        .cmd(vec!["stub_cmd".to_string()])
        .build()?;

    let handler = MyHandler {
        config,
        game_name: game_name.into(),
        sender: sender.clone(),
    };

    thread::spawn(move || {
        if let Err(error) = watch(&handler).context("Handler failed") {
            error!("Got error in watcher thread!!!");
            error!("Thread: {}, error: {:?}", handler.game_name, error);
        }
    });
    info!("Spawned watcher thread for {}", game_name);

    Ok(())
}
