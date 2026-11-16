use chrono::{DateTime, Utc};
use passuff_config::FocusRelation;
use passuff_core::{ActivityView, PullRequestView};

use crate::tui::app::{Overlay, Pane, TuiApp, WatchedRow};
use crate::tui::markdown::{render as render_markdown, MarkdownLine, MarkdownStyle};

pub fn activity_detail_lines(
    activity: &ActivityView,
    pull_request: Option<&PullRequestView>,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Activity {}#{}",
        activity.pull_request.name(),
        activity.pull_request.number()
    ));
    if let Some(occurred) = activity.occurred_at {
        lines.push(format!("When: {}", minute(occurred)));
        if occurred != activity.observed_at {
            lines.push(format!("Fetched: {}", minute(activity.observed_at)));
        }
    } else {
        lines.push(format!(
            "When: ~{} (fetch time)",
            minute(activity.observed_at)
        ));
    }
    lines.push(format!("Pull request {}", activity.pull_request));
    if let Some(pull_request) = pull_request {
        if !pull_request.title.is_empty() {
            lines.push(format!("Title: {}", pull_request.title));
        }
        if let Some(stack) = &pull_request.stack {
            lines.push(format!("stack {}", stack.number));
        }
    }
    lines.push(if activity.read {
        "read".to_string()
    } else {
        "unread".to_string()
    });
    lines.push(format!("Event: {}", activity.display));
    lines.push("Enter open in browser    esc/q close".to_string());
    lines.push("────".to_string());
    lines.push(body_label(activity));
    match &activity.body {
        Some(body) if !body.trim().is_empty() => {
            for line in render_markdown(body) {
                lines.push(line.text);
            }
        }
        _ => lines.push(activity.summary.clone()),
    }
    lines
}

pub fn pull_request_detail_lines(pull_request: &PullRequestView) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Pull request {}", pull_request.key));
    lines.push(format!(
        "Status: {}   CI: {}",
        pull_request.state.as_str(),
        pull_request.ci.as_str()
    ));
    lines.push(format!(
        "Creator: @{}",
        if pull_request.author.is_empty() {
            "unknown"
        } else {
            &pull_request.author
        }
    ));
    let assignee = pull_request
        .assignees
        .first()
        .map(|login| format!("@{login}"))
        .unwrap_or_else(|| "none".to_string());
    lines.push(format!("Assignee: {assignee}"));
    if let Some(stack) = &pull_request.stack {
        lines.push(format!(
            "Stack: {}/{} base {}",
            stack.position, stack.size, stack.base
        ));
    }
    lines.push("Enter open in browser    esc/q close".to_string());
    lines.push("────".to_string());
    lines.push("Description:".to_string());
    if pull_request.body.trim().is_empty() {
        lines.push("(no description)".to_string());
    } else {
        for line in render_markdown(&pull_request.body) {
            lines.push(line.text);
        }
    }
    lines
}

