use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use passuff_config::Urgency;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopMessage {
    pub title: String,
    pub body: String,
    pub urgency: String,
}

pub trait Delivery: Send + Sync {
    fn deliver(&self, message: &DesktopMessage) -> Result<(), String>;
}

#[derive(Default)]
pub struct RecordingDelivery {
    messages: Mutex<Vec<DesktopMessage>>,
    available: AtomicBool,
}

impl RecordingDelivery {
    pub fn available() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
            available: AtomicBool::new(true),
        }
    }

    pub fn unavailable() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
            available: AtomicBool::new(false),
        }
    }

    pub fn set_available(&self, available: bool) {
        self.available.store(available, Ordering::SeqCst);
    }

    pub fn messages(&self) -> Vec<DesktopMessage> {
        self.messages
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.clear();
        }
    }
}

impl Delivery for RecordingDelivery {
    fn deliver(&self, message: &DesktopMessage) -> Result<(), String> {
        if !self.available.load(Ordering::SeqCst) {
            return Err("notification service is unavailable".to_string());
        }
        if let Ok(mut messages) = self.messages.lock() {
            messages.push(message.clone());
        }
        Ok(())
    }
}

pub struct FileDelivery {
    path: std::path::PathBuf,
}

impl FileDelivery {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Delivery for FileDelivery {
    fn deliver(&self, message: &DesktopMessage) -> Result<(), String> {
        let line = serde_json::to_string(message).map_err(|error| error.to_string())?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| error.to_string())?;
        writeln!(file, "{line}").map_err(|error| error.to_string())
    }
}

pub struct SystemDelivery;

impl Delivery for SystemDelivery {
    fn deliver(&self, message: &DesktopMessage) -> Result<(), String> {
        let mut notification = notify_rust::Notification::new();
        notification.summary(&message.title).body(&message.body);
        notification.urgency(match message.urgency.as_str() {
            "low" => notify_rust::Urgency::Low,
            "critical" => notify_rust::Urgency::Critical,
            _ => notify_rust::Urgency::Normal,
        });
        notification
            .show()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

pub fn urgency_label(urgency: Urgency) -> &'static str {
    match urgency {
        Urgency::Low => "low",
        Urgency::Normal => "normal",
        Urgency::Critical => "critical",
    }
}
