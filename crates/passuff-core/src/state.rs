use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use passuff_config::{FocusRelation, WatchMode};
use serde::{Deserialize, Serialize};

use crate::model::{
    ActivityEvent, CiResult, EventKind, Fingerprint, PullRequestKey, PullRequestState, ReviewState,
    StackInfo,
};

pub const STATE_VERSION: u32 = 2;
pub const ACTIVITY_LIMIT: usize = 500;

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("cannot read state {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write state {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot quarantine state {path}: {source}")]
    Quarantine {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot decode state: {0}")]
    Decode(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatcherState {
    pub version: u32,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountState>,
    #[serde(default)]
    pub activity: Vec<ActivityRecord>,
    #[serde(default)]
    pub next_activity_id: u64,
    #[serde(default)]
    pub last_scope: Option<ScopeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountState {
    #[serde(default)]
    pub login: Option<String>,
    #[serde(default)]
    pub last_check: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pull_requests: BTreeMap<String, WatchedPullRequest>,
    #[serde(default)]
    pub accepted_probe: BTreeMap<String, Fingerprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedPullRequest {
    #[serde(default = "yes")]
    pub baseline: bool,
    #[serde(default = "open_state")]
    pub state: PullRequestState,
    #[serde(default)]
    pub ci: CiResult,
    #[serde(default)]
    pub terminal_reported: bool,
    #[serde(default)]
    pub ci_failure_seen: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub head_oid: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub merged_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub seen_comments: BTreeSet<String>,
    #[serde(default)]
    pub seen_reviews: BTreeSet<String>,
    #[serde(default)]
    pub seen_review_comments: BTreeSet<String>,
    #[serde(default)]
    pub stack: Option<StackInfo>,
    #[serde(default)]
    pub relations: Vec<FocusRelation>,
}

fn yes() -> bool {
    true
}

fn open_state() -> PullRequestState {
    PullRequestState::Open
}

impl Default for WatchedPullRequest {
    fn default() -> Self {
        Self {
            baseline: true,
            state: PullRequestState::Open,
            ci: CiResult::NotStarted,
            terminal_reported: false,
            ci_failure_seen: false,
            title: String::new(),
            body: String::new(),
            author: String::new(),
            assignees: Vec::new(),
            url: String::new(),
            head_oid: None,
            created_at: None,
            updated_at: None,
            merged_at: None,
            closed_at: None,
            seen_comments: BTreeSet::new(),
            seen_reviews: BTreeSet::new(),
            seen_review_comments: BTreeSet::new(),
            stack: None,
            relations: Vec::new(),
        }
    }
}

impl WatchedPullRequest {
    pub fn key<'a>(&self, map_key: &'a str) -> &'a str {
        map_key
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: String,
    pub observed_at: DateTime<Utc>,
    #[serde(default)]
    pub occurred_at: Option<DateTime<Utc>>,
    pub pull_request: PullRequestKey,
    pub summary: String,
    #[serde(default)]
    pub display: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub stack: Option<StackInfo>,
    #[serde(default)]
    pub kind: Option<EventKind>,
    #[serde(default)]
    pub author: Option<String>,
}

impl ActivityRecord {
    pub fn effective_time(&self) -> DateTime<Utc> {
        self.occurred_at.unwrap_or(self.observed_at)
    }

    pub fn is_approximate(&self) -> bool {
        self.occurred_at.is_none()
    }

    pub fn from_event(
        id: String,
        observed_at: DateTime<Utc>,
        key: &PullRequestKey,
        event: &ActivityEvent,
        stack: Option<StackInfo>,
    ) -> Self {
        Self {
            id,
            observed_at,
            occurred_at: event.occurred_at,
            pull_request: key.clone(),
            summary: event.summary.clone(),
            display: event.display_summary(),
            body: event.body.clone(),
            read: event.read,
            stack,
            kind: Some(event.kind.clone()),
            author: event.author.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeSnapshot {
    pub mode: WatchMode,
    pub repositories: Vec<String>,
}

impl ScopeSnapshot {
    pub fn matches(&self, mode: WatchMode, repositories: &[String]) -> bool {
        self.mode == mode
            && self.repositories.len() == repositories.len()
            && self.repositories.iter().all(|left| {
                repositories
                    .iter()
                    .any(|right| left.eq_ignore_ascii_case(right))
            })
    }
}

#[derive(Debug, Clone)]
pub struct LoadOutcome {
    pub state: WatcherState,
    pub migrated: bool,
    pub quarantined: bool,
}

pub trait SnapshotStore: Send + Sync {
    fn load(&self) -> Result<LoadOutcome, StateError>;
    fn save(&self, state: &WatcherState) -> Result<(), StateError>;
    fn path(&self) -> &Path;
}

#[derive(Debug, Clone)]
pub struct JsonStateStore {
    path: PathBuf,
}

impl JsonStateStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            version: STATE_VERSION,
            accounts: BTreeMap::new(),
            activity: Vec::new(),
            next_activity_id: 1,
            last_scope: None,
        }
    }

    pub fn account_mut(&mut self, label: &str) -> &mut AccountState {
        self.accounts.entry(label.to_string()).or_default()
    }

    pub fn push_activity(&mut self, mut record: ActivityRecord) {
        if record.id.is_empty() {
            record.id = format!("a{}", self.next_activity_id.max(1));
        }
        if let Some(number) = record
            .id
            .strip_prefix('a')
            .and_then(|rest| rest.parse::<u64>().ok())
        {
            self.next_activity_id = self.next_activity_id.max(number + 1);
        }
        self.activity.push(record);
        if self.activity.len() > ACTIVITY_LIMIT {
            let excess = self.activity.len() - ACTIVITY_LIMIT;
            self.activity.drain(0..excess);
        }
    }

    pub fn rebuild_next_activity_id(&mut self) {
        let maximum = self
            .activity
            .iter()
            .filter_map(|record| record.id.strip_prefix('a'))
            .filter_map(|number| number.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        self.next_activity_id = self.next_activity_id.max(maximum + 1).max(1);
    }
}

impl SnapshotStore for JsonStateStore {
    fn load(&self) -> Result<LoadOutcome, StateError> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LoadOutcome {
                    state: WatcherState::new(),
                    migrated: false,
                    quarantined: false,
                })
            }
            Err(source) => {
                return Err(StateError::Read {
                    path: self.path.clone(),
                    source,
                })
            }
        };

        let value: serde_json::Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(_) => return self.quarantine(),
        };
        let version = value
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1);
        if version > STATE_VERSION as u64 {
            return self.quarantine();
        }

        let mut state: WatcherState = match serde_json::from_value(value) {
            Ok(state) => state,
            Err(_) => return self.quarantine(),
        };
        let migrated = version < STATE_VERSION as u64;
        if migrated {
            for account in state.accounts.values_mut() {
                for watched in account.pull_requests.values_mut() {
                    if watched.state.is_terminal() {
                        watched.terminal_reported = true;
                    }
                    if watched.ci.is_failing() {
                        watched.ci_failure_seen = true;
                    }
                }
            }
            state.version = STATE_VERSION;
        }
        state.rebuild_next_activity_id();
        Ok(LoadOutcome {
            state,
            migrated,
            quarantined: false,
        })
    }

    fn save(&self, state: &WatcherState) -> Result<(), StateError> {
        if std::env::var_os("PASSUFF_TEST_FAIL_STATE_WRITE").is_some() {
            return Err(StateError::Write {
                path: self.path.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "test-injected write failure",
                ),
            });
        }
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|source| StateError::Write {
            path: self.path.clone(),
            source,
        })?;
        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state.json");
        let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
        let bytes = serde_json::to_vec_pretty(state)
            .map_err(|error| StateError::Decode(error.to_string()))?;
        {
            let mut options = fs::OpenOptions::new();
            options.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temporary)
                .map_err(|source| StateError::Write {
                    path: self.path.clone(),
                    source,
                })?;
            file.write_all(&bytes).map_err(|source| StateError::Write {
                path: self.path.clone(),
                source,
            })?;
            file.flush().map_err(|source| StateError::Write {
                path: self.path.clone(),
                source,
            })?;
        }
        fs::rename(&temporary, &self.path).map_err(|source| {
            let _ = fs::remove_file(&temporary);
            StateError::Write {
                path: self.path.clone(),
                source,
            }
        })?;
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl JsonStateStore {
    fn quarantine(&self) -> Result<LoadOutcome, StateError> {
        let quarantine = self
            .path
            .with_extension(format!("corrupt.{}.json", Utc::now().timestamp_millis()));
        fs::rename(&self.path, &quarantine).map_err(|source| StateError::Quarantine {
            path: self.path.clone(),
            source,
        })?;
        Ok(LoadOutcome {
            state: WatcherState::new(),
            migrated: false,
            quarantined: true,
        })
    }
}

