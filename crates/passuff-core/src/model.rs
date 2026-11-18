//! Domain values shared by observation, classification, and the read model.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use passuff_config::Relation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

impl PrState {
    pub fn human(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Merged => "merged",
            Self::Closed => "closed",
        }
    }

    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Open)
    }

    pub fn from_graphql(value: &str) -> Self {
        match value {
            "MERGED" => Self::Merged,
            "CLOSED" => Self::Closed,
            _ => Self::Open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiResult {
    Passing,
    Failing,
    Pending,
    NotStarted,
}

impl CiResult {
    pub fn human(self) -> &'static str {
        match self {
            Self::Passing => "passing",
            Self::Failing => "failing",
            Self::Pending => "pending",
            Self::NotStarted => "not started",
        }
    }

    pub fn from_graphql(value: Option<&str>) -> Self {
        match value.unwrap_or_default() {
            "SUCCESS" => Self::Passing,
            "FAILURE" | "ERROR" => Self::Failing,
            "PENDING" | "EXPECTED" => Self::Pending,
            _ => Self::NotStarted,
        }
    }

    pub fn is_failing(self) -> bool {
        matches!(self, Self::Failing)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackInfo {
    pub number: u64,
    pub size: u64,
    pub base: String,
    pub position: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestKey {
    pub owner: String,
    pub repository: String,
    pub number: u64,
}

impl PullRequestKey {
    pub fn parse(value: &str) -> Option<Self> {
        let (repository, number) = value.rsplit_once('#')?;
        let (owner, repository) = repository.split_once('/')?;
        let number = number.parse().ok()?;
        if owner.is_empty() || repository.is_empty() {
            return None;
        }
        Some(Self {
            owner: owner.to_string(),
            repository: repository.to_string(),
            number,
        })
    }

    pub fn repository_name(&self) -> String {
        format!("{}/{}", self.owner, self.repository)
    }

    pub fn short(&self) -> String {
        format!("{}#{}", self.repository, self.number)
    }

    pub fn display(&self) -> String {
        format!("{}/{}#{}", self.owner, self.repository, self.number)
    }

    pub fn web_url(&self) -> String {
        format!(
            "https://github.com/{}/{}#{}",
            self.owner, self.repository, self.number
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    Comment,
    ReviewComment,
    Review,
    NewPullRequest,
    CiFailure,
    CiRecovery,
    Merged,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: String,
    pub key: String,
    pub kind: ActivityKind,
    pub summary: String,
    pub display: String,
    pub event_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub stack: Option<StackInfo>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

impl ActivityRecord {
    pub fn shown_time(&self) -> DateTime<Utc> {
        self.occurred_at.unwrap_or(self.observed_at)
    }

    pub fn body_label(&self) -> String {
        match self.kind {
            ActivityKind::Comment => self
                .author
                .as_ref()
                .map(|author| format!("Comment by @{author}"))
                .unwrap_or_else(|| self.summary.clone()),
            ActivityKind::ReviewComment => self
                .author
                .as_ref()
                .map(|author| format!("Review comment by @{author}"))
                .unwrap_or_else(|| self.summary.clone()),
            ActivityKind::Review => self
                .author
                .as_ref()
                .map(|author| format!("Review {} by @{author}", review_state_label(&self.summary)))
                .unwrap_or_else(|| self.summary.clone()),
            _ => self.summary.clone(),
        }
    }
}

fn review_state_label(summary: &str) -> &'static str {
    // The summary is "Review <state> by @login"; only the state is needed here.
    if summary.starts_with("Review approved") {
        "approved"
    } else if summary.starts_with("Review changes requested") {
        "changes requested"
    } else if summary.starts_with("Review commented") {
        "commented"
    } else if summary.starts_with("Review pending") {
        "pending"
    } else if summary.starts_with("Review dismissed") {
        "dismissed"
    } else {
        "commented"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedPullRequest {
    pub key: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub author_is_bot: bool,
    #[serde(default)]
    pub assignees: Vec<String>,
    pub state: PrState,
    pub ci: CiResult,
    #[serde(default)]
    pub is_draft: bool,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub merged_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub stack: Option<StackInfo>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub relations_known: bool,
    #[serde(default = "default_true")]
    pub baseline: bool,
    #[serde(default)]
    pub state_change_reported: bool,
    #[serde(default)]
    pub was_failing: bool,
    #[serde(default)]
    pub terminal_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub seen_comments: BTreeSet<String>,
    #[serde(default)]
    pub seen_reviews: BTreeSet<String>,
    #[serde(default)]
    pub seen_review_comments: BTreeSet<String>,
    #[serde(default)]
    pub last_observed: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

impl WatchedPullRequest {
    pub fn base_entry(key: &str, state: PrState, ci: CiResult, title: &str) -> Self {
        Self {
            key: key.to_string(),
            node_id: None,
            title: title.to_string(),
            body: None,
            url: None,
            author: None,
            author_is_bot: false,
            assignees: Vec::new(),
            state,
            ci,
            is_draft: false,
            created_at: None,
            updated_at: None,
            merged_at: None,
            closed_at: None,
            stack: None,
            relations: Vec::new(),
            relations_known: false,
            baseline: true,
            state_change_reported: state.is_terminal(),
            was_failing: ci.is_failing(),
            terminal_at: None,
            seen_comments: BTreeSet::new(),
            seen_reviews: BTreeSet::new(),
            seen_review_comments: BTreeSet::new(),
            last_observed: None,
        }
    }

    pub fn repository(&self) -> String {
        PullRequestKey::parse(&self.key)
            .map(|key| key.repository_name())
            .unwrap_or_default()
    }

    /// A recorded relation set that never intersects the focus is hidden. A
    /// record with no recorded relations stays visible (R14 of change-watch-scope).
    pub fn in_focus(&self, focus: &passuff_config::FocusConfig) -> bool {
        if !self.relations_known || self.relations.is_empty() {
            return true;
        }
        self.relations
            .iter()
            .any(|relation| focus.contains(*relation))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountState {
    #[serde(default)]
    pub login: Option<String>,
    #[serde(default)]
    pub last_check: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pull_requests: BTreeMap<String, WatchedPullRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherState {
    pub version: u32,
    #[serde(default)]
    pub next_activity_id: u64,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountState>,
    #[serde(default)]
    pub activity: Vec<ActivityRecord>,
    /// Repositories whose retained records were reselected and whose next
    /// observation is a silent catch-up.
    #[serde(default)]
    pub catch_up: BTreeSet<String>,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            version: 2,
            next_activity_id: 1,
            accounts: BTreeMap::new(),
            activity: Vec::new(),
            catch_up: BTreeSet::new(),
        }
    }
}

impl WatcherState {
    pub fn allocate_activity_id(&mut self) -> String {
        let id = self.next_activity_id.max(1);
        self.next_activity_id = id + 1;
        format!("a{id}")
    }

    pub fn find_activity_mut(&mut self, id: &str) -> Option<&mut ActivityRecord> {
        self.activity.iter_mut().find(|record| record.id == id)
    }

    pub fn activity_contains(&self, key: &str, event_id: &str) -> bool {
        self.activity
            .iter()
            .any(|record| record.key == key && record.event_id == event_id)
    }

    pub fn cap_activity(&mut self) {
        const LIMIT: usize = 500;
        if self.activity.len() > LIMIT {
            let excess = self.activity.len() - LIMIT;
            self.activity.drain(0..excess);
        }
    }

    /// Union of resolved logins across accounts. Only accounts whose credential
    /// matched their expected login store a login.
    pub fn login_set(&self) -> BTreeSet<String> {
        self.accounts
            .values()
            .filter_map(|account| account.login.as_ref())
            .map(|login| login.to_lowercase())
            .collect()
    }

    pub fn all_watched(&self) -> Vec<&WatchedPullRequest> {
        let mut watched: Vec<&WatchedPullRequest> = self
            .accounts
            .values()
            .flat_map(|account| account.pull_requests.values())
            .collect();
        watched.sort_by(|left, right| left.key.cmp(&right.key));
        watched
    }

    pub fn watched_by_key(&self) -> BTreeMap<String, &WatchedPullRequest> {
        let mut by_key = BTreeMap::new();
        for watched in self.all_watched() {
            by_key.entry(watched.key.clone()).or_insert(watched);
        }
        by_key
    }

    pub fn known_repositories(&self) -> Vec<String> {
        let mut repositories: BTreeMap<String, String> = BTreeMap::new();
        for watched in self.all_watched() {
            let repository = watched.repository();
            if !repository.is_empty() {
                repositories
                    .entry(repository.to_ascii_lowercase())
                    .or_insert(repository);
            }
        }
        repositories.into_values().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub key: String,
    pub summaries: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    pub failures: Vec<AccountFailure>,
    pub baselines: Vec<String>,
    pub notifications: Vec<Notification>,
}

impl RefreshReport {
    pub fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct AccountFailure {
    pub label: String,
    pub message: String,
}
