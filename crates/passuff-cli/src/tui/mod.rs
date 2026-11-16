pub mod app;
pub mod keys;
pub mod markdown;
pub mod render;

use std::io::IsTerminal;
use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use passuff_config::{AppConfig, FocusRelation, ScopeEditor, WatchMode};
use passuff_core::InboxService;
use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;

use crate::args::TuiCli;
use crate::delivery::RecordingDelivery;
use crate::paths::PassuffPaths;
use crate::runtime::{CliRuntime, CommandOutcome, Ports};
use crate::setup::run_setup;
use crate::sound::RecordingSoundPlayer;
use app::{Overlay, Pane, TuiApp, UiPrefs};
use keys::{parse_keys, Key};

#[derive(Debug, Clone)]
pub struct TuiOptions {
    pub paths: PassuffPaths,
    pub snapshot: bool,
    pub observe: bool,
    pub observe_for: Option<u64>,
    pub scope_select: Option<String>,
    pub scope_deselect: Option<String>,
    pub scope_mode: Option<WatchMode>,
    pub focus_set: Option<Vec<FocusRelation>>,
    pub picker: bool,
    pub keys: Vec<Key>,
    pub no_desktop: bool,
    pub no_sound: bool,
}

impl TuiOptions {
    pub fn from_cli(cli: &TuiCli) -> Result<Self, String> {
        let paths = PassuffPaths::resolve(cli.config.clone(), cli.state.clone());
        let scope_mode = match cli.scope_mode.as_deref() {
            Some("all") => Some(WatchMode::All),
            Some("selected") => Some(WatchMode::Selected),
            Some(other) => return Err(format!("unknown scope mode: {other}")),
            None => None,
        };
        let focus_set = match cli.focus_set.as_deref() {
            Some(value) => {
                let mut relations = Vec::new();
                for part in value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                {
                    let relation = FocusRelation::parse(part)
                        .ok_or_else(|| format!("unknown focus relation: {part}"))?;
                    if !relations.contains(&relation) {
                        relations.push(relation);
                    }
                }
                Some(relations)
            }
            None => None,
        };
        let keys = match cli.keys.as_deref() {
            Some(value) => parse_keys(value)?,
            None => Vec::new(),
        };
        Ok(Self {
            paths,
            snapshot: cli.snapshot,
            observe: cli.observe,
            observe_for: cli.observe_for,
            scope_select: cli.scope_select.clone(),
            scope_deselect: cli.scope_deselect.clone(),
            scope_mode,
            focus_set,
            picker: cli.picker,
            keys,
            no_desktop: cli.no_desktop,
            no_sound: cli.no_sound,
        })
    }
}

