use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
pub enum SubCommand {}

#[derive(Clap, Debug)]
#[clap(
    name = "A Game Saver",
    about = "Save your games",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION")
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[clap(short, long)]
    pub config: Option<PathBuf>,
    //    #[clap(subcommand)]
    //    pub cmd: SubCommand,
}
