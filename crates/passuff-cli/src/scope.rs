use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};
use passuff_config::{AppConfig, ConfigLoader, WatchMode};
use passuff_core::{ScopeSnapshot, WatcherState};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeError {
    pub message: String,
    pub exit: i32,
}

impl ScopeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit: 2,
        }
    }
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

pub struct ScopeEditor {
    config_path: PathBuf,
}

impl ScopeEditor {
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn config(&self) -> Result<AppConfig, ScopeError> {
        ConfigLoader::load(&self.config_path)
            .map_err(|error| ScopeError::new(format!("cannot read configuration: {error}")))
    }

    pub fn snapshot(&self) -> Result<ScopeSnapshot, ScopeError> {
        Ok(ScopeSnapshot::from_watch(&self.config()?.core.watch))
    }

    pub fn select(&self, repository: &str) -> Result<ScopeSnapshot, ScopeError> {
        let repository = validate_repository(repository)?;
        let config = self.config()?;
        let watch = &config.core.watch;
        let mut repositories = if watch.mode == WatchMode::All {
            Vec::new()
        } else {
            watch.repositories.clone()
        };
        add_repository(&mut repositories, &repository);
        self.write(WatchMode::Selected, &repositories)
    }

    pub fn deselect(&self, repository: &str) -> Result<ScopeSnapshot, ScopeError> {
        let repository = validate_repository(repository)?;
        let config = self.config()?;
        let watch = &config.core.watch;
        if watch.mode == WatchMode::All {
            return Err(ScopeError::new(
                "switch to selected mode before deselecting a repository",
            ));
        }
        let mut repositories = watch.repositories.clone();
        repositories.retain(|existing| !existing.eq_ignore_ascii_case(&repository));
        self.write(WatchMode::Selected, &repositories)
    }

    pub fn set_mode(&self, mode: WatchMode) -> Result<ScopeSnapshot, ScopeError> {
        let config = self.config()?;
        self.write(mode, &config.core.watch.repositories)
    }

    pub fn add(&self, repository: &str) -> Result<ScopeSnapshot, ScopeError> {
        self.select(repository)
    }

    pub fn known_repositories(&self, config: &AppConfig, state: &WatcherState) -> Vec<String> {
        let mut known: Vec<String> = config.core.watch.repositories.clone();
        for account in state.accounts.values() {
            for pull_request in account.pull_requests.values() {
                known.push(pull_request.repository.clone());
            }
        }
        known.sort_by_key(|repository| repository.to_ascii_lowercase());
        known.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        known
    }

    /// Add a pull request to the ignore list.
    pub fn ignore_pull_request(&self, repository: &str, number: u64) -> Result<(), ScopeError> {
        self.update_ignored(repository, number, true)
    }

    /// Remove a pull request from the ignore list.
    pub fn unignore_pull_request(&self, repository: &str, number: u64) -> Result<(), ScopeError> {
        self.update_ignored(repository, number, false)
    }

