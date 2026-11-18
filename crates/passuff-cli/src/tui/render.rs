//! Frame rendering for the terminal inbox. `--snapshot` uses the same output
//! without ANSI control sequences.

use passuff_config::{Relation, WatchMode};
use passuff_core::model::{ActivityRecord, PullRequestKey, WatchedPullRequest};
use passuff_core::view;

use crate::util::{pad, short_time, truncate_chars};

use super::{overlay_size, App, Overlay, Pane, PickerEntry, WatchedRow};

pub fn frame(app: &App) -> Vec<String> {
    let (width, height) = app.size;
    if width < 40 || height < 12 {
        return vec!["Terminal too small. Resize to at least 40x12.".to_string()];
    }

    let mut canvas = vec![vec![' '; width as usize]; height as usize];

    let account_line = account_line(app);
    set_line(&mut canvas, 0, &account_line);
    set_line(
        &mut canvas,
        1,
        &format!("watch scope: {}", view::scope_label(&app.config)),
    );
    let (unread, total) = app.global_counts();
    let sort_arrow = if app.prefs.ascending { '↑' } else { '↓' };
    set_line(
        &mut canvas,
        2,
        &format!(
            "{unread} unread of {total}    sort: {} {}",
            app.prefs.sort.label(),
            sort_arrow
        ),
    );

    let pane_area = height.saturating_sub(6);
    let watched_height = pane_area / 2;
    let activity_height = pane_area - watched_height;
    let watched_content = watched_content(app, watched_height.saturating_sub(2) as usize);
    let activity_content = activity_content(app, activity_height.saturating_sub(2) as usize);

    let watched_box = pane_box(
        width,
        watched_height,
        "Watched pull requests",
        &watched_content,
    );
    let activity_box = pane_box(
        width,
        activity_height,
        &app.activity_pane_title(),
        &activity_content,
    );
    for (offset, line) in watched_box.iter().enumerate() {
        set_line(&mut canvas, 3 + offset as u16, line);
    }
    let activity_top = 3 + watched_height;
    for (offset, line) in activity_box.iter().enumerate() {
        set_line(&mut canvas, activity_top + offset as u16, line);
    }

    let ignored_selected = app
        .selected_watched_key()
        .map(|key| view::is_ignored(&app.config, &key))
        .unwrap_or(false);
    set_line(
        &mut canvas,
        height - 3,
        &format!(
            "Focus: {}  Tab switch pane",
            match app.focus {
                Pane::PullRequests => "PRs",
                Pane::Activity => "activity",
            }
        ),
    );
    let hints = match app.focus {
        Pane::PullRequests => {
            let ignore = if ignored_selected {
                "x unignore"
            } else {
                "x ignore"
            };
            format!("Enter open  {ignore}  s sort  f filter  h keys")
        }
        Pane::Activity => "Enter open  r read  a mark visible read  u unread  h keys".to_string(),
    };
    set_line(&mut canvas, height - 2, &hints);
    set_line(&mut canvas, height - 1, &app.status);

    if let Some(overlay) = &app.overlay {
        draw_overlay(app, overlay, &mut canvas);
    }

    canvas
        .into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

fn account_line(app: &App) -> String {
    let parts: Vec<String> = app
        .config
        .accounts
        .keys()
        .map(|label| {
            let login = app
                .state
                .accounts
                .get(label)
                .and_then(|account| account.login.as_deref())
                .map(|login| format!("@{login}"))
                .unwrap_or_else(|| "not resolved".to_string());
            format!("{label} {login}")
        })
        .collect();
    format!("Accounts: {}", parts.join(", "))
}

fn watched_content(app: &App, visible_rows: usize) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("  Key               Pos  State    CI          ☺".to_string());

    let rows = app.watched_rows();
    let mut pull_lines = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if matches!(row, WatchedRow::Pull(_)) {
            pull_lines.push(index);
        }
    }
    let selected_line = pull_lines.get(app.watched_selection).copied().unwrap_or(0);
    let visible = visible_rows.saturating_sub(1).max(1);
    let first = (selected_line + 1).saturating_sub(visible);

    if pull_lines.is_empty() {
        lines.push("  no watched pull requests".to_string());
        return lines;
    }

    let ignored_lookup = app.config.watch.ignored_pull_requests.clone();
    let watched = app.state.watched_by_key();
    for row in rows.iter().skip(first).take(visible) {
        match row {
            WatchedRow::StackHeader { number, base } => {
                lines.push(format!("  ☰ stack {number}  base {base}"));
            }
            WatchedRow::Pull(key) => {
                let Some(pull_request) = watched.get(key).copied() else {
                    continue;
                };
                lines.push(watched_row(
                    app,
                    pull_request,
                    ignored_lookup
                        .iter()
                        .any(|ignored| ignored.eq_ignore_ascii_case(key)),
                ));
            }
        }
    }
    lines
}