pub fn run_tui(options: TuiOptions, ports: Ports) -> CommandOutcome {
    let interactive_terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let mut config = match AppConfig::load(&options.paths.config) {
        Ok(config) => config,
        Err(error) => {
            if error_to_missing(&error) {
                if options.snapshot || !interactive_terminal {
                    return CommandOutcome {
                        exit_code: 2,
                        stderr: "passuff-tui: run passuff setup to create the configuration\n"
                            .to_string(),
                        ..CommandOutcome::default()
                    };
                }
                let mut input = std::io::stdin().lock();
                let mut output = std::io::stdout();
                let result = run_setup(&options.paths.config, &mut input, &mut output);
                if result.exit_code != 0 {
                    return CommandOutcome {
                        exit_code: result.exit_code,
                        stdout: result.stdout,
                        ..CommandOutcome::default()
                    };
                }
                match AppConfig::load(&options.paths.config) {
                    Ok(config) => config,
                    Err(error) => {
                        return CommandOutcome {
                            exit_code: 2,
                            stderr: format!("passuff-tui: configuration error: {error}\n"),
                            ..CommandOutcome::default()
                        }
                    }
                }
            } else {
                return CommandOutcome {
                    exit_code: 2,
                    stderr: format!("passuff-tui: configuration error: {error}\n"),
                    ..CommandOutcome::default()
                };
            }
        }
    };

    if let Err(message) = apply_overrides(&options, &mut config) {
        return CommandOutcome {
            exit_code: 2,
            stderr: format!("passuff-tui: {message}\n"),
            ..CommandOutcome::default()
        };
    }

    if options.picker && (options.snapshot || !interactive_terminal) {
        let view = match load_view(&config, &options) {
            Ok(view) => view,
            Err(outcome) => return outcome,
        };
        let prefs = UiPrefs::load(&options.paths.ui);
        let app = TuiApp::new(
            config,
            options.paths.config.clone(),
            options.paths.ui.clone(),
            Arc::new(passuff_core::JsonStateStore::new(
                options.paths.state.clone(),
            )),
            view,
            prefs,
            ports.browser.clone(),
        );
        return CommandOutcome {
            stdout: render::picker_text(&app),
            ..CommandOutcome::default()
        };
    }

    if !options.snapshot && !interactive_terminal {
        return CommandOutcome {
            exit_code: 3,
            stderr: "passuff-tui: requires a terminal (use --snapshot for scripts)\n".to_string(),
            ..CommandOutcome::default()
        };
    }

    let prefs = UiPrefs::load(&options.paths.ui);
    let effective = effective_config(&config, &prefs, &options);
    if options.observe || options.observe_for.is_some() {
        let runtime = CliRuntime::new(options.paths.clone(), ports.clone_ports());
        let _ = runtime.run_cycle_for(
            &effective,
            false,
            options.no_desktop || !prefs.desktop(),
            options.no_sound || !prefs.sound(),
        );
        if let Some(milliseconds) = options.observe_for {
            let deadline =
                std::time::Instant::now() + std::time::Duration::from_millis(milliseconds);
            while std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(250));
                if std::time::Instant::now() + std::time::Duration::from_millis(1000) <= deadline {
                    let prefs = UiPrefs::load(&options.paths.ui);
                    let _ = runtime.run_probe_and_targeted_with_flags(
                        false,
                        options.no_desktop || !prefs.desktop(),
                        options.no_sound || !prefs.sound(),
                    );
                }
            }
        }
        match AppConfig::load(&options.paths.config) {
            Ok(reloaded) => config = reloaded,
            Err(error) => {
                return CommandOutcome {
                    exit_code: 2,
                    stderr: format!("passuff-tui: configuration error: {error}\n"),
                    ..CommandOutcome::default()
                }
            }
        }
    }

    let view = match load_view(&config, &options) {
        Ok(view) => view,
        Err(outcome) => return outcome,
    };
    let store = Arc::new(passuff_core::JsonStateStore::new(
        options.paths.state.clone(),
    ));
    let mut app = TuiApp::new(
        config,
        options.paths.config.clone(),
        options.paths.ui.clone(),
        store,
        view,
        prefs,
        ports.browser.clone(),
    );
    for key in options.keys.iter().copied() {
        app.handle_key(key);
        if app.should_quit {
            break;
        }
    }

    if options.snapshot {
        return CommandOutcome {
            stdout: snapshot(&app),
            ..CommandOutcome::default()
        };
    }

    if options.picker {
        let selected = app.known_repositories().len();
        app.overlay = Some(Overlay::ScopePicker { selected });
    }
    match run_interactive(app, options, ports) {
        Ok(()) => CommandOutcome::default(),
        Err(error) => CommandOutcome {
            exit_code: 3,
            stderr: format!("passuff-tui: {error}\n"),
            ..CommandOutcome::default()
        },
    }
}

#[allow(clippy::result_large_err)]
fn load_view(
    config: &AppConfig,
    options: &TuiOptions,
) -> Result<passuff_core::InboxView, CommandOutcome> {
    let store = Arc::new(passuff_core::JsonStateStore::new(
        options.paths.state.clone(),
    ));
    let service = InboxService::new(config.clone(), store);
    service.view().map_err(|error| CommandOutcome {
        exit_code: 3,
        stderr: format!("passuff-tui: state error: {error}\n"),
        ..CommandOutcome::default()
    })
}

fn effective_config(config: &AppConfig, prefs: &UiPrefs, options: &TuiOptions) -> AppConfig {
    let mut config = config.clone();
    config.notifications.desktop =
        config.notifications.desktop && prefs.desktop() && !options.no_desktop;
    config.notifications.sound = config.notifications.sound && prefs.sound() && !options.no_sound;
    config
}

