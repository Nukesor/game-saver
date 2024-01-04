use anyhow::{Context, Result};
use clap::Parser;
use crossbeam_channel::unbounded;
use flexi_logger::Logger;
use log::{info, LevelFilter};

mod app;
mod cli;
mod config;
mod watcher;

use config::Config;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = cli::CliArguments::parse();
    init_app(opt.verbosity);

    let config = Config::new(&opt.config)?;

    // Create the mpsc channel that's used to send notifications from the file watcher thread
    // to the actual application loop.
    let (sender, receiver) = unbounded();

    // Spawn all file-change watchers.
    info!("Spawning watchers");
    watcher::spawn_watchers(&config, &sender)
        .await
        .context("Failed while spawning watchers")?;
    info!("All watchers have been spawned, waiting for updates");

    // Run the actual main app.
    app::run(config, receiver)?;

    Ok(())
}

/// Run all boilerplate initialization code that's unrelated to actual application logic.
fn init_app(verbosity: u8) {
    // Beautify panics for better debug output.
    better_panic::install();

    // This section handles Shutdown via SigTerm/SigInt process signals
    // Notify the TaskHandler, so it can shutdown gracefully.
    // The actual program exit will be done via the TaskHandler.
    ctrlc::set_handler(move || {
        std::process::exit(1);
    })
    .expect("Failed to set signal handler");

    // Set the verbosity level and initialize the logger.
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    let log_info = format!("{}, watchexec=warn", level.to_string().to_lowercase());
    Logger::try_with_str(log_info)
        .expect("Failed to init logger")
        .start()
        .expect("Failed to start logger");

    info!("Initialized logger with verbosity {}", level);
}
