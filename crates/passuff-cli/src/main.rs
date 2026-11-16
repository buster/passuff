use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use passuff::args::{Cli, Command};
use passuff::delivery::{FileDelivery, SystemDelivery};
use passuff::paths::PassuffPaths;
use passuff::runtime::{CliRuntime, CommandOutcome, Ports, SystemBrowserOpener};
use passuff::sound::SystemSoundPlayer;
use passuff_core::{SystemClock, SystemCredentialSource, SystemGatewayFactory};

fn main() {
    let cli = Cli::parse();
    let paths = PassuffPaths::resolve(cli.config.clone(), cli.state.clone());
    if cli.verbose {
        eprintln!("passuff: config {}", paths.config.display());
        eprintln!("passuff: state {}", paths.state.display());
    }
    let ports = real_ports();
    let interactive = ports.interactive;
    let runtime = Arc::new(CliRuntime::new(paths, ports));
    let no_desktop = cli.no_desktop;
    let no_sound = cli.no_sound;

    match cli.command.clone() {
        None => {
            let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .enable_io()
                .build()
                .expect("tokio runtime");
            tokio_runtime.block_on(watch_loop(runtime, interactive, no_desktop, no_sound));
        }
        Some(Command::Setup) => {
            let mut input = std::io::stdin().lock();
            let outcome = runtime.run_setup_command(&mut input);
            print_outcome(&outcome);
            std::process::exit(outcome.exit_code);
        }
        Some(_) => {
            let outcome = runtime.run(&cli);
            print_outcome(&outcome);
            std::process::exit(outcome.exit_code);
        }
    }
}

async fn watch_loop(runtime: Arc<CliRuntime>, interactive: bool, no_desktop: bool, no_sound: bool) {
    let outcome = tokio::task::spawn_blocking({
        let runtime = runtime.clone();
        move || runtime.run_watch_start_with_flags(no_desktop, no_sound)
    })
    .await
    .expect("watch start");
    print_outcome(&outcome);
    if outcome.exit_code == 2 {
        std::process::exit(2);
    }
    let mut interval = runtime
        .load_config()
        .map(|config| config.poll_interval_secs)
        .unwrap_or(60)
        .max(1);
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                std::process::exit(0);
            }
            _ = tokio::time::sleep(Duration::from_secs(interval)) => {
                if let Ok(config) = runtime.load_config() {
                    interval = config.poll_interval_secs.max(1);
                }
                let outcome = tokio::task::spawn_blocking({
                    let runtime = runtime.clone();
                    move || {
                        runtime.run_watch_cycle_reload_with_flags(
                            interactive,
                            no_desktop,
                            no_sound,
                        )
                    }
                })
                .await
                .expect("watch cycle");
                print_outcome(&outcome);
            }
        }
    }
}

fn real_ports() -> Ports {
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let delivery: Arc<dyn passuff::Delivery> =
        if let Ok(path) = std::env::var("PASSUFF_NOTIFICATION_LOG") {
            Arc::new(FileDelivery::new(path))
        } else {
            Arc::new(SystemDelivery)
        };
    Ports {
        gateway_factory: Arc::new(SystemGatewayFactory),
        credentials: Arc::new(SystemCredentialSource::new()),
        delivery,
        sound: Arc::new(SystemSoundPlayer),
        clock: Arc::new(SystemClock),
        browser: Arc::new(SystemBrowserOpener),
        interactive,
    }
}

fn print_outcome(outcome: &CommandOutcome) {
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(outcome.stdout.as_bytes());
    let _ = stdout.flush();
    let mut stderr = std::io::stderr();
    let _ = stderr.write_all(outcome.stderr.as_bytes());
    let _ = stderr.flush();
}
