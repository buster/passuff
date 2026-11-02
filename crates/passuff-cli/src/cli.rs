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

    /// Enable diagnostic logs on stderr (debug unless RUST_LOG is set)
    #[arg(long, global = true)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run one observation pass and print notifications
    Check,
    /// Show accounts, watch scope, watched pull requests, and unread activity
    Status,
    /// List unread activity, or all activity with --all
    Inbox {
        #[arg(long)]
        all: bool,
    },
    /// Mark one activity read by id, or every activity with --all
    Read {
        /// Activity id, for example a12
        id: Option<String>,
        #[arg(long, conflicts_with = "id")]
        all: bool,
    },
}
