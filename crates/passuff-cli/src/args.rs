use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "passuff",
    version,
    about = "Multi-account GitHub pull request inbox and desktop notifier",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, global = true, value_name = "PATH")]
    pub state: Option<PathBuf>,
    #[arg(long, global = true)]
    pub verbose: bool,
    #[arg(long, global = true)]
    pub no_desktop: bool,
    #[arg(long, global = true)]
    pub no_sound: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Run one observation pass and print notifications
    Check,
    /// Show accounts, watch scope, watched pull requests, and unread count
    Status,
    /// List unread activity, or all activity
    Inbox {
        #[arg(long)]
        all: bool,
    },
    /// Mark one activity record read, or every record
    Read {
        id: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Create the configuration interactively
    Setup,
    /// Send one test notification and play the configured sound
    NotifyTest,
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "passuff-tui",
    version,
    about = "Full-screen terminal inbox for passuff",
    disable_help_subcommand = true
)]
pub struct TuiCli {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub state: Option<PathBuf>,
    #[arg(long)]
    pub snapshot: bool,
    #[arg(long)]
    pub observe: bool,
    #[arg(long, value_name = "MILLIS")]
    pub observe_for: Option<u64>,
    #[arg(long, value_name = "OWNER/REPO")]
    pub scope_select: Option<String>,
    #[arg(long, value_name = "OWNER/REPO")]
    pub scope_deselect: Option<String>,
    #[arg(long, value_name = "all|selected")]
    pub scope_mode: Option<String>,
    #[arg(long, value_name = "RELATIONS")]
    pub focus_set: Option<String>,
    #[arg(long)]
    pub picker: bool,
    #[arg(long, value_name = "KEYS")]
    pub keys: Option<String>,
    #[arg(long)]
    pub no_desktop: bool,
    #[arg(long)]
    pub no_sound: bool,
}
