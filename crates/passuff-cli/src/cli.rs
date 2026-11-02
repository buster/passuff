use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "passuff",
    version,
    about = "Watch authored GitHub pull requests and notify on eligible activity"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, value_name = "PATH")]
    pub state: Option<PathBuf>,

    #[arg(long, global = true)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run one observation pass and print notifications
    Check,
}
