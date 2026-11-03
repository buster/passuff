use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent};
use passuff_config::{AppConfig, ConfigError, CoreConfig};
use passuff_core::{
    ActivityRecord, CoreError, InboxService, InboxView, PullRequestKey, PullRequestView,
};
use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, TableState, Wrap};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiAction {
    Continue,
    Quit,
    OpenUrl(String),
}

#[derive(Debug, Clone)]
pub struct ActivityDetail {
    pub record: ActivityRecord,
    pub pull_request: Option<PullRequestView>,
    pub url: Option<String>,
}

#[derive(Debug)]
struct StackGroup<'a> {
    repository: String,
    number: u64,
    base: String,
    size: u64,
    members: Vec<&'a PullRequestView>,
    unread: usize,
}

fn stack_groups(view: &InboxView) -> (Vec<StackGroup<'_>>, Vec<&PullRequestView>) {
    let mut groups: BTreeMap<(String, u64), Vec<&PullRequestView>> = BTreeMap::new();
    let mut standalone = Vec::new();
    for pull_request in &view.pull_requests {
        match pull_request.stack.as_ref() {
            Some(stack) => groups
                .entry((pull_request.key.repository.clone(), stack.number))
                .or_default()
                .push(pull_request),
            None => standalone.push(pull_request),
        }
    }
    let mut result = Vec::new();
    for ((repository, number), mut members) in groups {
        members.sort_by_key(|pull_request| {
            pull_request
                .stack
                .as_ref()
                .map(|stack| stack.position)
                .unwrap_or(0)
        });
        let first = members
            .first()
            .and_then(|pull_request| pull_request.stack.as_ref())
            .expect("stack member");
        let unread = view
            .activity
            .iter()
            .filter(|record| !record.read && members.iter().any(|member| member.key == record.key))
            .count();
        result.push(StackGroup {
            repository,
            number,
            base: first.base.clone(),
            size: first.size,
            members,
            unread,
        });
    }
    standalone.sort_by(|left, right| left.key.cmp(&right.key));
    (result, standalone)
}

fn activity_stack(view: &InboxView, key: &PullRequestKey) -> Option<u64> {
    view.pull_requests
        .iter()
        .find(|pull_request| &pull_request.key == key)
        .and_then(|pull_request| pull_request.stack.as_ref().map(|stack| stack.number))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Watched,
    Activity,
}

pub struct TuiApp {
    view: InboxView,
    selected: usize,
    focus: Pane,
    pr_selected: usize,
    selected_pr: Option<PullRequestKey>,
    status: Option<String>,
    popup: Option<ActivityDetail>,
}

