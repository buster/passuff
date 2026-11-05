use std::cell::Cell as StdCell;
use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent};
use passuff_config::{AppConfig, ConfigError, CoreConfig};
use passuff_core::{
    excerpt, ActivityEvent, ActivityRecord, CoreError, InboxService, InboxView, PrState,
    PullRequestKey, PullRequestView,
};
use ratatui::backend::TestBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, TableState};
use ratatui::{Frame, Terminal};

use crate::delivery::NotificationSwitches;
use crate::theme;
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
    QuitSave,
    QuitDiscard,
    OpenUrl(String),
    ToggleIgnoredView,
    ToggleIgnore(PullRequestKey),
    ResetView,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TuiPreferences {
    pub sort: String,
    pub descending: bool,
    pub filter: Option<String>,
    #[serde(default)]
    pub unread_only: bool,
    #[serde(default)]
    pub show_ignored: bool,
    #[serde(default = "enabled_by_default")]
    pub desktop: bool,
    #[serde(default = "enabled_by_default")]
    pub sound: bool,
}

fn enabled_by_default() -> bool {
    true
}

impl Default for TuiPreferences {
    fn default() -> Self {
        Self {
            sort: "updated".to_string(),
            descending: true,
            filter: None,
            unread_only: false,
            show_ignored: false,
            desktop: true,
            sound: true,
        }
    }
}

fn sort_from_preference(value: &str) -> SortMode {
    match value {
        "name" => SortMode::Name,
        "state" => SortMode::State,
        "age" => SortMode::Age,
        _ => SortMode::Updated,
    }
}

fn filter_from_preference(value: &str) -> Option<ActivityFilter> {
    let (kind, rest) = value.split_once(':')?;
    match kind {
        "pr" => {
            let (repository, number) = rest.rsplit_once('#')?;
            Some(ActivityFilter::PullRequest(PullRequestKey::new(
                repository.to_string(),
                number.parse().ok()?,
            )))
        }
        "stack" => {
            let (repository, number) = rest.rsplit_once('#')?;
            Some(ActivityFilter::Stack(
                repository.to_string(),
                number.parse().ok()?,
            ))
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Updated,
    Name,
    State,
    Age,
}

impl SortMode {
    fn label(&self) -> &'static str {
        match self {
            SortMode::Updated => "updated",
            SortMode::Name => "name",
            SortMode::State => "state",
            SortMode::Age => "age",
        }
    }

    fn next(&self) -> Self {
        match self {
            SortMode::Updated => SortMode::Name,
            SortMode::Name => SortMode::State,
            SortMode::State => SortMode::Age,
            SortMode::Age => SortMode::Updated,
        }
    }

    fn default_descending(&self) -> bool {
        matches!(self, SortMode::Updated | SortMode::Age)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityFilter {
    PullRequest(PullRequestKey),
    Stack(String, u64),
}

impl ActivityFilter {
    fn label(&self) -> String {
        match self {
            ActivityFilter::PullRequest(key) => key.to_string(),
            ActivityFilter::Stack(repository, number) => {
                format!("{repository} stack {number}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActivityDetail {
    pub record: ActivityRecord,
    pub changes: Vec<ActivityRecord>,
    pub pull_request: Option<PullRequestView>,
    pub url: Option<String>,
}

#[derive(Debug)]
struct StackGroup<'a> {
    repository: String,
    number: u64,
    members: Vec<&'a PullRequestView>,
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
        result.push(StackGroup {
            repository,
            number,
            members,
        });
    }
    result.sort_by(|left, right| {
        newest(left.members.iter().copied())
            .cmp(newest(right.members.iter().copied()))
            .reverse()
            .then_with(|| left.repository.cmp(&right.repository))
            .then_with(|| left.number.cmp(&right.number))
    });
    standalone.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.key.cmp(&right.key))
    });
    (result, standalone)
}

fn state_rank(state: &PrState) -> u8 {
    match state {
        PrState::Open => 0,
        PrState::Merged => 1,
        PrState::Closed => 2,
    }
}

fn ordered_units<'a>(
    groups: &[StackGroup<'a>],
    standalone: &[&'a PullRequestView],
) -> Vec<Vec<&'a PullRequestView>> {
    let mut units: Vec<(String, PullRequestKey, Vec<&PullRequestView>)> = groups
        .iter()
        .map(|group| {
            (
                newest(group.members.iter().copied()).to_string(),
                group
                    .members
                    .first()
                    .map(|pull_request| pull_request.key.clone())
                    .unwrap_or_else(|| PullRequestKey::new(String::new(), 0)),
                group.members.clone(),
            )
        })
        .collect();
    for pull_request in standalone {
        units.push((
            pull_request.updated_at.clone(),
            pull_request.key.clone(),
            vec![pull_request],
        ));
    }
    units.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    units.into_iter().map(|(_, _, rows)| rows).collect()
}

fn newest<'a>(pull_requests: impl Iterator<Item = &'a PullRequestView>) -> &'a str {
    pull_requests
        .map(|pull_request| pull_request.updated_at.as_str())
        .max()
        .unwrap_or("")
}

const PULL_REQUEST_COLUMN_WIDTH: usize = 18;
const PULL_REQUEST_LABEL_WIDTH: usize = PULL_REQUEST_COLUMN_WIDTH - 2;

fn centered_cell(text: &str) -> Cell<'static> {
    Cell::new(Text::from(text.to_string()).alignment(Alignment::Center))
}

fn tail_label(text: &str, width: usize) -> String {
    let characters: Vec<char> = text.chars().collect();
    if characters.len() <= width {
        return text.to_string();
    }
    let tail: String = characters[characters.len() - (width - 1)..]
        .iter()
        .collect();
    format!("…{tail}")
}

