use chrono::{DateTime, SecondsFormat, Utc};
use passuff_config::NotificationsConfig;
use passuff_core::{
    AccountView, ActivityView, InboxView, Notification, PullRequestView, RefreshReport,
};

use crate::delivery::{urgency_label, DesktopMessage};

pub struct RenderedRefresh {
    pub stdout: String,
    pub stderr: String,
    pub digest: Option<DesktopMessage>,
}

pub fn render_refresh(
    report: &RefreshReport,
    notifications: &NotificationsConfig,
) -> RenderedRefresh {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for failure in &report.account_failures {
        stderr.push_str(&format!("passuff: {}\n", failure.message));
    }
    for baseline in &report.baselines {
        stdout.push_str(&format!(
            "passuff: Fresh baseline recorded for {baseline}\n"
        ));
    }
    if report.notifications.is_empty() {
        stdout.push_str("passuff: no notifications\n");
    } else if report.notifications.len() == 1 {
        let notification = &report.notifications[0];
        let count = notification.activity_count();
        stdout.push_str(&format!("passuff: notification for {}\n", notification.key));
        stdout.push_str(&format!(
            "passuff: {count} {} on {}\n",
            plural(count, "activity", "activities"),
            notification.key
        ));
        for event in &notification.events {
            stdout.push_str(&format!("  {}\n", event.summary));
        }
    } else {
        let total: usize = report
            .notifications
            .iter()
            .map(Notification::activity_count)
            .sum();
        let pull_requests = report.notifications.len();
        stdout.push_str(&format!(
            "passuff: {total} activities in {pull_requests} pull requests\n"
        ));
        for notification in &report.notifications {
            stdout.push_str(&format!("passuff: notification for {}\n", notification.key));
            stdout.push_str(&format!(
                "  {}: {} {}\n",
                notification.key,
                notification.activity_count(),
                plural(notification.activity_count(), "activity", "activities")
            ));
            for event in &notification.events {
                stdout.push_str(&format!("    {}\n", event.summary));
            }
        }
    }
    let digest = digest_message(report, notifications);
    RenderedRefresh {
        stdout,
        stderr,
        digest,
    }
}

pub fn digest_message(
    report: &RefreshReport,
    notifications: &NotificationsConfig,
) -> Option<DesktopMessage> {
    if report.notifications.is_empty() {
        return None;
    }
    let urgency = urgency_label(notifications.urgency).to_string();
    if report.notifications.len() == 1 {
        let notification = &report.notifications[0];
        let count = notification.activity_count();
        let title = if count == 1 {
            format!("New activity on {}", notification.key)
        } else {
            format!("{count} activities on {}", notification.key)
        };
        let mut body = String::new();
        if count > 1 {
            body.push_str(&format!("{count} activities on {}\n", notification.key));
        }
        for event in &notification.events {
            body.push_str(&event.summary);
            body.push('\n');
        }
        return Some(DesktopMessage {
            title,
            body: body.trim_end().to_string(),
            urgency,
        });
    }
    let total: usize = report
        .notifications
        .iter()
        .map(Notification::activity_count)
        .sum();
    let pull_requests = report.notifications.len();
    let title = format!("{total} activities in {pull_requests} pull requests");
    let mut body = format!("{title}\n");
    for notification in report.notifications.iter().take(5) {
        body.push_str(&format!(
            "{}: {} {}\n",
            notification.key,
            notification.activity_count(),
            plural(notification.activity_count(), "activity", "activities")
        ));
    }
    if report.notifications.len() > 5 {
        body.push_str(&format!(
            "and {} more pull requests\n",
            report.notifications.len() - 5
        ));
    }
    Some(DesktopMessage {
        title,
        body: body.trim_end().to_string(),
        urgency,
    })
}

pub fn render_status(view: &InboxView) -> String {
    let mut output = String::new();
    output.push_str("passuff: accounts\n");
    for account in &view.accounts {
        output.push_str(&format!(
            "  {} {}  last check {}\n",
            account.label,
            account
                .login
                .as_deref()
                .map(|login| format!("@{login}"))
                .unwrap_or_else(|| "not resolved".to_string()),
            account
                .last_check
                .map(format_timestamp)
                .unwrap_or_else(|| "never".to_string())
        ));
    }
    output.push_str(&format!("passuff: watch scope: {}\n", view.scope_label));
    if view.watched.is_empty() {
        output.push_str("passuff: no watched pull requests\n");
    } else {
        output.push_str("passuff: watched pull requests\n");
        for pull_request in &view.watched {
            output.push_str(&format!(
                "  {} {} CI {}  Pull request {}{}\n",
                pull_request.key,
                pull_request.state.as_str(),
                pull_request.ci.as_str(),
                pull_request.key,
                if pull_request.title.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", pull_request.title)
                }
            ));
        }
    }
    output.push_str(&format!(
        "passuff: {} unread of {}\n",
        view.unread, view.total
    ));
    output
}

pub fn render_inbox(view: &InboxView, all: bool) -> String {
    let records: Vec<&ActivityView> = view
        .activity
        .iter()
        .filter(|record| !record.ignored)
        .filter(|record| all || !record.read)
        .collect();
    if records.is_empty() {
        return if all {
            "passuff: no activity\n".to_string()
        } else {
            "passuff: no unread activity\n".to_string()
        };
    }
    let mut output = String::new();
    for record in records {
        let time = record
            .occurred_at
            .map(format_timestamp)
            .unwrap_or_else(|| format!("~{}", format_timestamp(record.observed_at)));
        output.push_str(&format!(
            "  {}  {}  {}  {}  {}\n",
            record.id,
            time,
            record.pull_request,
            record.summary,
            if record.read { "read" } else { "unread" }
        ));
    }
    output
}

pub fn format_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

#[allow(dead_code)]
fn _account(_: &AccountView, _: &PullRequestView) {}
