use std::process::ExitCode;
use std::sync::Arc;

use passuff_config::{AppConfig, ConfigError};

use crate::delivery::NotificationDelivery;
use crate::sound::SoundOutcome;
use crate::transcript::TranscriptRenderer;
use crate::RunOutcome;

/// Send one notification through the configured desktop and sound surfaces.
pub async fn notify_test(
    config: Result<AppConfig, ConfigError>,
    delivery: Arc<dyn NotificationDelivery>,
    no_desktop: bool,
    no_sound: bool,
) -> RunOutcome {
    let renderer = TranscriptRenderer;
    let mut config = match config {
        Ok(config) => config,
        Err(error) => {
            return RunOutcome {
                exit: ExitCode::from(2),
                status: 2,
                report: None,
                transcript: renderer.render_config_error(&error.to_string()),
                emitted: false,
            };
        }
    };
    if no_desktop {
        config.notifications.desktop = false;
    }
    if no_sound {
        config.notifications.sound = false;
    }

    let rendered = renderer.render_test_notification(config.notifications.sound);
    let report = delivery.deliver(&rendered, &config.notifications).await;
    let mut lines = Vec::new();
    let mut failed = false;
    let mut enabled = false;

    if config.notifications.desktop {
        enabled = true;
        if report.desktop_delivered {
            lines.push("passuff: desktop notification sent".to_string());
        } else {
            failed = true;
            if report.errors.is_empty() {
                lines.push(renderer.render_delivery_error("the notification was not delivered"));
            } else {
                for error in &report.errors {
                    if let Some(detail) = error.strip_prefix("desktop notification failed: ") {
                        lines.push(renderer.render_delivery_error(detail));
                    } else {
                        lines.push(renderer.render_delivery_error(error));
                    }
                }
            }
        }
    } else {
        lines.push("passuff: desktop notifications are disabled".to_string());
    }

    if config.notifications.sound {
        enabled = true;
        match report.sound.as_ref() {
            Some(SoundOutcome::Played) => lines.push("passuff: sound played".to_string()),
            Some(SoundOutcome::Failed(detail)) => {
                failed = true;
                lines.push(renderer.render_sound_unavailable(detail));
            }
            Some(SoundOutcome::Unavailable) => {
                failed = true;
                lines.push(renderer.render_sound_unavailable("no sound player found"));
            }
            Some(SoundOutcome::Disabled) | None => {
                failed = true;
                lines.push(renderer.render_sound_unavailable("the sound was not played"));
            }
        }
    } else {
        lines.push("passuff: sound is disabled".to_string());
    }

    if !enabled {
        lines.push(
            "passuff: nothing to test: desktop notifications and sound are disabled".to_string(),
        );
    }
    let status = if !enabled {
        2
    } else if failed {
        1
    } else {
        0
    };
    RunOutcome {
        exit: ExitCode::from(status as u8),
        status,
        report: None,
        transcript: lines.join("\n"),
        emitted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::notify_test;
    use crate::delivery::{DeliveryReport, NotificationDelivery};
    use crate::sound::SoundOutcome;
    use crate::transcript::RenderedNotification;
    use async_trait::async_trait;
    use passuff_config::{
        AppConfig, ConfigError, CoreConfig, EventToggles, NotificationsConfig, WatchConfig,
    };
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    struct CapturingDelivery {
        report: DeliveryReport,
        notifications: Mutex<Vec<(String, String)>>,
    }

    impl CapturingDelivery {
        fn new(report: DeliveryReport) -> Self {
            Self {
                report,
                notifications: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl NotificationDelivery for CapturingDelivery {
        async fn deliver(
            &self,
            rendered: &RenderedNotification,
            _config: &NotificationsConfig,
        ) -> DeliveryReport {
            self.notifications
                .lock()
                .expect("notifications")
                .push((rendered.title.clone(), rendered.body.clone()));
            self.report.clone()
        }
    }

    fn config(notifications: NotificationsConfig) -> AppConfig {
        AppConfig {
            poll_interval_secs: 60,
            reconciliation_interval_secs: 3600,
            core: CoreConfig {
                accounts: BTreeMap::new(),
                events: EventToggles::default(),
                watch: WatchConfig::default(),
            },
            notifications,
        }
    }

    fn delivery(report: DeliveryReport) -> Arc<CapturingDelivery> {
        Arc::new(CapturingDelivery::new(report))
    }

    #[tokio::test]
    async fn a_test_notification_reports_every_enabled_surface() {
        let delivery = delivery(DeliveryReport {
            desktop_delivered: true,
            sound: Some(SoundOutcome::Played),
            errors: Vec::new(),
        });
        let outcome = notify_test(
            Ok(config(NotificationsConfig::default())),
            delivery.clone(),
            false,
            false,
        )
        .await;
        assert_eq!(outcome.status, 0);
        assert!(outcome.transcript.contains("desktop notification sent"));
        assert!(outcome.transcript.contains("sound played"));
        let notifications = delivery.notifications.lock().expect("notifications");
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].0, "passuff: notify test");
        assert!(notifications[0]
            .1
            .contains("passuff notifications are working"));
    }

    #[tokio::test]
    async fn a_test_notification_reports_a_failed_surface() {
        let delivery = delivery(DeliveryReport {
            desktop_delivered: false,
            sound: Some(SoundOutcome::Unavailable),
            errors: vec!["desktop notification failed: permission denied".to_string()],
        });
        let outcome = notify_test(
            Ok(config(NotificationsConfig::default())),
            delivery,
            false,
            false,
        )
        .await;
        assert_eq!(outcome.status, 1);
        assert!(outcome
            .transcript
            .contains("desktop notification failed: permission denied"));
        assert!(outcome.transcript.contains("sound unavailable"));
    }

    #[tokio::test]
    async fn a_test_notification_reports_disabled_surfaces() {
        let delivery = delivery(DeliveryReport::default());
        let outcome = notify_test(
            Ok(config(NotificationsConfig {
                desktop: false,
                sound: false,
                ..NotificationsConfig::default()
            })),
            delivery,
            false,
            false,
        )
        .await;
        assert_eq!(outcome.status, 2);
        assert!(outcome
            .transcript
            .contains("desktop notifications are disabled"));
        assert!(outcome.transcript.contains("sound is disabled"));
    }

    #[tokio::test]
    async fn a_test_notification_reports_a_configuration_error() {
        let outcome = notify_test(
            Err(ConfigError::Validation("invalid configuration".to_string())),
            delivery(DeliveryReport::default()),
            false,
            false,
        )
        .await;
        assert_eq!(outcome.status, 2);
        assert!(outcome.transcript.contains("configuration error"));
    }
}