    fn update_ignored(&self, repository: &str, number: u64, add: bool) -> Result<(), ScopeError> {
        let entry = format!("{repository}#{number}");
        let text = fs::read_to_string(&self.config_path)
            .map_err(|error| ScopeError::new(format!("cannot read configuration: {error}")))?;
        let mut document: toml_edit::DocumentMut = text
            .parse()
            .map_err(|error| ScopeError::new(format!("cannot read configuration: {error}")))?;
        if document.get("watch").is_none() {
            document["watch"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        let watch = document["watch"]
            .as_table_mut()
            .ok_or_else(|| ScopeError::new("cannot write configuration: [watch] is not a table"))?;
        let mut entries: Vec<String> = watch
            .get("ignored_pull_requests")
            .and_then(toml_edit::Item::as_array)
            .map(|array| {
                array
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if add {
            if !entries
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&entry))
            {
                entries.push(entry);
            }
        } else {
            entries.retain(|existing| !existing.eq_ignore_ascii_case(&entry));
        }
        let mut array = toml_edit::Array::new();
        for entry in entries {
            array.push(entry);
        }
        watch["ignored_pull_requests"] = toml_edit::value(array);
        self.write_document(document)
    }

    fn write_document(&self, document: toml_edit::DocumentMut) -> Result<(), ScopeError> {
        let temporary = self.config_path.with_extension("toml.tmp");
        fs::write(&temporary, document.to_string())
            .map_err(|error| ScopeError::new(format!("cannot write configuration: {error}")))?;
        if let Err(error) = fs::rename(&temporary, &self.config_path) {
            let _ = fs::remove_file(&temporary);
            return Err(ScopeError::new(format!(
                "cannot write configuration: {error}"
            )));
        }
        Ok(())
    }

    fn write(&self, mode: WatchMode, repositories: &[String]) -> Result<ScopeSnapshot, ScopeError> {
        let text = fs::read_to_string(&self.config_path)
            .map_err(|error| ScopeError::new(format!("cannot read configuration: {error}")))?;
        let mut document: toml_edit::DocumentMut = text
            .parse()
            .map_err(|error| ScopeError::new(format!("cannot read configuration: {error}")))?;
        if document.get("watch").is_none() {
            document["watch"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        let watch = document["watch"]
            .as_table_mut()
            .ok_or_else(|| ScopeError::new("cannot write configuration: [watch] is not a table"))?;
        watch["mode"] = toml_edit::value(match mode {
            WatchMode::All => "all",
            WatchMode::Selected => "selected",
        });
        let mut array = toml_edit::Array::new();
        for repository in repositories {
            array.push(repository.clone());
        }
        watch["repositories"] = toml_edit::value(array);
        self.write_document(document)?;
        Ok(ScopeSnapshot {
            mode,
            repositories: repositories.to_vec(),
        })
    }
}

fn validate_repository(repository: &str) -> Result<String, ScopeError> {
    let repository = repository.trim();
    let valid = !repository.is_empty()
        && !repository.chars().any(char::is_whitespace)
        && repository.matches('/').count() == 1
        && !repository.starts_with('/')
        && !repository.ends_with('/');
    if !valid {
        return Err(ScopeError::new("must be owner/repo"));
    }
    Ok(repository.to_string())
}

fn add_repository(repositories: &mut Vec<String>, repository: &str) {
    if !repositories
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(repository))
    {
        repositories.push(repository.to_string());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerOutcome {
    Continue,
    Close,
}

pub struct ScopePicker {
    pub mode: WatchMode,
    pub repositories: Vec<String>,
    pub selected: BTreeSet<String>,
    pub known: Vec<String>,
    pub cursor: usize,
    pub input: String,
    pub insert: bool,
    pub error: Option<String>,
}

impl ScopePicker {
    pub fn new(config: &AppConfig, known: Vec<String>) -> Self {
        Self {
            mode: config.core.watch.mode,
            repositories: config.core.watch.repositories.clone(),
            selected: config
                .core
                .watch
                .repositories
                .iter()
                .map(|repository| repository.to_ascii_lowercase())
                .collect(),
            known,
            cursor: 0,
            input: String::new(),
            insert: false,
            error: None,
        }
    }

    pub fn reload(&mut self, config: &AppConfig) {
        self.mode = config.core.watch.mode;
        self.repositories = config.core.watch.repositories.clone();
        self.selected = config
            .core
            .watch
            .repositories
            .iter()
            .map(|repository| repository.to_ascii_lowercase())
            .collect();
        if self.cursor >= self.known.len() {
            self.cursor = self.known.len().saturating_sub(1);
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let block = Block::bordered()
            .title(Line::from(vec![
                Span::styled(" passuff", theme::bright().add_modifier(Modifier::BOLD)),
                Span::styled(" — Watch scope ", theme::dim()),
            ]))
            .border_style(theme::border());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(4),
            Constraint::Length(3),
        ])
        .split(inner);

        let mode = match self.mode {
            WatchMode::All => "all",
            WatchMode::Selected => "selected",
        };
        let mut mode_line = vec![
            Span::styled("Mode: ", theme::dim()),
            Span::styled(mode.to_string(), theme::accent_bold()),
            Span::styled("   (m switches all/selected)", theme::dim()),
        ];
        if self.mode == WatchMode::All {
            mode_line.push(Span::styled(
                "   every known repository is observed",
                theme::dim(),
            ));
        } else if self.repositories.is_empty() {
            mode_line.push(Span::styled("   watching nothing", theme::dim()));
        }
        frame.render_widget(Paragraph::new(Line::from(mode_line)), chunks[0]);

        let mut lines: Vec<Line> = Vec::new();
        if self.known.is_empty() {
            lines.push(Line::from(Span::styled(
                "(no known repositories — type owner/name)",
                theme::dim(),
            )));
        }
        for (index, repository) in self.known.iter().enumerate() {
            let checked = self.mode == WatchMode::All
                || self.selected.contains(&repository.to_ascii_lowercase());
            let marker = if checked { "[x]" } else { "[ ]" };
            let cursor = if index == self.cursor { "▶ " } else { "  " };
            lines.push(Line::from(vec![
                Span::styled(cursor, theme::gold_bold()),
                Span::styled(
                    format!("{marker} "),
                    if checked {
                        theme::green()
                    } else {
                        theme::dim()
                    },
                ),
                Span::styled(
                    repository.clone(),
                    if index == self.cursor {
                        theme::bright()
                    } else {
                        theme::body()
                    },
                ),
            ]));
        }
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled(
                if self.insert {
                    "Add (insert): "
                } else {
                    "Add: "
                },
                theme::accent(),
            ),
            Span::styled(self.input.clone(), theme::bright()),
        ]));
        if let Some(error) = &self.error {
            lines.push(Line::from(vec![
                Span::styled("Error: ", theme::gold()),
                Span::styled(error.clone(), theme::gold()),
            ]));
        }
        frame.render_widget(Paragraph::new(lines), chunks[1]);
        let entries = [
            ("↑↓", "move"),
            ("type", "owner/name"),
            ("enter", "add"),
            ("space", "toggle"),
            ("m", "mode"),
            ("esc/q", "close"),
        ];
        let mut footer = Vec::new();
        for (index, (key, label)) in entries.iter().enumerate() {
            if index > 0 {
                footer.push(Span::styled(" · ", theme::dim()));
            }
            footer.push(Span::styled((*key).to_string(), theme::key()));
            footer.push(Span::styled(format!(" {label}"), theme::dim()));
        }
        frame.render_widget(Paragraph::new(Line::from(footer)), chunks[2]);
    }

    pub fn snapshot(&self) -> String {
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = ratatui::Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| self.render(frame))
            .expect("draw snapshot");
        let buffer = terminal.backend().buffer();
        (0..30)
            .map(|y| {
                (0..100)
                    .map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()).unwrap_or(" "))
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        editor: &ScopeEditor,
    ) -> Result<PickerOutcome, ScopeError> {
        self.error = None;
        if self.insert {
            match key.code {
                KeyCode::Esc => self.insert = false,
                KeyCode::Enter => {
                    let repository = self.input.trim().to_string();
                    if !repository.is_empty() {
                        match editor.add(&repository) {
                            Ok(snapshot) => {
                                self.apply(snapshot);
                                self.input.clear();
                                self.insert = false;
                                self.reload_known(editor);
                            }
                            Err(error) => self.error = Some(error.message),
                        }
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(character) => self.input.push(character),
                _ => {}
            }
            return Ok(PickerOutcome::Continue);
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Ok(PickerOutcome::Close),
            KeyCode::Char('i') => self.insert = true,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.known.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Char('m') => {
                let mode = match self.mode {
                    WatchMode::All => WatchMode::Selected,
                    WatchMode::Selected => WatchMode::All,
                };
                match editor.set_mode(mode) {
                    Ok(snapshot) => self.apply(snapshot),
                    Err(error) => self.error = Some(error.message),
                }
            }
            KeyCode::Char(' ') => {
                if let Some(repository) = self.known.get(self.cursor).cloned() {
                    let result = if self.mode == WatchMode::All {
                        editor.select(&repository)
                    } else if self.selected.contains(&repository.to_ascii_lowercase()) {
                        editor.deselect(&repository)
                    } else {
                        editor.select(&repository)
                    };
                    match result {
                        Ok(snapshot) => self.apply(snapshot),
                        Err(error) => self.error = Some(error.message),
                    }
                }
            }
            _ => {}
        }
        Ok(PickerOutcome::Continue)
    }

    fn apply(&mut self, snapshot: ScopeSnapshot) {
        self.mode = snapshot.mode;
        self.repositories = snapshot.repositories;
        self.selected = self
            .repositories
            .iter()
            .map(|repository| repository.to_ascii_lowercase())
            .collect();
    }

    fn reload_known(&mut self, editor: &ScopeEditor) {
        if let Ok(config) = editor.config() {
            let state = passuff_core::WatcherState::default();
            self.known = editor.known_repositories(&config, &state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(dir: &std::path::Path) -> ScopeEditor {
        let path = dir.join("config.toml");
        std::fs::write(&path, "[accounts.personal]\ntoken_env = \"A\"\n").expect("config");
        ScopeEditor::new(path)
    }

    #[test]
    fn select_deselect_mode_and_add_persist() {
        let dir = tempfile::tempdir().expect("dir");
        let editor = editor(dir.path());
        let snapshot = editor.select("acme/api").expect("select");
        assert_eq!(snapshot.mode, WatchMode::Selected);
        assert_eq!(snapshot.repositories, vec!["acme/api"]);
        editor.select("acme/web").expect("select second");
        editor.deselect("ACME/API").expect("deselect");
        assert_eq!(
            editor.snapshot().expect("snapshot").repositories,
            vec!["acme/web"]
        );
        editor.set_mode(WatchMode::All).expect("all");
        assert_eq!(editor.snapshot().expect("snapshot").mode, WatchMode::All);
        editor.add("acme/new").expect("add");
        let snapshot = editor.snapshot().expect("snapshot");
        assert_eq!(snapshot.mode, WatchMode::Selected);
        assert!(snapshot.repositories.iter().any(|r| r == "acme/new"));
    }

    #[test]
    fn invalid_repository_is_rejected() {
        let dir = tempfile::tempdir().expect("dir");
        let editor = editor(dir.path());
        let error = editor.select("acme").expect_err("invalid");
        assert_eq!(error.message, "must be owner/repo");
        assert_eq!(error.exit, 2);
    }

    #[test]
    fn write_failure_keeps_the_previous_scope() {
        let dir = tempfile::tempdir().expect("dir");
        let editor = editor(dir.path());
        editor.select("acme/api").expect("select");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500))
                .expect("readonly");
        }
        let error = editor.select("acme/web").expect_err("write failure");
        assert!(
            error.message.contains("cannot write configuration"),
            "{error:?}"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))
                .expect("restore");
        }
        assert_eq!(
            editor.snapshot().expect("snapshot").repositories,
            vec!["acme/api"]
        );
    }
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn picker_switches_mode_toggles_and_accepts_inserted_names() {
        let dir = tempfile::tempdir().expect("dir");
        let editor = editor(dir.path());
        let config = editor.config().expect("config");
        let mut picker = ScopePicker::new(&config, vec!["acme/api".to_string()]);
        picker
            .handle_key(key(KeyCode::Char('m')), &editor)
            .expect("mode");
        assert_eq!(picker.mode, WatchMode::Selected);
        picker
            .handle_key(key(KeyCode::Char(' ')), &editor)
            .expect("toggle");
        assert!(picker.selected.contains("acme/api"));
        picker
            .handle_key(key(KeyCode::Char('i')), &editor)
            .expect("insert");
        for character in "acme/j".chars() {
            picker
                .handle_key(key(KeyCode::Char(character)), &editor)
                .expect("type");
        }
        picker
            .handle_key(key(KeyCode::Enter), &editor)
            .expect("add");
        assert!(editor
            .snapshot()
            .expect("snapshot")
            .repositories
            .iter()
            .any(|repository| repository == "acme/j"));
        assert_eq!(
            picker.handle_key(key(KeyCode::Esc), &editor).expect("esc"),
            PickerOutcome::Close
        );
    }
}