fn watched_row(app: &App, pull_request: &WatchedPullRequest, ignored: bool) -> String {
    let marker = if app.selected_key.as_deref() == Some(pull_request.key.as_str()) {
        "▶ "
    } else {
        "  "
    };
    let key = truncate_key(&pull_request.key);
    let position = pull_request
        .stack
        .as_ref()
        .map(|stack| format!("☰ {}/{}", stack.position, stack.size))
        .unwrap_or_default();
    let people = people_label(pull_request);
    let title = pull_request.title.clone();
    format!(
        "  {}{}{} {:<6} {:<8} {:<11} ☺ {}{}",
        if ignored { "⊘ " } else { "" },
        marker,
        pad(&key, 16),
        if pull_request.stack.is_some() {
            position
        } else {
            String::new()
        },
        pull_request.state.human(),
        format!("CI {}", pull_request.ci.human()),
        people,
        if title.is_empty() {
            String::new()
        } else {
            format!("  {title}")
        }
    )
}

fn people_label(pull_request: &WatchedPullRequest) -> String {
    let creator = pull_request.author.clone();
    let assignees = &pull_request.assignees;
    if let Some(creator) = &creator {
        let creator_is_assignee = assignees
            .iter()
            .any(|assignee| assignee.eq_ignore_ascii_case(creator));
        if assignees.is_empty() || creator_is_assignee {
            return format!("@{creator}");
        }
        return format!(
            "@{} → {}",
            creator,
            assignees
                .iter()
                .map(|assignee| format!("@{assignee}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    assignees
        .iter()
        .map(|assignee| format!("@{assignee}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn truncate_key(key: &str) -> String {
    if key.chars().count() <= 16 {
        return key.to_string();
    }
    let suffix: String = key
        .chars()
        .rev()
        .take(15)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{suffix}")
}

fn activity_content(app: &App, visible_rows: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let (unread, total) = app.activity_counts();
    let filtered = app
        .filtered_from()
        .map(|from| format!(" (filtered from {from})"))
        .unwrap_or_default();
    lines.push(format!("{unread} unread of {total}{filtered}"));

    let records = app.activity_filtered();
    if records.is_empty() {
        lines.push("no activity yet".to_string());
        return lines;
    }
    let visible = visible_rows.saturating_sub(1).max(1);
    let first = (app.activity_selection + 1).saturating_sub(visible);
    for record in records.iter().skip(first).take(visible) {
        let marker = if app.selected_activity_id.as_deref() == Some(record.id.as_str()) {
            "▶"
        } else {
            " "
        };
        let time = match record.occurred_at {
            Some(time) => short_time(time),
            None => format!("~{}", short_time(record.observed_at)),
        };
        lines.push(format!(
            "  {marker} {}  {}  Pull request {}  {}",
            record.id, time, record.key, record.display
        ));
    }
    lines
}

fn pane_box(width: u16, height: u16, title: &str, content: &[String]) -> Vec<String> {
    let width = width as usize;
    let height = height as usize;
    let inner_width = width.saturating_sub(2);
    let mut lines = Vec::with_capacity(height);
    lines.push(border_top(width, title));
    for index in 0..height.saturating_sub(2) {
        let text = content.get(index).map(String::as_str).unwrap_or("");
        let text = sanitize(text);
        lines.push(format!(
            "│{}│",
            pad(&truncate_chars(&text, inner_width), inner_width)
        ));
    }
    lines.push(format!("└{}┘", "─".repeat(inner_width)));
    lines
}

fn border_top(width: usize, title: &str) -> String {
    let mut characters: Vec<char> = vec!['┌', '─', ' '];
    characters.extend(title.chars());
    characters.push(' ');
    if characters.len() < width.saturating_sub(1) {
        characters.extend(std::iter::repeat_n(
            '─',
            width.saturating_sub(1) - characters.len(),
        ));
    }
    characters.truncate(width.saturating_sub(1));
    characters.push('┐');
    characters.into_iter().collect()
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn set_line(canvas: &mut [Vec<char>], y: u16, text: &str) {
    let Some(row) = canvas.get_mut(y as usize) else {
        return;
    };
    let width = row.len();
    for cell in row.iter_mut() {
        *cell = ' ';
    }
    for (index, character) in text.chars().take(width).enumerate() {
        row[index] = if character.is_control() {
            ' '
        } else {
            character
        };
    }
}

fn draw_overlay(app: &App, overlay: &Overlay, canvas: &mut [Vec<char>]) {
    let (width, height) = app.size;
    let (overlay_width, overlay_height) = overlay_size((width, height));
    let content = app.detail_lines();
    let inner_height = overlay_height.saturating_sub(2) as usize;
    let inner_width = overlay_width.saturating_sub(2) as usize;
    let max_scroll = content.len().saturating_sub(inner_height);
    let scroll = app.overlay_scroll.min(max_scroll);
    let title = match overlay {
        Overlay::ActivityDetail { id } => app
            .state
            .activity
            .iter()
            .find(|record| &record.id == id)
            .map(|record| {
                PullRequestKey::parse(&record.key)
                    .map(|key| format!("Activity {}", key.short()))
                    .unwrap_or_else(|| format!("Activity {}", record.key))
            })
            .unwrap_or_else(|| "Activity".to_string()),
        Overlay::PullRequestDetail { key } => format!("Pull request {key}"),
        _ => app.overlay_title(),
    };

    let mut lines = Vec::with_capacity(overlay_height as usize);
    lines.push(border_top(overlay_width as usize, &title));
    for index in 0..inner_height {
        let text = content
            .get(scroll + index)
            .map(String::as_str)
            .unwrap_or("");
        lines.push(format!(
            "│{}│",
            pad(&truncate_chars(text, inner_width), inner_width)
        ));
    }
    lines.push(format!("└{}┘", "─".repeat(inner_width)));

    let x = (width.saturating_sub(overlay_width)) / 2;
    let y = (height.saturating_sub(overlay_height)) / 2;
    for (offset, line) in lines.iter().enumerate() {
        let target_y = y as usize + offset;
        let Some(row) = canvas.get_mut(target_y) else {
            continue;
        };
        for (index, character) in line.chars().enumerate() {
            let target_x = x as usize + index;
            if target_x < row.len() {
                row[target_x] = character;
            }
        }
    }
}

pub fn activity_detail_lines(app: &App, record: &ActivityRecord) -> Vec<String> {
    let watched = app.state.watched_by_key();
    let pull_request = watched.get(&record.key).copied();
    let mut lines = Vec::new();
    let short = PullRequestKey::parse(&record.key)
        .map(|key| key.short())
        .unwrap_or_else(|| record.key.clone());
    lines.push(format!("Activity {short}"));

    match record.occurred_at {
        Some(occurred) => {
            lines.push(format!("When: {}", short_time(occurred)));
            if occurred.timestamp() != record.observed_at.timestamp() {
                lines.push(format!("Fetched: {}", short_time(record.observed_at)));
            }
        }
        None => {
            lines.push(format!(
                "When: ~{} (fetch time)",
                short_time(record.observed_at)
            ));
        }
    }

    lines.push(format!("Pull request {}", record.key));
    if let Some(pull_request) = pull_request {
        if !pull_request.title.is_empty() {
            lines.push(format!("Title: {}", pull_request.title));
        }
        lines.push(format!(
            "State: {}   CI: {}",
            pull_request.state.human(),
            pull_request.ci.human()
        ));
    }
    if let Some(stack) = record
        .stack
        .clone()
        .or_else(|| pull_request.and_then(|pull_request| pull_request.stack.clone()))
    {
        lines.push(format!("stack {}", stack.number));
    }
    lines.push(if record.read {
        "read".to_string()
    } else {
        "unread".to_string()
    });
    lines.push(format!("Event: {}", record.display));
    lines.push("Enter open in browser    esc/q close".to_string());
    lines.push("────".to_string());
    lines.push(record.body_label());
    if let Some(body) = &record.body {
        if !body.trim().is_empty() {
            lines.extend(markdown_lines(body));
        }
    }
    lines
}

pub fn pull_request_detail_lines(app: &App, pull_request: &WatchedPullRequest) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Pull request {}", pull_request.key));
    if !pull_request.title.is_empty() {
        lines.push(format!("Title: {}", pull_request.title));
    }
    lines.push(format!(
        "Status: {}   CI: {}",
        pull_request.state.human(),
        pull_request.ci.human()
    ));
    lines.push(format!(
        "Creator: @{}",
        pull_request.author.as_deref().unwrap_or("unknown")
    ));
    if !pull_request.assignees.is_empty() {
        lines.push(format!(
            "Assignee: {}",
            pull_request
                .assignees
                .iter()
                .map(|assignee| format!("@{assignee}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(stack) = &pull_request.stack {
        lines.push(format!("stack {}  base {}", stack.number, stack.base));
    }
    if let Some(url) = &pull_request.url {
        lines.push(format!("URL: {url}"));
    }
    lines.push("────".to_string());
    lines.push("Description:".to_string());
    match &pull_request.body {
        Some(body) if !body.trim().is_empty() => lines.extend(markdown_lines(body)),
        _ => lines.push("(no description)".to_string()),
    }
    let _ = app;
    lines
}

pub fn help_lines(pane: Pane) -> Vec<String> {
    match pane {
        Pane::Activity => vec![
            "Activity pane".to_string(),
            "j/k, up/down  move".to_string(),
            "PgUp/PgDn  move ten rows".to_string(),
            "Home/End  first or last".to_string(),
            "Enter  open detail".to_string(),
            "r  mark read".to_string(),
            "a  mark visible read".to_string(),
            "u  unread only".to_string(),
            "Tab  switch pane".to_string(),
            "h/F1  keys".to_string(),
            "q  quit".to_string(),
        ],
        Pane::PullRequests => vec![
            "Pull request pane".to_string(),
            "j/k, up/down  move".to_string(),
            "PgUp/PgDn  move ten rows".to_string(),
            "Home/End  first or last".to_string(),
            "Enter  open detail".to_string(),
            "x  ignore or unignore".to_string(),
            "s/S  cycle or reverse sort".to_string(),
            "f/F  filter activity".to_string(),
            "left/right  ignored activity".to_string(),
            "Tab  switch pane".to_string(),
            "h/F1  keys".to_string(),
            "q  quit".to_string(),
        ],
    }
}

pub fn settings_lines(app: &App) -> Vec<String> {
    let marker = |index: usize| {
        if app.settings_selection == index {
            "▶"
        } else {
            " "
        }
    };
    vec![
        "Settings".to_string(),
        format!(
            "{} Desktop notifications: {}",
            marker(0),
            if app.prefs.desktop { "on" } else { "off" }
        ),
        format!(
            "{} Sound: {}",
            marker(1),
            if app.prefs.sound { "on" } else { "off" }
        ),
        String::new(),
        "space/Enter toggle    esc close".to_string(),
    ]
}

pub fn picker_lines(app: &App) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("Watch scope".to_string());
    lines.push(format!(
        "Mode: {}",
        match app.config.watch.effective_mode() {
            WatchMode::All => "all",
            WatchMode::Selected => "selected",
        }
    ));
    lines.push("Focus".to_string());
    let entries = app.picker_entries();
    let mut index = 0usize;
    for relation in Relation::ALL {
        let selected = app.config.focus.contains(relation);
        lines.push(format!(
            "{} [{}] {}",
            marker(app.picker_selection, index),
            if selected { "x" } else { " " },
            view::relation_label(relation)
        ));
        index += 1;
    }
    lines.push("Repositories".to_string());
    let repositories: Vec<PickerEntry> = entries
        .into_iter()
        .filter(|entry| matches!(entry, PickerEntry::Repository(_)))
        .collect();
    if repositories.is_empty() {
        lines.push("(no known repositories)".to_string());
    }
    for entry in repositories {
        if let PickerEntry::Repository(repository) = entry {
            let selected = app
                .config
                .watch
                .repositories
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&repository));
            lines.push(format!(
                "{} [{}] {}",
                marker(app.picker_selection, index),
                if selected { "x" } else { " " },
                repository
            ));
            index += 1;
        }
    }
    lines
}

fn marker(selection: usize, index: usize) -> &'static str {
    if selection == index {
        "▶"
    } else {
        " "
    }
}

/// A small Markdown subset: headings, bullets, task boxes, code fences,
/// horizontal rules, and inline emphasis markers.
pub fn markdown_lines(body: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_code = false;
    let mut previous_blank = false;

    for raw in body.lines() {
        let trimmed = raw.trim_end();
        if trimmed.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
                previous_blank = false;
            }
            continue;
        }
        if trimmed.trim().is_empty() {
            if !previous_blank && !lines.is_empty() {
                lines.push(String::new());
                previous_blank = true;
            }
            continue;
        }
        previous_blank = false;
        let leading = trimmed.len() - trimmed.trim_start().len();
        let indent = " ".repeat(leading.min(8));
        let content = trimmed.trim_start();
        if content.trim() == "---" || content.trim() == "***" {
            lines.push(format!("{indent}────"));
            continue;
        }
        if let Some(text) = content.strip_prefix("###") {
            lines.push(format!("{indent}{}", inline(text.trim())));
            continue;
        }
        if let Some(text) = content.strip_prefix("##") {
            lines.push(format!("{indent}{}", inline(text.trim())));
            continue;
        }
        if let Some(text) = content.strip_prefix('#') {
            lines.push(format!("{indent}{}", inline(text.trim())));
            continue;
        }
        if let Some(text) = content
            .strip_prefix("- [ ] ")
            .or_else(|| content.strip_prefix("* [ ] "))
        {
            lines.push(format!("{indent}• ☐ {}", inline(text)));
            continue;
        }
        if let Some(text) = content
            .strip_prefix("- [x] ")
            .or_else(|| content.strip_prefix("- [X] "))
            .or_else(|| content.strip_prefix("* [x] "))
        {
            lines.push(format!("{indent}• ☑ {}", inline(text)));
            continue;
        }
        if let Some(text) = content
            .strip_prefix("- ")
            .or_else(|| content.strip_prefix("* "))
        {
            lines.push(format!("{indent}• {}", inline(text)));
            continue;
        }
        if let Some((number, rest)) = numbered_item(content) {
            lines.push(format!("{indent}{number}. {}", inline(rest)));
            continue;
        }
        lines.push(format!("{indent}{}", inline(content)));
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

fn numbered_item(content: &str) -> Option<(&str, &str)> {
    let (number, rest) = content.split_once('.')?;
    if number.chars().all(|character| character.is_ascii_digit())
        && !number.is_empty()
        && rest.starts_with(' ')
    {
        Some((number, rest.trim_start()))
    } else {
        None
    }
}

fn inline(value: &str) -> String {
    let mut result = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '`' => {}
            '*' if characters.peek() == Some(&'*') => {
                characters.next();
            }
            '_' if characters.peek() == Some(&'_') => {
                characters.next();
            }
            '[' => {
                let mut label = String::new();
                for next in characters.by_ref() {
                    if next == ']' {
                        break;
                    }
                    label.push(next);
                }
                if characters.peek() == Some(&'(') {
                    let mut depth = 0;
                    for next in characters.by_ref() {
                        if next == '(' {
                            depth += 1;
                        } else if next == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                    }
                }
                result.push_str(&label);
            }
            other => result.push(other),
        }
    }
    result
}
