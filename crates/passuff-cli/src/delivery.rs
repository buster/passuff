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

        let mut notification = notify_rust::Notification::new();
        notification
            .appname("passuff")
            .summary(&rendered.title)
            .body(&rendered.body);
        #[cfg(not(target_os = "macos"))]
        {
            let urgency = match config.urgency {
                Urgency::Low => notify_rust::Urgency::Low,
                Urgency::Normal => notify_rust::Urgency::Normal,
                Urgency::Critical => notify_rust::Urgency::Critical,
            };
            notification.urgency(urgency);
        }
        if matches!(report.sound, Some(SoundOutcome::Unavailable)) {
            notification.sound_name("message-new-instant");
        }
        match notification.show() {
            Ok(_) => report.desktop_delivered = true,
            Err(error) => report
                .errors
                .push(format!("desktop notification failed: {error}")),
        }
        report
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
