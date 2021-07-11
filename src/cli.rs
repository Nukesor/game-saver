use std::path::PathBuf;

use clap::Clap;

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
    pub verbosity: u8,

    #[clap(short, long)]
    /// You can explicitly specify a configuration path.
    /// Otherwise the default path in "~/.local/share" will be used.
    pub config: Option<PathBuf>,
}