fn apply_overrides(options: &TuiOptions, config: &mut AppConfig) -> Result<(), String> {
    if let Some(repository) = &options.scope_select {
        ScopeEditor::select_repository(&options.paths.config, repository)
            .map_err(|error| error.to_string())?;
        *config = AppConfig::load(&options.paths.config).map_err(|error| error.to_string())?;
    }
    if let Some(repository) = &options.scope_deselect {
        if config.watch.effective_mode() == WatchMode::All {
            return Err("scope-deselect requires selected mode".to_string());
        }
        ScopeEditor::deselect_repository(&options.paths.config, repository)
            .map_err(|error| error.to_string())?;
        *config = AppConfig::load(&options.paths.config).map_err(|error| error.to_string())?;
    }
    if let Some(mode) = options.scope_mode {
        ScopeEditor::set_mode(&options.paths.config, mode).map_err(|error| error.to_string())?;
        *config = AppConfig::load(&options.paths.config).map_err(|error| error.to_string())?;
    }
    if let Some(relations) = &options.focus_set {
        ScopeEditor::set_focus(&options.paths.config, relations)
            .map_err(|error| error.to_string())?;
        *config = AppConfig::load(&options.paths.config).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn error_to_missing(error: &passuff_config::ConfigError) -> bool {
    matches!(error, passuff_config::ConfigError::Missing { .. })
}

pub fn snapshot(app: &TuiApp) -> String {
    snapshot_with_size(app, 120, 40)
}

pub fn snapshot_with_size(app: &TuiApp, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => return format!("passuff-tui: {error}\n"),
    };
    let _ = terminal.draw(|frame| draw(app, frame));
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            line.push_str(buffer.cell((x, y)).map(|cell| cell.symbol()).unwrap_or(" "));
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }
    output
}

fn draw(app: &TuiApp, frame: &mut ratatui::Frame<'_>) {
    if frame.area().width < 40 || frame.area().height < 12 {
        frame.render_widget(
            Paragraph::new("Terminal too small. Resize to at least 40x12."),
            frame.area(),
        );
        return;
    }
    if let Some((title, lines, scroll)) = render::overlay_lines(app) {
        let width = frame.area().width.saturating_sub(4).clamp(20, 100);
        let height = frame.area().height.saturating_sub(2).clamp(6, 30);
        let area = centered(frame.area(), width, height);
        frame.render_widget(Clear, area);
        let viewport = area.height.saturating_sub(2) as usize;
        app.detail_viewport.set(viewport.max(1));
        let max_scroll = lines.len().saturating_sub(viewport);
        let scroll = scroll.min(max_scroll);
        let block_title = match &app.status {
            Some(status) => format!("{title} - {}", status.replace('\n', " ")),
            None => title,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                block_title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        let text = Text::from(
            lines
                .into_iter()
                .skip(scroll)
                .take(viewport)
                .map(|line| {
                    let style = match line.style {
                        markdown::MarkdownStyle::Heading => Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                        markdown::MarkdownStyle::Dim | markdown::MarkdownStyle::Code => {
                            Style::default().fg(Color::DarkGray)
                        }
                        markdown::MarkdownStyle::Rule => Style::default().fg(Color::DarkGray),
                        markdown::MarkdownStyle::Normal => Style::default(),
                    };
                    Line::from(Span::styled(line.text, style))
                })
                .collect::<Vec<_>>(),
        );
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Min(4),
            Constraint::Length(3),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new(plain_lines(render::header_lines(app))),
        chunks[0],
    );

    let watched_focused = app.focus == Pane::Watched;
    let watched_block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_style(watched_focused))
        .title(Span::styled(
            "Watched pull requests",
            Style::default().fg(Color::Cyan),
        ));
    let watched_inner = watched_block.inner(chunks[1]);
    frame.render_widget(watched_block, chunks[1]);
    frame.render_widget(
        Paragraph::new(pane_lines(render::watched_lines(
            app,
            watched_inner.height as usize,
        ))),
        watched_inner,
    );

    let activity_focused = app.focus == Pane::Activity;
    let activity_block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_style(activity_focused))
        .title(Span::styled(
            render::activity_title(app),
            Style::default().fg(Color::Cyan),
        ));
    let activity_inner = activity_block.inner(chunks[2]);
    frame.render_widget(activity_block, chunks[2]);
    frame.render_widget(
        Paragraph::new(pane_lines(render::activity_lines(
            app,
            activity_inner.height as usize,
        ))),
        activity_inner,
    );

    frame.render_widget(
        Paragraph::new(footer_lines(render::footer_lines(app))),
        chunks[3],
    );
}

fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn plain_lines(lines: Vec<String>) -> Text<'static> {
    Text::from(
        lines
            .into_iter()
            .map(|line| Line::from(Span::raw(line)))
            .collect::<Vec<_>>(),
    )
}

fn pane_lines(lines: Vec<String>) -> Text<'static> {
    Text::from(
        lines
            .into_iter()
            .map(|line| {
                let style = if line.starts_with("▶") || line.contains("▶ ") {
                    Style::default().fg(Color::Yellow)
                } else if line.contains("no watched pull requests")
                    || line.contains("no activity yet")
                {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(line, style))
            })
            .collect::<Vec<_>>(),
    )
}

