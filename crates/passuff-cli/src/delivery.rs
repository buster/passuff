#[cfg(feature = "test-support")]
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "test-support")]
use std::sync::Mutex;

use async_trait::async_trait;
use passuff_config::NotificationsConfig;
#[cfg(not(target_os = "macos"))]
use passuff_config::Urgency;
#[cfg(feature = "test-support")]
use serde_json::json;

use crate::sound::{SoundOutcome, SoundPlayer};
use crate::transcript::RenderedNotification;

#[derive(Debug, Clone, Default)]
pub struct DeliveryReport {
    pub desktop_delivered: bool,
    pub sound: Option<SoundOutcome>,
    pub errors: Vec<String>,
}

#[async_trait]
pub trait NotificationDelivery: Send + Sync {
    async fn deliver(
        &self,
        rendered: &RenderedNotification,
        config: &NotificationsConfig,
    ) -> DeliveryReport;
}

#[derive(Clone)]
pub struct DesktopDelivery {
    sound: Arc<dyn SoundPlayer>,
}

impl DesktopDelivery {
    pub fn new(sound: Arc<dyn SoundPlayer>) -> Self {
        Self { sound }
    }
}

#[async_trait]
impl NotificationDelivery for DesktopDelivery {
    async fn deliver(
        &self,
        rendered: &RenderedNotification,
        config: &NotificationsConfig,
    ) -> DeliveryReport {
        let mut report = DeliveryReport::default();
        let sound = if config.sound {
            Some(self.sound.play(&config.sound_file).await)
        } else {
            Some(SoundOutcome::Disabled)
        };
        report.sound = sound.clone();

        if !config.desktop {
            return report;
        }

        #[cfg(target_os = "macos")]
        {
            match show_macos_notification(&rendered.title, &rendered.body).await {
                Ok(()) => report.desktop_delivered = true,
                Err(error) => report
                    .errors
                    .push(format!("desktop notification failed: {error}")),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut notification = notify_rust::Notification::new();
            notification
                .appname("passuff")
                .summary(&rendered.title)
                .body(&rendered.body);
            let urgency = match config.urgency {
                Urgency::Low => notify_rust::Urgency::Low,
                Urgency::Normal => notify_rust::Urgency::Normal,
                Urgency::Critical => notify_rust::Urgency::Critical,
            };
            notification.urgency(urgency);
            if matches!(report.sound, Some(SoundOutcome::Unavailable)) {
                notification.sound_name("message-new-instant");
            }
            match notification.show() {
                Ok(_) => report.desktop_delivered = true,
                Err(error) => report
                    .errors
                    .push(format!("desktop notification failed: {error}")),
            }
        }
        report
    }
}

#[cfg(any(target_os = "macos", test))]
fn macos_notification_args(title: &str, body: &str) -> Vec<String> {
    vec![
        "-e".to_string(),
        "on run argv".to_string(),
        "-e".to_string(),
        "display notification (item 1 of argv) with title (item 2 of argv)".to_string(),
        "-e".to_string(),
        "end run".to_string(),
        body.to_string(),
        title.to_string(),
    ]
}

#[cfg(target_os = "macos")]
async fn show_macos_notification(title: &str, body: &str) -> Result<(), String> {
    let status = tokio::process::Command::new("osascript")
        .args(macos_notification_args(title, body))
        .status()
        .await
        .map_err(|error| format!("osascript failed: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("osascript exited with {status}"))
    }
}

#[cfg(feature = "test-support")]
#[derive(Clone)]
pub struct RecordingDelivery {
    sound: Arc<dyn SoundPlayer>,
    log: Arc<Mutex<Vec<serde_json::Value>>>,
    sink: Option<PathBuf>,
}

#[cfg(feature = "test-support")]
impl RecordingDelivery {
    pub fn new(sound: Arc<dyn SoundPlayer>, log: Arc<Mutex<Vec<serde_json::Value>>>) -> Self {
        Self {
            sound,
            log,
            sink: None,
        }
    }

    pub fn with_sink(sound: Arc<dyn SoundPlayer>, sink: PathBuf) -> Self {
        Self {
            sound,
            log: Arc::new(Mutex::new(Vec::new())),
            sink: Some(sink),
        }
    }
}

#[cfg(feature = "test-support")]
#[async_trait]
impl NotificationDelivery for RecordingDelivery {
    async fn deliver(
        &self,
        rendered: &RenderedNotification,
        config: &NotificationsConfig,
    ) -> DeliveryReport {
        let mut report = DeliveryReport::default();
        if config.sound {
            report.sound = Some(self.sound.play(&config.sound_file).await);
        } else {
            report.sound = Some(SoundOutcome::Disabled);
        }
        if config.desktop {
            let record = json!({
                "pr": rendered.title.trim_start_matches("passuff: "),
                "title": rendered.title,
                "body": rendered.body,
                "sound": config.sound,
            });
            self.log.lock().expect("delivery log").push(record.clone());
            if let Some(sink) = &self.sink {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(sink)
                {
                    use std::io::Write;
                    let _ = writeln!(file, "{record}");
                }
            }
            report.desktop_delivered = true;
        }
        report
    }
}

#[cfg(feature = "test-support")]
#[derive(Debug, Clone, Copy)]
pub struct FailingDelivery;

#[cfg(feature = "test-support")]
#[async_trait]
impl NotificationDelivery for FailingDelivery {
    async fn deliver(
        &self,
        _rendered: &RenderedNotification,
        _config: &NotificationsConfig,
    ) -> DeliveryReport {
        DeliveryReport {
            desktop_delivered: false,
            sound: None,
            errors: vec!["desktop notification failed: unavailable".to_string()],
        }
    }
}

/// Suppress desktop notifications and/or sound for one run without changing configuration.
#[derive(Clone)]
pub struct MutedDelivery {
    inner: Arc<dyn NotificationDelivery>,
    desktop: bool,
    sound: bool,
}

impl MutedDelivery {
    pub fn new(inner: Arc<dyn NotificationDelivery>, desktop: bool, sound: bool) -> Self {
        Self {
            inner,
            desktop,
            sound,
        }
    }
}

#[async_trait]
impl NotificationDelivery for MutedDelivery {
    async fn deliver(
        &self,
        rendered: &RenderedNotification,
        config: &NotificationsConfig,
    ) -> DeliveryReport {
        if self.desktop && self.sound {
            return self.inner.deliver(rendered, config).await;
        }
        let mut config = config.clone();
        if !self.desktop {
            config.desktop = false;
        }
        if !self.sound {
            config.sound = false;
        }
        self.inner.deliver(rendered, &config).await
    }
}

pub fn mute_delivery(
    delivery: Arc<dyn NotificationDelivery>,
    desktop: bool,
    sound: bool,
) -> Arc<dyn NotificationDelivery> {
    if desktop && sound {
        delivery
    } else {
        Arc::new(MutedDelivery::new(delivery, desktop, sound))
    }
}

/// Build the delivery adapter shared by the watch command and the terminal inbox.
pub fn build_delivery(sound: Arc<crate::sound::SystemSoundPlayer>) -> Arc<dyn NotificationDelivery> {
    #[cfg(feature = "test-support")]
    {
        if let Ok(sink) = std::env::var("PASSUFF_NOTIFICATION_SINK") {
            return Arc::new(RecordingDelivery::with_sink(
                sound,
                std::path::PathBuf::from(sink),
            ));
        }
    }
    Arc::new(DesktopDelivery::new(sound))
}

#[cfg(test)]
mod tests {
    use super::macos_notification_args;

    #[test]
    fn macos_notifications_pass_the_body_then_the_title() {
        let args = macos_notification_args(
            "passuff: notify test",
            "If you can read this, passuff notifications are working.",
        );
        assert!(args.iter().any(|arg| arg == "on run argv"));
        assert!(args
            .iter()
            .any(|arg| arg == "display notification (item 1 of argv) with title (item 2 of argv)"));
        assert_eq!(
            args[args.len() - 2],
            "If you can read this, passuff notifications are working."
        );
        assert_eq!(args[args.len() - 1], "passuff: notify test");
    }
}

#[cfg(test)]
mod mute_tests {
    use super::{DeliveryReport, MutedDelivery, NotificationDelivery};
    use crate::transcript::RenderedNotification;
    use async_trait::async_trait;
    use passuff_config::NotificationsConfig;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Capture {
        seen: Mutex<Option<NotificationsConfig>>,
    }

    #[async_trait]
    impl NotificationDelivery for Capture {
        async fn deliver(
            &self,
            _rendered: &RenderedNotification,
            config: &NotificationsConfig,
        ) -> DeliveryReport {
            *self.seen.lock().expect("seen") = Some(config.clone());
            DeliveryReport::default()
        }
    }

    fn rendered() -> RenderedNotification {
        RenderedNotification {
            title: "title".to_string(),
            body: "body".to_string(),
            transcript: "title\nbody".to_string(),
            sound: true,
        }
    }

    async fn seen(desktop: bool, sound: bool) -> (bool, bool) {
        let capture = Arc::new(Capture::default());
        let inner: Arc<dyn NotificationDelivery> = capture.clone();
        MutedDelivery::new(inner, desktop, sound)
            .deliver(&rendered(), &NotificationsConfig::default())
            .await;
        let config = capture.seen.lock().expect("seen").clone().expect("config");
        (config.desktop, config.sound)
    }

    #[tokio::test]
    async fn muting_the_desktop_keeps_the_sound() {
        assert_eq!(seen(false, true).await, (false, true));
    }

    #[tokio::test]
    async fn muting_the_sound_keeps_the_desktop() {
        assert_eq!(seen(true, false).await, (true, false));
    }

    #[tokio::test]
    async fn leaving_both_enabled_passes_the_config_through() {
        assert_eq!(seen(true, true).await, (true, true));
    }
}