impl TuiApp {
    pub fn new(view: InboxView) -> Self {
        Self {
            view,
            selected: 0,
            focus: Pane::Activity,
            pr_selected: 0,
            selected_pr: None,
            status: None,
            popup: None,
        }
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    /// Show a failure line in the footer until the next successful update.
    pub fn set_status(&mut self, status: Option<String>) {
        self.status = status;
    }

    pub fn focus(&self) -> Pane {
        self.focus
    }

    pub fn selected_summary(&self) -> Option<String> {
        self.view
            .activity
            .get(self.selected)
            .map(|record| activity_summary(&record.event))
    }

    pub fn selected_pull_request(&self) -> Option<&PullRequestView> {
        self.ordered_pull_requests().get(self.pr_selected).copied()
    }

    fn ordered_pull_requests(&self) -> Vec<&PullRequestView> {
        let (groups, standalone) = stack_groups(&self.view);
        let mut ordered: Vec<&PullRequestView> = groups
            .iter()
            .flat_map(|group| group.members.iter().copied())
            .collect();
        ordered.extend(standalone);
        ordered
    }

    fn clamp_pr_selection(&mut self, preserve: Option<PullRequestKey>) {
        let (index, key) = {
            let ordered = self.ordered_pull_requests();
            if let Some(key) = preserve {
                if let Some(index) = ordered.iter().position(|pull_request| pull_request.key == key) {
                    (index, Some(key))
                } else {
                    let index = self.pr_selected.min(ordered.len().saturating_sub(1));
                    (index, ordered.get(index).map(|pull_request| pull_request.key.clone()))
                }
            } else {
                let index = self.pr_selected.min(ordered.len().saturating_sub(1));
                (index, ordered.get(index).map(|pull_request| pull_request.key.clone()))
            }
        };
        self.pr_selected = index;
        self.selected_pr = key;
    }

    pub async fn reload(
        &mut self,
        inbox: &dyn InboxService,
        config: &CoreConfig,
    ) -> Result<(), CoreError> {
        let selected_pr = self.selected_pr.clone();
        self.view = inbox.view(config).await?;
        if self.selected >= self.view.activity.len() {
            self.selected = self.view.activity.len().saturating_sub(1);
        }
        self.clamp_pr_selection(selected_pr);
        Ok(())
    }

    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        inbox: &dyn InboxService,
        config: &CoreConfig,
    ) -> Result<TuiAction, CoreError> {
        if self.popup.is_some() {
            return match key.code {
                KeyCode::Esc => {
                    self.popup = None;
                    Ok(TuiAction::Continue)
                }
                KeyCode::Enter => Ok(self
                    .popup
                    .as_ref()
                    .and_then(|detail| detail.url.clone())
                    .map(TuiAction::OpenUrl)
                    .unwrap_or(TuiAction::Continue)),
                _ => Ok(TuiAction::Continue),
            };
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(TuiAction::Quit),
            KeyCode::Tab | KeyCode::BackTab => {
                self.focus = match self.focus {
                    Pane::Watched => Pane::Activity,
                    Pane::Activity => Pane::Watched,
                };
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.focus {
                    Pane::Watched => {
                        let next = {
                            let ordered = self.ordered_pull_requests();
                            if self.pr_selected + 1 < ordered.len() {
                                Some((self.pr_selected + 1, ordered[self.pr_selected + 1].key.clone()))
                            } else {
                                None
                            }
                        };
                        if let Some((index, key)) = next {
                            self.pr_selected = index;
                            self.selected_pr = Some(key);
                        }
                    }
                    Pane::Activity => {
                        if self.selected + 1 < self.view.activity.len() {
                            self.selected += 1;
                        }
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.focus {
                    Pane::Watched => {
                        let index = self.pr_selected.saturating_sub(1);
                        let key = self
                            .ordered_pull_requests()
                            .get(index)
                            .map(|pull_request| pull_request.key.clone());
                        self.pr_selected = index;
                        self.selected_pr = key;
                    }
                    Pane::Activity => {
                        self.selected = self.selected.saturating_sub(1);
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Enter if self.focus == Pane::Activity => {
                if let Some(record) = self.view.activity.get(self.selected).cloned() {
                    let pull_request = self
                        .view
                        .pull_requests
                        .iter()
                        .find(|pull_request| pull_request.key == record.key)
                        .cloned();
                    let url = pull_request
                        .as_ref()
                        .map(|pull_request| pull_request.url.clone());
                    self.popup = Some(ActivityDetail {
                        record,
                        pull_request,
                        url,
                    });
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('r') if self.focus == Pane::Activity => {
                if let Some(record) = self.view.activity.get(self.selected) {
                    inbox.mark_read(&record.id).await?;
                    self.reload(inbox, config).await?;
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('a') if self.focus == Pane::Activity => {
                inbox.mark_all_read().await?;
                self.reload(inbox, config).await?;
                Ok(TuiAction::Continue)
            }
            _ => Ok(TuiAction::Continue),
        }
    }

    fn footer_keys(&self) -> &'static str {
        match self.focus {
            Pane::Watched => "Focus: PRs   Tab activity   j/k move   p scope   q quit",
            Pane::Activity => {
                "Focus: activity   Tab PRs   j/k move   enter detail   r read   a all   p scope   q quit"
            }
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let outer = Block::bordered()
            .title(
                Line::from(" passuff — Inbox ")
                    .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .border_style(Style::new().fg(Color::Gray));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        let controls_height = if self.status.is_some() { 4 } else { 3 };
        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Percentage(30),
            Constraint::Min(5),
            Constraint::Length(controls_height),
        ])
        .split(inner);

        let accounts = self
            .view
            .accounts
            .iter()
            .map(|account| {
                let login = account
                    .login
                    .as_deref()
                    .map(|login| format!("@{login}"))
                    .unwrap_or_else(|| "not resolved".to_string());
                format!("{} {}", account.label, login)
            })
            .collect::<Vec<_>>()
            .join("   |   ");
        frame.render_widget(
            Paragraph::new(format!(
                "{accounts}   |   watch scope: {}",
                self.view.scope_label
            )),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new(format!(
                "{} unread of {}   |   Focus: activity",
                self.view.unread,
                self.view.activity.len()
            ))
            .style(Style::new().fg(Color::Yellow)),
            chunks[1],
        );

        let watched_block = Block::bordered()
            .title(" Watched pull requests ")
            .border_style(Style::new().fg(Color::Gray));
        if self.view.pull_requests.is_empty() {
            frame.render_widget(
                Paragraph::new("no watched pull requests").block(watched_block),
                chunks[2],
            );
        } else {
            let (groups, standalone) = stack_groups(&self.view);
            let summaries: Vec<String> = groups
                .iter()
                .map(|group| {
                    format!(
                        "{} stack {} | base {} | size {} | {} unread in stack",
                        group.repository, group.number, group.base, group.size, group.unread
                    )
                })
                .collect();
            let inner = watched_block.inner(chunks[2]);
            frame.render_widget(watched_block, chunks[2]);
            let summary_height = (summaries.len() as u16).min(inner.height.saturating_sub(1));
            let watched_layout =
                Layout::vertical([Constraint::Length(summary_height), Constraint::Min(1)])
                    .split(inner);
            if summary_height > 0 {
                frame.render_widget(Paragraph::new(summaries.join("\n")), watched_layout[0]);
            }
            let mut rows: Vec<Row> = Vec::new();
            for group in &groups {
                for pull_request in &group.members {
                    let stack = pull_request.stack.as_ref().expect("stack member");
                    rows.push(Row::new(vec![
                        Cell::from(pull_request.key.to_string()),
                        Cell::from(state_label(&pull_request.state)),
                        Cell::from(ci_label(&pull_request.ci_state)),
                        Cell::from(format!("position {}", stack.position)),
                        Cell::from(format!("stack {}", stack.number)),
                        Cell::from(pull_request.title.clone()),
                    ]));
                }
            }
            for pull_request in &standalone {
                rows.push(Row::new(vec![
                    Cell::from(pull_request.key.to_string()),
                    Cell::from(state_label(&pull_request.state)),
                    Cell::from(ci_label(&pull_request.ci_state)),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(pull_request.title.clone()),
                ]));
            }
            let table = Table::new(
                rows,
                [
                    Constraint::Length(18),
                    Constraint::Length(8),
                    Constraint::Length(12),
                    Constraint::Length(11),
                    Constraint::Length(10),
                    Constraint::Min(10),
                ],
            )
            .header(
                Row::new(["Pull request", "State", "CI", "Position", "Stack", "Title"])
                    .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD))
            .highlight_symbol("▶ ");
            let selected = if self.focus == Pane::Watched {
                Some(self.pr_selected)
            } else {
                None
            };
            let mut state = TableState::default().with_selected(selected);
            frame.render_stateful_widget(table, watched_layout[1], &mut state);
        }

        let activity_block = Block::bordered()
            .title(
                Line::from(" Activity ")
                    .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .border_style(Style::new().fg(Color::Green));
        if self.view.activity.is_empty() {
            frame.render_widget(
                Paragraph::new("no activity yet").block(activity_block),
                chunks[3],
            );
        } else {
            let rows: Vec<Row> = self
                .view
                .activity
                .iter()
                .map(|record| {
                    Row::new(vec![
                        Cell::from(record.id.clone()),
                        Cell::from(record.observed_at.clone()),
                        Cell::from(record.key.to_string()),
                        Cell::from(if record.read { "read" } else { "unread" }),
                        Cell::from(
                            activity_stack(&self.view, &record.key)
                                .map(|number| format!("stack {number}"))
                                .unwrap_or_default(),
                        ),
                        Cell::from(activity_summary(&record.event)),
                    ])
                })
                .collect();
            let table = Table::new(
                rows,
                [
                    Constraint::Length(5),
                    Constraint::Length(27),
                    Constraint::Length(14),
                    Constraint::Length(7),
                    Constraint::Length(10),
                    Constraint::Min(20),
                ],
            )
            .header(
                Row::new(["ID", "Observed", "Pull request", "State", "Stack", "Event"])
                    .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .block(activity_block)
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD))
            .highlight_symbol("▶ ");
            let selected = if self.focus == Pane::Activity {
                Some(self.selected)
            } else {
                None
            };
            let mut state = TableState::default().with_selected(selected);
            frame.render_stateful_widget(table, chunks[3], &mut state);
        }

        let controls = Block::bordered()
            .title(" Controls ")
            .border_style(Style::new().fg(Color::Gray));
        let controls_inner = controls.inner(chunks[4]);
        frame.render_widget(controls, chunks[4]);
        match self.status.as_deref() {
            Some(status) => {
                let lines = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
                    .split(controls_inner);
                frame.render_widget(Paragraph::new(self.footer_keys()), lines[0]);
                frame.render_widget(
                    Paragraph::new(status).style(Style::new().fg(Color::Yellow)),
                    lines[1],
                );
            }
            None => {
                frame.render_widget(Paragraph::new(self.footer_keys()), controls_inner);
            }
        }
        if self.popup.is_some() {
            self.render_popup(frame);
        }
    }

    fn render_popup(&self, frame: &mut Frame) {
        let Some(detail) = self.popup.as_ref() else {
            return;
        };
        let area = frame.area();
        let width = (area.width * 90 / 100)
            .min(area.width.saturating_sub(2))
            .max(20);
        let observed = detail
            .record
            .observed_at
            .get(..19)
            .unwrap_or(&detail.record.observed_at);
        let key_style = Style::new()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Activity ", key_style),
                Span::raw(detail.record.id.clone()),
            ]),
            Line::from(format!(
                "{}   {}",
                observed,
                activity_summary(&detail.record.event)
            )),
        ];
        match detail.pull_request.as_ref() {
            Some(pull_request) => {
                lines.push(Line::from(format!(
                    "{}   {}",
                    pull_request.key, pull_request.title
                )));
                let mut state_line = format!(
                    "State: {}   {}",
                    state_label(&pull_request.state),
                    ci_label(&pull_request.ci_state)
                );
                if let Some(stack) = pull_request.stack.as_ref() {
                    state_line.push_str(&format!(
                        "   Stack: {} position {}",
                        stack.number, stack.position
                    ));
                }
                lines.push(
                    Line::from(state_line)
                        .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
                );
            }
            None => lines.push(Line::from(format!(
                "{}   pull request not in view",
                detail.record.key
            ))),
        }
        lines.push(
            Line::from(format!(
                "Read: {}",
                if detail.record.read { "read" } else { "unread" }
            ))
            .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
        );
        let hint = Line::from(vec![
            Span::styled("Enter", key_style),
            Span::raw(" open in browser   "),
            Span::styled("Esc", key_style),
            Span::raw(" close"),
        ]);
        let inner_width = usize::from(width.saturating_sub(2)).max(1);
        let rows: usize = lines
            .iter()
            .map(|line| {
                let length: usize = line.spans.iter().map(|span| span.content.chars().count()).sum();
                length.max(1).div_ceil(inner_width)
            })
            .sum();
        let height = (rows as u16 + 3)
            .max(7)
            .min(area.height.saturating_sub(2).max(7));
        let rect = Rect::new(
            (area.width - width) / 2,
            (area.height - height) / 2,
            width,
            height,
        );
        let block = Block::bordered()
            .title(
                Line::from(" Activity ")
                    .style(Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .border_style(Style::new().fg(Color::Green));
        let inner = block.inner(rect);
        frame.render_widget(Clear, rect);
        frame.render_widget(block, rect);
        let content = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(1),
        );
        let hint_area = Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(1),
            inner.width,
            1,
        );
        frame.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: false }),
            content,
        );
        frame.render_widget(Paragraph::new(hint), hint_area);
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
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use crossterm::event::KeyModifiers;
    use passuff_config::ConfigLoader;
    use passuff_core::{
        AccountView, ActivityEvent, ActivityRecord, CiState, PrState, PullRequestKey,
        PullRequestStack, PullRequestView, ReadOutcome, StateError,
    };

    use super::*;

    struct FakeInbox {
        views: Mutex<VecDeque<InboxView>>,
        fail_view: bool,
        reads: AtomicUsize,
    }

    impl FakeInbox {
        fn new(views: Vec<InboxView>) -> Self {
            Self {
                views: Mutex::new(views.into()),
                fail_view: false,
                reads: AtomicUsize::new(0),
            }
        }

        fn failing_view() -> Self {
            Self {
                views: Mutex::new(VecDeque::new()),
                fail_view: true,
                reads: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl InboxService for FakeInbox {
        async fn view(&self, _config: &CoreConfig) -> Result<InboxView, CoreError> {
            if self.fail_view {
                return Err(CoreError::State(StateError::Read {
                    path: "state.json".into(),
                    message: "read-only".into(),
                }));
            }
            Ok(self
                .views
                .lock()
                .expect("views")
                .pop_front()
                .expect("scripted view"))
        }

        async fn mark_read(&self, _id: &str) -> Result<ReadOutcome, CoreError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            Ok(ReadOutcome {
                found: true,
                changed: 1,
                unread: 0,
            })
        }

        async fn mark_all_read(&self) -> Result<ReadOutcome, CoreError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            Ok(ReadOutcome {
                found: true,
                changed: 1,
                unread: 0,
            })
        }
    }

    fn core_config() -> CoreConfig {
        CoreConfig {
            accounts: Default::default(),
            events: Default::default(),
            watch: Default::default(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn view(activity: Vec<ActivityRecord>) -> InboxView {
        InboxView {
            accounts: vec![AccountView {
                label: "personal".to_string(),
                login: Some("octo-personal".to_string()),
                last_check_at: None,
            }],
            scope: Vec::new(),
            scope_label: "all authored pull requests".to_string(),
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

    fn pr_key(key: &str) -> PullRequestKey {
        let (repository, number) = key.split_once('#').expect("repository#number");
        PullRequestKey::new(repository.to_string(), number.parse().expect("number"))
    }

    fn stacked(key: &str, number: u64, size: u64, position: u64, base: &str) -> PullRequestView {
        PullRequestView {
            key: pr_key(key),
            state: PrState::Open,
            ci_state: CiState::Success,
            title: format!("Pull request {key}"),
            url: format!("https://github.com/{key}"),
            stack: Some(PullRequestStack {
                number,
                size,
                position,
                base: base.to_string(),
            }),
        }
    }

    fn standalone(key: &str) -> PullRequestView {
        PullRequestView {
            key: pr_key(key),
            state: PrState::Open,
            ci_state: CiState::Success,
            title: format!("Pull request {key}"),
            url: format!("https://github.com/{key}"),
            stack: None,
        }
    }

    #[test]
    fn the_activity_detail_wraps_a_long_title() {
        let title = "Set up passuff interactively when no configuration exists and validate every account before writing";
        let mut pull_request = standalone("acme/api#42");
        pull_request.title = title.to_string();
        let record = record_on("acme/api#42", "a1", false);
        let mut app = TuiApp::new(view(vec![record.clone()]));
        app.view.pull_requests.push(pull_request.clone());
        app.popup = Some(ActivityDetail {
            record,
            pull_request: Some(pull_request),
            url: None,
        });
        let snapshot = app.snapshot();
        assert!(
            snapshot.contains("before writing"),
            "long title was clipped: {snapshot}"
        );
        assert!(snapshot.contains("open in browser"));
        assert!(snapshot.contains("Esc close"));
        let lines: Vec<&str> = snapshot.lines().collect();
        let hint_row = lines
            .iter()
            .position(|line| line.contains("open in browser"))
            .expect("hint line");
        assert!(
            lines[hint_row + 1].contains("└"),
            "hint is not on the bottom line: {}", lines[hint_row + 1]
        );
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| app.render(frame))
            .expect("draw popup");
        let buffer = terminal.backend().buffer();
        let green_frame = (0..30).any(|y| {
            (0..100).any(|x| {
                let cell = &buffer[(x, y)];
                cell.fg == Color::Green && cell.symbol() == "┌"
            })
        });
        assert!(green_frame, "the popup frame is not green");
    }

    fn record_on(key: &str, id: &str, read: bool) -> ActivityRecord {
        ActivityRecord {
            id: id.to_string(),
            key: pr_key(key),
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
        assert!(frame.contains("▶ a2"), "{frame}");
        assert!(!frame.contains("▶ a1"), "{frame}");
        assert!(frame.contains("New comment from @alice"), "{frame}");
    }

    #[test]
    fn snapshot_shows_the_configured_watch_scope() {
        let mut view = view(Vec::new());
        view.scope = vec!["acme/api".to_string(), "acme/web".to_string()];
        view.scope_label = "acme/api, acme/web".to_string();
        let frame = TuiApp::new(view).snapshot();
        assert!(frame.contains("watch scope: acme/api, acme/web"), "{frame}");
    }

    #[tokio::test]
    async fn reload_clamps_the_selection_when_the_list_shrinks() {
        let mut app = TuiApp::new(view(vec![record("a2", false), record("a1", false)]));
        app.selected = 1;
        let inbox = FakeInbox::new(vec![view(vec![record("a2", true)])]);
        app.reload(&inbox, &core_config()).await.expect("reload");
        assert_eq!(app.selected, 0);
        assert_eq!(
            app.selected_summary().as_deref(),
            Some("New comment from @alice")
        );
    }

    #[tokio::test]
    async fn a_failed_reload_after_a_read_is_reported() {
        let mut app = TuiApp::new(view(vec![record("a2", false)]));
        let inbox = FakeInbox::failing_view();
        let error = app
            .handle_key(key(KeyCode::Char('r')), &inbox, &core_config())
            .await
            .expect_err("reload failure");
        assert!(matches!(error, CoreError::State(_)));
        assert_eq!(inbox.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn an_unbound_key_changes_nothing() {
        let mut app = TuiApp::new(view(vec![record("a2", false), record("a1", false)]));
        let inbox = FakeInbox::new(Vec::new());
        let action = app
            .handle_key(key(KeyCode::Char('x')), &inbox, &core_config())
            .await
            .expect("continue");
        assert_eq!(action, TuiAction::Continue);
        assert_eq!(app.selected, 0);
        assert_eq!(inbox.reads.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn load_view_maps_config_errors_to_exit_two() {
        let config = ConfigLoader::parse("[accounts.work]\n");
        let error = load_view(&config, &FakeInbox::new(Vec::new()))
            .await
            .expect_err("configuration error");
        assert_eq!(error.exit, 2);
        assert!(error.message.contains("configuration error"), "{error:?}");
    }

    #[tokio::test]
    async fn load_view_maps_state_errors_to_exit_three() {
        let config = ConfigLoader::parse("[accounts.personal]\ntoken_env = \"TOKEN\"\n");
        let error = load_view(&config, &FakeInbox::failing_view())
            .await
            .expect_err("state error");
        assert_eq!(error.exit, 3);
        assert!(error.message.contains("state error"), "{error:?}");
    }
    #[test]
    fn snapshot_groups_stack_members_in_position_order() {
        let mut view = view(Vec::new());
        view.pull_requests = vec![
            stacked("acme/api#42", 7, 2, 2, "main"),
            stacked("acme/api#43", 7, 2, 1, "main"),
        ];
        let frame = TuiApp::new(view).snapshot();
        assert!(
            frame.contains("acme/api stack 7 | base main | size 2 | 0 unread in stack"),
            "{frame}"
        );
        let first = frame.find("position 1").expect("position 1");
        let second = frame.find("position 2").expect("position 2");
        assert!(first < second, "{frame}");
    }

    #[test]
    fn snapshot_keeps_a_position_gap() {
        let mut view = view(Vec::new());
        view.pull_requests = vec![
            stacked("acme/api#42", 7, 3, 1, "main"),
            stacked("acme/api#44", 7, 3, 3, "main"),
        ];
        let frame = TuiApp::new(view).snapshot();
        assert!(frame.contains("size 3"), "{frame}");
        assert!(frame.contains("position 1"), "{frame}");
        assert!(frame.contains("position 3"), "{frame}");
        assert!(!frame.contains("position 2"), "{frame}");
    }

    #[test]
    fn snapshot_renders_a_standalone_row_without_a_stack_summary() {
        let mut view = view(Vec::new());
        view.pull_requests = vec![standalone("acme/api#42")];
        let frame = TuiApp::new(view).snapshot();
        assert!(frame.contains("acme/api#42"), "{frame}");
        assert!(!frame.contains("stack 7"), "{frame}");
        assert!(!frame.contains("base main"), "{frame}");
    }

    #[test]
    fn snapshot_counts_unread_per_stack() {
        let mut view = view(vec![
            record_on("acme/api#40", "a1", false),
            record_on("acme/api#42", "a2", false),
            record_on("acme/api#43", "a3", true),
        ]);
        view.pull_requests = vec![
            stacked("acme/api#42", 7, 2, 1, "main"),
            stacked("acme/api#43", 7, 2, 2, "main"),
            standalone("acme/api#40"),
        ];
        let frame = TuiApp::new(view).snapshot();
        assert!(frame.contains("2 unread of 3"), "{frame}");
        assert!(frame.contains("1 unread in stack"), "{frame}");
    }

    #[test]
    fn snapshot_labels_activity_with_its_stack() {
        let mut view = view(vec![record_on("acme/api#43", "a1", false)]);
        view.pull_requests = vec![stacked("acme/api#43", 7, 2, 2, "main")];
        let frame = TuiApp::new(view).snapshot();
        let activity = frame
            .lines()
            .find(|line| line.contains("New comment from @alice"))
            .expect("activity row");
        assert!(activity.contains("stack 7"), "{frame}");
    }

    #[test]
    fn snapshot_separates_the_same_stack_number_across_repositories() {
        let mut view = view(Vec::new());
        view.pull_requests = vec![
            stacked("acme/api#42", 7, 1, 1, "main"),
            stacked("acme/web#40", 7, 1, 1, "main"),
        ];
        let frame = TuiApp::new(view).snapshot();
        assert!(frame.contains("acme/api stack 7"), "{frame}");
        assert!(frame.contains("acme/web stack 7"), "{frame}");
    }
}
