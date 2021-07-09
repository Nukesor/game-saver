use anyhow::{Context, Result};
use clap::Clap;
use log::LevelFilter;
use pretty_env_logger::formatted_builder;
use watchexec::{
    config::{Config, ConfigBuilder},
    error::Result as WatchexecResult,
    pathop::PathOp,
    run::{watch, ExecHandler, Handler},
};

mod cli;

fn main() -> Result<()> {
    // Parse commandline options.
    let opt = cli::CliArguments::parse();
    init_app(opt.verbose);

    watcher()?;
    Ok(())
}

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
}

fn watcher() -> Result<()> {
    let config = ConfigBuilder::default()
        .clear_screen(true)
        .run_initially(true)
        .paths(vec!["/home/nuke/temp".into()])
        .cmd(vec!["ls".into()])
        .build()?;

    let handler = MyHandler(ExecHandler::new(config)?);
    println!("ROFL");
    watch(&handler).context("Handler failed")
}

struct MyHandler(ExecHandler);

impl Handler for MyHandler {
    fn args(&self) -> Config {
        self.0.args()
    }

    fn on_manual(&self) -> WatchexecResult<bool> {
        println!("Inital update");
        Ok(true)
    }

    fn on_update(&self, ops: &[PathOp]) -> WatchexecResult<bool> {
        println!("File updated: {:?}", ops);
        Ok(true)
    }
}
