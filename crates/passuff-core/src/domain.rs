use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use passuff_config::{Involvement, WatchConfig, WatchMode};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub label: String,
    pub login: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PullRequestKey {
    pub repository: String,
    pub number: u64,
}

impl PullRequestKey {
    pub fn new(repository: impl Into<String>, number: u64) -> Self {
        Self {
            repository: repository.into(),
            number,
        }
    }
}

impl fmt::Display for PullRequestKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.repository, self.number)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

impl PrState {
    pub fn is_terminal(&self) -> bool {
        !matches!(self, PrState::Open)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CiState {
    Success,
    Failure,
    Error,
    Pending,
    None,
}

impl CiState {
    pub fn failed(&self) -> bool {
        matches!(self, CiState::Failure | CiState::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiResult {
    pub state: CiState,
    pub failed_checks: Vec<String>,
}

impl Default for CiResult {
    fn default() -> Self {
        Self {
            state: CiState::None,
            failed_checks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub author: String,
    pub state: String,
    pub body: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeSnapshot {
    pub mode: WatchMode,
    pub repositories: Vec<String>,
}

impl ScopeSnapshot {
    pub fn from_watch(watch: &WatchConfig) -> Self {
        Self {
            mode: watch.mode,
            repositories: watch.repositories.clone(),
        }
    }

    pub fn allows_repository(&self, repository: &str) -> bool {
        self.mode == WatchMode::All
            || self
                .repositories
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(repository))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestStack {
    pub number: u64,
    pub size: u64,
    pub position: u64,
    pub base: String,
}

#[derive(Debug, Clone)]
pub struct AuthoredPullRequest {
    pub key: PullRequestKey,
    pub node_id: String,
    pub author: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub state: PrState,
    pub merged: bool,
    pub is_draft: bool,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub base_ref: String,
    pub merged_by: Option<String>,
    pub stack: Option<PullRequestStack>,
    pub head_oid: String,
    pub ci: CiResult,
    pub comments: Vec<Comment>,
    pub reviews: Vec<Review>,
    pub review_comments: Vec<ReviewComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    pub recorded_at: String,
    pub seen_comment_ids: Vec<String>,
    pub seen_review_ids: Vec<String>,
    pub seen_review_comment_ids: Vec<String>,
    pub head_oid: String,
    pub ci_state: CiState,
    pub state: PrState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentObservation {
    pub seen_comment_ids: Vec<String>,
    pub seen_review_ids: Vec<String>,
    pub seen_review_comment_ids: Vec<String>,
    pub head_oid: String,
    pub ci_state: CiState,
    pub ci_failed: bool,
    pub state: PrState,
    pub state_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchedPullRequest {
    pub repository: String,
    pub number: u64,
    pub node_id: String,
    pub author: String,
    pub title: String,
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub is_draft: bool,
    #[serde(default)]
    pub merged_by: Option<String>,
    pub baseline: Baseline,
    pub current: CurrentObservation,
    #[serde(default)]
    pub stack: Option<PullRequestStack>,
    #[serde(default)]
    pub involvements: Vec<Involvement>,
}

impl WatchedPullRequest {
    pub fn key(&self) -> PullRequestKey {
        PullRequestKey::new(self.repository.clone(), self.number)
    }

    pub fn from_baseline(pr: &AuthoredPullRequest, recorded_at: String) -> Self {
        Self {
            repository: pr.key.repository.clone(),
            number: pr.key.number,
            node_id: pr.node_id.clone(),
            author: pr.author.clone(),
            title: pr.title.clone(),
            url: pr.url.clone(),
            created_at: pr.created_at.clone(),
            updated_at: pr.updated_at.clone(),
            description: pr.body.clone(),
            assignees: pr.assignees.clone(),
            is_draft: pr.is_draft,
            merged_by: pr.merged_by.clone(),
            baseline: Baseline {
                recorded_at,
                seen_comment_ids: pr.comments.iter().map(|c| c.id.clone()).collect(),
                seen_review_ids: pr.reviews.iter().map(|r| r.id.clone()).collect(),
                seen_review_comment_ids: pr.review_comments.iter().map(|c| c.id.clone()).collect(),
                head_oid: pr.head_oid.clone(),
                ci_state: pr.ci.state.clone(),
                state: pr.state.clone(),
            },
            stack: pr.stack.clone(),
            involvements: Vec::new(),
            current: CurrentObservation {
                seen_comment_ids: pr.comments.iter().map(|c| c.id.clone()).collect(),
                seen_review_ids: pr.reviews.iter().map(|r| r.id.clone()).collect(),
                seen_review_comment_ids: pr.review_comments.iter().map(|c| c.id.clone()).collect(),
                head_oid: pr.head_oid.clone(),
                ci_state: pr.ci.state.clone(),
                ci_failed: pr.ci.state.failed(),
                state: pr.state.clone(),
                state_reported: pr.state.is_terminal(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityEvent {
    Comment {
        author: String,
        #[serde(default, alias = "body_excerpt")]
        body: String,
    },
    ReviewComment {
        author: String,
        #[serde(default, alias = "body_excerpt")]
        body: String,
    },
    Review {
        author: String,
        state: String,
        #[serde(default)]
        body: String,
    },
    CiFailure {
        failed_checks: Vec<String>,
    },
    CiRecovery,
    StateChange {
        state: PrState,
        #[serde(default)]
        actor: Option<String>,
    },
    Opened {
        author: String,
    },
}

/// The single-line preview of a comment stored in an activity record.
pub fn excerpt(body: &str) -> String {
    let single_line = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = single_line.chars();
    let excerpt: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        format!("{excerpt}...")
    } else {
        excerpt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub key: PullRequestKey,
    pub events: Vec<ActivityEvent>,
}

/// The GitHub GraphQL allowance reported by a probe response.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApiAllowance {
    pub cost: u32,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: String,
}

/// One open authored pull request as seen by a change probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbePullRequest {
    pub repository: String,
    pub number: u64,
    pub node_id: String,
    pub updated_at: String,
    pub created_at: String,
    pub head_oid: String,
    pub ci_state: Option<CiState>,
    pub state: PrState,
}

/// One account's change probe result, without observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeProbe {
    pub label: String,
    pub login: String,
    pub pull_requests: Vec<ProbePullRequest>,
}

/// The outcome of comparing a probe with the last successful cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSignal {
    pub changed: bool,
    pub allowance: Option<ApiAllowance>,
    pub failures: Vec<AccountFailure>,
}

#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    pub notifications: Vec<Notification>,
    pub account_failures: Vec<AccountFailure>,
    pub baselines_recorded: Vec<PullRequestKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountFailure {
    pub label: String,
    pub kind: AccountFailureKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountFailureKind {
    Authentication,
    RateLimited { reset_at: String },
    Timeout,
    Transport,
    MalformedResponse,
    Credential,
    Identity,
}

#[derive(Debug, Clone, Error)]
pub enum StateError {
    #[error("cannot read state {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("cannot write state {path}: {message}")]
    Write { path: PathBuf, message: String },
    #[error("cannot quarantine state {path}: {message}")]
    Quarantine { path: PathBuf, message: String },
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("state error: {0}")]
    State(#[from] StateError),
    #[error("probe failed: {0}")]
    Probe(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatcherState {
    pub version: u32,
    pub accounts: BTreeMap<String, AccountState>,
    #[serde(default)]
    pub activity: Vec<ActivityRecord>,
    #[serde(default)]
    pub next_activity_id: u64,
    #[serde(default)]
    pub last_scope: Option<ScopeSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: String,
    pub key: PullRequestKey,
    pub observed_at: String,
    #[serde(default)]
    pub occurred_at: Option<String>,
    pub event: ActivityEvent,
    pub read: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountState {
    pub login: String,
    pub last_check_at: Option<String>,
    pub pull_requests: BTreeMap<String, WatchedPullRequest>,
}

#[derive(Debug, Clone, Default)]
pub struct LoginSet(HashSet<String>);

impl LoginSet {
    pub fn new(logins: impl IntoIterator<Item = String>) -> Self {
        Self(
            logins
                .into_iter()
                .map(|login| login.to_ascii_lowercase())
                .collect(),
        )
    }

    pub fn contains(&self, login: &str) -> bool {
        self.0.contains(&login.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::{excerpt, ActivityEvent};

    #[test]
    fn an_activity_comment_loads_a_legacy_body_excerpt() {
        let event: ActivityEvent = serde_json::from_str(
            r#"{"Comment":{"author":"alice","body_excerpt":"a stored preview"}}"#,
        )
        .expect("legacy event");
        assert_eq!(
            event,
            ActivityEvent::Comment {
                author: "alice".to_string(),
                body: "a stored preview".to_string()
            }
        );
    }

    #[test]
    fn the_excerpt_collapses_lines_and_caps_its_length() {
        assert_eq!(excerpt("one\n\ntwo"), "one two");
        let long = "word ".repeat(40);
        let preview = excerpt(&long);
        assert!(preview.ends_with("..."), "{preview}");
        assert_eq!(preview.chars().count(), 83);
    }
}
