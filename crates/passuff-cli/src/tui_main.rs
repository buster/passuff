use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use crossterm::event::{read, Event, KeyEvent};
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
    let app = TuiApp::new(view);
    if args.snapshot {
        println!("{}", app.snapshot());
        return;
    }
    if !std::io::stdout().is_terminal() {
        eprintln!("passuff: terminal UI requires a terminal (use --snapshot)");
        std::process::exit(3);
    }
    let config = config.expect("validated configuration");
    enable_raw_mode().expect("raw mode");
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("alternate screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let mut app = app;
    loop {
        terminal.draw(|frame| app.render(frame)).expect("draw");
        let event = tokio::task::spawn_blocking(read)
            .await
            .expect("event task")
            .expect("event");
        let Event::Key(key) = event else {
            continue;
        };
        match handle(&mut app, key, &inbox, &config.core).await {
            Ok(TuiAction::Quit) => break,
            Ok(TuiAction::Continue) => {}
            Err(error) => {
                restore(&mut terminal);
                eprintln!("passuff: state error: {error}");
                std::process::exit(3);
            }
        }
    }
    restore(&mut terminal);
}

async fn handle(
    app: &mut TuiApp,
    key: KeyEvent,
    inbox: &dyn passuff_core::InboxService,
    config: &passuff_config::CoreConfig,
) -> Result<TuiAction, passuff_core::CoreError> {
    app.handle_key(key, inbox, config).await
}

fn restore(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
}