fn people_label(author: &str, assignees: &[String]) -> String {
    let mut others: Vec<&str> = Vec::new();
    for assignee in assignees {
        if assignee.eq_ignore_ascii_case(author)
            || others
                .iter()
                .any(|seen| seen.eq_ignore_ascii_case(assignee))
        {
            continue;
        }
        others.push(assignee);
    }
    let creator = format!("@{author}");
    if others.is_empty() {
        creator
    } else {
        format!(
            "{creator} → {}",
            others
                .iter()
                .map(|login| format!("@{login}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn event_line(event: &ActivityEvent) -> String {
    match event {
        ActivityEvent::Comment { author, body } => {
            let preview = excerpt(body);
            if preview.trim().is_empty() {
                format!("comment by @{author}")
            } else {
                format!("comment by @{author}: {preview}")
            }
        }
        ActivityEvent::ReviewComment { author, body } => {
            let preview = excerpt(body);
            if preview.trim().is_empty() {
                format!("review comment by @{author}")
            } else {
                format!("review comment by @{author}: {preview}")
            }
        }
        ActivityEvent::Review { author, state, .. } => format!("review by @{author}: {state}"),
        ActivityEvent::CiFailure { failed_checks } if !failed_checks.is_empty() => {
            format!("CI failing: {}", failed_checks.join(", "))
        }
        ActivityEvent::CiFailure { .. } => "CI failing".to_string(),
        ActivityEvent::CiRecovery => "CI recovered".to_string(),
        ActivityEvent::Opened { author } => format!("opened by @{author}"),
        ActivityEvent::StateChange { state, actor } => match state {
            PrState::Merged => actor
                .as_ref()
                .map(|actor| format!("merged by @{actor}"))
                .unwrap_or_else(|| "merged".to_string()),
            PrState::Closed => "closed".to_string(),
            PrState::Open => "reopened".to_string(),
        },
    }
}

fn strip_html_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    loop {
        match rest.find("<!--") {
            Some(start) => {
                out.push_str(&rest[..start]);
                match rest[start + 4..].find("-->") {
                    Some(end) => rest = &rest[start + 4 + end + 3..],
                    None => return out,
                }
            }
            None => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

fn parse_link(chars: &[char], bracket: usize) -> Option<(String, usize)> {
    let mut label = String::new();
    let mut index = bracket + 1;
    while index < chars.len() {
        match chars[index] {
            ']' => {
                if chars.get(index + 1) != Some(&'(') {
                    return None;
                }
                let mut end = index + 2;
                while end < chars.len() && chars[end] != ')' {
                    end += 1;
                }
                if end < chars.len() {
                    return Some((label, end + 1));
                }
                return None;
            }
            character => label.push(character),
        }
        index += 1;
    }
    None
}

fn strip_inline_markdown(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        let image = character == '!' && chars.get(index + 1) == Some(&'[');
        if character == '[' || image {
            let bracket = if image { index + 1 } else { index };
            if let Some((label, next)) = parse_link(&chars, bracket) {
                out.push_str(&label);
                index = next;
                continue;
            }
        }
        match character {
            '`' => {}
            '~' if chars.get(index + 1) == Some(&'~') => index += 1,
            '*' if chars.get(index + 1) == Some(&'*') => index += 1,
            '_' if chars.get(index + 1) == Some(&'_') => index += 1,
            _ => out.push(character),
        }
        index += 1;
    }
    out
}

fn heading_text(line: &str) -> Option<&str> {
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = line[hashes..].strip_prefix(' ')?;
    Some(rest.trim_end_matches('#').trim_end())
}

fn is_horizontal_rule(line: &str) -> bool {
    let compact: String = line
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    compact.len() >= 3
        && compact
            .chars()
            .all(|character| character == '-' || character == '*' || character == '_')
}

fn list_item(line: &str) -> Option<(String, &str)> {
    for marker in ["- ", "* ", "+ "] {
        if let Some(content) = line.strip_prefix(marker) {
            let (prefix, content) = if let Some(content) = content.strip_prefix("[ ] ") {
                ("☐ ", content)
            } else if let Some(content) = content
                .strip_prefix("[x] ")
                .or_else(|| content.strip_prefix("[X] "))
            {
                ("☑ ", content)
            } else {
                ("• ", content)
            };
            return Some((prefix.to_string(), content));
        }
    }
    let digits = line
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    if digits > 0 {
        if let Some(content) = line[digits..].strip_prefix(". ") {
            return Some((format!("{}. ", &line[..digits]), content));
        }
    }
    None
}

fn push_wrapped(
    lines: &mut Vec<Line<'static>>,
    text: &str,
    prefix: &str,
    width: usize,
    style: Style,
) {
    let indent = " ".repeat(prefix.chars().count());
    let available = width.saturating_sub(indent.chars().count()).max(1);
    for (index, chunk) in wrap_text(text, available).iter().enumerate() {
        let lead = if index == 0 { prefix } else { indent.as_str() };
        lines.push(Line::from(Span::styled(format!("{lead}{chunk}"), style)));
    }
}

fn render_description(text: &str, width: usize) -> Vec<Line<'static>> {
    let width = width.max(1);
    let heading_style = theme::green_bold();
    let code_style = theme::dim();
    let body_style = theme::body();
    let mut lines: Vec<Line<'static>> = Vec::new();
    let body = strip_html_comments(text);
    let mut in_code = false;
    for raw in body.lines() {
        let line = raw.trim_end();
        let start = line.trim_start();
        if start.starts_with("```") || start.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            if line.is_empty() {
                lines.push(Line::default());
            } else {
                for chunk in wrap_text(line, width) {
                    lines.push(Line::from(Span::styled(chunk, code_style)));
                }
            }
            continue;
        }
        if start.is_empty() {
            if lines.last().is_some_and(|line| !line.spans.is_empty()) {
                lines.push(Line::default());
            }
            continue;
        }
        if let Some(heading) = heading_text(start) {
            let text = strip_inline_markdown(heading);
            for chunk in wrap_text(&text, width) {
                lines.push(Line::from(Span::styled(chunk, heading_style)));
            }
            continue;
        }
        if is_horizontal_rule(start) {
            lines.push(Line::from(Span::styled("─".repeat(width), theme::rule())));
            continue;
        }
        if let Some((prefix, content)) = list_item(start) {
            let text = strip_inline_markdown(content);
            push_wrapped(&mut lines, &text, &prefix, width, body_style);
            continue;
        }
        let text = strip_inline_markdown(start);
        for chunk in wrap_text(&text, width) {
            lines.push(Line::from(Span::styled(chunk, body_style)));
        }
    }
    while lines.last().is_some_and(|line| line.spans.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(Line::default());
    }
    lines
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };
        if candidate <= width {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if word.chars().count() > width {
            let mut chunk = String::new();
            for character in word.chars() {
                if chunk.chars().count() == width {
                    lines.push(std::mem::take(&mut chunk));
                }
                chunk.push(character);
            }
            current = chunk;
        } else {
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn short_time(observed_at: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(observed_at)
        .map(|time| time.format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| observed_at.get(..16).unwrap_or(observed_at).to_string())
}

fn short_record_time(record: &ActivityRecord) -> String {
    let time = short_time(passuff_core::effective_time(record));
    if passuff_core::is_approximate(record) {
        format!("~{time}")
    } else {
        time
    }
}

fn activity_time_line(record: &ActivityRecord) -> Line<'static> {
    let mut spans = vec![
        Span::styled("When: ", theme::accent()),
        Span::styled(short_record_time(record), theme::dim()),
    ];
    if passuff_core::is_approximate(record) {
        spans.push(Span::styled(" (fetch time)", theme::dim()));
    } else {
        let fetched = short_time(&record.observed_at);
        if fetched != short_time(passuff_core::effective_time(record)) {
            spans.push(Span::styled("   Fetched: ", theme::accent()));
            spans.push(Span::styled(fetched, theme::dim()));
        }
    }
    Line::from(spans)
}

fn activity_label(record: &ActivityRecord) -> String {
    let repository = record
        .key
        .repository
        .rsplit('/')
        .next()
        .unwrap_or(&record.key.repository);
    format!("{repository}#{}", record.key.number)
}

fn activity_stack(view: &InboxView, key: &PullRequestKey) -> Option<u64> {
    view.pull_requests
        .iter()
        .find(|pull_request| &pull_request.key == key)
        .and_then(|pull_request| pull_request.stack.as_ref().map(|stack| stack.number))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollAction {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

fn scroll_action(code: KeyCode) -> Option<ScrollAction> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(ScrollAction::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(ScrollAction::Down),
        KeyCode::PageUp => Some(ScrollAction::PageUp),
        KeyCode::PageDown => Some(ScrollAction::PageDown),
        KeyCode::Home => Some(ScrollAction::Home),
        KeyCode::End => Some(ScrollAction::End),
        _ => None,
    }
}

fn apply_scroll(offset: u16, action: ScrollAction, max: u16, page: u16) -> u16 {
    match action {
        ScrollAction::Up => offset.saturating_sub(1),
        ScrollAction::Down => offset.saturating_add(1).min(max),
        ScrollAction::PageUp => offset.saturating_sub(page.max(1)),
        ScrollAction::PageDown => offset.saturating_add(page.max(1)).min(max),
        ScrollAction::Home => 0,
        ScrollAction::End => max,
    }
}

fn activity_popup_width(area: Rect) -> u16 {
    (area.width * 90 / 100)
        .min(area.width.saturating_sub(2))
        .max(20)
}

fn activity_popup_height(area: Rect, rows: usize) -> u16 {
    (rows as u16 + 3)
        .max(7)
        .min(area.height.saturating_sub(2).max(7))
}

fn activity_popup_content_height(area: Rect, rows: usize) -> u16 {
    activity_popup_height(area, rows).saturating_sub(3)
}

fn pr_popup_width(area: Rect) -> u16 {
    (area.width * 80 / 100)
        .min(area.width.saturating_sub(2))
        .max(20)
}

fn pr_popup_height(area: Rect, rows: usize) -> u16 {
    let max_height = (area.height * 80 / 100)
        .min(area.height.saturating_sub(2))
        .max(8);
    (rows as u16 + 3).min(max_height)
}

fn pr_popup_content_height(area: Rect, rows: usize) -> u16 {
    pr_popup_height(area, rows).saturating_sub(3)
}

fn push_wrapped_styled(lines: &mut Vec<Line<'static>>, text: &str, width: usize, style: Style) {
    for chunk in wrap_text(text, width) {
        lines.push(Line::from(chunk).style(style));
    }
}

fn indent_lines(lines: Vec<Line<'static>>, indent: usize) -> Vec<Line<'static>> {
    if indent == 0 {
        return lines;
    }
    let prefix = " ".repeat(indent);
    lines
        .into_iter()
        .map(|line| {
            let mut spans = Vec::with_capacity(line.spans.len() + 1);
            spans.push(Span::raw(prefix.clone()));
            spans.extend(line.spans);
            Line::from(spans)
        })
        .collect()
}

fn separator(width: usize) -> Line<'static> {
    Line::from(Span::styled("─".repeat(width), theme::rule()))
}

fn activity_body(event: &ActivityEvent) -> Option<(String, &str)> {
    let (label, body) = match event {
        ActivityEvent::Comment { author, body } => (format!("Comment by @{author}"), body.as_str()),
        ActivityEvent::ReviewComment { author, body } => {
            (format!("Review comment by @{author}"), body.as_str())
        }
        ActivityEvent::Review {
            author,
            state,
            body,
        } => (format!("Review by @{author} ({state})"), body.as_str()),
        _ => return None,
    };
    if body.trim().is_empty() {
        None
    } else {
        Some((label, body))
    }
}

fn activity_detail_lines(detail: &ActivityDetail, width: usize) -> Vec<Line<'static>> {
    let title_style = theme::accent_bold();
    let label_style = theme::accent();
    let value_style = theme::dim();
    let mut lines = vec![Line::from(vec![
        Span::styled("Activity ", title_style),
        Span::styled(activity_label(&detail.record), value_style),
    ])];
    match detail.pull_request.as_ref() {
        Some(pull_request) => {
            push_wrapped_styled(
                &mut lines,
                &format!("{}   {}", pull_request.key, pull_request.title),
                width,
                value_style,
            );
            lines.push(activity_time_line(&detail.record));
            let mut state = vec![
                Span::styled("State: ", label_style),
                Span::styled(state_label(&pull_request.state).to_string(), value_style),
                Span::raw("   "),
                Span::styled(ci_label(&pull_request.ci_state).to_string(), value_style),
            ];
            if let Some(stack) = pull_request.stack.as_ref() {
                state.push(Span::raw("   "));
                state.push(Span::styled("Stack: ", label_style));
                state.push(Span::styled(
                    format!("{} position {}", stack.number, stack.position),
                    value_style,
                ));
            }
            lines.push(Line::from(state));
        }
        None => {
            push_wrapped_styled(
                &mut lines,
                &format!("{}   pull request not in view", detail.record.key),
                width,
                value_style,
            );
            lines.push(activity_time_line(&detail.record));
        }
    }
    if let Some((label, body)) = activity_body(&detail.record.event) {
        lines.push(separator(width));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(label, title_style),
        ]));
        lines.extend(indent_lines(
            render_description(body, width.saturating_sub(2).max(1)),
            2,
        ));
    }
    lines.push(Line::default());
    push_wrapped_styled(
        &mut lines,
        &format!("Changes on {}: {}", detail.record.key, detail.changes.len()),
        width,
        theme::dim(),
    );
    for change in &detail.changes {
        push_wrapped_styled(
            &mut lines,
            &format!(
                "{}   {}   {}",
                short_record_time(change),
                event_line(&change.event),
                if change.read { "read" } else { "unread" }
            ),
            width,
            theme::dim(),
        );
    }
    lines
}

fn pr_detail_lines(pull_request: &PullRequestView, width: usize) -> Vec<Line<'static>> {
    let title_style = theme::accent_bold();
    let label_style = theme::accent();
    let value_style = theme::dim();
    let label = |text: &str| Span::styled(text.to_string(), label_style);
    let value = |text: String| Span::styled(text, value_style);

    let mut status = state_label(&pull_request.state).to_string();
    if pull_request.is_draft {
        status.push_str(", draft");
    }
    let ci = ci_label(&pull_request.ci_state);
    let ci = ci.strip_prefix("CI ").unwrap_or(ci).to_string();
    let creator = format!(
        "@{} ({})",
        pull_request.author,
        short_time(&pull_request.created_at)
    );
    let assignees = if pull_request.assignees.is_empty() {
        "none".to_string()
    } else {
        pull_request
            .assignees
            .iter()
            .map(|login| format!("@{login}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut lines = vec![
        Line::from(vec![
            label("Status: "),
            value(status),
            Span::raw("   "),
            label("CI: "),
            value(ci),
        ]),
        Line::from(vec![
            label("Creator: "),
            value(creator),
            Span::raw("   "),
            label("Assignee: "),
            value(assignees),
        ]),
    ];
    if let Some(stack) = pull_request.stack.as_ref() {
        lines.push(Line::from(vec![
            label("Stack: "),
            value(format!(
                "{}/{}   base {}",
                stack.position, stack.size, stack.base
            )),
        ]));
    }
    lines.push(separator(width));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("Description:", title_style),
    ]));
    let description = if pull_request.description.trim().is_empty() {
        "(no description)".to_string()
    } else {
        pull_request.description.clone()
    };
    lines.extend(indent_lines(
        render_description(&description, width.saturating_sub(2).max(1)),
        2,
    ));
    lines
}

fn settings_row(label: &str, enabled: bool, selected: bool) -> Line<'static> {
    let marker = if selected { "▶ " } else { "  " };
    let label_style = if selected {
        theme::gold_bold()
    } else {
        theme::bright()
    };
    let state_style = if enabled {
        theme::green()
    } else {
        theme::dim()
    };
    Line::from(vec![
        Span::styled(marker, theme::gold_bold()),
        Span::styled(format!("{label:<22}"), label_style),
        Span::styled(if enabled { "on" } else { "off" }, state_style),
    ])
}

