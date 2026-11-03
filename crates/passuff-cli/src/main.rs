mod cli;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use passuff_cli::delivery::DesktopDelivery;
use passuff_cli::inbox_cli::InboxCli;
use passuff_cli::sound::SystemSoundPlayer;
use passuff_cli::{system_credentials, CliPorts, CliRuntime};
use passuff_config::ConfigLoader;
use passuff_core::{GitHubApiAdapter, Inbox, JsonStateStore};

use crate::cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    passuff_cli::init_logging(args.verbose);

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

    tracing::debug!(
        config = %config_path.display(),
        state = %state_path.display(),
        "starting passuff"
    );
    let interactive =
        args.command.is_none() && std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if matches!(args.command, Some(Command::Setup)) {
        let mut output = std::io::stdout();
        let result = passuff_cli::setup::SetupSession::new(
            config_path.clone(),
            std::io::stdin().lock(),
            &mut output,
        )
        .run();
        match result {
            Ok(()) => std::process::exit(0),
            Err(error) => std::process::exit(error.exit),
        }
    }
    let state_store = Arc::new(JsonStateStore::new(state_path));
    let config = ConfigLoader::load(&config_path);
    let inbox_cli = || InboxCli::new(Arc::new(Inbox::new(state_store.clone())));
    let build_runtime = |interactive_credentials: bool| {
        let credentials = system_credentials(interactive_credentials);
        let sound = Arc::new(SystemSoundPlayer::new());
        let delivery = build_delivery(sound);
        CliRuntime::new(CliPorts {
            gateway: Arc::new(GitHubApiAdapter::new()),
            store: state_store.clone(),
            credentials,
            delivery,
        })
        .with_config_path(config_path.clone())
    };

    let outcome = match args.command {
        Some(Command::Setup) => unreachable!("setup handled before config load"),
        Some(Command::Check) => build_runtime(false).run_check(config).await,
        Some(Command::Status) => inbox_cli().status(config).await,
        Some(Command::Inbox { all }) => inbox_cli().inbox(config, all).await,
        Some(Command::Read { id, all }) => inbox_cli().read(config, id, all).await,
        None => build_runtime(interactive).run_watch(config).await,
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