fn footer_lines(lines: Vec<String>) -> Text<'static> {
    Text::from(
        lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let style = if index == 0 {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if line.contains("no activity") {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(line, style))
            })
            .collect::<Vec<_>>(),
    )
}
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

fn run_interactive(mut app: TuiApp, options: TuiOptions, ports: Ports) -> Result<(), String> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let stop = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel::<ObservationUpdate>();
    let runtime = Arc::new(CliRuntime::new(options.paths.clone(), ports));
    let thread_stop = stop.clone();
    let thread_options = options.clone();
    let startup_done = options.observe || options.observe_for.is_some();
    let observer = std::thread::spawn(move || {
        let mut startup = startup_done;
        let mut last_reconciliation = Instant::now();
        let mut delay = Duration::from_secs(1);
        let mut defer = false;
        while !thread_stop.load(Ordering::SeqCst) {
            let deadline = Instant::now() + delay;
            while Instant::now() < deadline {
                if thread_stop.load(Ordering::SeqCst) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            if thread_stop.load(Ordering::SeqCst) {
                break;
            }
            let config = match runtime.load_config() {
                Ok(config) => config,
                Err(error) => {
                    let _ = sender.send(ObservationUpdate::Error(format!(
                        "configuration error: {error}"
                    )));
                    delay = Duration::from_secs(1);
                    continue;
                }
            };
            let prefs = UiPrefs::load(&thread_options.paths.ui);
            let no_desktop = thread_options.no_desktop || !prefs.desktop();
            let no_sound = thread_options.no_sound || !prefs.sound();
            let now = Instant::now();
            let reconciliation_due = now.duration_since(last_reconciliation).as_secs()
                >= config.reconciliation_interval_secs;
            let outcome = if defer {
                runtime.run_probe_only()
            } else if !startup || reconciliation_due {
                startup = true;
                last_reconciliation = now;
                runtime.run_cycle_for(&config, false, no_desktop, no_sound)
            } else {
                runtime.run_probe_and_targeted_with_flags(false, no_desktop, no_sound)
            };
            defer = outcome
                .allowance
                .as_ref()
                .map(passuff_core::defer_pass)
                .unwrap_or(false);
            delay = outcome
                .allowance
                .as_ref()
                .map(|allowance| passuff_core::next_delay(allowance, chrono::Utc::now()))
                .unwrap_or_else(|| Duration::from_secs(1))
                .max(Duration::from_secs(1));
            let _ = sender.send(ObservationUpdate::Outcome {
                stderr: outcome.stderr,
            });
        }
    });

    enable_raw_mode().map_err(|error| error.to_string())?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).map_err(|error| error.to_string())?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    app.interactive_mode = true;
    let result = (|| -> Result<(), String> {
        loop {
            terminal
                .draw(|frame| draw(&app, frame))
                .map_err(|error| error.to_string())?;
            if app.should_quit {
                break;
            }
            while let Ok(update) = receiver.try_recv() {
                match update {
                    ObservationUpdate::Error(message) => app.status = Some(message),
                    ObservationUpdate::Outcome { stderr } => {
                        if !stderr.trim().is_empty() {
                            app.status = Some(stderr.trim().to_string());
                        }
                        app.reload_config();
                        let _ = app.reload_view();
                    }
                }
            }
            if crossterm::event::poll(Duration::from_millis(150))
                .map_err(|error| error.to_string())?
            {
                let event = crossterm::event::read().map_err(|error| error.to_string())?;
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat {
                        if let Some(action) = map_key(key.code) {
                            app.handle_key(action);
                        }
                    }
                }
            }
        }
        app.save_preferences();
        Ok(())
    })();
    stop.store(true, Ordering::SeqCst);
    drop(receiver);
    let _ = observer.join();
    disable_raw_mode().map_err(|error| error.to_string())?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|error| error.to_string())?;
    result
}

enum ObservationUpdate {
    Error(String),
    Outcome { stderr: String },
}

fn map_key(code: KeyCode) -> Option<Key> {
    match code {
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::BackTab => Some(Key::BackTab),
        KeyCode::Down => Some(Key::Down),
        KeyCode::Up => Some(Key::Up),
        KeyCode::PageDown => Some(Key::PageDown),
        KeyCode::PageUp => Some(Key::PageUp),
        KeyCode::Home => Some(Key::Home),
        KeyCode::End => Some(Key::End),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Esc => Some(Key::Esc),
        KeyCode::Left => Some(Key::Left),
        KeyCode::Right => Some(Key::Right),
        KeyCode::F(1) => Some(Key::Help),
        KeyCode::Char(character) => Some(match character {
            'j' => Key::Down,
            'k' => Key::Up,
            'q' => Key::Quit,
            'r' => Key::MarkRead,
            'a' => Key::MarkAllRead,
            'u' => Key::ToggleUnread,
            'c' => Key::ClearFilter,
            'C' => Key::ResetView,
            'S' => Key::SortReverse,
            's' => Key::SortNext,
            'x' => Key::ToggleIgnore,
            'f' => Key::FilterPullRequest,
            'F' => Key::FilterStack,
            'h' | 'H' => Key::Help,
            'p' => Key::OpenPicker,
            ',' => Key::OpenSettings,
            ' ' => Key::Space,
            other => Key::Char(other),
        }),
        _ => None,
    }
}