pub fn header_lines(app: &TuiApp) -> Vec<String> {
    let accounts = app
        .view
        .accounts
        .iter()
        .map(|account| match &account.login {
            Some(login) => format!("{} @{}", account.label, login),
            None => account.label.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let (unread, total) = app.visible_counts();
    vec![
        format!("Accounts: {accounts}"),
        format!("watch scope: {}", app.view.scope_label),
        format!(
            "{unread} unread of {total}    sort: {} {}",
            app.sort.label(),
            app.direction.arrow()
        ),
    ]
}

/// Watched-pane contents. The first line is the column header; the caller
/// supplies the inner height after the pane border and title.
pub fn watched_lines(app: &TuiApp, available: usize) -> Vec<String> {
    if available == 0 {
        return Vec::new();
    }
    let mut lines = vec!["  Key               Pos  State    CI          ☺".to_string()];
    let rows_available = available.saturating_sub(1);
    if rows_available == 0 {
        return lines;
    }

    let watched_rows = app.watched_rows();
    if watched_rows.is_empty() {
        lines.push("  no watched pull requests".to_string());
        return lines;
    }

    let selected_key = app
        .sorted_pull_requests()
        .get(app.watched_selection)
        .map(|pr| pr.key.clone());
    let selected_display = watched_rows
        .iter()
        .position(|row| match row {
            WatchedRow::PullRequest(pr) => Some(&pr.key) == selected_key.as_ref(),
            WatchedRow::StackHeader { .. } => false,
        })
        .unwrap_or(0);
    let watched_start = selected_display
        .saturating_add(1)
        .saturating_sub(rows_available);
    for row in watched_rows.iter().skip(watched_start).take(rows_available) {
        match row {
            WatchedRow::StackHeader { repository, stack } => {
                lines.push(format!(
                    "  ☰ {} stack {} · base {} · size {}",
                    repository, stack.number, stack.base, stack.size
                ));
            }
            WatchedRow::PullRequest(pr) => {
                let marker = if Some(&pr.key) == selected_key.as_ref() {
                    "▶ "
                } else {
                    "  "
                };
                let ignored = if pr.ignored { "⊘ " } else { "" };
                let stack = pr
                    .stack_label()
                    .map(|label| format!("{label:>4}"))
                    .unwrap_or_else(|| "    ".to_string());
                let people = people_label(pr);
                lines.push(format!(
                    "  {marker}{ignored}{:<16} {stack}  {:<7}  CI {:<10}  ☺ {people}  {}",
                    truncate_key(&pr.key.to_string()),
                    pr.state.as_str(),
                    pr.ci.as_str(),
                    pr.title
                ));
            }
        }
    }
    lines
}

pub fn activity_title(app: &TuiApp) -> String {
    format!("Activity · {}", activity_filter_label(app))
}

/// Activity-pane contents. The first line is the visible count; the caller
/// supplies the inner height after the pane border and title.
pub fn activity_lines(app: &TuiApp, available: usize) -> Vec<String> {
    if available == 0 {
        return Vec::new();
    }
    let (unread, total) = app.visible_counts();
    let filtered = app
        .filtered_from()
        .filter(|from| *from != total)
        .map(|from| format!(" (filtered from {from})"))
        .unwrap_or_default();
    let mut lines = vec![format!("{unread} unread of {total}{filtered}")];
    let rows_available = available.saturating_sub(1);
    if rows_available == 0 {
        return lines;
    }
    let activity = app.visible_activity();
    if activity.is_empty() {
        lines.push("  no activity yet - Tab to watched PRs".to_string());
        return lines;
    }
    let start = app
        .activity_selection
        .saturating_add(1)
        .saturating_sub(rows_available);
    for (index, record) in activity.iter().enumerate().skip(start).take(rows_available) {
        let marker = if index == app.activity_selection {
            "▶ "
        } else {
            "  "
        };
        let when = if let Some(occurred) = record.occurred_at {
            minute(occurred)
        } else {
            format!("~{}", minute(record.observed_at))
        };
        lines.push(format!(
            "  {marker}{}  {when}  Pull request {}  {}",
            record.id, record.pull_request, record.display
        ));
    }
    lines
}

pub fn footer_lines(app: &TuiApp) -> Vec<String> {
    let focus = match app.focus {
        Pane::Watched => "Focus: PRs",
        Pane::Activity => "Focus: activity",
    };
    let mut focus_line = format!("{focus}    Tab switch pane");
    if app.focus == Pane::Activity && app.visible_activity().is_empty() {
        focus_line.push_str("    (activity empty - Tab to watched PRs)");
    }
    let x_label = if app
        .selected_pull_request_owned()
        .map(|pr| pr.ignored)
        .unwrap_or(false)
    {
        "x unignore"
    } else {
        "x ignore"
    };
    let hints = match app.focus {
        Pane::Watched => {
            format!("j/k move  enter  {x_label}  s sort  f filter  h help  , settings  q quit")
        }
        Pane::Activity => {
            "j/k move  enter  r read  a read visible  f filter  u unread  h help  q quit"
                .to_string()
        }
    };
    vec![focus_line, hints, app.status.clone().unwrap_or_default()]
}

pub fn overlay_lines(app: &TuiApp) -> Option<(String, Vec<MarkdownLine>, usize)> {
    let overlay = app.overlay.as_ref()?;
    match overlay {
        Overlay::ActivityDetail {
            activity,
            pull_request,
            scroll,
        } => {
            let lines = activity_detail_lines(activity, pull_request.as_ref())
                .into_iter()
                .map(|text| MarkdownLine {
                    text,
                    style: MarkdownStyle::Normal,
                })
                .collect();
            Some((
                format!(
                    "Activity {}#{}",
                    activity.pull_request.name(),
                    activity.pull_request.number()
                ),
                lines,
                *scroll,
            ))
        }
        Overlay::PullRequestDetail {
            pull_request,
            scroll,
        } => {
            let lines = pull_request_detail_lines(pull_request)
                .into_iter()
                .map(|text| MarkdownLine {
                    text,
                    style: MarkdownStyle::Normal,
                })
                .collect();
            Some((format!("Pull request {}", pull_request.key), lines, *scroll))
        }
        Overlay::Help => {
            let lines = if app.focus == Pane::Watched {
                vec![
                    "Pull request pane",
                    "j/k, arrows    move",
                    "enter          pull request detail",
                    "x              ignore or unignore",
                    "left/right     hide or show ignored",
                    "s/S            sort or reverse",
                    "f/F            filter activity",
                    "u/c/C          unread/clear/reset",
                    ", settings     p scope picker",
                    "tab            switch to activity",
                    "esc/q          close",
                ]
            } else {
                vec![
                    "Activity pane",
                    "j/k, arrows    move",
                    "enter          activity detail",
                    "r              mark selected read",
                    "a              mark visible read",
                    "u              unread only",
                    "f/F            filter activity",
                    "u/c/C          unread/clear/reset",
                    ", settings     p scope picker",
                    "tab            switch to watched PRs",
                    "esc/q          close",
                ]
            };
            Some((
                "Keys".to_string(),
                lines
                    .into_iter()
                    .map(|text| MarkdownLine {
                        text: text.to_string(),
                        style: MarkdownStyle::Normal,
                    })
                    .collect(),
                0,
            ))
        }
        Overlay::Settings { selected } => {
            let lines = vec![
                format!(
                    "{} Desktop notifications: {}",
                    if *selected == 0 { ">" } else { " " },
                    if app.prefs.desktop() { "on" } else { "off" }
                ),
                format!(
                    "{} Sound: {}",
                    if *selected == 1 { ">" } else { " " },
                    if app.prefs.sound() { "on" } else { "off" }
                ),
                "space/enter toggle    esc/q close".to_string(),
            ];
            Some((
                "Settings".to_string(),
                lines
                    .into_iter()
                    .map(|text| MarkdownLine {
                        text,
                        style: MarkdownStyle::Normal,
                    })
                    .collect(),
                0,
            ))
        }
        Overlay::ScopePicker { selected } => {
            Some(("Watch scope".to_string(), picker_lines(app, *selected), 0))
        }
        Overlay::SavePrompt => Some((
            "Save".to_string(),
            vec![
                MarkdownLine {
                    text: "Save view and notification settings?".to_string(),
                    style: MarkdownStyle::Normal,
                },
                MarkdownLine {
                    text: "y save and quit    n discard and quit    esc/q back".to_string(),
                    style: MarkdownStyle::Normal,
                },
            ],
            0,
        )),
    }
}

pub fn picker_lines(app: &TuiApp, selected: usize) -> Vec<MarkdownLine> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Mode: {}",
        app.config.watch.effective_mode().as_str()
    ));
    let repositories = app.known_repositories();
    for (index, repository) in repositories.iter().enumerate() {
        let marked = app
            .config
            .watch
            .repositories
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(repository));
        lines.push(format!(
            "{}[{}] {repository}",
            if selected == index { "> " } else { "  " },
            if marked { "x" } else { " " }
        ));
    }
    lines.push("Focus".to_string());
    for (index, relation) in FocusRelation::ALL.iter().enumerate() {
        let marked = app.config.focus.relations.contains(relation);
        lines.push(format!(
            "{}[{}] {}",
            if selected == repositories.len() + index {
                "> "
            } else {
                "  "
            },
            if marked { "x" } else { " " },
            relation.label()
        ));
    }
    lines.push("space toggle  m mode  j/k move  esc/q close".to_string());
    lines
        .into_iter()
        .map(|text| MarkdownLine {
            text,
            style: MarkdownStyle::Normal,
        })
        .collect()
}

