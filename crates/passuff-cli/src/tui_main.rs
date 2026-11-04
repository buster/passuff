use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use passuff_cli::browser::{BrowserOpener, SystemBrowserOpener};
use passuff_cli::delivery::build_delivery;
use passuff_cli::live::{ObservationSession, ObservationUpdate};
use passuff_cli::scope::{ScopeEditor, ScopePicker};
use passuff_cli::sound::SystemSoundPlayer;
use passuff_cli::tui::{load_view, TuiAction, TuiApp, TuiPreferences};
use passuff_cli::{system_credentials, CliPorts, CliRuntime};
use passuff_config::{AppConfig, ConfigLoader, WatchMode};
use passuff_core::{GitHubApiAdapter, Inbox, JsonStateStore, SnapshotStore};
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
    #[arg(long)]
    observe: bool,
    #[arg(long, value_name = "MILLIS")]
    observe_for: Option<u64>,
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
    #[arg(long, value_name = "KEYS")]
    keys: Option<String>,
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

    if !config_path.exists() {
        if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
            let mut output = std::io::stdout();
            let result = passuff_cli::setup::SetupSession::new(
                config_path.clone(),
                std::io::stdin().lock(),
                &mut output,
            )
            .run();
            if let Err(error) = result {
                std::process::exit(error.exit);
            }
        } else {
            eprintln!("passuff: no configuration; run passuff setup");
            std::process::exit(2);
        }
    }
    let mut config = ConfigLoader::load(&config_path);
    let inbox = Inbox::new(Arc::new(JsonStateStore::new(state_path.clone())));
    let view = match load_view(&config, &inbox).await {
        Ok(view) => view,
        Err(error) => {
            eprintln!("{}", error.message);
            std::process::exit(error.exit);
        }
    };
    let mut app = TuiApp::new(view);

    if args.observe || args.observe_for.is_some() {
        let active = config.as_ref().expect("validated configuration").clone();
        let mut session = match ObservationSession::open(
            runtime_for(&config_path, &state_path),
            active.clone(),
            config_path.clone(),
        )
        .await
        {
            Ok(session) => session,
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(3);
            }
        };
        let first = session.tick().await;
        config = Ok(first.config.clone());
        if let Err(error) = app.reload(&inbox, &first.config.core).await {
            eprintln!("passuff: state error: {error}");
            std::process::exit(3);
        }
        app.set_status(first.failure);
        if let Some(millis) = args.observe_for {
            observe_for(&mut app, &inbox, &mut session, first.next_delay, millis).await;
        }
    }

    if let Some(keys) = args.keys.as_deref() {
        let mut active = config.as_ref().expect("validated configuration").clone();
        for key in keys.split(',').map(str::trim).filter(|key| !key.is_empty()) {
            let code = match key {
                "tab" => KeyCode::Tab,
                "backtab" => KeyCode::BackTab,
                "j" | "down" => KeyCode::Char('j'),
                "k" | "up" => KeyCode::Char('k'),
                "r" => KeyCode::Char('r'),
                "a" => KeyCode::Char('a'),
                "q" => KeyCode::Char('q'),
                "enter" => KeyCode::Enter,
                "esc" => KeyCode::Esc,
                "left" => KeyCode::Left,
                "right" => KeyCode::Right,
                "pageup" => KeyCode::PageUp,
                "pagedown" => KeyCode::PageDown,
                "home" => KeyCode::Home,
                "end" => KeyCode::End,
                "x" => KeyCode::Char('x'),
                other => {
                    eprintln!("passuff-tui: unknown key {other}");
                    std::process::exit(2);
                }
            };
            match app
                .handle_key(
                    KeyEvent::new(code, KeyModifiers::NONE),
                    &inbox,
                    &active.core,
                )
                .await
            {
                Ok(action) => {
                    if let Err(message) = apply_action(
                        &mut app,
                        action,
                        &inbox,
                        &mut active,
                        &editor,
                        &SystemBrowserOpener::default(),
                    )
                    .await
                    {
                        eprintln!("{message}");
                        std::process::exit(3);
                    }
                }
                Err(error) => {
                    eprintln!("passuff: state error: {error}");
                    std::process::exit(3);
                }
            }
        }
    }
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
    let preferences_path = state_path.with_file_name("ui.json");
    let saved_preferences = load_preferences(&preferences_path).unwrap_or_default();
    app.apply_preferences(&saved_preferences);
    if let Err(message) = run_terminal(
        &mut app,
        &inbox,
        config,
        &editor,
        &state_store,
        args.picker,
        runtime_for(&config_path, &state_path),
        config_path,
        &preferences_path,
        &saved_preferences,
    )
    .await
    {
        eprintln!("{message}");
        std::process::exit(3);
    }
}

