//! Desktop notification delivery. `PASSUFF_NOTIFICATION_LOG` replaces the
//! platform surface and appends one JSON object per digest.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use passuff_config::Urgency;

use crate::render::Digest;

#[derive(Debug, Clone)]
pub struct Surfaces {
    pub desktop: bool,
    pub sound: bool,
    pub urgency: Urgency,
    pub sound_file: String,
}

#[derive(Debug, Default)]
pub struct DeliveryReport {
    pub desktop_error: Option<String>,
    pub sound_error: Option<String>,
}

pub fn deliver(digest: &Digest, surfaces: &Surfaces) -> DeliveryReport {
    let mut report = DeliveryReport::default();
    if surfaces.desktop {
        if let Err(error) = send_desktop(&digest.title, &digest.body, surfaces.urgency) {
            report.desktop_error = Some(error);
        }
    }
    if surfaces.sound {
        if let Err(error) = crate::sound::play(&surfaces.sound_file) {
            report.sound_error = Some(error.to_string());
        }
    }
    report
}

pub fn send_desktop(title: &str, body: &str, urgency: Urgency) -> Result<(), String> {
    if let Ok(log) = std::env::var("PASSUFF_NOTIFICATION_LOG") {
        if !log.is_empty() {
            return append_log(Path::new(&log), title, body, urgency);
        }
    }

    #[cfg(target_os = "linux")]
    {
        send_linux(title, body, urgency)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = urgency;
        send_macos(title, body)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (title, body, urgency);
        Err("no desktop notification surface on this platform".to_string())
    }
}

fn append_log(path: &Path, title: &str, body: &str, urgency: Urgency) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create notification log directory: {error}"))?;
        }
    }
    #[derive(serde::Serialize)]
    struct LogLine<'a> {
        title: &'a str,
        body: &'a str,
        urgency: &'a str,
    }
    let line = serde_json::to_string(&LogLine {
        title,
        body: body.trim_end_matches('\n'),
        urgency: urgency.as_str(),
    })
    .map_err(|error| format!("cannot serialize notification: {error}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("cannot open notification log {}: {error}", path.display()))?;
    writeln!(file, "{line}")
        .map_err(|error| format!("cannot write notification log {}: {error}", path.display()))
}

#[cfg(target_os = "linux")]
fn send_linux(title: &str, body: &str, urgency: Urgency) -> Result<(), String> {
    let Some(program) = find_in_path("notify-send") else {
        return Err("notify-send is unavailable".to_string());
    };
    let status = Command::new(program)
        .arg(format!("--urgency={}", urgency.as_str()))
        .arg("--app-name=passuff")
        .arg(title)
        .arg(body)
        .output()
        .map_err(|error| format!("cannot start notify-send: {error}"))?;
    if status.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&status.stderr).trim().to_string();
        Err(if detail.is_empty() {
            "notify-send failed".to_string()
        } else {
            format!("notify-send failed: {detail}")
        })
    }
}

#[cfg(target_os = "macos")]
fn send_macos(title: &str, body: &str) -> Result<(), String> {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        escape_applescript(body),
        escape_applescript(title)
    );
    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|error| format!("cannot start osascript: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("osascript failed".to_string())
    }
}

#[cfg(target_os = "macos")]
fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn find_in_path(program: &str) -> Option<PathBuf> {
    let path = Path::new(program);
    if path.components().count() > 1 || path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    let search = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&search) {
        let candidate = directory.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
