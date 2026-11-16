use std::io::{IsTerminal, Write};
use std::sync::Arc;

use clap::Parser;
use passuff::args::TuiCli;
use passuff::delivery::{FileDelivery, SystemDelivery};
use passuff::runtime::{Ports, SystemBrowserOpener};
use passuff::sound::SystemSoundPlayer;
use passuff::tui::{run_tui, TuiOptions};
use passuff_core::{SystemClock, SystemCredentialSource, SystemGatewayFactory};

fn main() {
    let cli = TuiCli::parse();
    let options = match TuiOptions::from_cli(&cli) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("passuff-tui: {error}");
            std::process::exit(2);
        }
    };
    let ports = real_ports();
    let outcome = run_tui(options, ports);
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(outcome.stdout.as_bytes());
    let _ = stdout.flush();
    let mut stderr = std::io::stderr();
    let _ = stderr.write_all(outcome.stderr.as_bytes());
    let _ = stderr.flush();
    std::process::exit(outcome.exit_code);
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
