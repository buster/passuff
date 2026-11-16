use std::path::PathBuf;

use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct PassuffPaths {
    pub config: PathBuf,
    pub state: PathBuf,
    pub ui: PathBuf,
}

impl PassuffPaths {
    pub fn resolve(config: Option<PathBuf>, state: Option<PathBuf>) -> Self {
        let project = ProjectDirs::from("dev", "passuff", "passuff");
        let config_default = project
            .as_ref()
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("passuff.toml"));
        let state_default = project
            .as_ref()
            .and_then(|dirs| dirs.state_dir().map(|dir| dir.join("state.json")))
            .or_else(|| {
                project
                    .as_ref()
                    .map(|dirs| dirs.data_local_dir().join("state.json"))
            })
            .unwrap_or_else(|| PathBuf::from("passuff-state.json"));
        let state = state.unwrap_or(state_default);
        let ui = state
            .parent()
            .map(|parent| parent.join("ui.json"))
            .unwrap_or_else(|| PathBuf::from("ui.json"));
        Self {
            config: config.unwrap_or(config_default),
            state,
            ui,
        }
    }
}

pub fn bundled_sound_path() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("dev", "passuff", "passuff") {
        return dirs.cache_dir().join("notification.wav");
    }
    PathBuf::from("notification.wav")
}

pub const BUNDLED_SOUND_BYTES: &[u8] = include_bytes!("../assets/notification.wav");

pub fn ensure_bundled_sound() -> Result<PathBuf, String> {
    let path = bundled_sound_path();
    if path.is_file() {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(&path, BUNDLED_SOUND_BYTES).map_err(|error| error.to_string())?;
    Ok(path)
}
