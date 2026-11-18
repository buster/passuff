//! Versioned JSON watcher state with atomic replacement and quarantine of
//! corrupt or unknown-version files.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use crate::model::{AccountState, CiResult, PrState, WatchedPullRequest, WatcherState};

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug)]
pub struct LoadedState {
    pub state: WatcherState,
    pub existed: bool,
    /// The file was migrated or quarantined and must be written back.
    pub needs_save: bool,
}

#[derive(Debug, Deserialize)]
struct VersionOneState {
    #[allow(dead_code)]
    version: u32,
    #[serde(default)]
    accounts: BTreeMap<String, VersionOneAccount>,
}

#[derive(Debug, Deserialize)]
struct VersionOneAccount {
    #[serde(default)]
    login: Option<String>,
    #[serde(default)]
    last_check: Option<DateTime<Utc>>,
    #[serde(default)]
    pull_requests: BTreeMap<String, VersionOnePullRequest>,
}

#[derive(Debug, Deserialize)]
struct VersionOnePullRequest {
    state: PrState,
    ci: CiResult,
    #[serde(default)]
    title: String,
}

pub fn load(path: &Path) -> Result<LoadedState, StateError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LoadedState {
                state: WatcherState::default(),
                existed: false,
                needs_save: false,
            });
        }
        Err(error) => {
            return Err(StateError::Io(format!(
                "cannot read state {}: {error}",
                path.display()
            )));
        }
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => return quarantine(path),
    };

    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => return quarantine(path),
    };

    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or_default();

    match version {
        1 => {
            let version_one: VersionOneState = match serde_json::from_value(value) {
                Ok(state) => state,
                Err(_) => return quarantine(path),
            };
            let mut state = WatcherState::default();
            let now = Utc::now();
            for (label, account) in version_one.accounts {
                let mut migrated = AccountState {
                    login: account.login,
                    last_check: account.last_check,
                    pull_requests: BTreeMap::new(),
                };
                for (key, pull_request) in account.pull_requests {
                    let mut watched = WatchedPullRequest::base_entry(
                        &key,
                        pull_request.state,
                        pull_request.ci,
                        &pull_request.title,
                    );
                    if pull_request.state.is_terminal() {
                        watched.terminal_at = Some(now);
                    }
                    migrated.pull_requests.insert(key, watched);
                }
                state.accounts.insert(label, migrated);
            }
            Ok(LoadedState {
                state,
                existed: true,
                needs_save: true,
            })
        }
        2 => match serde_json::from_value::<WatcherState>(value) {
            Ok(mut state) => {
                state.version = 2;
                normalize(&mut state);
                Ok(LoadedState {
                    state,
                    existed: true,
                    needs_save: false,
                })
            }
            Err(_) => quarantine(path),
        },
        _ => quarantine(path),
    }
}

fn normalize(state: &mut WatcherState) {
    if state.next_activity_id == 0 {
        let highest = state
            .activity
            .iter()
            .filter_map(|record| record.id.strip_prefix('a'))
            .filter_map(|number| number.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        state.next_activity_id = highest + 1;
    }
    state.cap_activity();
}

fn quarantine(path: &Path) -> Result<LoadedState, StateError> {
    let destination = quarantine_path(path);
    if let Err(error) = fs::rename(path, &destination) {
        return Err(StateError::Io(format!(
            "cannot set aside corrupt state {}: {error}",
            path.display()
        )));
    }
    Ok(LoadedState {
        state: WatcherState::default(),
        existed: true,
        needs_save: true,
    })
}

pub fn quarantine_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_else(|| "state.json".into());
    name.push(".corrupt");
    path.with_file_name(name)
}

pub fn save(path: &Path, state: &WatcherState) -> Result<(), StateError> {
    let mut state = state.clone();
    state.version = 2;
    let text = serde_json::to_string_pretty(&state)
        .map_err(|error| StateError::Invalid(format!("cannot serialize state: {error}")))?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            StateError::Io(format!(
                "cannot create state directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    let temporary = temporary_path(path);
    if let Err(error) = fs::write(&temporary, text) {
        let _ = fs::remove_file(&temporary);
        return Err(StateError::Io(format!(
            "cannot write state {}: {error}",
            path.display()
        )));
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(StateError::Io(format!(
            "cannot replace state {}: {error}",
            path.display()
        )));
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "state.json".to_string());
    path.with_file_name(format!(".{name}.tmp-{}", std::process::id()))
}