trait ClonePorts {
    fn clone_ports(&self) -> Ports;
}

impl ClonePorts for Ports {
    fn clone_ports(&self) -> Ports {
        Ports {
            gateway_factory: self.gateway_factory.clone(),
            credentials: self.credentials.clone(),
            delivery: self.delivery.clone(),
            sound: self.sound.clone(),
            clock: self.clock.clone(),
            browser: self.browser.clone(),
            interactive: self.interactive,
        }
    }
}

#[allow(dead_code)]
fn _default_recording() -> (RecordingDelivery, RecordingSoundPlayer) {
    (
        RecordingDelivery::available(),
        RecordingSoundPlayer::available(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::RecordingBrowserOpener;
    use passuff_config::WatchMode;
    use passuff_core::{
        AccountView, ActivityView, CiResult, EventKind, InboxView, JsonStateStore, PullRequestKey,
        PullRequestState, PullRequestView,
    };

    fn sample_view() -> InboxView {
        let key = PullRequestKey::new("acme/api", 42);
        let now = chrono::Utc::now();
        InboxView {
            accounts: vec![AccountView {
                label: "personal".to_string(),
                login: Some("octo-personal".to_string()),
                last_check: None,
            }],
            scope_label: "all authored pull requests".to_string(),
            scope_mode: WatchMode::All,
            scope_repositories: Vec::new(),
            watched: vec![PullRequestView {
                key: key.clone(),
                state: PullRequestState::Open,
                ci: CiResult::Passing,
                title: "Add the thing".to_string(),
                body: "A description".to_string(),
                author: "octo-personal".to_string(),
                assignees: vec!["alice".to_string()],
                stack: None,
                relations: Vec::new(),
                created_at: Some(now),
                updated_at: Some(now),
                ignored: false,
                url: key.url(),
            }],
            activity: vec![ActivityView {
                id: "a1".to_string(),
                observed_at: now,
                occurred_at: Some(now),
                pull_request: key,
                summary: "New comment from @alice".to_string(),
                display: "comment by @alice".to_string(),
                body: Some("Body".to_string()),
                read: false,
                stack: None,
                kind: Some(EventKind::Comment),
                author: Some("alice".to_string()),
                ignored: false,
            }],
            known_repositories: vec!["acme/api".to_string()],
            unread: 1,
            total: 1,
        }
    }

    fn sample_app() -> TuiApp {
        TuiApp::new(
            passuff_config::AppConfig::default(),
            std::path::PathBuf::from("/tmp/passuff-config.toml"),
            std::path::PathBuf::from("/tmp/passuff-ui.json"),
            Arc::new(JsonStateStore::new("/tmp/passuff-state.json")),
            sample_view(),
            UiPrefs::default(),
            Arc::new(RecordingBrowserOpener::default()),
        )
    }

    #[test]
    fn renders_both_panes_and_footer_at_80x24() {
        let app = sample_app();
        let output = snapshot_with_size(&app, 80, 24);
        assert_eq!(output.lines().count(), 24);
        for expected in [
            "Accounts: personal @octo-personal",
            "watch scope: all authored pull requests",
            "1 unread of 1",
            "Watched pull requests",
            "Activity · all",
            "acme/api#42",
            "comment by @alice",
            "Focus: activity",
        ] {
            assert!(
                output.contains(expected),
                "missing {expected:?} in 80x24 frame:\n{output}"
            );
        }
        let focus_line = output
            .lines()
            .position(|line| line.contains("Focus: activity"))
            .expect("focus line");
        assert!(focus_line < 23);
    }

    #[test]
    fn renders_an_empty_activity_hint_at_80x24() {
        let mut app = sample_app();
        app.view.activity.clear();
        app.view.unread = 0;
        app.view.total = 0;
        let output = snapshot_with_size(&app, 80, 24);
        assert!(output.contains("no activity yet"));
        assert!(output.contains("Tab to watched PRs"));
        assert!(output.contains("Focus: activity"));
    }
}
