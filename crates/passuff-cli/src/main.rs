mod cli;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use passuff_cli::delivery::DesktopDelivery;
use passuff_cli::sound::SystemSoundPlayer;
use passuff_cli::{system_credentials, CliPorts, CliRuntime};
use passuff_config::ConfigLoader;
use passuff_core::{GitHubApiAdapter, JsonStateStore};

use crate::cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    if args.verbose {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }

    let dirs = directories::ProjectDirs::from("dev", "passuff", "passuff");
    let config_path = args.config.unwrap_or_else(|| {
        dirs.as_ref()
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("passuff.toml"))
    });
    let state_path = args.state.unwrap_or_else(|| {
        dirs.as_ref()
            .map(|dirs| {
                dirs.state_dir()
                    .unwrap_or_else(|| dirs.data_dir())
                    .join("state.json")
            })
            .unwrap_or_else(|| PathBuf::from("passuff-state.json"))
    });

    let interactive =
        args.command.is_none() && std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let credentials = system_credentials(interactive);
    let sound = Arc::new(SystemSoundPlayer::new());
    let delivery = build_delivery(sound);
    let ports = CliPorts {
        gateway: Arc::new(GitHubApiAdapter::new()),
        store: Arc::new(JsonStateStore::new(state_path)),
        credentials,
        delivery,
    };
    let runtime = CliRuntime::new(ports);
    let config = ConfigLoader::load(&config_path);

    let outcome = match args.command {
        Some(Command::Check) => runtime.run_check(config).await,
        None => runtime.run_watch(config).await,
    };
    if !outcome.emitted && !outcome.transcript.is_empty() {
        println!("{}", outcome.transcript);
    }
    std::process::exit(outcome.status);
}

fn build_delivery(
    sound: Arc<SystemSoundPlayer>,
) -> Arc<dyn passuff_cli::delivery::NotificationDelivery> {
    #[cfg(feature = "test-support")]
    {
        if let Ok(sink) = std::env::var("PASSUFF_NOTIFICATION_SINK") {
            return Arc::new(passuff_cli::delivery::RecordingDelivery::with_sink(
                sound,
                PathBuf::from(sink),
            ));
        }
    }
    Arc::new(DesktopDelivery::new(sound))
}
