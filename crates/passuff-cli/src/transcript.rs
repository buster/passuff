use passuff_core::{
    AccountFailure, ActivityEvent, Notification, PrState, PullRequestKey, RefreshReport,
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

    pub fn render_report(&self, report: &RefreshReport, sound: bool) -> String {
        let mut lines = Vec::new();
        for key in &report.baselines_recorded {
            lines.push(self.render_baseline(key));
        }
        for notification in &report.notifications {
            lines.push(self.render_notification(notification, sound).transcript);
        }
        for failure in &report.account_failures {
            lines.push(self.render_account_failure(failure));
        }
        if report.notifications.is_empty() && report.baselines_recorded.is_empty() {
            lines.push(self.render_no_changes());
        }
        lines.join("\n")
    }
}

fn render_event(event: &ActivityEvent) -> String {
    match event {
        ActivityEvent::Comment { author, .. } => format!("New comment from @{author}"),
        ActivityEvent::ReviewComment { author, .. } => {
            format!("Review comment from @{author}")
        }
        ActivityEvent::Review { author, state } => format!("Review {state} by @{author}"),
        ActivityEvent::CiFailure { failed_checks } if !failed_checks.is_empty() => {
            format!("CI failing: {}", failed_checks.join(", "))
        }
        ActivityEvent::CiFailure { .. } => "CI failing".to_string(),
        ActivityEvent::CiRecovery => "CI recovered".to_string(),
        ActivityEvent::StateChange { state } => match state {
            PrState::Merged => "Merged".to_string(),
            PrState::Closed => "Closed".to_string(),
            PrState::Open => "Reopened".to_string(),
        },
    }
}
