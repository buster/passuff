use crossterm::event::{KeyCode, KeyEvent};
use passuff_config::{AppConfig, ConfigError, CoreConfig};
use passuff_core::{CoreError, InboxService, InboxView};
use ratatui::backend::TestBackend;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::{Frame, Terminal};

use crate::transcript::{activity_summary, ci_label, state_label};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiLoadError {
    pub exit: i32,
    pub message: String,
}

pub async fn load_view(
    config: &Result<AppConfig, ConfigError>,
    inbox: &dyn InboxService,
) -> Result<InboxView, TuiLoadError> {
    let config = config.as_ref().map_err(|error| TuiLoadError {
        exit: 2,
        message: format!("passuff: configuration error: {error}"),
    })?;
    inbox
        .view(&config.core)
        .await
        .map_err(|error| TuiLoadError {
            exit: 3,
            message: format!("passuff: state error: {error}"),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiAction {
    Continue,
    Quit,
}

pub struct TuiApp {
    view: InboxView,
    selected: usize,
}

impl TuiApp {
    pub fn new(view: InboxView) -> Self {
        Self { view, selected: 0 }
    }

    pub fn view(&self) -> &InboxView {
        &self.view
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_summary(&self) -> Option<String> {
        self.view
            .activity
            .get(self.selected)
            .map(|record| activity_summary(&record.event))
    }

    pub async fn reload(
        &mut self,
        inbox: &dyn InboxService,
        config: &CoreConfig,
    ) -> Result<(), CoreError> {
        self.view = inbox.view(config).await?;
        if self.selected >= self.view.activity.len() {
            self.selected = self.view.activity.len().saturating_sub(1);
        }
        Ok(())
    }

    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        inbox: &dyn InboxService,
        config: &CoreConfig,
    ) -> Result<TuiAction, CoreError> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(TuiAction::Quit),
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < self.view.activity.len() {
                    self.selected += 1;
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('r') => {
                if let Some(record) = self.view.activity.get(self.selected) {
                    inbox.mark_read(&record.id).await?;
                    self.reload(inbox, config).await?;
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('a') => {
                inbox.mark_all_read().await?;
                self.reload(inbox, config).await?;
                Ok(TuiAction::Continue)
            }
            _ => Ok(TuiAction::Continue),
        }
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "passuff — inbox   {} unread of {}",
            self.view.unread,
            self.view.activity.len()
        )];
        lines.push(String::new());
        for account in &self.view.accounts {
            let login = account
                .login
                .as_deref()
                .map(|login| format!("@{login}"))
                .unwrap_or_else(|| "not resolved".to_string());
            lines.push(format!("accounts: {} {}", account.label, login));
        }
        let scope = if self.view.scope.is_empty() {
            "all authored pull requests".to_string()
        } else {
            self.view.scope.join(", ")
        };
        lines.push(format!("watch scope: {scope}"));
        lines.push(String::new());
        if self.view.pull_requests.is_empty() {
            lines.push("no watched pull requests".to_string());
        } else {
            lines.push("watched pull requests".to_string());
            for pull_request in &self.view.pull_requests {
                lines.push(format!(
                    "  {} {} {} {}",
                    pull_request.key,
                    state_label(&pull_request.state),
                    ci_label(&pull_request.ci_state),
                    pull_request.title
                ));
            }
        }
        lines.push(String::new());
        if self.view.activity.is_empty() {
            lines.push("no activity yet".to_string());
        } else {
            lines.push("activity".to_string());
            for (index, record) in self.view.activity.iter().enumerate() {
                let marker = if index == self.selected { ">" } else { " " };
                lines.push(format!(
                    "{} {} {} {} {} {}",
                    marker,
                    record.id,
                    record.observed_at,
                    record.key,
                    if record.read { "read" } else { "unread" },
                    activity_summary(&record.event)
                ));
            }
        }
        lines.push(String::new());
        lines.push("q quit  j/k move  r read  a read all".to_string());
        lines
    }

    pub fn render(&self, frame: &mut Frame) {
        let lines: Vec<Line> = self.lines().into_iter().map(Line::from).collect();
        frame.render_widget(Paragraph::new(lines), frame.area());
    }

    pub fn snapshot(&self) -> String {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal");
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use passuff_core::{AccountView, ActivityEvent, ActivityRecord, PullRequestKey};

    fn view(activity: Vec<ActivityRecord>) -> InboxView {
        InboxView {
            accounts: vec![AccountView {
                label: "personal".to_string(),
                login: Some("octo-personal".to_string()),
                last_check_at: None,
            }],
            scope: Vec::new(),
            pull_requests: Vec::new(),
            unread: activity.iter().filter(|record| !record.read).count(),
            activity,
        }
    }

    fn record(id: &str, read: bool) -> ActivityRecord {
        ActivityRecord {
            id: id.to_string(),
            key: PullRequestKey::new("acme/api".to_string(), 42),
            observed_at: "2026-11-02T10:00:00Z".to_string(),
            event: ActivityEvent::Comment {
                author: "alice".to_string(),
                body_excerpt: "hi".to_string(),
            },
            read,
        }
    }

    #[test]
    fn snapshot_shows_the_empty_inbox() {
        let app = TuiApp::new(view(Vec::new()));
        let frame = app.snapshot();
        assert!(frame.contains("no activity yet"), "{frame}");
        assert!(frame.contains("0 unread of 0"), "{frame}");
    }

    #[test]
    fn snapshot_marks_the_selected_activity() {
        let app = TuiApp::new(view(vec![record("a2", false), record("a1", true)]));
        let frame = app.snapshot();
        assert!(frame.contains("> a2"), "{frame}");
        assert!(!frame.contains("> a1"), "{frame}");
        assert!(frame.contains("New comment from @alice"), "{frame}");
    }
}
