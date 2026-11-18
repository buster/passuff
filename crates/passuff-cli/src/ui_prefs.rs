//! TUI preferences in `ui.json`, next to the state file.

use std::path::Path;

use passuff_config::AppConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortMode {
    Updated,
    Name,
    State,
    Age,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Updated => "updated",
            Self::Name => "name",
            Self::State => "state",
            Self::Age => "age",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Updated => Self::Name,
            Self::Name => Self::State,
            Self::State => Self::Age,
            Self::Age => Self::Updated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrefs {
    #[serde(default = "default_sort")]
    pub sort: SortMode,
    #[serde(default = "default_true")]
    pub ascending: bool,
    #[serde(default)]
    pub unread_only: bool,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub show_ignored: bool,
    #[serde(default = "default_true")]
    pub desktop: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
}

fn default_sort() -> SortMode {
    SortMode::Updated
}

fn default_true() -> bool {
    true
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            sort: SortMode::Updated,
            ascending: true,
            unread_only: false,
            filter: None,
            show_ignored: false,
            desktop: true,
            sound: true,
        }
    }
}

impl UiPrefs {
    pub fn load(path: &Path, config: &AppConfig) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| Self::from_config(config)),
            Err(_) => Self::from_config(config),
        }
    }

    fn from_config(config: &AppConfig) -> Self {
        Self {
            desktop: config.notifications.desktop,
            sound: config.notifications.sound,
            ..Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("cannot create directory: {error}"))?;
            }
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|error| format!("cannot serialize preferences: {error}"))?;
        let temporary = path.with_file_name(format!(".ui.json.tmp-{}", std::process::id()));
        std::fs::write(&temporary, text)
            .map_err(|error| format!("cannot write preferences: {error}"))?;
        std::fs::rename(&temporary, path)
            .map_err(|error| format!("cannot replace preferences: {error}"))
    }
}
