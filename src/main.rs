use std::{thread::sleep, time::Duration};

use anyhow::{Context, Result};
use clap::Clap;
use crossbeam_channel::unbounded;
use log::{debug, info, LevelFilter};
use pretty_env_logger::formatted_builder;

mod cli;
mod config;
mod ui;
mod watcher;

use config::Config;

fn main() -> Result<()> {
    // Parse commandline options.
    let opt = cli::CliArguments::parse();
    init_app(opt.verbosity);

    let config = Config::new(&opt.config)?;

    // Create the mpsc channel that's used to send notifications from the file watcher thread
    // to the actual application loop.
    let (sender, receiver) = unbounded();

    // Spawn all file-change watchers.
    info!("Spawning watchers");
    watcher::spawn_watchers(&config, &sender).context("Failed while spawning watchers")?;

    ui::run_tui(&config)?;

    info!("All watchers are spawned, waiting for updates");
    loop {
        sleep(Duration::from_secs(1));
        let updates = receiver.recv();

        for update in updates {
            ("Watchers spawned, waiting for updates");
            debug!(
                "{:?}: Detected changes for game {}",
                update.time, update.game_name
            );
            for path in update.locations {
                debug!("Change in path {:?}", path);
            }

            debug!("");
        }
    }
}

/// Run all boilerplate initialization code that's unrelated to actual application logic.
fn init_app(verbosity: u8) {
    // Beautify panics for better debug output.
    better_panic::install();

    // Set the verbosity level and initialize the logger.
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    let mut builder = formatted_builder();
    builder.filter(None, level).init();
    info!("Initialized logger with verbosity {}", level);
}