fn load_preferences(path: &Path) -> Option<TuiPreferences> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_preferences(path: &Path, preferences: &TuiPreferences) -> Result<(), String> {
    let text = serde_json::to_string_pretty(preferences)
        .map_err(|error| format!("cannot save view settings: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, text)
        .map_err(|error| format!("cannot save view settings: {error}"))?;
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("cannot save view settings: {error}"));
    }
    Ok(())
}

fn runtime_for(config_path: &Path, state_path: &Path) -> CliRuntime {
    let sound = Arc::new(SystemSoundPlayer::new());
    CliRuntime::new(CliPorts {
        gateway: Arc::new(GitHubApiAdapter::new()),
        store: Arc::new(JsonStateStore::new(state_path.to_path_buf())),
        credentials: system_credentials(false),
        delivery: build_delivery(sound),
    })
    .with_config_path(config_path.to_path_buf())
}

async fn observe_for(
    app: &mut TuiApp,
    inbox: &Inbox,
    session: &mut ObservationSession,
    mut delay: Duration,
    millis: u64,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(millis);
    loop {
        let now = std::time::Instant::now();
        if now + delay > deadline {
            break;
        }
        tokio::time::sleep(delay).await;
        let update = session.tick().await;
        let view_config = display_config(&update.config, app.show_ignored());
        if app.reload(inbox, &view_config.core).await.is_err() {
            break;
        }
        app.set_status(update.failure);
        delay = update.next_delay;
    }
}

fn display_config(config: &AppConfig, show_ignored: bool) -> AppConfig {
    let mut adjusted = config.clone();
    if show_ignored {
        adjusted.core.watch.ignored_pull_requests.clear();
    }
    adjusted
}

async fn apply_action(
    app: &mut TuiApp,
    action: TuiAction,
    inbox: &Inbox,
    config: &mut AppConfig,
    editor: &ScopeEditor,
    opener: &dyn BrowserOpener,
) -> Result<bool, String> {
    match action {
        TuiAction::Continue => Ok(false),
        TuiAction::Quit | TuiAction::QuitDiscard => Ok(false),
        TuiAction::QuitSave => Ok(true),
        TuiAction::OpenUrl(url) => {
            match opener.open(&url) {
                Ok(()) => app.close_popup(),
                Err(error) => app.set_status(Some(format!("could not open browser: {error}"))),
            }
            Ok(false)
        }
        TuiAction::ToggleIgnoredView | TuiAction::ResetView => {
            let view_config = display_config(config, app.show_ignored());
            app.reload(inbox, &view_config.core)
                .await
                .map_err(|error| format!("passuff: state error: {error}"))?;
            Ok(false)
        }
        TuiAction::ToggleIgnore(key) => {
            let ignored = config.core.watch.ignores(&key.repository, key.number);
            let result = if ignored {
                editor.unignore_pull_request(&key.repository, key.number)
            } else {
                editor.ignore_pull_request(&key.repository, key.number)
            };
            if let Err(error) = result {
                app.set_status(Some(error.message));
                return Ok(false);
            }
            *config = ConfigLoader::load(editor.config_path())
                .map_err(|error| format!("passuff-tui: configuration error: {error}"))?;
            let view_config = display_config(config, app.show_ignored());
            app.reload(inbox, &view_config.core)
                .await
                .map_err(|error| format!("passuff: state error: {error}"))?;
            Ok(false)
        }
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

fn spawn_input() -> tokio::sync::mpsc::UnboundedReceiver<Event> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || loop {
        match read() {
            Ok(event) => {
                if sender.send(event).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    });
    receiver
}

enum ObservationMessage {
    Fetching,
    Update(Box<ObservationUpdate>),
}

fn spawn_observation(
    mut session: ObservationSession,
) -> tokio::sync::mpsc::UnboundedReceiver<ObservationMessage> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut delay = Duration::ZERO;
        loop {
            tokio::time::sleep(delay).await;
            if sender.send(ObservationMessage::Fetching).is_err() {
                break;
            }
            let update = session.tick().await;
            delay = update.next_delay;
            if sender
                .send(ObservationMessage::Update(Box::new(update)))
                .is_err()
            {
                break;
            }
        }
    });
    receiver
}

