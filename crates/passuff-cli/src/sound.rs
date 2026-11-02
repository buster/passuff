use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use passuff_config::SoundFile;

const BUNDLED_WAV: &[u8] = include_bytes!("../assets/notification.wav");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundOutcome {
    Played,
    Disabled,
    Unavailable,
    Failed(String),
}

#[async_trait]
pub trait SoundPlayer: Send + Sync {
    async fn play(&self, file: &SoundFile) -> SoundOutcome;
}

#[derive(Debug, Clone)]
pub struct SystemSoundPlayer {
    bundled_path: PathBuf,
}

impl SystemSoundPlayer {
    pub fn new() -> Self {
        let bundled_path = cache_path();
        if let Some(parent) = bundled_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&bundled_path, BUNDLED_WAV).is_err() {
            let fallback = std::env::temp_dir().join("passuff-notification.wav");
            let _ = std::fs::write(&fallback, BUNDLED_WAV);
            return Self {
                bundled_path: fallback,
            };
        }
        Self { bundled_path }
    }

    fn path_for(&self, file: &SoundFile) -> Option<PathBuf> {
        match file {
            SoundFile::Default => Some(self.bundled_path.clone()),
            SoundFile::Path(path) => Some(path.clone()),
        }
    }
}

#[async_trait]
impl SoundPlayer for SystemSoundPlayer {
    async fn play(&self, file: &SoundFile) -> SoundOutcome {
        let Some(path) = self.path_for(file) else {
            return SoundOutcome::Unavailable;
        };
        if !Path::new(&path).is_file() {
            return SoundOutcome::Failed(format!("sound file does not exist: {}", path.display()));
        }
        let players: &[&str] = if cfg!(target_os = "macos") {
            &["afplay"]
        } else {
            &["paplay", "pw-play", "aplay", "canberra-gtk-play"]
        };
        let mut last_error = None;
        for player in players {
            let mut command = tokio::process::Command::new(player);
            if *player == "canberra-gtk-play" {
                command.arg("-f").arg(&path);
            } else {
                command.arg(&path);
            }
            match tokio::time::timeout(Duration::from_secs(5), command.status()).await {
                Ok(Ok(status)) if status.success() => return SoundOutcome::Played,
                Ok(Ok(status)) => {
                    last_error = Some(format!("{player} exited with {status}"));
                }
                Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Ok(Err(error)) => {
                    last_error = Some(format!("{player} failed: {error}"));
                }
                Err(_) => {
                    last_error = Some(format!("{player} timed out"));
                }
            }
        }
        match last_error {
            Some(error) => SoundOutcome::Failed(error),
            None => SoundOutcome::Unavailable,
        }
    }
}

#[cfg(feature = "test-support")]
#[derive(Debug, Clone)]
pub struct RecordingSoundPlayer {
    log: Arc<Mutex<Vec<String>>>,
}

#[cfg(feature = "test-support")]
impl RecordingSoundPlayer {
    pub fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self { log }
    }
}

#[cfg(feature = "test-support")]
#[async_trait]
impl SoundPlayer for RecordingSoundPlayer {
    async fn play(&self, file: &SoundFile) -> SoundOutcome {
        self.log.lock().expect("sound log").push(match file {
            SoundFile::Default => "default".to_string(),
            SoundFile::Path(path) => path.display().to_string(),
        });
        SoundOutcome::Played
    }
}

fn cache_path() -> PathBuf {
    if let Some(dirs) = directories::ProjectDirs::from("dev", "passuff", "passuff") {
        if let Some(cache) = dirs.cache_dir().to_str() {
            return PathBuf::from(cache).join("notification.wav");
        }
    }
    std::env::temp_dir().join("passuff-notification.wav")
}

#[cfg(feature = "test-support")]
#[derive(Debug, Clone, Copy)]
pub struct UnavailableSoundPlayer;

#[cfg(feature = "test-support")]
#[async_trait]
impl SoundPlayer for UnavailableSoundPlayer {
    async fn play(&self, _file: &SoundFile) -> SoundOutcome {
        SoundOutcome::Unavailable
    }
}