pub fn picker_text(app: &TuiApp) -> String {
    let mut output = String::new();
    output.push_str("passuff-tui: Watch scope\n");
    for line in picker_lines(app, 0) {
        output.push_str(&line.text);
        output.push('\n');
    }
    output
}

fn minute(value: DateTime<Utc>) -> String {
    value.format("%m-%d %H:%M").to_string()
}

fn people_label(pull_request: &PullRequestView) -> String {
    let creator = if pull_request.author.is_empty() {
        "@unknown".to_string()
    } else {
        format!("@{}", pull_request.author)
    };
    match pull_request.assignees.first() {
        Some(assignee) if !assignee.eq_ignore_ascii_case(&pull_request.author) => {
            format!("{creator} → @{assignee}")
        }
        _ => creator,
    }
}

fn truncate_key(key: &str) -> String {
    let characters: Vec<char> = key.chars().collect();
    if characters.len() <= 16 {
        return key.to_string();
    }
    let tail: String = characters[characters.len() - 15..].iter().collect();
    format!("…{tail}")
}

fn activity_filter_label(app: &TuiApp) -> String {
    let mut parts = Vec::new();
    if let Some(key) = &app.filter_pull_request {
        parts.push(key.to_string());
    }
    if let Some(stack) = app.filter_stack {
        let repository = app
            .sorted_pull_requests()
            .iter()
            .find(|pr| pr.stack.as_ref().map(|stack| stack.number) == Some(stack))
            .map(|pr| pr.key.repository().to_string())
            .unwrap_or_default();
        parts.push(format!("{repository} stack {stack}"));
    }
    if app.unread_only {
        parts.push("unread".to_string());
    }
    if parts.is_empty() {
        "all".to_string()
    } else {
        parts.join(" · ")
    }
}

fn body_label(activity: &ActivityView) -> String {
    match &activity.kind {
        Some(passuff_core::EventKind::Comment) => activity
            .author
            .as_deref()
            .map(|author| format!("Comment by @{author}"))
            .unwrap_or_else(|| "Comment".to_string()),
        Some(passuff_core::EventKind::ReviewComment) => activity
            .author
            .as_deref()
            .map(|author| format!("Review comment by @{author}"))
            .unwrap_or_else(|| "Review comment".to_string()),
        Some(passuff_core::EventKind::Review { state }) => activity
            .author
            .as_deref()
            .map(|author| format!("Review {} by @{author}", state.label()))
            .unwrap_or_else(|| "Review".to_string()),
        _ => activity.summary.clone(),
    }
}
