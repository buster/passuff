use passuff_core::{
    AccountFailure, ActivityEvent, ActivityRecord, CiState, InboxView, Notification, PrState,
    PullRequestKey, ReadOutcome,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedNotification {
    pub title: String,
    pub body: String,
    pub transcript: String,
    pub sound: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TranscriptRenderer;

impl TranscriptRenderer {
    pub fn render_notification(
        &self,
        notification: &Notification,
        sound: bool,
    ) -> RenderedNotification {
        let title = format!("passuff: {}", notification.key);
        let lines: Vec<String> = notification.events.iter().map(render_event).collect();
        let body = lines.join("\n");
        let mut transcript = title.clone();
        for line in &lines {
            transcript.push('\n');
            transcript.push_str("  ");
            transcript.push_str(line);
        }
        RenderedNotification {
            title,
            body,
            transcript,
            sound,
        }
    }

    pub fn render_account_failure(&self, failure: &AccountFailure) -> String {
        format!(
            "passuff: account {} failed: {}",
            failure.label, failure.detail
        )
    }

    pub fn render_baseline(&self, key: &PullRequestKey) -> String {
        format!("passuff: Fresh baseline recorded for {key}")
    }

    pub fn render_no_changes(&self) -> String {
        "passuff: no notifications".to_string()
    }

    pub fn render_test_notification(&self, sound: bool) -> RenderedNotification {
        let title = "passuff: notify test".to_string();
        let body = "If you can read this, passuff notifications are working.".to_string();
        RenderedNotification {
            transcript: format!("{title}\n{body}"),
            title,
            body,
            sound,
        }
    }

    pub fn render_watching(&self) -> String {
        "passuff: watching".to_string()
    }

    pub fn render_delivery_error(&self, detail: &str) -> String {
        format!("passuff: desktop notification failed: {detail}")
    }

    pub fn render_sound_unavailable(&self, detail: &str) -> String {
        format!("passuff: sound unavailable: {detail}")
    }

    pub fn render_config_error(&self, detail: &str) -> String {
        format!("passuff: configuration error: {detail}")
    }

    pub fn render_state_error(&self, detail: &str) -> String {
        format!("passuff: state error: {detail}")
    }

    pub fn render_status(&self, view: &InboxView) -> String {
        let mut lines = vec!["passuff: accounts".to_string()];
        for account in &view.accounts {
            let login = account
                .login
                .as_deref()
                .map(|login| format!("@{login}"))
                .unwrap_or_else(|| "not resolved".to_string());
            let last = account.last_check_at.as_deref().unwrap_or("never");
            lines.push(format!("  {} {} last check {}", account.label, login, last));
        }
        lines.push(format!("passuff: watch scope: {}", view.scope_label));
        if view.pull_requests.is_empty() {
            lines.push("passuff: no watched pull requests".to_string());
        } else {
            lines.push("passuff: watched pull requests".to_string());
            for pull_request in &view.pull_requests {
                lines.push(format!(
                    "  {} {} {} {}",
                    pull_request.key,
                    state_label(&pull_request.state),
                    ci_label(&pull_request.ci_state),
                    pull_request.title
                ));
            }
        }
        lines.push(format!(
            "passuff: activity: {} unread of {}",
            view.unread,
            view.activity.len()
        ));
        lines.join("\n")
    }

    pub fn render_inbox(&self, view: &InboxView, all: bool) -> String {
        let records: Vec<&ActivityRecord> = view
            .activity
            .iter()
            .filter(|record| all || !record.read)
            .collect();
        if records.is_empty() {
            return if all && view.activity.is_empty() {
                "passuff: no activity".to_string()
            } else {
                "passuff: no unread activity".to_string()
            };
        }
        let mut lines = vec!["passuff: inbox".to_string()];
        for record in records {
            lines.push(format!(
                "  {} {} {} {} {}",
                record.id,
                record.observed_at,
                record.key,
                if record.read { "read" } else { "unread" },
                render_event(&record.event)
            ));
        }
        lines.join("\n")
    }

    pub fn render_read(&self, outcome: ReadOutcome) -> String {
        let noun = if outcome.changed == 1 {
            "activity record"
        } else {
            "activity records"
        };
        format!(
            "passuff: {} {} read\npassuff: {} unread remaining",
            outcome.changed, noun, outcome.unread
        )
    }

    pub fn render_unknown_activity(&self, id: &str) -> String {
        format!("passuff: no activity with id {id}")
    }
}

pub fn state_label(state: &PrState) -> &'static str {
    match state {
        PrState::Open => "open",
        PrState::Merged => "merged",
        PrState::Closed => "closed",
    }
}

pub fn ci_label(state: &CiState) -> &'static str {
    match state {
        CiState::Success => "CI passing",
        CiState::Failure | CiState::Error => "CI failing",
        CiState::Pending => "CI pending",
        CiState::None => "CI not started",
    }
}

pub fn activity_summary(event: &ActivityEvent) -> String {
    render_event(event)
}

fn render_event(event: &ActivityEvent) -> String {
    match event {
        ActivityEvent::Comment { author, .. } => format!("New comment from @{author}"),
        ActivityEvent::ReviewComment { author, .. } => {
            format!("Review comment from @{author}")
        }
        ActivityEvent::Review { author, state, .. } => format!("Review {state} by @{author}"),
        ActivityEvent::CiFailure { failed_checks } if !failed_checks.is_empty() => {
            format!("CI failing: {}", failed_checks.join(", "))
        }
        ActivityEvent::CiFailure { .. } => "CI failing".to_string(),
        ActivityEvent::CiRecovery => "CI recovered".to_string(),
        ActivityEvent::Opened { author } => format!("New pull request by @{author}"),
        ActivityEvent::StateChange { state, .. } => match state {
            PrState::Merged => "Merged".to_string(),
            PrState::Closed => "Closed".to_string(),
            PrState::Open => "Reopened".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_labels_cover_every_state_and_ci_result() {
        assert_eq!(state_label(&PrState::Open), "open");
        assert_eq!(state_label(&PrState::Merged), "merged");
        assert_eq!(state_label(&PrState::Closed), "closed");
        assert_eq!(ci_label(&CiState::Success), "CI passing");
        assert_eq!(ci_label(&CiState::Failure), "CI failing");
        assert_eq!(ci_label(&CiState::Error), "CI failing");
        assert_eq!(ci_label(&CiState::Pending), "CI pending");
        assert_eq!(ci_label(&CiState::None), "CI not started");
    }
}
