use anyhow::{Context, Result};
use clap::Clap;
use crossbeam_channel::unbounded;
use log::{info, LevelFilter};
use pretty_env_logger::formatted_builder;

mod app;
mod cli;
mod config;
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
    info!("All watchers have been spawned, waiting for updates");

    // Run the actual main app.
    app::run(config, receiver)?;

    Ok(())
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
