use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub trait SoundPlayer: Send + Sync {
    fn play(&self, file: &Path) -> Result<(), String>;
}

#[derive(Default)]
pub struct RecordingSoundPlayer {
    plays: Mutex<Vec<PathBuf>>,
    available: AtomicBool,
}

impl RecordingSoundPlayer {
    pub fn available() -> Self {
        Self {
            plays: Mutex::new(Vec::new()),
            available: AtomicBool::new(true),
        }
    }

    pub fn unavailable() -> Self {
        Self {
            plays: Mutex::new(Vec::new()),
            available: AtomicBool::new(false),
        }
    }

    pub fn set_available(&self, available: bool) {
        self.available.store(available, Ordering::SeqCst);
    }

    pub fn plays(&self) -> Vec<PathBuf> {
        self.plays
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut plays) = self.plays.lock() {
            plays.clear();
        }
    }
}

impl SoundPlayer for RecordingSoundPlayer {
    fn play(&self, file: &Path) -> Result<(), String> {
        if !self.available.load(Ordering::SeqCst) {
            return Err("no sound player available".to_string());
        }
        if let Ok(mut plays) = self.plays.lock() {
            plays.push(file.to_path_buf());
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SystemSoundPlayer;

impl SoundPlayer for SystemSoundPlayer {
    fn play(&self, file: &Path) -> Result<(), String> {
        if let Ok(player) = std::env::var("PASSUFF_SOUND_PLAYER") {
            return run_player(&player, &[file.as_os_str().to_owned()]);
        }
        #[cfg(target_os = "macos")]
        {
            return run_player("afplay", &[file.as_os_str().to_owned()]);
        }
        #[cfg(target_os = "linux")]
        {
            let file_arg = file.as_os_str().to_owned();
            for player in ["paplay", "pw-play", "aplay"] {
                if command_exists(player) {
                    if let Ok(()) = run_player(player, std::slice::from_ref(&file_arg)) {
                        return Ok(());
                    }
                }
            }
            if command_exists("canberra-gtk-play") {
                return run_player(
                    "canberra-gtk-play",
                    &[
                        std::ffi::OsString::from("--file"),
                        file.as_os_str().to_owned(),
                    ],
                );
            }
            Err("no sound player available".to_string())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let _ = file;
            Err("sound is not supported on this platform".to_string())
        }
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_player(name: &str, args: &[std::ffi::OsString]) -> Result<(), String> {
    let status = Command::new(name)
        .args(args)
        .status()
        .map_err(|error| format!("{name}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{name} exited with {status}"))
    }
}
