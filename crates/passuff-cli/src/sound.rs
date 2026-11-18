//! Sound playback through a system player. The bundled sound is embedded and
//! written to a temporary file only when the workspace asset is not present at
//! runtime.

use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use crate::delivery::find_in_path;

const BUNDLED: &[u8] = include_bytes!("../assets/notification.wav");
const ASSET_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/notification.wav");

#[derive(Debug)]
pub enum SoundError {
    Unavailable(String),
    Failed(String),
}

impl fmt::Display for SoundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(detail) => {
                write!(formatter, "sound unavailable: {detail}")
            }
            Self::Failed(detail) => write!(formatter, "sound failed: {detail}"),
        }
    }
}

pub fn play(sound_file: &str) -> Result<(), SoundError> {
    let file = resolve_sound_file(sound_file)?;
    let (program, mut arguments) = player()
        .ok_or_else(|| SoundError::Unavailable("no system audio player was found".to_string()))?;
    arguments.push(file.to_string_lossy().into_owned());
    let output = Command::new(&program)
        .args(&arguments)
        .output()
        .map_err(|error| {
            SoundError::Unavailable(format!("cannot start {}: {error}", program.display()))
        })?;
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(SoundError::Failed(if detail.is_empty() {
            format!("{} exited with {}", program.display(), output.status)
        } else {
            detail
        }))
    }
}

fn resolve_sound_file(sound_file: &str) -> Result<PathBuf, SoundError> {
    if sound_file == "default" {
        let asset = PathBuf::from(ASSET_PATH);
        if asset.is_file() {
            return Ok(asset);
        }
        let temporary =
            std::env::temp_dir().join(format!("passuff-notification-{}.wav", std::process::id()));
        std::fs::write(&temporary, BUNDLED).map_err(|error| {
            SoundError::Unavailable(format!("cannot write bundled sound: {error}"))
        })?;
        return Ok(temporary);
    }
    let path = passuff_config::expand_home(sound_file);
    if path.is_file() {
        Ok(path)
    } else {
        Err(SoundError::Unavailable(format!(
            "sound file is missing: {}",
            path.display()
        )))
    }
}

fn player() -> Option<(PathBuf, Vec<String>)> {
    if let Ok(value) = std::env::var("PASSUFF_SOUND_PLAYER") {
        if value.trim().is_empty() {
            return None;
        }
        let direct = PathBuf::from(&value);
        if direct.is_file() {
            return Some((direct, Vec::new()));
        }
        let mut parts = value.split_whitespace();
        let program = parts.next()?;
        let arguments = parts.map(str::to_string).collect();
        let program = find_in_path(program)?;
        return Some((program, arguments));
    }

    #[cfg(target_os = "macos")]
    let candidates = ["afplay"];
    #[cfg(target_os = "linux")]
    let candidates = ["paplay", "pw-play", "aplay", "canberra-gtk-play"];
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let candidates: [&str; 0] = [];

    for candidate in candidates {
        if let Some(program) = find_in_path(candidate) {
            return Some((program, Vec::new()));
        }
    }
    None
}

/// The bundled sound path as documented by the product.
pub fn bundled_path() -> PathBuf {
    PathBuf::from(ASSET_PATH)
}
