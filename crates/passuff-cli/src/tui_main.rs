use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use crossterm::event::{read, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use passuff_cli::scope::{ScopeEditor, ScopePicker};
use passuff_cli::tui::{load_view, TuiAction, TuiApp};
use passuff_config::{AppConfig, ConfigLoader, WatchMode};
use passuff_core::{Inbox, JsonStateStore, SnapshotStore};
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
    #[arg(long, value_name = "OWNER/REPO")]
    scope_select: Option<String>,
    #[arg(long, value_name = "OWNER/REPO")]
    scope_deselect: Option<String>,
    #[arg(long, value_name = "all|selected")]
    scope_mode: Option<String>,
    #[arg(long, value_name = "OWNER/REPO")]
    scope_add: Option<String>,
    #[arg(long)]
    picker: bool,
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

    let editor = ScopeEditor::new(config_path.clone());
    let state_store = JsonStateStore::new(state_path.clone());
    if let Some(mode) = args.scope_mode.as_deref() {
        let mode = match mode {
            "all" => WatchMode::All,
            "selected" => WatchMode::Selected,
            _ => {
                eprintln!("passuff-tui: --scope-mode must be all or selected");
                std::process::exit(2);
            }
        };
        if let Err(error) = editor.set_mode(mode) {
            eprintln!("passuff-tui: {}", error.message);
            std::process::exit(error.exit);
        }
    }
    if let Some(repository) = args.scope_select.as_deref() {
        if let Err(error) = editor.select(repository) {
            eprintln!("passuff-tui: {}", error.message);
            std::process::exit(error.exit);
        }
    }
    if let Some(repository) = args.scope_add.as_deref() {
        if let Err(error) = editor.add(repository) {
            eprintln!("passuff-tui: {}", error.message);
            std::process::exit(error.exit);
        }
    }
    if let Some(repository) = args.scope_deselect.as_deref() {
        if let Err(error) = editor.deselect(repository) {
            eprintln!("passuff-tui: {}", error.message);
            std::process::exit(error.exit);
        }
    }

    let config = ConfigLoader::load(&config_path);
    let inbox = Inbox::new(Arc::new(JsonStateStore::new(state_path.clone())));
    let view = match load_view(&config, &inbox).await {
        Ok(view) => view,
        Err(error) => {
            eprintln!("{}", error.message);
            std::process::exit(error.exit);
        }
    };
    let mut app = TuiApp::new(view);
    if args.snapshot {
        if args.picker {
            let config = config.expect("validated configuration");
            let state = state_store.load().await.unwrap_or_default();
            let known = editor.known_repositories(&config, &state);
            let picker = ScopePicker::new(&config, known);
            println!("{}", picker.snapshot());
        } else {
            println!("{}", app.snapshot());
        }
        return;
    }
    if !std::io::stdout().is_terminal() {
        eprintln!("passuff-tui: terminal UI requires a terminal (use --snapshot)");
        std::process::exit(3);
    }
    let config = config.expect("validated configuration");
    if let Err(message) =
        run_terminal(&mut app, &inbox, config, &editor, &state_store, args.picker).await
    {
        eprintln!("{message}");
        std::process::exit(3);
    }
}

async fn known_repositories(
    editor: &ScopeEditor,
    config: &AppConfig,
    store: &JsonStateStore,
) -> Vec<String> {
    let state = store.load().await.unwrap_or_default();
    editor.known_repositories(config, &state)
}

async fn reload_view(
    app: &mut TuiApp,
    inbox: &Inbox,
    config_path: &std::path::Path,
) -> Result<AppConfig, String> {
    let config = ConfigLoader::load(config_path)
        .map_err(|error| format!("passuff-tui: configuration error: {error}"))?;
    let view = load_view(&Ok(config.clone()), inbox)
        .await
        .map_err(|error| error.message)?;
    *app = TuiApp::new(view);
    Ok(config)
}

async fn run_terminal(
    app: &mut TuiApp,
    inbox: &Inbox,
    mut config: AppConfig,
    editor: &ScopeEditor,
    store: &JsonStateStore,
    start_picker: bool,
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
    let mut known = known_repositories(editor, &config, store).await;
    let mut picker = start_picker.then(|| ScopePicker::new(&config, known.clone()));
    let outcome = loop {
        if let Some(active) = picker.as_mut() {
            if let Err(error) = terminal.draw(|frame| active.render(frame)) {
                break Err(format!("passuff-tui: draw: {error}"));
            }
            let event = match tokio::task::spawn_blocking(read).await {
                Ok(Ok(event)) => event,
                Ok(Err(error)) => break Err(format!("passuff-tui: event: {error}")),
                Err(error) => break Err(format!("passuff-tui: event task: {error}")),
            };
            let Event::Key(key) = event else { continue };
            match active.handle_key(key, editor) {
                Ok(passuff_cli::scope::PickerOutcome::Close) => {
                    picker = None;
                }
                Ok(passuff_cli::scope::PickerOutcome::Continue) => {}
                Err(error) => {
                    eprintln!("passuff-tui: {}", error.message);
                }
            }
            match reload_view(app, inbox, editor.config_path()).await {
                Ok(reloaded) => {
                    config = reloaded;
                    known = known_repositories(editor, &config, store).await;
                    if let Some(active) = picker.as_mut() {
                        active.known = known.clone();
                        active.reload(&config);
                    }
                }
                Err(message) => break Err(message),
            }
            continue;
        }

        if let Err(error) = terminal.draw(|frame| app.render(frame)) {
            break Err(format!("passuff-tui: draw: {error}"));
        }
        let event = match tokio::task::spawn_blocking(read).await {
            Ok(Ok(event)) => event,
            Ok(Err(error)) => break Err(format!("passuff-tui: event: {error}")),
            Err(error) => break Err(format!("passuff-tui: event task: {error}")),
        };
        let Event::Key(key) = event else { continue };
        if matches!(key.code, crossterm::event::KeyCode::Char('p')) {
            picker = Some(ScopePicker::new(&config, known.clone()));
            continue;
        }
        match app.handle_key(key, inbox, &config.core).await {
            Ok(TuiAction::Quit) => break Ok(()),
            Ok(TuiAction::Continue) => {}
            Err(error) => break Err(format!("passuff: state error: {error}")),
        }
    };
    restore(&mut terminal);
    outcome
}

fn restore(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
}
