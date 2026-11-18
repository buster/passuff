//! Plain-text transcripts. Every line is written to stdout except account
//! failures and delivery failures, which the runtime writes to stderr.

use passuff_config::AppConfig;
use passuff_core::model::{AccountFailure, Notification, RefreshReport};
use passuff_core::view;
use passuff_core::{ActivityRecord, WatcherState};

use crate::util::{activity_word, record_word, rfc3339};

#[derive(Debug, Clone)]
pub struct Digest {
    pub title: String,
    pub body: String,
}

pub fn status(config: &AppConfig, state: &WatcherState) -> String {
    let mut lines = Vec::new();
    lines.push("passuff: accounts".to_string());

    for label in config.accounts.keys() {
        let stored = state.accounts.get(label);
        let login = stored
            .and_then(|account| account.login.as_deref())
            .map(|login| format!("@{login}"))
            .unwrap_or_else(|| "not resolved".to_string());
        let last_check = stored
            .and_then(|account| account.last_check)
            .map(rfc3339)
            .unwrap_or_else(|| "never".to_string());
        lines.push(format!("  {label} {login}  last check {last_check}"));
    }

    lines.push(format!(
        "passuff: watch scope: {}",
        view::scope_label(config)
    ));

    let watched = state.watched_by_key();
    let login_set = state.login_set();
    let mut rows: Vec<&passuff_core::WatchedPullRequest> = watched
        .into_values()
        .filter(|pull_request| view::watched_visible(config, pull_request, &login_set))
        .collect();
    rows.sort_by(|left, right| left.key.cmp(&right.key));

    if rows.is_empty() {
        lines.push("passuff: no watched pull requests".to_string());
    } else {
        lines.push("passuff: watched pull requests".to_string());
        for pull_request in rows {
            let title = if pull_request.title.is_empty() {
                String::new()
            } else {
                format!(" - {}", pull_request.title)
            };
            lines.push(format!(
                "  {} {} CI {}  Pull request {}{}",
                pull_request.key,
                pull_request.state.human(),
                pull_request.ci.human(),
                pull_request.key,
                title
            ));
        }
    }

    let (unread, total) = view::unread_counts(config, state);
    lines.push(format!("passuff: {unread} unread of {total}"));
    lines.join("\n")
}

pub fn inbox(config: &AppConfig, state: &WatcherState, all: bool) -> String {
    let records = view::visible_activity(config, state);
    let selected: Vec<&ActivityRecord> = records
        .into_iter()
        .filter(|record| all || !record.read)
        .collect();
    if selected.is_empty() {
        return if all {
            "passuff: no activity".to_string()
        } else {
            "passuff: no unread activity".to_string()
        };
    }
    selected
        .into_iter()
        .map(|record| {
            let time = match record.occurred_at {
                Some(time) => rfc3339(time),
                None => format!("~{}", rfc3339(record.observed_at)),
            };
            format!(
                "  {}  {}  {}  {}  {}",
                record.id,
                time,
                record.key,
                record.summary,
                if record.read { "read" } else { "unread" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn read_message(changed: usize) -> String {
    format!("passuff: {changed} activity {} read", record_word(changed))
}

pub fn check_transcript(report: &RefreshReport) -> String {
    let mut lines = Vec::new();
    for key in &report.baselines {
        lines.push(format!("passuff: Fresh baseline recorded for {key}"));
    }
    if report.notifications.is_empty() {
        lines.push("passuff: no notifications".to_string());
        return lines.join("\n");
    }

    if report.notifications.len() == 1 {
        let notification = &report.notifications[0];
        lines.push(format!("passuff: notification for {}", notification.key));
        lines.push(format!(
            "passuff: {} {} on {}",
            notification.summaries.len(),
            activity_word(notification.summaries.len()),
            notification.key
        ));
        for summary in &notification.summaries {
            lines.push(format!("  {summary}"));
        }
    } else {
        let total: usize = report
            .notifications
            .iter()
            .map(|notification| notification.summaries.len())
            .sum();
        lines.push(format!(
            "passuff: {total} activities in {} pull requests",
            report.notifications.len()
        ));
        for notification in &report.notifications {
            lines.push(format!("passuff: notification for {}", notification.key));
            lines.push(format!(
                "  {}: {} {}",
                notification.key,
                notification.summaries.len(),
                activity_word(notification.summaries.len())
            ));
            for summary in &notification.summaries {
                lines.push(format!("    {summary}"));
            }
        }
    }
    lines.join("\n")
}

pub fn account_failure_line(failure: &AccountFailure) -> String {
    format!(
        "passuff: account {} failed: {}",
        failure.label, failure.message
    )
}

pub fn digest(notifications: &[Notification]) -> Option<Digest> {
    if notifications.is_empty() {
        return None;
    }
    if notifications.len() == 1 {
        let notification = &notifications[0];
        if notification.summaries.len() == 1 {
            return Some(Digest {
                title: format!("New activity on {}", notification.key),
                body: notification.summaries[0].clone(),
            });
        }
        let count = notification.summaries.len();
        let title = format!("{count} activities on {}", notification.key);
        let mut body = vec![title.clone()];
        body.extend(notification.summaries.iter().cloned());
        return Some(Digest {
            title,
            body: body.join("\n"),
        });
    }

    let total: usize = notifications
        .iter()
        .map(|notification| notification.summaries.len())
        .sum();
    let title = format!(
        "{total} activities in {} pull requests",
        notifications.len()
    );
    let mut body = vec![title.clone()];
    for notification in notifications.iter().take(5) {
        body.push(format!(
            "{}: {} {}",
            notification.key,
            notification.summaries.len(),
            activity_word(notification.summaries.len())
        ));
    }
    if notifications.len() > 5 {
        body.push(format!(
            "and {} more pull requests",
            notifications.len() - 5
        ));
    }
    Some(Digest {
        title,
        body: body.join("\n"),
    })
}
