//! passuff CLI and terminal adapters.

pub mod args;
pub mod delivery;
pub mod paths;
pub mod render;
pub mod runtime;
pub mod setup;
pub mod sound;
pub mod tui;

pub use args::{Cli, Command, TuiCli};
pub use delivery::{Delivery, DesktopMessage, FileDelivery, RecordingDelivery, SystemDelivery};
pub use paths::{ensure_bundled_sound, PassuffPaths};
pub use runtime::{
    BrowserOpener, CliRuntime, CommandOutcome, Ports, RecordingBrowserOpener, SystemBrowserOpener,
};
pub use sound::{RecordingSoundPlayer, SoundPlayer, SystemSoundPlayer};
pub use tui::{run_tui, snapshot as tui_snapshot, TuiOptions};
