use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use crossbeam_channel::Sender;
use log::{error, info};
use watchexec::{
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    error::CriticalError,
    event::{
        filekind::{FileEventKind, ModifyKind},
        Tag,
    },
    handler::PrintDebug,
    Watchexec,
};
use watchexec_filterer_globset::GlobsetFilterer;

use crate::config::{Config, GameConfig};

/// This is th message that will be send via the mpsc channel as soon as files change.
#[derive(Debug)]
pub struct Update {
    pub game_name: String,
    pub locations: Vec<PathBuf>,
    pub time: DateTime<Local>,
}

/// Convenience wrapper around `spawn_watcher` for multiple watchers.
pub async fn spawn_watchers(config: &Config, sender: &Sender<Update>) -> Result<()> {
    for (name, game_config) in &config.games {
        if !game_config.savegame_location().exists() {
            error!("Cannot find savegame_location for game {}", name);
            continue;
        }
        info!("Building watcher for {}", name);
        spawn_watcher(name, game_config, sender).await?;
    }

    Ok(())
}

/// Create a new watcher from a GameConfig and spin it of in its own thread.
/// As soon as files change, the handler sends notifications via the mpsc channel.
async fn spawn_watcher(
    game_name: &str,
    game_config: &GameConfig,
    sender: &Sender<Update>,
) -> Result<()> {
    let mut init = InitConfig::default();
    init.on_error(PrintDebug(std::io::stderr()));

    // Create the filter that enforces all ignored globs from the configuration file.
    let ignores: Vec<(String, Option<PathBuf>)> = game_config
        .ignored_files
        .iter()
        .map(|glob| (glob.clone(), None))
        .collect();
    let globset_filterer = GlobsetFilterer::new(
        game_config.savegame_location(),
        Vec::new(),
        ignores,
        Vec::new(),
        Vec::new(),
    );
    let globset_filterer = globset_filterer
        .await
        .context("Failed to init globset filter for game {game_name}")?;

    // Initialize the runtime configuration for watchexec
    let mut runtime = RuntimeConfig::default();
    runtime
        .pathset(vec![game_config.savegame_location()])
        .filterer(Arc::new(globset_filterer));

    // Define the handler that's called if any changes are detected.
    let sender = sender.clone();
    let game_name_clone = game_name.to_string();
    runtime.on_action(move |action: Action| {
        let sender_clone = sender.clone();
        let game_name_clone = game_name_clone.clone();
        async move {
            // Only trigger on File event types that're interesting for us.
            let mut should_trigger = false;
            let mut locations = Vec::new();
            for event in action.events.iter() {
                let mut interesting_event = false;
                for tag in &event.tags {
                    if let Tag::FileEventKind(fek) = tag {
                        match fek {
                            FileEventKind::Access(_) => continue,
                            FileEventKind::Modify(ModifyKind::Name(_)) => interesting_event = true,
                            FileEventKind::Modify(ModifyKind::Metadata(_)) => continue,
                            FileEventKind::Modify(_) => interesting_event = true,
                            FileEventKind::Create(_) => interesting_event = true,
                            FileEventKind::Remove(_) => continue,
                            _ => continue,
                        };
                    }
                }

                // Handle all interesting events.
                if interesting_event {
                    should_trigger = true;
                    event
                        .paths()
                        .for_each(|(path, _filetype)| locations.push(path.to_path_buf()))
                }
            }

            // If anything interesting happened, notify the main program about it.
            locations.dedup();
            if should_trigger {
                sender_clone
                    .send(Update {
                        game_name: game_name_clone,
                        locations,
                        time: Local::now(),
                    })
                    .expect("Failed to send update.");
            }

            action.outcome(Outcome::DoNothing);

            Ok::<(), CriticalError>(())
        }
    });

    // Init and spawn the watcher in new async task.
    let watcher = Watchexec::new(init, runtime)?;
    let game_name_clone = game_name.to_string();
    tokio::spawn(async move {
        if let Err(err) = watcher.main().await {
            eprintln!("Error in file watcher for game {game_name_clone}:\n{err:?}");
        };

        println!("Exiting file watcher worker for {game_name_clone}");
    });
    info!("Spawned watcher thread for {}", game_name);

    Ok(())
}
