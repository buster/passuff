use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use crossterm::event::{read, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use passuff_cli::tui::{load_view, TuiAction, TuiApp};
use passuff_config::ConfigLoader;
use passuff_core::{Inbox, JsonStateStore};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

#[derive(Debug, Parser)]
#[command(
    name = "passuff-tui",
    version,
    about = "Browse the passuff activity inbox"
)]
struct Args {
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    state: Option<PathBuf>,
    #[arg(long)]
    snapshot: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
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

    let config = ConfigLoader::load(&config_path);
    let inbox = Inbox::new(Arc::new(JsonStateStore::new(state_path)));
    let view = match load_view(&config, &inbox).await {
        Ok(view) => view,
        Err(error) => {
            eprintln!("{}", error.message);
            std::process::exit(error.exit);
        }
    };
    let mut app = TuiApp::new(view);
    if args.snapshot {
        println!("{}", app.snapshot());
        return;
    }
    if !std::io::stdout().is_terminal() {
        eprintln!("passuff-tui: terminal UI requires a terminal (use --snapshot)");
        std::process::exit(3);
    }
    let config = config.expect("validated configuration");
    if let Err(message) = run_terminal(&mut app, &inbox, &config.core).await {
        eprintln!("{message}");
        std::process::exit(3);
    }
}

async fn run_terminal(
    app: &mut TuiApp,
    inbox: &dyn passuff_core::InboxService,
    config: &passuff_config::CoreConfig,
) -> Result<(), String> {
    enable_raw_mode().map_err(|error| format!("passuff-tui: raw mode: {error}"))?;
    let mut stdout = std::io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(format!("passuff-tui: alternate screen: {error}"));
    }
    let mut terminal = match Terminal::new(CrosstermBackend::new(stdout)) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = disable_raw_mode();
            return Err(format!("passuff-tui: terminal: {error}"));
        }
    };
    let outcome = drive(app, inbox, config, &mut terminal).await;
    restore(&mut terminal);
    outcome
}

async fn drive(
    app: &mut TuiApp,
    inbox: &dyn passuff_core::InboxService,
    config: &passuff_config::CoreConfig,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| app.render(frame))
            .map_err(|error| format!("passuff-tui: draw: {error}"))?;
        let event = tokio::task::spawn_blocking(read)
            .await
            .map_err(|error| format!("passuff-tui: event task: {error}"))?
            .map_err(|error| format!("passuff-tui: event: {error}"))?;
        let Event::Key(key) = event else {
            continue;
        };
        match app.handle_key(key, inbox, config).await {
            Ok(TuiAction::Quit) => return Ok(()),
            Ok(TuiAction::Continue) => {}
            Err(error) => return Err(format!("passuff: state error: {error}")),
        }
    }
}

fn restore(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
}