pub fn review_label(state: ReviewState) -> &'static str {
    state.label()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn history_is_bounded() {
        let mut state = WatcherState::new();
        for index in 0..510 {
            state.push_activity(ActivityRecord {
                id: format!("a{}", index + 1),
                observed_at: Utc::now(),
                occurred_at: None,
                pull_request: PullRequestKey::new("acme/api", 1),
                summary: format!("event {index}"),
                display: String::new(),
                body: None,
                read: false,
                stack: None,
                kind: None,
                author: None,
            });
        }
        assert_eq!(state.activity.len(), ACTIVITY_LIMIT);
        assert_eq!(state.activity.first().unwrap().summary, "event 10");
    }

    #[test]
    fn version_one_migrates_to_version_two() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("state.json");
        fs::write(
            &path,
            r#"{
              "version": 1,
              "accounts": {
                "personal": {
                  "login": "octo-personal",
                  "last_check": "2026-11-02T12:00:00Z",
                  "pull_requests": {
                    "acme/api#42": {"state": "merged", "ci": "failing"}
                  }
                }
              }
            }"#,
        )
        .unwrap();
        let store = JsonStateStore::new(&path);
        let outcome = store.load().unwrap();
        assert!(outcome.migrated);
        let watched = outcome.state.accounts["personal"].pull_requests["acme/api#42"].clone();
        assert!(watched.terminal_reported);
        assert!(watched.ci_failure_seen);
    }
}