async fn run_terminal(
    app: &mut TuiApp,
    inbox: &Inbox,
    mut config: AppConfig,
    editor: &ScopeEditor,
    store: &JsonStateStore,
    start_picker: bool,
    runtime: CliRuntime,
    config_path: PathBuf,
    preferences_path: &std::path::Path,
    saved_preferences: &TuiPreferences,
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
    let session = ObservationSession::open(runtime, config.clone(), config_path)
        .await
        .map_err(|message| {
            restore(&mut terminal);
            message
        })?;
    let mut input = spawn_input();
    let mut observation = spawn_observation(session);
    let mut known = known_repositories(editor, &config, store).await;
    let mut picker = start_picker.then(|| ScopePicker::new(&config, known.clone()));
    let mut next_update_at: Option<std::time::Instant> = None;
    let outcome = loop {
        let drawn = match picker.as_mut() {
            Some(active) => terminal.draw(|frame| active.render(frame)),
            None => terminal.draw(|frame| app.render(frame)),
        };
        if let Err(error) = drawn {
            break Err(format!("passuff-tui: draw: {error}"));
        }
        tokio::select! {
            maybe_event = input.recv() => {
                let Some(Event::Key(key)) = maybe_event else { continue };
                if let Some(active) = picker.as_mut() {
                    match active.handle_key(key, editor) {
                        Ok(passuff_cli::scope::PickerOutcome::Close) => {
                            picker = None;
                        }
                        Ok(passuff_cli::scope::PickerOutcome::Continue) => {}
                        Err(error) => {
                            eprintln!("passuff-tui: {}", error.message);
                        }
                    }
                    match ConfigLoader::load(editor.config_path()) {
                        Ok(reloaded) => {
                            config = reloaded;
                            known = known_repositories(editor, &config, store).await;
                            if let Some(active) = picker.as_mut() {
                                active.known = known.clone();
                                active.reload(&config);
                            }
                            let view_config = display_config(&config, app.show_ignored());
                            if let Err(error) = app.reload(inbox, &view_config.core).await {
                                break Err(format!("passuff: state error: {error}"));
                            }
                        }
                        Err(error) => break Err(format!("passuff-tui: configuration error: {error}")),
                    }
                    continue;
                }
                if matches!(key.code, KeyCode::Char('p')) {
                    picker = Some(ScopePicker::new(&config, known.clone()));
                    continue;
                }
                match app.handle_key(key, inbox, &config.core).await {
                    Ok(TuiAction::Quit) => {
                        if app.preferences() != *saved_preferences {
                            app.show_save_prompt();
                            continue;
                        }
                        break Ok(());
                    }
                    Ok(TuiAction::QuitSave) => {
                        save_preferences(preferences_path, &app.preferences())?;
                        break Ok(());
                    }
                    Ok(TuiAction::QuitDiscard) => {
                        break Ok(());
                    }
                    Ok(action) => {
                        match apply_action(
                            app,
                            action,
                            inbox,
                            &mut config,
                            editor,
                            &SystemBrowserOpener::default(),
                        )
                        .await
                        {
                            Ok(true) => break Ok(()),
                            Ok(false) => {}
                            Err(message) => break Err(message),
                        }
                    }
                    Err(error) => break Err(format!("passuff: state error: {error}")),
                }
            }
            maybe_update = observation.recv() => {
                let Some(message) = maybe_update else { continue };
                let update = match message {
                    ObservationMessage::Fetching => {
                        app.set_fetching(true);
                        continue;
                    }
                    ObservationMessage::Update(update) => {
                        app.set_fetching(false);
                        *update
                    }
                };
                next_update_at = Some(std::time::Instant::now() + update.next_delay);
                app.set_countdown(Some(update.next_delay));
                if let Some(allowance) = update.allowance.as_ref() {
                    app.set_budget(allowance.remaining, allowance.limit);
                }
                config = update.config;
                if update.changed {
                    known = known_repositories(editor, &config, store).await;
                    if let Some(active) = picker.as_mut() {
                        active.known = known.clone();
                        active.reload(&config);
                    }
                    let view_config = display_config(&config, app.show_ignored());
                    if let Err(error) = app.reload(inbox, &view_config.core).await {
                        break Err(format!("passuff: state error: {error}"));
                    }
                }
                app.set_status(update.failure);
            }
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                if let Some(at) = next_update_at {
                    app.set_countdown(Some(
                        at.saturating_duration_since(std::time::Instant::now()),
                    ));
                }
                app.advance_spinner();
            }
        }
    };
    restore(&mut terminal);
    outcome
}

fn restore(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
}
