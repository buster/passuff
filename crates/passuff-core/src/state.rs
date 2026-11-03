use std::fs;
use std::io::Write;
use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;

use crate::domain::{StateError, WatcherState};

#[async_trait]
pub trait SnapshotStore: Send + Sync {
    async fn load(&self) -> Result<WatcherState, StateError>;
    async fn save(&self, state: &WatcherState) -> Result<(), StateError>;
}

#[derive(Debug, Clone)]
pub struct JsonStateStore {
    path: PathBuf,
}

impl JsonStateStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn quarantine_path(&self) -> PathBuf {
        let stamp = Utc::now().timestamp();
        let name = self
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "state.json".to_string());
        self.path.with_file_name(format!("{name}.corrupt.{stamp}"))
    }

    fn quarantine(&self) -> Result<(), StateError> {
        fs::rename(&self.path, self.quarantine_path()).map_err(|error| StateError::Quarantine {
            path: self.path.clone(),
            message: error.to_string(),
        })
    }
}

fn normalize_terminal_states(state: &mut WatcherState) {
    for account in state.accounts.values_mut() {
        for pull_request in account.pull_requests.values_mut() {
            if pull_request.current.state.is_terminal() {
                pull_request.current.state_reported = true;
            }
        }
    }
}

fn normalize_next_activity_id(state: &mut WatcherState) {
    let max_id = state
        .activity
        .iter()
        .filter_map(|record| record.id.strip_prefix('a'))
        .filter_map(|suffix| suffix.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    state.next_activity_id = state.next_activity_id.max(max_id);
}

#[async_trait]
impl SnapshotStore for JsonStateStore {
    async fn load(&self) -> Result<WatcherState, StateError> {
        tracing::debug!(path = %self.path.display(), "loading state");
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(WatcherState {
                    version: 2,
                    accounts: Default::default(),
                    activity: Vec::new(),
                    next_activity_id: 0,
                });
            }
            Err(error) => {
                return Err(StateError::Read {
                    path: self.path.clone(),
                    message: error.to_string(),
                });
            }
        };

        let parsed: Result<WatcherState, _> = serde_json::from_str(&text);
        match parsed {
            Ok(mut state) if state.version == 1 || state.version == 2 => {
                let migrate = state.version == 1;
                state.version = 2;
                normalize_terminal_states(&mut state);
                normalize_next_activity_id(&mut state);
                if migrate {
                    self.save(&state).await?;
                }
                tracing::debug!(accounts = state.accounts.len(), "state loaded");
                Ok(state)
            }
            Ok(_) | Err(_) => {
                self.quarantine()?;
                Ok(WatcherState {
                    version: 2,
                    accounts: Default::default(),
                    activity: Vec::new(),
                    next_activity_id: 0,
                })
            }
        }
    }

    async fn save(&self, state: &WatcherState) -> Result<(), StateError> {
        tracing::debug!(accounts = state.accounts.len(), "saving state");
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| StateError::Write {
                path: self.path.clone(),
                message: error.to_string(),
            })?;
        }
        let payload = serde_json::to_vec_pretty(state).map_err(|error| StateError::Write {
            path: self.path.clone(),
            message: error.to_string(),
        })?;
        let temporary = self.path.with_file_name(format!(
            ".{}.{}.tmp",
            self.path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "state.json".to_string()),
            std::process::id()
        ));
        {
            let mut file = fs::File::create(&temporary).map_err(|error| StateError::Write {
                path: self.path.clone(),
                message: error.to_string(),
            })?;
            file.write_all(&payload)
                .map_err(|error| StateError::Write {
                    path: self.path.clone(),
                    message: error.to_string(),
                })?;
            file.sync_all().map_err(|error| StateError::Write {
                path: self.path.clone(),
                message: error.to_string(),
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o600);
                fs::set_permissions(&temporary, permissions).map_err(|error| {
                    StateError::Write {
                        path: self.path.clone(),
                        message: error.to_string(),
                    }
                })?;
            }
        }
        fs::rename(&temporary, &self.path).map_err(|error| StateError::Write {
            path: self.path.clone(),
            message: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AccountState, ActivityRecord, AuthoredPullRequest, CiResult, PullRequestKey,
        WatchedPullRequest,
    };

    fn merged_pull_request() -> AuthoredPullRequest {
        AuthoredPullRequest {
            key: PullRequestKey::new("acme/api".to_string(), 42),
            node_id: "node-42".to_string(),
            author: "octo".to_string(),
            title: "Pull request".to_string(),
            url: "https://github.com/acme/api/pull/42".to_string(),
            created_at: "2026-11-02T09:00:00Z".to_string(),
            updated_at: "2026-11-02T09:00:00Z".to_string(),
            state: crate::domain::PrState::Merged,
            merged: true,
            is_draft: false,
            assignees: Vec::new(),
            labels: Vec::new(),
            milestone: None,
            base_ref: "main".to_string(),
            stack: None,
            head_oid: "abc123".to_string(),
            ci: CiResult::default(),
            comments: Vec::new(),
            reviews: Vec::new(),
            review_comments: Vec::new(),
        }
    }

    #[tokio::test]
    async fn missing_file_is_empty_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonStateStore::new(dir.path().join("state.json"));
        let state = store.load().await.unwrap();
        assert_eq!(state.version, 2);
        assert!(state.accounts.is_empty());
    }

    #[tokio::test]
    async fn round_trips_state_and_quarantines_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let store = JsonStateStore::new(&path);
        let mut state = WatcherState {
            version: 1,
            accounts: Default::default(),
            activity: Vec::new(),
            next_activity_id: 0,
        };
        state.accounts.insert(
            "personal".to_string(),
            AccountState {
                login: "octo-personal".to_string(),
                last_check_at: None,
                pull_requests: Default::default(),
            },
        );
        store.save(&state).await.unwrap();
        let loaded = store.load().await.unwrap();
        assert_eq!(loaded.accounts["personal"].login, "octo-personal");

        fs::write(&path, "{ not json").unwrap();
        let loaded = store.load().await.unwrap();
        assert!(loaded.accounts.is_empty());
        let quarantined = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains(".corrupt."));
        assert!(quarantined);
    }

    #[tokio::test]
    async fn unknown_version_is_quarantined_and_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let store = JsonStateStore::new(&path);
        fs::write(&path, r#"{"version": 999, "accounts": {}}"#).unwrap();
        let loaded = store.load().await.unwrap();
        assert_eq!(loaded.version, 2);
        assert!(loaded.accounts.is_empty());
        let quarantined = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains(".corrupt."));
        assert!(quarantined);
    }

    #[tokio::test]
    async fn version_one_migrates_and_preserves_watched_pull_requests() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let merged = merged_pull_request();
        let watched =
            WatchedPullRequest::from_baseline(&merged, "2026-11-02T10:00:00Z".to_string());
        let mut state = WatcherState {
            version: 1,
            accounts: Default::default(),
            activity: Vec::new(),
            next_activity_id: 0,
        };
        state.accounts.insert(
            "work".to_string(),
            AccountState {
                login: "octo".to_string(),
                last_check_at: Some("2026-11-02T10:00:00Z".to_string()),
                pull_requests: [("acme/api#42".to_string(), watched)].into(),
            },
        );
        std::fs::write(&path, serde_json::to_string(&state).unwrap()).unwrap();
        let store = JsonStateStore::new(&path);
        let loaded = store.load().await.unwrap();
        assert_eq!(loaded.version, 2);
        assert!(loaded.accounts["work"]
            .pull_requests
            .contains_key("acme/api#42"));
        assert!(loaded.activity.is_empty());
        let on_disk: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(on_disk["version"], 2);
    }

    #[tokio::test]
    async fn next_activity_id_is_recomputed_from_existing_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let state = WatcherState {
            version: 2,
            accounts: Default::default(),
            activity: vec![ActivityRecord {
                id: "a7".to_string(),
                key: PullRequestKey::new("acme/api".to_string(), 42),
                observed_at: "2026-11-02T10:00:00Z".to_string(),
                event: crate::domain::ActivityEvent::CiRecovery,
                read: false,
            }],
            next_activity_id: 3,
        };
        std::fs::write(&path, serde_json::to_string(&state).unwrap()).unwrap();
        let store = JsonStateStore::new(&path);
        let loaded = store.load().await.unwrap();
        assert_eq!(loaded.next_activity_id, 7);
    }

    #[tokio::test]
    async fn a_legacy_terminal_observation_is_marked_reported_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let merged = merged_pull_request();
        let mut watched = WatchedPullRequest::from_baseline(&merged, "checked".to_string());
        watched.current.state_reported = false;
        let mut state = WatcherState {
            version: 1,
            accounts: Default::default(),
            activity: Vec::new(),
            next_activity_id: 0,
        };
        state.accounts.insert(
            "work".to_string(),
            AccountState {
                login: "octo".to_string(),
                last_check_at: Some("2026-11-02T10:00:00Z".to_string()),
                pull_requests: [("acme/api#42".to_string(), watched)].into(),
            },
        );
        let store = JsonStateStore::new(&path);
        store.save(&state).await.unwrap();
        let loaded = store.load().await.unwrap();
        assert!(
            loaded.accounts["work"].pull_requests["acme/api#42"]
                .current
                .state_reported
        );
    }

    #[tokio::test]
    async fn save_reports_a_write_failure() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("blocker"), b"x").unwrap();
        let store = JsonStateStore::new(dir.path().join("blocker/state.json"));
        let state = WatcherState {
            version: 1,
            accounts: Default::default(),
            activity: Vec::new(),
            next_activity_id: 0,
        };
        assert!(matches!(
            store.save(&state).await,
            Err(StateError::Write { .. })
        ));
    }
}
