use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    pub config: PathBuf,
    pub state: PathBuf,
}

impl Paths {
    pub fn resolve(config: Option<PathBuf>, state: Option<PathBuf>) -> Self {
        Self {
            config: config.unwrap_or_else(passuff_config::config_path),
            state: state.unwrap_or_else(passuff_config::state_path),
        }
    }

    pub fn ui(&self) -> PathBuf {
        self.state.with_file_name("ui.json").to_path_buf()
    }

    pub fn ui_for(state: &Path) -> PathBuf {
        state.with_file_name("ui.json")
    }
}