fn popup_hint() -> Line<'static> {
    Line::from(vec![
        Span::styled("Enter", theme::key()),
        Span::styled(" open in browser   ", theme::dim()),
        Span::styled("↑↓/PgUp/PgDn", theme::key()),
        Span::styled(" scroll   ", theme::dim()),
        Span::styled("Esc/q", theme::key()),
        Span::styled(" close", theme::dim()),
    ])
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
    countdown: Option<std::time::Duration>,
    fetching: bool,
    spinner: usize,
    budget: Option<(u32, u32)>,
    popup: Option<ActivityDetail>,
    sort: SortMode,
    descending: bool,
    activity_filter: Option<ActivityFilter>,
    unread_only: bool,
    help: Option<Pane>,
    pr_detail: Option<PullRequestView>,
    show_ignored: bool,
    save_prompt: bool,
    viewport: StdCell<(u16, u16)>,
    popup_scroll: StdCell<u16>,
    pr_detail_scroll: StdCell<u16>,
    settings: Option<usize>,
    notifications: NotificationSwitches,
    ignored_pull_requests: Vec<String>,
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
            countdown: None,
            fetching: false,
            spinner: 0,
            budget: None,
            popup: None,
            sort: SortMode::Updated,
            descending: true,
            activity_filter: None,
            unread_only: false,
            help: None,
            pr_detail: None,
            show_ignored: false,
            save_prompt: false,
            viewport: StdCell::new((0, 0)),
            popup_scroll: StdCell::new(0),
            pr_detail_scroll: StdCell::new(0),
            settings: None,
            notifications: NotificationSwitches::new(true, true),
            ignored_pull_requests: Vec::new(),
        }
    }

    pub fn show_ignored(&self) -> bool {
        self.show_ignored
    }

    pub fn set_show_ignored(&mut self, show_ignored: bool) {
        self.show_ignored = show_ignored;
    }

    pub fn set_ignored_pull_requests(&mut self, ignored: &[String]) {
        self.ignored_pull_requests = ignored.to_vec();
    }

    fn is_ignored(&self, key: &PullRequestKey) -> bool {
        let entry = key.to_string();
        self.ignored_pull_requests
            .iter()
            .any(|ignored| ignored.eq_ignore_ascii_case(&entry))
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
        self.pr_detail = None;
        self.popup_scroll.set(0);
        self.pr_detail_scroll.set(0);
    }

    fn scroll_viewport(&self) -> (u16, u16) {
        match self.viewport.get() {
            (0, 0) => (100, 30),
            size => size,
        }
    }

    fn scroll_activity_detail(&self, action: ScrollAction) {
        let Some(detail) = self.popup.as_ref() else {
            return;
        };
        let (width, height) = self.scroll_viewport();
        let area = Rect::new(0, 0, width, height);
        let popup_width = activity_popup_width(area);
        let inner_width = usize::from(popup_width.saturating_sub(2)).max(1);
        let rows = activity_detail_lines(detail, inner_width).len();
        let content_height = activity_popup_content_height(area, rows).max(1);
        let max_scroll = (rows as u16).saturating_sub(content_height);
        self.popup_scroll.set(apply_scroll(
            self.popup_scroll.get(),
            action,
            max_scroll,
            content_height,
        ));
    }

    fn scroll_pr_detail(&self, action: ScrollAction) {
        let Some(pull_request) = self.pr_detail.as_ref() else {
            return;
        };
        let (width, height) = self.scroll_viewport();
        let area = Rect::new(0, 0, width, height);
        let popup_width = pr_popup_width(area);
        let inner_width = usize::from(popup_width.saturating_sub(2)).max(1);
        let rows = pr_detail_lines(pull_request, inner_width).len();
        let content_height = pr_popup_content_height(area, rows).max(1);
        let max_scroll = (rows as u16).saturating_sub(content_height);
        self.pr_detail_scroll.set(apply_scroll(
            self.pr_detail_scroll.get(),
            action,
            max_scroll,
            content_height,
        ));
    }

    fn sync_pr_selection(&mut self) {
        let Some(record) = self.visible_activity().get(self.selected).copied() else {
            return;
        };
        let key = record.key.clone();
        if let Some(index) = self
            .ordered_pull_requests()
            .iter()
            .position(|pull_request| pull_request.key == key)
        {
            self.pr_selected = index;
            self.selected_pr = Some(key);
        }
    }

    /// Show a failure line in the footer until the next successful update.
    pub fn set_status(&mut self, status: Option<String>) {
        self.status = status;
    }

    /// Show the time until the next observation.
    pub fn set_countdown(&mut self, countdown: Option<std::time::Duration>) {
        self.countdown = countdown;
    }

    pub fn set_fetching(&mut self, fetching: bool) {
        self.fetching = fetching;
        if !fetching {
            self.spinner = 0;
        }
    }

    pub fn advance_spinner(&mut self) {
        if self.fetching {
            self.spinner = self.spinner.wrapping_add(1);
        }
    }

    pub fn set_budget(&mut self, remaining: u32, limit: u32) {
        self.budget = Some((remaining, limit));
    }

    pub fn focus(&self) -> Pane {
        self.focus
    }

    fn focus_label(&self) -> &'static str {
        match self.focus {
            Pane::Watched => "PRs",
            Pane::Activity => "activity",
        }
    }

    fn pane_border_style(&self, pane: Pane) -> Style {
        if self.focus == pane {
            Style::new().fg(Color::Green)
        } else {
            theme::border()
        }
    }

    fn pane_title_style(&self, pane: Pane) -> Style {
        if self.focus == pane {
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            theme::dim()
        }
    }

    fn row_highlight_style(&self) -> Style {
        theme::gold_bold()
    }

    fn visible_activity(&self) -> Vec<&ActivityRecord> {
        let records = self
            .view
            .activity
            .iter()
            .filter(|record| !self.unread_only || !record.read);
        match self.activity_filter.as_ref() {
            None => records.collect(),
            Some(ActivityFilter::PullRequest(key)) => {
                records.filter(|record| &record.key == key).collect()
            }
            Some(ActivityFilter::Stack(repository, number)) => {
                let members: Vec<&PullRequestKey> = self
                    .view
                    .pull_requests
                    .iter()
                    .filter(|pull_request| {
                        pull_request.stack.as_ref().is_some_and(|stack| {
                            stack.number == *number
                                && pull_request.key.repository.eq_ignore_ascii_case(repository)
                        })
                    })
                    .map(|pull_request| &pull_request.key)
                    .collect();
                records
                    .filter(|record| members.contains(&&record.key))
                    .collect()
            }
        }
    }

    pub fn sort_mode(&self) -> SortMode {
        self.sort
    }

    pub fn descending(&self) -> bool {
        self.descending
    }

    pub fn activity_filter(&self) -> Option<&ActivityFilter> {
        self.activity_filter.as_ref()
    }

    pub fn preferences(&self) -> TuiPreferences {
        TuiPreferences {
            sort: self.sort.label().to_string(),
            descending: self.descending,
            filter: self.activity_filter.as_ref().map(|filter| match filter {
                ActivityFilter::PullRequest(key) => format!("pr:{key}"),
                ActivityFilter::Stack(repository, number) => {
                    format!("stack:{repository}#{number}")
                }
            }),
            unread_only: self.unread_only,
            show_ignored: self.show_ignored,
            desktop: self.notifications.desktop(),
            sound: self.notifications.sound(),
        }
    }

    pub fn apply_preferences(&mut self, preferences: &TuiPreferences) {
        self.sort = sort_from_preference(&preferences.sort);
        self.descending = preferences.descending;
        self.activity_filter = preferences
            .filter
            .as_deref()
            .and_then(filter_from_preference);
        self.unread_only = preferences.unread_only;
        self.show_ignored = preferences.show_ignored;
        self.notifications.set_desktop(preferences.desktop);
        self.notifications.set_sound(preferences.sound);
    }

    pub fn notification_switches(&self) -> NotificationSwitches {
        self.notifications.clone()
    }

    pub fn show_save_prompt(&mut self) {
        self.save_prompt = true;
    }

    fn sort_units<'a>(
        &self,
        units: Vec<Vec<&'a PullRequestView>>,
    ) -> Vec<Vec<&'a PullRequestView>> {
        let mut units = units;
        units.sort_by(|left, right| {
            let left = left[0];
            let right = right[0];
            let ordering = match self.sort {
                SortMode::Updated => left.updated_at.cmp(&right.updated_at),
                SortMode::Age => left.created_at.cmp(&right.created_at),
                SortMode::Name => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
                SortMode::State => state_rank(&left.state).cmp(&state_rank(&right.state)),
            };
            let ordering = if self.descending {
                ordering.reverse()
            } else {
                ordering
            };
            ordering.then_with(|| left.key.cmp(&right.key))
        });
        units
    }

    pub fn selected_summary(&self) -> Option<String> {
        self.visible_activity()
            .get(self.selected)
            .map(|record| activity_summary(&record.event))
    }

    pub fn selected_pull_request(&self) -> Option<&PullRequestView> {
        self.ordered_pull_requests().get(self.pr_selected).copied()
    }

    fn ordered_pull_requests(&self) -> Vec<&PullRequestView> {
        let (groups, standalone) = stack_groups(&self.view);
        self.sort_units(ordered_units(&groups, &standalone))
            .into_iter()
            .flatten()
            .collect()
    }

    fn clamp_pr_selection(&mut self, preserve: Option<PullRequestKey>) {
        let (index, key) = {
            let ordered = self.ordered_pull_requests();
            if let Some(key) = preserve {
                if let Some(index) = ordered
                    .iter()
                    .position(|pull_request| pull_request.key == key)
                {
                    (index, Some(key))
                } else {
                    let index = self.pr_selected.min(ordered.len().saturating_sub(1));
                    (
                        index,
                        ordered
                            .get(index)
                            .map(|pull_request| pull_request.key.clone()),
                    )
                }
            } else {
                let index = self.pr_selected.min(ordered.len().saturating_sub(1));
                (
                    index,
                    ordered
                        .get(index)
                        .map(|pull_request| pull_request.key.clone()),
                )
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
        let visible = self.visible_activity().len();
        if self.selected >= visible {
            self.selected = visible.saturating_sub(1);
        }
        self.clamp_pr_selection(selected_pr);
        if self.focus == Pane::Activity {
            self.sync_pr_selection();
        }
        Ok(())
    }

    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        inbox: &dyn InboxService,
        config: &CoreConfig,
    ) -> Result<TuiAction, CoreError> {
        if self.save_prompt {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.save_prompt = false;
                    Ok(TuiAction::QuitSave)
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.save_prompt = false;
                    Ok(TuiAction::QuitDiscard)
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.save_prompt = false;
                    Ok(TuiAction::Continue)
                }
                _ => Ok(TuiAction::Continue),
            };
        }
        if let Some(selected) = self.settings {
            match key.code {
                KeyCode::Esc | KeyCode::Char(',') | KeyCode::Char('q') => self.settings = None,
                KeyCode::Char('j') | KeyCode::Down => {
                    self.settings = Some((selected + 1) % 2);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.settings = Some((selected + 1) % 2);
                }
                KeyCode::Char(' ') | KeyCode::Enter => match selected {
                    0 => self.notifications.toggle_desktop(),
                    1 => self.notifications.toggle_sound(),
                    _ => {}
                },
                _ => {}
            }
            return Ok(TuiAction::Continue);
        }
        if self.help.is_some() {
            if matches!(
                key.code,
                KeyCode::Esc
                    | KeyCode::Char('q')
                    | KeyCode::Char('h')
                    | KeyCode::Char('H')
                    | KeyCode::F(1)
            ) {
                self.help = None;
            }
            return Ok(TuiAction::Continue);
        }
        if self.pr_detail.is_some() {
            if let Some(action) = scroll_action(key.code) {
                self.scroll_pr_detail(action);
                return Ok(TuiAction::Continue);
            }
            return match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.pr_detail = None;
                    self.pr_detail_scroll.set(0);
                    Ok(TuiAction::Continue)
                }
                KeyCode::Enter => Ok(self
                    .pr_detail
                    .as_ref()
                    .map(|pull_request| TuiAction::OpenUrl(pull_request.url.clone()))
                    .unwrap_or(TuiAction::Continue)),
                _ => Ok(TuiAction::Continue),
            };
        }
        if self.popup.is_some() {
            if let Some(action) = scroll_action(key.code) {
                self.scroll_activity_detail(action);
                return Ok(TuiAction::Continue);
            }
            return match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.popup = None;
                    self.popup_scroll.set(0);
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
                                Some((
                                    self.pr_selected + 1,
                                    ordered[self.pr_selected + 1].key.clone(),
                                ))
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
                        if self.selected + 1 < self.visible_activity().len() {
                            self.selected += 1;
                        }
                        self.sync_pr_selection();
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Left | KeyCode::Right if self.focus == Pane::Watched => {
                self.show_ignored = !self.show_ignored;
                Ok(TuiAction::ToggleIgnoredView)
            }
            KeyCode::Home => {
                match self.focus {
                    Pane::Watched => {
                        self.pr_selected = 0;
                        self.selected_pr = self
                            .ordered_pull_requests()
                            .first()
                            .map(|pull_request| pull_request.key.clone());
                    }
                    Pane::Activity => {
                        self.selected = 0;
                        self.sync_pr_selection();
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::End => {
                match self.focus {
                    Pane::Watched => {
                        let (index, key) = {
                            let ordered = self.ordered_pull_requests();
                            let index = ordered.len().saturating_sub(1);
                            (
                                index,
                                ordered
                                    .get(index)
                                    .map(|pull_request| pull_request.key.clone()),
                            )
                        };
                        self.pr_selected = index;
                        self.selected_pr = key;
                    }
                    Pane::Activity => {
                        self.selected = self.visible_activity().len().saturating_sub(1);
                        self.sync_pr_selection();
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::PageDown => {
                match self.focus {
                    Pane::Watched => {
                        let (index, key) = {
                            let ordered = self.ordered_pull_requests();
                            let index =
                                (self.pr_selected + 10).min(ordered.len().saturating_sub(1));
                            (
                                index,
                                ordered
                                    .get(index)
                                    .map(|pull_request| pull_request.key.clone()),
                            )
                        };
                        self.pr_selected = index;
                        self.selected_pr = key;
                    }
                    Pane::Activity => {
                        self.selected = (self.selected + 10)
                            .min(self.visible_activity().len().saturating_sub(1));
                        self.sync_pr_selection();
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::PageUp => {
                match self.focus {
                    Pane::Watched => {
                        let index = self.pr_selected.saturating_sub(10);
                        self.pr_selected = index;
                        self.selected_pr = self
                            .ordered_pull_requests()
                            .get(index)
                            .map(|pull_request| pull_request.key.clone());
                    }
                    Pane::Activity => {
                        self.selected = self.selected.saturating_sub(10);
                        self.sync_pr_selection();
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('x') if self.focus == Pane::Watched => Ok(self
                .selected_pull_request()
                .map(|pull_request| TuiAction::ToggleIgnore(pull_request.key.clone()))
                .unwrap_or(TuiAction::Continue)),
            KeyCode::Char('s') if self.focus == Pane::Watched => {
                self.sort = self.sort.next();
                self.descending = self.sort.default_descending();
                let preserve = self.selected_pr.clone();
                self.clamp_pr_selection(preserve);
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('S') if self.focus == Pane::Watched => {
                self.descending = !self.descending;
                let preserve = self.selected_pr.clone();
                self.clamp_pr_selection(preserve);
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('f') if self.focus == Pane::Watched => {
                let next = self
                    .selected_pull_request()
                    .map(|pull_request| ActivityFilter::PullRequest(pull_request.key.clone()));
                if let Some(next) = next {
                    if self.activity_filter.as_ref() == Some(&next) {
                        self.activity_filter = None;
                    } else {
                        self.activity_filter = Some(next);
                    }
                    self.selected = 0;
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('F') if self.focus == Pane::Watched => {
                let filter = self.selected_pull_request().map(|pull_request| {
                    pull_request
                        .stack
                        .as_ref()
                        .map(|stack| {
                            ActivityFilter::Stack(pull_request.key.repository.clone(), stack.number)
                        })
                        .unwrap_or_else(|| ActivityFilter::PullRequest(pull_request.key.clone()))
                });
                if let Some(filter) = filter {
                    if self.activity_filter.as_ref() == Some(&filter) {
                        self.activity_filter = None;
                    } else {
                        self.activity_filter = Some(filter);
                    }
                    self.selected = 0;
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('u') => {
                self.unread_only = !self.unread_only;
                self.selected = 0;
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('c') => {
                self.activity_filter = None;
                self.unread_only = false;
                self.selected = 0;
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('C') => {
                self.sort = SortMode::Updated;
                self.descending = true;
                self.activity_filter = None;
                self.unread_only = false;
                self.show_ignored = false;
                self.selected = 0;
                Ok(TuiAction::ResetView)
            }
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::F(1) => {
                self.help = Some(self.focus);
                Ok(TuiAction::Continue)
            }
            KeyCode::Char(',') => {
                self.settings = Some(0);
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
                        self.sync_pr_selection();
                    }
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Enter if self.focus == Pane::Watched => {
                if let Some(pull_request) = self.selected_pull_request().cloned() {
                    self.popup = None;
                    self.pr_detail_scroll.set(0);
                    self.pr_detail = Some(pull_request);
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Enter if self.focus == Pane::Activity => {
                let selected_record = self
                    .visible_activity()
                    .get(self.selected)
                    .map(|record| (*record).clone());
                if let Some(record) = selected_record {
                    let pull_request = self
                        .view
                        .pull_requests
                        .iter()
                        .find(|pull_request| pull_request.key == record.key)
                        .cloned();
                    let url = pull_request
                        .as_ref()
                        .map(|pull_request| pull_request.url.clone());
                    let changes = self
                        .view
                        .activity
                        .iter()
                        .filter(|change| change.key == record.key)
                        .cloned()
                        .collect();
                    self.pr_detail = None;
                    self.popup_scroll.set(0);
                    self.popup = Some(ActivityDetail {
                        record,
                        changes,
                        pull_request,
                        url,
                    });
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('r') if self.focus == Pane::Activity => {
                let record = self
                    .visible_activity()
                    .get(self.selected)
                    .map(|record| (*record).clone());
                if let Some(record) = record {
                    inbox.mark_read(&record.id).await?;
                    self.reload(inbox, config).await?;
                }
                Ok(TuiAction::Continue)
            }
            KeyCode::Char('a') if self.focus == Pane::Activity => {
                if self.activity_filter.is_some() {
                    let ids: Vec<String> = self
                        .visible_activity()
                        .iter()
                        .map(|record| record.id.clone())
                        .collect();
                    for id in ids {
                        inbox.mark_read(&id).await?;
                    }
                } else {
                    inbox.mark_all_read().await?;
                }
                self.reload(inbox, config).await?;
                Ok(TuiAction::Continue)
            }
            _ => Ok(TuiAction::Continue),
        }
    }

    fn footer_keys(&self) -> Line<'static> {
        let selected_ignored = self
            .selected_pull_request()
            .is_some_and(|pull_request| self.is_ignored(&pull_request.key));
        let entries: Vec<(&str, &str)> = match self.focus {
            Pane::Watched => vec![
                ("Tab", "activity"),
                ("↑↓", "move"),
                ("enter", "detail"),
                (
                    "x",
                    if selected_ignored {
                        "unignore"
                    } else {
                        "ignore"
                    },
                ),
                ("◀/▶", "view"),
                ("p", "scope"),
                (",", "opts"),
                ("q", "quit"),
            ],
            Pane::Activity => vec![
                ("Tab", "PRs"),
                ("↑↓", "move"),
                ("enter", "detail"),
                ("r", "read"),
                ("a", "all"),
                ("u", "unread"),
                ("p", "scope"),
                (",", "opts"),
                ("q", "quit"),
            ],
        };
        let mut spans = Vec::new();
        for (index, (key, label)) in entries.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled(" · ", theme::dim()));
            }
            spans.push(Span::styled((*key).to_string(), theme::key()));
            spans.push(Span::styled(format!(" {label}"), theme::dim()));
        }
        Line::from(spans)
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        self.viewport.set((area.width, area.height));
        let outer = Block::bordered()
            .title(Line::from(vec![
                Span::styled(" passuff ", theme::bright().add_modifier(Modifier::BOLD)),
                Span::styled("— Inbox ", theme::dim()),
            ]))
            .border_style(theme::border());
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

        let mut header = Vec::new();
        for (index, account) in self.view.accounts.iter().enumerate() {
            if index > 0 {
                header.push(Span::styled("   |   ", theme::dim()));
            }
            let login = account
                .login
                .as_deref()
                .map(|login| format!("@{login}"))
                .unwrap_or_else(|| "not resolved".to_string());
            header.push(Span::styled(account.label.clone(), theme::bright()));
            header.push(Span::raw(" "));
            header.push(Span::styled(login, theme::accent()));
        }
        header.push(Span::styled("   |   ", theme::dim()));
        header.push(Span::styled("watch scope: ", theme::dim()));
        header.push(Span::styled(self.view.scope_label.clone(), theme::bright()));
        frame.render_widget(Paragraph::new(Line::from(header)), chunks[0]);
        let visible = self.visible_activity();
        let unread = visible.iter().filter(|record| !record.read).count();
        let filtering = self.activity_filter.is_some() || self.unread_only;
        let mut spans = vec![
            Span::styled(unread.to_string(), theme::gold_bold()),
            Span::styled(" unread of ", theme::dim()),
            Span::styled(visible.len().to_string(), theme::bright()),
        ];
        if filtering {
            spans.push(Span::styled(
                format!("   (filtered from {})", self.view.activity.len()),
                theme::dim(),
            ));
        }
        spans.push(Span::styled("   |   ", theme::dim()));
        spans.push(Span::styled("Focus: ", theme::dim()));
        spans.push(Span::styled(self.focus_label(), theme::accent_bold()));
        if self.fetching {
            const FRAMES: [&str; 4] = ["◇", "◆", "◈", "◆"];
            spans.push(Span::styled("   |   ", theme::dim()));
            spans.push(Span::styled(
                FRAMES[self.spinner % FRAMES.len()],
                theme::green_bold(),
            ));
            spans.push(Span::styled(" fetching", theme::dim()));
        } else if let Some(remaining) = self.countdown {
            let seconds = if remaining.as_secs() >= 10 {
                format!("{}s", remaining.as_secs())
            } else {
                format!("{:.1}s", remaining.as_secs_f64())
            };
            spans.push(Span::styled(format!("   |   ↻ {seconds}"), theme::dim()));
        }
        if let Some((remaining, limit)) = self.budget {
            spans.push(Span::styled(
                format!("   ·   {remaining}/{limit}"),
                theme::dim(),
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), chunks[1]);

        let watched_mode = if self.show_ignored { "all" } else { "filtered" };
        let direction = if self.descending { "↓" } else { "↑" };
        let watched_block = Block::bordered()
            .title(
                Line::from(format!(
                    " Watched pull requests · {watched_mode} · sort: {} {direction} ",
                    self.sort.label()
                ))
                .style(self.pane_title_style(Pane::Watched)),
            )
            .border_style(self.pane_border_style(Pane::Watched));
        if self.view.pull_requests.is_empty() {
            frame.render_widget(
                Paragraph::new("no watched pull requests").block(watched_block),
                chunks[2],
            );
        } else {
            let (groups, standalone) = stack_groups(&self.view);
            let mut entries: Vec<(&PullRequestView, String, String, Option<String>, bool)> =
                Vec::new();
            let mut has_stack = false;
            let mut stack_width = 1;
            let mut people_width = 1;
            for unit in self.sort_units(ordered_units(&groups, &standalone)) {
                for pull_request in unit {
                    let key = tail_label(&pull_request.key.to_string(), PULL_REQUEST_LABEL_WIDTH);
                    let people = people_label(&pull_request.author, &pull_request.assignees);
                    let stack = pull_request
                        .stack
                        .as_ref()
                        .map(|stack| format!("{}/{}", stack.position, stack.size));
                    if let Some(stack) = stack.as_ref() {
                        has_stack = true;
                        stack_width = stack_width.max(stack.chars().count());
                    }
                    people_width = people_width.max(people.chars().count());
                    let ignored = self.is_ignored(&pull_request.key);
                    entries.push((pull_request, key, people, stack, ignored));
                }
            }
            let mut constraints = vec![
                Constraint::Length(PULL_REQUEST_COLUMN_WIDTH as u16),
                Constraint::Length(6),
                Constraint::Length(12),
            ];
            let mut header = vec!["Pull request", "State", "CI"];
            if has_stack {
                constraints.push(Constraint::Length(stack_width as u16));
                header.push("☰");
            }
            constraints.push(Constraint::Length(people_width as u16));
            header.push("☺");
            constraints.push(Constraint::Min(15));
            header.push("Title");
            let mut rows: Vec<Row> = Vec::new();
            for (pull_request, key, people, stack, ignored) in entries {
                let mut cells = vec![
                    Cell::from(key),
                    Cell::from(state_label(&pull_request.state)),
                    Cell::from(ci_label(&pull_request.ci_state)),
                ];
                if has_stack {
                    cells.push(Cell::from(stack.unwrap_or_default()));
                }
                cells.push(Cell::from(people));
                let title = if ignored {
                    format!("⊘ {}", pull_request.title)
                } else {
                    pull_request.title.clone()
                };
                cells.push(Cell::from(title));
                let style = if ignored {
                    theme::dim()
                } else {
                    Style::default()
                };
                rows.push(Row::new(cells).style(style));
            }
            let table = Table::new(rows, constraints)
                .header(
                    Row::new(header.into_iter().map(centered_cell).collect::<Vec<_>>())
                        .style(theme::accent_bold()),
                )
                .row_highlight_style(self.row_highlight_style())
                .highlight_symbol("▶ ");
            let table = table.block(watched_block);
            let mut state = TableState::default().with_selected(Some(self.pr_selected));
            frame.render_stateful_widget(table, chunks[2], &mut state);
        }

        let activity_title = {
            let mut parts = Vec::new();
            if let Some(filter) = self.activity_filter.as_ref() {
                parts.push(filter.label());
            }
            if self.unread_only {
                parts.push("unread".to_string());
            }
            if parts.is_empty() {
                " Activity ".to_string()
            } else {
                format!(" Activity · {} ", parts.join(" · "))
            }
        };
        let activity_block = Block::bordered()
            .title(Line::from(activity_title).style(self.pane_title_style(Pane::Activity)))
            .border_style(self.pane_border_style(Pane::Activity));
        let visible_activity = self.visible_activity();
        if visible_activity.is_empty() {
            let empty = if self.activity_filter.is_some() || self.unread_only {
                "no activity for this filter"
            } else {
                "no activity yet"
            };
            frame.render_widget(Paragraph::new(empty).block(activity_block), chunks[3]);
        } else {
            let rows: Vec<Row> = visible_activity
                .iter()
                .map(|record| {
                    Row::new(vec![
                        Cell::from(short_record_time(record)),
                        Cell::from(event_line(&record.event)),
                        Cell::from(
                            activity_stack(&self.view, &record.key)
                                .map(|number| format!("stack {number}"))
                                .unwrap_or_default(),
                        ),
                        Cell::from(if record.read { "read" } else { "unread" }),
                    ])
                })
                .collect();
            let table = Table::new(
                rows,
                [
                    Constraint::Length(12),
                    Constraint::Min(40),
                    Constraint::Length(10),
                    Constraint::Length(6),
                ],
            )
            .header(
                Row::new(
                    ["When", "Event", "Stack", "Read"]
                        .into_iter()
                        .map(centered_cell)
                        .collect::<Vec<_>>(),
                )
                .style(theme::accent_bold()),
            )
            .block(activity_block)
            .row_highlight_style(self.row_highlight_style())
            .highlight_symbol("▶ ");
            let mut state = TableState::default().with_selected(Some(self.selected));
            frame.render_stateful_widget(table, chunks[3], &mut state);
        }

        let controls = Block::bordered()
            .title(Line::from(" Controls ").style(theme::accent()))
            .border_style(theme::border());
        let controls_inner = controls.inner(chunks[4]);
        frame.render_widget(controls, chunks[4]);
        match self.status.as_deref() {
            Some(status) => {
                let lines = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
                    .split(controls_inner);
                frame.render_widget(Paragraph::new(self.footer_keys()), lines[0]);
                frame.render_widget(Paragraph::new(status).style(theme::gold()), lines[1]);
            }
            None => {
                frame.render_widget(Paragraph::new(self.footer_keys()), controls_inner);
            }
        }
        if self.popup.is_some() {
            self.render_popup(frame);
        }
        if self.pr_detail.is_some() {
            self.render_pr_popup(frame);
        }
        if self.help.is_some() {
            self.render_help(frame);
        }
        if self.settings.is_some() {
            self.render_settings(frame);
        }
        if self.save_prompt {
            let area = frame.area();
            let width = 54.min(area.width.saturating_sub(2)).max(20);
            let rect = Rect::new((area.width - width) / 2, area.height / 2, width, 3);
            frame.render_widget(Clear, rect);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("Save sort and filter settings?  ", theme::bright()),
                    Span::styled("y", theme::key()),
                    Span::styled(" yes  ", theme::dim()),
                    Span::styled("n", theme::key()),
                    Span::styled(" no  ", theme::dim()),
                    Span::styled("Esc", theme::key()),
                    Span::styled(" cancel", theme::dim()),
                ]))
                .block(
                    Block::bordered()
                        .title(Line::from(" Leave ").style(theme::accent()))
                        .border_style(theme::border()),
                ),
                rect,
            );
        }
    }

    fn render_settings(&self, frame: &mut Frame) {
        let Some(selected) = self.settings else {
            return;
        };
        let area = frame.area();
        let width = 56.min(area.width.saturating_sub(2)).max(24);
        let height = 5;
        let rect = Rect::new(
            (area.width.saturating_sub(width)) / 2,
            (area.height.saturating_sub(height)) / 2,
            width,
            height,
        );
        let block = Block::bordered()
            .title(Line::from(" Settings ").style(theme::accent_bold()))
            .border_style(theme::border());
        let inner = block.inner(rect);
        frame.render_widget(Clear, rect);
        frame.render_widget(block, rect);
        let rows = vec![
            settings_row(
                "Desktop notifications",
                self.notifications.desktop(),
                selected == 0,
            ),
            settings_row("Sound", self.notifications.sound(), selected == 1),
        ];
        let chunks =
            Layout::vertical([Constraint::Length(rows.len() as u16), Constraint::Length(1)])
                .split(inner);
        frame.render_widget(Paragraph::new(rows), chunks[0]);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("space", theme::key()),
                Span::styled(" toggle   ", theme::dim()),
                Span::styled("esc/q", theme::key()),
                Span::styled(" close", theme::dim()),
            ])),
            chunks[1],
        );
    }

    fn render_help(&self, frame: &mut Frame) {
        let Some(pane) = self.help else {
            return;
        };
        let area = frame.area();
        let width = (area.width * 70 / 100)
            .min(area.width.saturating_sub(2))
            .max(20);
        let heading_style = theme::accent_bold();
        let mut lines = vec![
            Line::from(Span::styled(
                match pane {
                    Pane::Watched => "Pull request pane",
                    Pane::Activity => "Activity pane",
                },
                heading_style,
            )),
            Line::from(""),
        ];
        let entries: &[(&str, &str)] = match pane {
            Pane::Watched => &[
                ("Tab", "switch pane"),
                ("↑↓", "move selection"),
                ("PgUp/PgDn", "move by ten"),
                ("Home/End", "first / last row"),
                ("Enter", "pull request detail"),
                ("x", "ignore / unignore"),
                ("left/right", "filtered / all"),
                ("s / S", "sort field / reverse"),
                ("f / F", "filter activities to PR / stack"),
                ("u", "unread activities only"),
                ("c / C", "clear filters / reset view"),
                ("p", "watch scope"),
                (",", "settings"),
                ("h / F1", "this help"),
                ("q", "quit"),
            ],
            Pane::Activity => &[
                ("Tab", "switch pane"),
                ("↑↓", "move selection"),
                ("PgUp/PgDn", "move by ten"),
                ("Home/End", "first / last row"),
                ("Enter", "activity detail"),
                ("r", "mark selected read"),
                ("a", "mark visible read"),
                ("f / F", "filter to PR / stack"),
                ("u", "unread activities only"),
                ("c / C", "clear filters / reset view"),
                (",", "settings"),
                ("h / F1", "this help"),
                ("q", "quit"),
            ],
        };
        for (keys, description) in entries {
            lines.push(Line::from(vec![
                Span::styled(format!("{keys:<12} "), theme::key()),
                Span::styled((*description).to_string(), theme::dim()),
            ]));
        }
        let inner_width = usize::from(width.saturating_sub(2)).max(1);
        let rows: usize = lines
            .iter()
            .map(|line| {
                let length: usize = line
                    .spans
                    .iter()
                    .map(|span| span.content.chars().count())
                    .sum();
                length.max(1).div_ceil(inner_width)
            })
            .sum();
        let height = (rows as u16 + 3)
            .max(8)
            .min(area.height.saturating_sub(2).max(8));
        let rect = Rect::new(
            (area.width - width) / 2,
            (area.height - height) / 2,
            width,
            height,
        );
        let block = Block::bordered()
            .title(Line::from(" Keys ").style(theme::accent_bold()))
            .border_style(theme::border());
        frame.render_widget(Clear, rect);
        frame.render_widget(Paragraph::new(lines).block(block), rect);
    }

    fn render_popup(&self, frame: &mut Frame) {
        let Some(detail) = self.popup.as_ref() else {
            return;
        };
        let area = frame.area();
        let width = activity_popup_width(area);
        let inner_width = usize::from(width.saturating_sub(2)).max(1);
        let lines = activity_detail_lines(detail, inner_width);
        let height = activity_popup_height(area, lines.len());
        let rect = Rect::new(
            (area.width.saturating_sub(width)) / 2,
            (area.height.saturating_sub(height)) / 2,
            width,
            height,
        );
        let block = Block::bordered()
            .title(Line::from(" Activity ").style(theme::accent_bold()))
            .border_style(theme::border());
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
        let max_scroll = (lines.len() as u16).saturating_sub(content.height);
        let scroll = self.popup_scroll.get().min(max_scroll);
        self.popup_scroll.set(scroll);
        frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), content);
        frame.render_widget(Paragraph::new(popup_hint()), hint_area);
    }

    fn render_pr_popup(&self, frame: &mut Frame) {
        let Some(pull_request) = self.pr_detail.as_ref() else {
            return;
        };
        let area = frame.area();
        let width = pr_popup_width(area);
        let inner_width = usize::from(width.saturating_sub(2)).max(1);
        let lines = pr_detail_lines(pull_request, inner_width);
        let height = pr_popup_height(area, lines.len());
        let rect = Rect::new(
            (area.width.saturating_sub(width)) / 2,
            (area.height.saturating_sub(height)) / 2,
            width,
            height,
        );
        let block = Block::bordered()
            .title(Line::from(vec![
                Span::styled(" PR", theme::bright().add_modifier(Modifier::BOLD)),
                Span::styled(" - ", theme::dim()),
                Span::styled(pull_request.key.to_string(), theme::accent_bold()),
                Span::styled(" ", theme::dim()),
            ]))
            .border_style(theme::border());
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
        let max_scroll = (lines.len() as u16).saturating_sub(content.height);
        let scroll = self.pr_detail_scroll.get().min(max_scroll);
        self.pr_detail_scroll.set(scroll);
        frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), content);
        frame.render_widget(Paragraph::new(popup_hint()), hint_area);
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
            focus: Default::default(),
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
            occurred_at: None,
            event: ActivityEvent::Comment {
                author: "alice".to_string(),
                body: "hi".to_string(),
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
            author: "octo-personal".to_string(),
            created_at: "2026-11-02T11:00:00Z".to_string(),
            description: String::new(),
            assignees: Vec::new(),
            is_draft: false,
            updated_at: "2026-11-02T11:00:00Z".to_string(),
            ignored: false,
            involvements: Vec::new(),
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
            author: "octo-personal".to_string(),
            created_at: "2026-11-02T11:00:00Z".to_string(),
            description: String::new(),
            assignees: Vec::new(),
            is_draft: false,
            updated_at: "2026-11-02T11:00:00Z".to_string(),
            ignored: false,
            involvements: Vec::new(),
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
            record: record.clone(),
            changes: vec![record],
            pull_request: Some(pull_request),
            url: None,
        });
        let snapshot = app.snapshot();
        assert!(
            snapshot.contains("before writing"),
            "long title was clipped: {snapshot}"
        );
        assert!(snapshot.contains("open in browser"));
        assert!(snapshot.contains("Esc/q close"));
        assert!(snapshot.contains("Activity api#42"), "{snapshot}");
        assert!(!snapshot.contains("Activity a1"), "{snapshot}");
        let lines: Vec<&str> = snapshot.lines().collect();
        let hint_row = lines
            .iter()
            .position(|line| line.contains("open in browser"))
            .expect("hint line");
        assert!(
            lines[hint_row + 1].contains("└"),
            "hint is not on the bottom line: {}",
            lines[hint_row + 1]
        );
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| app.render(frame))
            .expect("draw popup");
        let buffer = terminal.backend().buffer();
        let accent_title = (0..30).any(|y| {
            (0..100).any(|x| {
                let cell = &buffer[(x, y)];
                cell.fg == theme::ACCENT
                    && cell.modifier.contains(Modifier::BOLD)
                    && cell.symbol() == "A"
            })
        });
        assert!(accent_title, "the popup title is not accent");
    }

    #[test]
    fn the_most_recently_updated_rows_come_first() {
        let mut older = stacked("acme/api#1", 7, 2, 1, "main");
        older.updated_at = "2026-11-02T10:00:00Z".to_string();
        let mut older_sibling = stacked("acme/api#2", 7, 2, 2, "main");
        older_sibling.updated_at = "2026-11-02T10:00:00Z".to_string();
        let mut newest = standalone("acme/api#3");
        newest.updated_at = "2026-11-03T12:00:00Z".to_string();
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![older, older_sibling, newest];
        let ordered = app.ordered_pull_requests();
        assert_eq!(ordered[0].key.to_string(), "acme/api#3");
        assert_eq!(ordered[1].key.to_string(), "acme/api#1");
        let snapshot = app.snapshot();
        assert!(
            snapshot.find("acme/api#3").unwrap() < snapshot.find("acme/api#1").unwrap(),
            "newest row is not first: {snapshot}"
        );
    }

    fn record_on(key: &str, id: &str, read: bool) -> ActivityRecord {
        ActivityRecord {
            id: id.to_string(),
            key: pr_key(key),
            observed_at: "2026-11-02T10:00:00Z".to_string(),
            occurred_at: None,
            event: ActivityEvent::Comment {
                author: "alice".to_string(),
                body: "hi".to_string(),
            },
            read,
        }
    }

    #[test]
    fn older_preferences_enable_notifications() {
        let preferences: TuiPreferences = serde_json::from_str(
            r#"{"sort":"updated","descending":true,"filter":null,"unread_only":false,"show_ignored":false}"#,
        )
        .expect("legacy preferences");
        assert!(preferences.desktop);
        assert!(preferences.sound);
    }

    #[tokio::test]
    async fn q_closes_every_dialog() {
        let mut app = TuiApp::new(view(Vec::new()));
        let inbox = FakeInbox::new(Vec::new());
        let record = record("a1", false);
        app.popup = Some(ActivityDetail {
            record: record.clone(),
            changes: vec![record],
            pull_request: None,
            url: None,
        });
        app.handle_key(key(KeyCode::Char('q')), &inbox, &core_config())
            .await
            .expect("activity popup");
        assert!(app.popup.is_none());

        app.pr_detail = Some(standalone("acme/api#42"));
        app.handle_key(key(KeyCode::Char('q')), &inbox, &core_config())
            .await
            .expect("pull request dialog");
        assert!(app.pr_detail.is_none());

        app.help = Some(Pane::Watched);
        app.handle_key(key(KeyCode::Char('q')), &inbox, &core_config())
            .await
            .expect("help");
        assert!(app.help.is_none());

        app.settings = Some(0);
        app.handle_key(key(KeyCode::Char('q')), &inbox, &core_config())
            .await
            .expect("settings");
        assert!(app.settings.is_none());

        app.show_save_prompt();
        app.handle_key(key(KeyCode::Char('q')), &inbox, &core_config())
            .await
            .expect("save prompt");
        assert!(!app.save_prompt);
    }

    #[tokio::test]
    async fn the_settings_dialog_toggles_notifications() {
        let mut app = TuiApp::new(view(Vec::new()));
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Char(',')), &inbox, &core_config())
            .await
            .expect("open settings");
        let frame = app.snapshot();
        assert!(frame.contains("Desktop notifications"), "{frame}");
        assert!(frame.contains("Sound"), "{frame}");
        assert!(frame.contains("Settings"), "{frame}");
        assert!(app.notification_switches().desktop());
        assert!(app.notification_switches().sound());

        app.handle_key(key(KeyCode::Char(' ')), &inbox, &core_config())
            .await
            .expect("toggle desktop");
        assert!(!app.notification_switches().desktop());
        assert!(app.notification_switches().sound());

        app.handle_key(key(KeyCode::Char('j')), &inbox, &core_config())
            .await
            .expect("select sound");
        app.handle_key(key(KeyCode::Char(' ')), &inbox, &core_config())
            .await
            .expect("toggle sound");
        assert!(!app.notification_switches().sound());
        let frame = app.snapshot();
        assert!(frame.contains("off"), "{frame}");
        let preferences = app.preferences();
        assert!(!preferences.desktop);
        assert!(!preferences.sound);

        app.handle_key(key(KeyCode::Esc), &inbox, &core_config())
            .await
            .expect("close settings");
        let frame = app.snapshot();
        assert!(!frame.contains("Desktop notifications"), "{frame}");
    }

    #[test]
    fn snapshot_shows_the_next_update_countdown() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.set_countdown(Some(std::time::Duration::from_secs(42)));
        let frame = app.snapshot();
        assert!(frame.contains("↻ 42s"), "{frame}");
        app.set_countdown(Some(std::time::Duration::from_millis(1_440)));
        app.set_budget(4_840, 5_000);
        let frame = app.snapshot();
        assert!(frame.contains("↻ 1.4s"), "{frame}");
        assert!(frame.contains("4840/5000"), "{frame}");
    }

    #[test]
    fn snapshot_shows_a_spinner_while_fetching() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.set_fetching(true);
        let frame = app.snapshot();
        assert!(frame.contains("fetching"), "{frame}");
        assert!(frame.contains('◆') || frame.contains('◇'), "{frame}");
    }

    #[test]
    fn snapshot_shows_the_empty_inbox() {
        let app = TuiApp::new(view(Vec::new()));
        let frame = app.snapshot();
        assert!(frame.contains("no activity yet"), "{frame}");
        assert!(frame.contains("0 unread of 0"), "{frame}");
    }

    #[test]
    fn the_inbox_title_has_no_emoji() {
        let app = TuiApp::new(view(Vec::new()));
        let frame = app.snapshot();
        assert!(frame.contains(" passuff — Inbox "), "{frame}");
        assert!(!frame.contains('☞'), "{frame}");
    }

    #[test]
    fn snapshot_marks_the_selected_activity() {
        let app = TuiApp::new(view(vec![
            record_on("acme/api#43", "a2", false),
            record_on("acme/api#42", "a1", true),
        ]));
        let frame = app.snapshot();
        let selected = frame
            .lines()
            .find(|line| line.contains("▶"))
            .expect("selected row");
        assert_eq!(frame.matches("▶").count(), 1, "{frame}");
        assert_eq!(selected, selected, "{frame}");
        assert!(frame.contains("comment by @alice"), "{frame}");
    }

    fn pane_border(app: &TuiApp, title: &str) -> (Color, Modifier) {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal.draw(|frame| app.render(frame)).expect("draw");
        let buffer = terminal.backend().buffer();
        let y = (0..30)
            .find(|y| {
                (0..100)
                    .map(|x| {
                        buffer
                            .cell((x, *y))
                            .map(|cell| cell.symbol())
                            .unwrap_or(" ")
                    })
                    .collect::<String>()
                    .contains(title)
            })
            .expect("pane title");
        (buffer[(1, y)].fg, buffer[(1, y)].modifier)
    }

    #[tokio::test]
    async fn the_green_frame_follows_the_focused_pane() {
        let mut view = view(vec![record_on("acme/api#42", "a1", false)]);
        view.pull_requests = vec![standalone("acme/api#42")];
        let mut app = TuiApp::new(view);
        let inbox = FakeInbox::new(Vec::new());

        assert_eq!(pane_border(&app, " Activity ").0, Color::Green);
        let unfocused = pane_border(&app, " Watched pull requests");
        assert_eq!(unfocused.0, Color::Reset);
        assert!(unfocused.1.contains(Modifier::DIM));

        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");

        assert_eq!(pane_border(&app, " Watched pull requests").0, Color::Green);
        let unfocused = pane_border(&app, " Activity ");
        assert_eq!(unfocused.0, Color::Reset);
        assert!(unfocused.1.contains(Modifier::DIM));
    }

    #[test]
    fn the_tui_uses_the_shared_watn_palette() {
        let mut view = view(vec![record("a1", false)]);
        view.pull_requests = vec![standalone("acme/api#42")];
        let app = TuiApp::new(view);
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| app.render(frame)).expect("draw");
        let buffer = terminal.backend().buffer();
        let find = |needle: &str| -> (u16, u16) {
            for y in 0..30 {
                let row: String = (0..100).map(|x| buffer[(x, y)].symbol()).collect();
                if let Some(byte_offset) = row.find(needle) {
                    let x = row[..byte_offset].chars().count();
                    return (x as u16, y);
                }
            }
            panic!("missing {needle:?}");
        };

        let (x, y) = find("Tab");
        assert_eq!(buffer[(x, y)].fg, theme::ACCENT);
        assert!(buffer[(x, y)].modifier.contains(Modifier::BOLD));

        let (x, y) = find("▶");
        assert_eq!(buffer[(x, y)].fg, theme::GOLD);
        assert_eq!(buffer[(x, y)].bg, Color::Reset);
        assert!(buffer[(x, y)].modifier.contains(Modifier::BOLD));

        let (x, y) = find("1 unread of 1");
        assert_eq!(buffer[(x, y)].fg, theme::GOLD);
        assert!(buffer[(x, y)].modifier.contains(Modifier::BOLD));

        let (x, y) = find("Pull request");
        assert_eq!(buffer[(x, y)].fg, theme::ACCENT);
        assert!(buffer[(x, y)].modifier.contains(Modifier::BOLD));
    }

    #[tokio::test]
    async fn the_activity_arrow_stays_while_the_pull_request_pane_has_focus() {
        let mut view = view(vec![record_on("acme/api#42", "a1", false)]);
        view.pull_requests = vec![standalone("acme/api#42")];
        let mut app = TuiApp::new(view);
        let inbox = FakeInbox::new(Vec::new());
        let activity_row = |frame: &str| {
            frame
                .lines()
                .find(|line| line.contains("comment by @alice"))
                .expect("activity row")
                .to_string()
        };

        assert!(activity_row(&app.snapshot()).contains('▶'));
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        let frame = app.snapshot();
        assert!(activity_row(&frame).contains('▶'), "{frame}");
        let pr_row = frame
            .lines()
            .find(|line| line.contains("acme/api#42"))
            .expect("pull request row");
        assert!(pr_row.contains('▶'), "{frame}");
        assert!(frame.contains("|   Focus: PRs"), "{frame}");
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
    async fn home_and_end_jump_to_the_ends_of_the_watched_list() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![
            standalone("acme/api#1"),
            standalone("acme/api#2"),
            standalone("acme/api#3"),
        ];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::End), &inbox, &core_config())
            .await
            .expect("end");
        assert_eq!(app.pr_selected, 2);
        assert_eq!(
            app.selected_pull_request()
                .map(|pull_request| pull_request.key.to_string()),
            Some("acme/api#3".to_string())
        );
        app.handle_key(key(KeyCode::Home), &inbox, &core_config())
            .await
            .expect("home");
        assert_eq!(app.pr_selected, 0);
        assert_eq!(
            app.selected_pull_request()
                .map(|pull_request| pull_request.key.to_string()),
            Some("acme/api#1".to_string())
        );
    }

    #[tokio::test]
    async fn home_and_end_jump_to_the_ends_of_the_activity_list() {
        let mut app = TuiApp::new(view(vec![
            record_on("acme/api#42", "a1", false),
            record_on("acme/api#42", "a2", false),
            record_on("acme/api#42", "a3", false),
        ]));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::End), &inbox, &core_config())
            .await
            .expect("end");
        assert_eq!(app.selected, 2);
        app.handle_key(key(KeyCode::Home), &inbox, &core_config())
            .await
            .expect("home");
        assert_eq!(app.selected, 0);
    }

    #[tokio::test]
    async fn the_watched_table_centers_its_headers() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        let frame = app.snapshot();
        let header = frame
            .lines()
            .find(|line| line.contains("Pull request"))
            .expect("header row");
        assert!(header.contains("  Pull request"), "{frame}");
        assert!(header.contains("  CI "), "{frame}");
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

    #[test]
    fn preferences_round_trip_through_apply() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.sort = SortMode::Name;
        app.descending = false;
        app.activity_filter = Some(ActivityFilter::PullRequest(pr_key("acme/api#42")));
        app.unread_only = true;
        app.show_ignored = true;
        let preferences = app.preferences();
        let mut restored = TuiApp::new(view(Vec::new()));
        restored.apply_preferences(&preferences);
        assert_eq!(restored.preferences(), preferences);
        assert_eq!(restored.sort_mode(), SortMode::Name);
        assert!(!restored.descending());
    }

    #[tokio::test]
    async fn sorting_by_name_orders_the_watched_rows() {
        let mut app = TuiApp::new(view(Vec::new()));
        let mut zebra = standalone("acme/api#1");
        zebra.title = "Zebra".to_string();
        let mut alpha = standalone("acme/api#2");
        alpha.title = "Alpha".to_string();
        app.view.pull_requests = vec![zebra, alpha];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Char('s')), &inbox, &core_config())
            .await
            .expect("sort");
        let frame = app.snapshot();
        assert!(
            frame.find("Alpha").unwrap() < frame.find("Zebra").unwrap(),
            "{frame}"
        );
        assert!(frame.contains("sort: name ↑"), "{frame}");
    }

    #[tokio::test]
    async fn f_filters_the_activity_to_the_selected_pull_request() {
        let mut app = TuiApp::new(view(vec![
            record_on("acme/api#42", "a1", false),
            record_on("acme/api#43", "a2", false),
        ]));
        app.view.pull_requests = vec![standalone("acme/api#42"), standalone("acme/api#43")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Char('f')), &inbox, &core_config())
            .await
            .expect("filter");
        let frame = app.snapshot();
        assert!(frame.contains("Activity · acme/api#42"), "{frame}");
        assert!(frame.contains("1 unread of 1"), "{frame}");
        assert!(frame.contains("(filtered from 2)"), "{frame}");
        app.handle_key(key(KeyCode::Char('c')), &inbox, &core_config())
            .await
            .expect("clear");
        assert!(!app.snapshot().contains("filtered from"));
    }

    #[tokio::test]
    async fn f_filters_the_activity_to_the_whole_stack() {
        let mut app = TuiApp::new(view(vec![
            record_on("acme/api#42", "a1", false),
            record_on("acme/api#43", "a2", false),
        ]));
        app.view.pull_requests = vec![
            stacked("acme/api#42", 7, 2, 1, "main"),
            stacked("acme/api#43", 7, 2, 2, "main"),
        ];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Char('F')), &inbox, &core_config())
            .await
            .expect("stack filter");
        let frame = app.snapshot();
        assert!(frame.contains("Activity · acme/api stack 7"), "{frame}");
        assert!(frame.contains("2 unread of 2"), "{frame}");
    }

    #[tokio::test]
    async fn u_filters_the_activity_to_unread_records() {
        let mut app = TuiApp::new(view(vec![
            record_on("acme/api#42", "a1", false),
            record_on("acme/api#42", "a2", true),
        ]));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Char('u')), &inbox, &core_config())
            .await
            .expect("unread filter");
        let frame = app.snapshot();
        assert!(frame.contains("Activity · unread"), "{frame}");
        assert!(frame.contains("1 unread of 1"), "{frame}");
        assert!(frame.contains("(filtered from 2)"), "{frame}");
        app.handle_key(key(KeyCode::Char('c')), &inbox, &core_config())
            .await
            .expect("clear");
        let frame = app.snapshot();
        assert!(frame.contains("1 unread of 2"), "{frame}");
        assert!(!frame.contains("filtered from"), "{frame}");
    }

    #[test]
    fn tail_label_keeps_the_end_of_a_long_key() {
        let key = "very-long-organization/very-long-repository#1234";
        assert_eq!(tail_label(key, 16), "…repository#1234");
        assert_eq!(tail_label("acme/api#42", 16), "acme/api#42");
    }

    fn rendered(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn description_rendering_keeps_paragraphs_and_wraps() {
        let lines = render_description("Summary\nTwo changes\n\n\nFirst\nSecond", 80);
        assert_eq!(
            rendered(&lines),
            vec!["Summary", "Two changes", "", "First", "Second"]
        );
        let lines = render_description("one long line that wraps", 8);
        assert_eq!(rendered(&lines), vec!["one long", "line", "that", "wraps"]);
        let lines = render_description("\n\nlead\n\n", 80);
        assert_eq!(rendered(&lines), vec!["lead"]);
    }

    #[test]
    fn markdown_headings_and_lists_lose_their_markers() {
        let lines = render_description("## Summary\n- one\n- two", 40);
        assert_eq!(rendered(&lines), vec!["Summary", "• one", "• two"]);
        assert_eq!(lines[0].spans[0].style.fg, Some(theme::GREEN));
    }

    #[test]
    fn markdown_inline_markers_and_links_render_as_text() {
        let lines = render_description(
            "**bold** and `code` and [link](https://example.com) and ~~old~~",
            60,
        );
        assert_eq!(rendered(&lines), vec!["bold and code and link and old"]);
    }

    #[test]
    fn markdown_code_fences_render_without_fences() {
        let lines = render_description("```rust\nlet x = 1;\n```", 40);
        assert_eq!(rendered(&lines), vec!["let x = 1;"]);
        assert!(lines[0].spans[0].style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn markdown_bullets_wrap_with_a_hanging_indent() {
        let lines = render_description("- alpha beta gamma delta", 14);
        assert_eq!(rendered(&lines), vec!["• alpha beta", "  gamma delta"]);
    }

    #[test]
    fn markdown_numbered_items_keep_their_number() {
        let lines = render_description("1. first\n2. second", 20);
        assert_eq!(rendered(&lines), vec!["1. first", "2. second"]);
    }

    #[test]
    fn people_label_collapses_creator_and_assignees() {
        assert_eq!(people_label("alice", &[]), "@alice");
        assert_eq!(people_label("alice", &["alice".to_string()]), "@alice");
        assert_eq!(
            people_label("alice", &["Alice".to_string(), "bob".to_string()]),
            "@alice → @bob"
        );
        assert_eq!(
            people_label("alice", &["bob".to_string(), "bob".to_string()]),
            "@alice → @bob"
        );
        assert_eq!(
            people_label("alice", &["bob".to_string(), "carol".to_string()]),
            "@alice → @bob, @carol"
        );
    }

    #[tokio::test]
    async fn the_watched_table_shows_the_creator_and_assignees() {
        let mut app = TuiApp::new(view(vec![record_on("acme/api#42", "a1", false)]));
        let mut pull_request = standalone("acme/api#42");
        pull_request.assignees = vec!["alice".to_string()];
        app.view.pull_requests = vec![pull_request];
        let frame = app.snapshot();
        assert!(frame.contains("☺"), "{frame}");
        assert!(frame.contains("@octo-personal → @alice"), "{frame}");
    }

    #[tokio::test]
    async fn a_long_pull_request_key_keeps_its_number_visible() {
        let key = "very-long-organization/very-long-repository#1234";
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![standalone(key)];
        let frame = app.snapshot();
        assert!(frame.contains("…repository#1234"), "{frame}");
        assert!(frame.contains("▶ …repository#1234"), "{frame}");
    }

    #[tokio::test]
    async fn the_stack_symbol_appears_only_when_a_visible_pull_request_is_stacked() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        let frame = app.snapshot();
        assert!(frame.contains("☺"), "{frame}");
        assert!(!frame.contains("☰"), "{frame}");
        app.view.pull_requests = vec![stacked("acme/api#42", 7, 2, 1, "main")];
        let frame = app.snapshot();
        assert!(frame.contains("☰"), "{frame}");
        assert!(frame.contains("1/2"), "{frame}");
    }

    #[tokio::test]
    async fn h_opens_the_keys_for_the_focused_pane() {
        let mut app = TuiApp::new(view(Vec::new()));
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Char('h')), &inbox, &core_config())
            .await
            .expect("help");
        let frame = app.snapshot();
        assert!(frame.contains("Pull request pane"), "{frame}");
        assert!(frame.contains("sort field / reverse"), "{frame}");
        app.handle_key(key(KeyCode::Esc), &inbox, &core_config())
            .await
            .expect("close help");
        assert!(!app.snapshot().contains("Pull request pane"));
    }

    #[tokio::test]
    async fn x_in_the_watched_pane_returns_the_ignore_action() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        let action = app
            .handle_key(key(KeyCode::Char('x')), &inbox, &core_config())
            .await
            .expect("x");
        assert_eq!(action, TuiAction::ToggleIgnore(pr_key("acme/api#42")));
    }

    #[tokio::test]
    async fn an_ignored_pull_request_stays_visible_with_the_unignore_control() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        app.set_ignored_pull_requests(&["acme/api#42".to_string()]);
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        let frame = app.snapshot();
        assert!(
            frame.contains("⊘"),
            "the ignored marker is missing: {frame}"
        );
        assert!(frame.contains("x unignore"), "{frame}");
    }

    #[tokio::test]
    async fn left_and_right_toggle_the_ignored_view() {
        let mut app = TuiApp::new(view(Vec::new()));
        app.view.pull_requests = vec![standalone("acme/api#42")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        let action = app
            .handle_key(key(KeyCode::Right), &inbox, &core_config())
            .await
            .expect("right");
        assert_eq!(action, TuiAction::ToggleIgnoredView);
        assert!(app.show_ignored());
        app.handle_key(key(KeyCode::Left), &inbox, &core_config())
            .await
            .expect("left");
        assert!(!app.show_ignored());
    }

    #[tokio::test]
    async fn reload_moves_the_pull_request_cursor_with_the_selected_activity() {
        let mut app = TuiApp::new(view(vec![record_on("acme/api#42", "a1", false)]));
        app.view.pull_requests = vec![standalone("acme/api#42"), standalone("acme/api#43")];
        let mut next = view(vec![
            record_on("acme/api#43", "a2", false),
            record_on("acme/api#42", "a1", false),
        ]);
        next.pull_requests = vec![standalone("acme/api#42"), standalone("acme/api#43")];
        let inbox = FakeInbox::new(vec![next]);
        app.reload(&inbox, &core_config()).await.expect("reload");
        assert_eq!(
            app.selected_pull_request().map(|pr| pr.key.to_string()),
            Some("acme/api#43".to_string())
        );
    }

    #[tokio::test]
    async fn reload_does_not_move_the_cursor_while_the_pull_request_pane_is_focused() {
        let mut app = TuiApp::new(view(vec![record_on("acme/api#42", "a1", false)]));
        app.view.pull_requests = vec![standalone("acme/api#42"), standalone("acme/api#43")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.selected_pr = Some(pr_key("acme/api#42"));
        app.pr_selected = 0;
        let mut next = view(vec![
            record_on("acme/api#43", "a2", false),
            record_on("acme/api#42", "a1", false),
        ]);
        next.pull_requests = vec![standalone("acme/api#42"), standalone("acme/api#43")];
        let inbox = FakeInbox::new(vec![next]);
        app.reload(&inbox, &core_config()).await.expect("reload");
        assert_eq!(
            app.selected_pull_request().map(|pr| pr.key.to_string()),
            Some("acme/api#42".to_string())
        );
    }

    #[tokio::test]
    async fn moving_the_activity_selection_moves_the_pull_request_cursor() {
        let mut app = TuiApp::new(view(vec![
            record_on("acme/api#43", "a2", false),
            record_on("acme/api#42", "a1", true),
        ]));
        app.view.pull_requests = vec![standalone("acme/api#42"), standalone("acme/api#43")];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Char('j')), &inbox, &core_config())
            .await
            .expect("j");
        assert_eq!(
            app.selected_pull_request().map(|pr| pr.key.to_string()),
            Some("acme/api#42".to_string())
        );
    }

    #[tokio::test]
    async fn page_keys_move_the_watched_selection_by_ten() {
        let mut app = TuiApp::new(view(Vec::new()));
        for number in 1..=25 {
            app.view
                .pull_requests
                .push(standalone(&format!("acme/api#{number}")));
        }
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::PageDown), &inbox, &core_config())
            .await
            .expect("page down");
        assert_eq!(app.pr_selected, 10);
        app.handle_key(key(KeyCode::PageUp), &inbox, &core_config())
            .await
            .expect("page up");
        assert_eq!(app.pr_selected, 0);
    }

    #[tokio::test]
    async fn the_pull_request_detail_shows_metadata_and_closes() {
        let mut app = TuiApp::new(view(Vec::new()));
        let mut pull_request = standalone("acme/api#42");
        pull_request.description = "Adds the live inbox".to_string();
        pull_request.assignees = vec!["alice".to_string()];
        app.view.pull_requests = vec![pull_request];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Enter), &inbox, &core_config())
            .await
            .expect("enter");
        let frame = app.snapshot();
        assert!(frame.contains("Creator: @octo-personal"), "{frame}");
        assert!(frame.contains("Assignee: @alice"), "{frame}");
        assert!(frame.contains("Status: open"), "{frame}");
        assert!(frame.contains("CI: passing"), "{frame}");
        assert!(frame.contains("Adds the live inbox"), "{frame}");
        let action = app
            .handle_key(key(KeyCode::Enter), &inbox, &core_config())
            .await
            .expect("open");
        assert_eq!(
            action,
            TuiAction::OpenUrl("https://github.com/acme/api#42".to_string())
        );
        app.handle_key(key(KeyCode::Esc), &inbox, &core_config())
            .await
            .expect("esc");
        assert!(!app.snapshot().contains("Created by @octo-personal"));
    }

    #[tokio::test]
    async fn the_pull_request_description_keeps_paragraphs_apart() {
        let mut app = TuiApp::new(view(Vec::new()));
        let mut pull_request = standalone("acme/api#42");
        pull_request.description =
            "Summary\nFirst line\nSecond line\n\nSeparate paragraph".to_string();
        app.view.pull_requests = vec![pull_request];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Enter), &inbox, &core_config())
            .await
            .expect("enter");
        let frame = app.snapshot();
        let lines: Vec<&str> = frame.lines().collect();
        let summary = lines
            .iter()
            .find(|line| line.contains("Summary"))
            .expect("summary line");
        assert!(!summary.contains("First line"), "{frame}");
        let second = lines
            .iter()
            .position(|line| line.contains("Second line"))
            .expect("second line");
        assert!(
            lines[second + 1].chars().all(|c| c == '│' || c == ' '),
            "paragraph break missing: {frame}"
        );
    }

    #[tokio::test]
    async fn a_long_pull_request_description_scrolls_to_the_end() {
        let mut app = TuiApp::new(view(Vec::new()));
        let mut pull_request = standalone("acme/api#42");
        pull_request.description = (1..=40)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        app.view.pull_requests = vec![pull_request];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Enter), &inbox, &core_config())
            .await
            .expect("enter");
        let frame = app.snapshot();
        assert!(frame.contains("PR - acme/api#42"), "{frame}");
        assert!(frame.contains("line 1"), "{frame}");
        assert!(!frame.contains("line 40"), "{frame}");
        app.handle_key(key(KeyCode::End), &inbox, &core_config())
            .await
            .expect("end");
        let frame = app.snapshot();
        assert!(frame.contains("line 40"), "{frame}");
        app.handle_key(key(KeyCode::Home), &inbox, &core_config())
            .await
            .expect("home");
        let frame = app.snapshot();
        assert!(frame.contains("line 1"), "{frame}");
        assert!(!frame.contains("line 40"), "{frame}");
    }

    #[tokio::test]
    async fn the_pull_request_detail_recedes_metadata_and_indents_the_description() {
        let mut app = TuiApp::new(view(Vec::new()));
        let mut pull_request = standalone("acme/api#42");
        pull_request.description = "Summary\n\nThe body.".to_string();
        app.view.pull_requests = vec![pull_request];
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Tab), &inbox, &core_config())
            .await
            .expect("tab");
        app.handle_key(key(KeyCode::Enter), &inbox, &core_config())
            .await
            .expect("enter");
        let frame = app.snapshot();
        assert!(
            frame.contains("Status: open   CI: passing"),
            "metadata is not compact: {frame}"
        );
        assert!(
            frame.contains("Creator: @octo-personal (11-02 11:00)   Assignee: none"),
            "metadata is not compact: {frame}"
        );
        assert!(
            frame.lines().any(|line| line.contains("│  Summary")),
            "the description is not indented: {frame}"
        );
        assert!(
            frame.lines().any(|line| line.contains('─')),
            "the separator is missing: {frame}"
        );

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| app.render(frame)).expect("draw");
        let buffer = terminal.backend().buffer();
        let mut label_is_accent = false;
        let mut value_is_dim = false;
        for y in 0..30 {
            let row: String = (0..100).map(|x| buffer[(x, y)].symbol()).collect();
            if let Some(byte_offset) = row.find("Status: ") {
                let offset = row[..byte_offset].chars().count();
                let value = offset + "Status: ".chars().count();
                label_is_accent = buffer[(offset as u16, y)].fg == theme::ACCENT;
                value_is_dim = buffer[(value as u16, y)].modifier.contains(Modifier::DIM);
            }
        }
        assert!(label_is_accent, "the metadata label is not accent");
        assert!(value_is_dim, "the metadata value is not dim");
    }

    #[test]
    fn the_activity_detail_shows_the_comment_body() {
        let mut record = record("a1", false);
        record.event = ActivityEvent::Comment {
            author: "alice".to_string(),
            body: "## Summary\n\nThe full body mentions the zebra marker.".to_string(),
        };
        let mut app = TuiApp::new(view(vec![record.clone()]));
        app.popup = Some(ActivityDetail {
            record: record.clone(),
            changes: vec![record],
            pull_request: None,
            url: None,
        });
        let frame = app.snapshot();
        assert!(
            frame.contains("The full body mentions the zebra marker."),
            "{frame}"
        );
        assert!(frame.contains("Comment by @alice"), "{frame}");
    }

    #[tokio::test]
    async fn scrolling_works_before_the_first_render() {
        let mut record = record("a1", false);
        record.event = ActivityEvent::Comment {
            author: "alice".to_string(),
            body: (1..=40)
                .map(|line| format!("comment line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let mut app = TuiApp::new(view(vec![record.clone()]));
        app.popup = Some(ActivityDetail {
            record: record.clone(),
            changes: vec![record],
            pull_request: None,
            url: None,
        });
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::End), &inbox, &core_config())
            .await
            .expect("end");
        let frame = app.snapshot();
        assert!(frame.contains("comment line 40"), "{frame}");
    }

    #[tokio::test]
    async fn the_activity_detail_scrolls_through_a_long_comment() {
        let mut record = record("a1", false);
        record.event = ActivityEvent::Comment {
            author: "alice".to_string(),
            body: (1..=40)
                .map(|line| format!("comment line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let mut app = TuiApp::new(view(vec![record.clone()]));
        app.popup = Some(ActivityDetail {
            record: record.clone(),
            changes: vec![record],
            pull_request: None,
            url: None,
        });
        let frame = app.snapshot();
        assert!(frame.contains("comment line 1"), "{frame}");
        assert!(!frame.contains("comment line 40"), "{frame}");
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::End), &inbox, &core_config())
            .await
            .expect("end");
        let frame = app.snapshot();
        assert!(frame.contains("comment line 40"), "{frame}");
        app.handle_key(key(KeyCode::Home), &inbox, &core_config())
            .await
            .expect("home");
        let frame = app.snapshot();
        assert!(frame.contains("comment line 1"), "{frame}");
        assert!(!frame.contains("comment line 40"), "{frame}");
    }

    #[tokio::test]
    async fn the_detail_lists_all_changes_for_the_pull_request() {
        let first = record_on("acme/api#42", "a1", false);
        let mut second = record_on("acme/api#42", "a2", false);
        second.event = ActivityEvent::CiFailure {
            failed_checks: vec!["build".to_string()],
        };
        let mut app = TuiApp::new(view(vec![first, second]));
        let inbox = FakeInbox::new(Vec::new());
        app.handle_key(key(KeyCode::Enter), &inbox, &core_config())
            .await
            .expect("enter");
        let frame = app.snapshot();
        assert!(frame.contains("Changes on acme/api#42: 2"), "{frame}");
        assert!(frame.contains("comment by @alice"), "{frame}");
        assert!(frame.contains("CI failing: build"), "{frame}");
    }

    #[tokio::test]
    async fn the_save_prompt_answers_discard_save_and_cancel() {
        let mut app = TuiApp::new(view(Vec::new()));
        let inbox = FakeInbox::new(Vec::new());
        app.show_save_prompt();
        assert_eq!(
            app.handle_key(key(KeyCode::Char('n')), &inbox, &core_config())
                .await
                .expect("discard"),
            TuiAction::QuitDiscard
        );
        app.show_save_prompt();
        assert_eq!(
            app.handle_key(key(KeyCode::Char('y')), &inbox, &core_config())
                .await
                .expect("save"),
            TuiAction::QuitSave
        );
        app.show_save_prompt();
        assert_eq!(
            app.handle_key(key(KeyCode::Esc), &inbox, &core_config())
                .await
                .expect("cancel"),
            TuiAction::Continue
        );
        assert_eq!(
            app.handle_key(key(KeyCode::Char('n')), &inbox, &core_config())
                .await
                .expect("unbound"),
            TuiAction::Continue
        );
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
        assert!(!frame.contains("unread in stack"), "{frame}");
        let first = frame.find("1/2").expect("1/2");
        let second = frame.find("2/2").expect("2/2");
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
        assert!(frame.contains("1/3"), "{frame}");
        assert!(frame.contains("3/3"), "{frame}");
        assert!(!frame.contains("2/3"), "{frame}");
    }

    #[test]
    fn snapshot_renders_a_standalone_row_without_a_stack_summary() {
        let mut view = view(Vec::new());
        view.pull_requests = vec![standalone("acme/api#42")];
        let frame = TuiApp::new(view).snapshot();
        assert!(frame.contains("acme/api#42"), "{frame}");
        assert!(!frame.contains("1/1"), "{frame}");
        assert!(!frame.contains("unread in stack"), "{frame}");
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
        assert!(!frame.contains("unread in stack"), "{frame}");
    }

    #[test]
    fn snapshot_labels_activity_with_its_stack() {
        let mut view = view(vec![record_on("acme/api#43", "a1", false)]);
        view.pull_requests = vec![stacked("acme/api#43", 7, 2, 2, "main")];
        let frame = TuiApp::new(view).snapshot();
        let activity = frame
            .lines()
            .find(|line| line.contains("comment by @alice"))
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
        assert!(frame.contains("acme/api#42"), "{frame}");
        assert!(frame.contains("acme/web#40"), "{frame}");
        assert!(!frame.contains("unread in stack"), "{frame}");
    }
}
