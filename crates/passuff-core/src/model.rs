use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use passuff_config::FocusRelation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PullRequestKey {
    repository: String,
    number: u64,
}

impl PullRequestKey {
    pub fn new(repository: impl Into<String>, number: u64) -> Self {
        Self {
            repository: repository.into().to_ascii_lowercase(),
            number,
        }
    }

    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn name(&self) -> &str {
        self.repository
            .split_once('/')
            .map(|(_, name)| name)
            .unwrap_or(&self.repository)
    }

    pub fn url(&self) -> String {
        format!("https://github.com/{}#{}", self.repository, self.number)
    }
}

impl fmt::Display for PullRequestKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.repository, self.number)
    }
}

impl FromStr for PullRequestKey {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (repository, number) = value
            .rsplit_once('#')
            .ok_or_else(|| format!("invalid pull request key: {value}"))?;
        let number: u64 = number
            .parse()
            .map_err(|_| format!("invalid pull request number: {value}"))?;
        if repository.matches('/').count() != 1
            || repository.starts_with('/')
            || repository.ends_with('/')
        {
            return Err(format!("invalid pull request key: {value}"));
        }
        Ok(Self::new(repository, number))
    }
}

impl Serialize for PullRequestKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PullRequestKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Open,
    Merged,
    Closed,
}

impl PullRequestState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Merged => "merged",
            Self::Closed => "closed",
        }
    }

    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Open)
    }

    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "merged" => Self::Merged,
            "closed" => Self::Closed,
            _ => Self::Open,
        }
    }
}

impl fmt::Display for PullRequestState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CiResult {
    Passing,
    Failing,
    Pending,
    #[default]
    #[serde(rename = "not started")]
    NotStarted,
}

impl CiResult {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passing => "passing",
            Self::Failing => "failing",
            Self::Pending => "pending",
            Self::NotStarted => "not started",
        }
    }

    pub fn is_failing(self) -> bool {
        matches!(self, Self::Failing)
    }

    pub fn parse(value: Option<&str>) -> Self {
        match value.unwrap_or_default().to_ascii_uppercase().as_str() {
            "SUCCESS" | "SUCCESSFUL" | "PASSING" => Self::Passing,
            "FAILURE" | "FAILING" | "ERROR" | "TIMED_OUT" | "CANCELLED" => Self::Failing,
            "PENDING" | "EXPECTED" | "IN_PROGRESS" | "QUEUED" | "WAITING" => Self::Pending,
            _ => Self::NotStarted,
        }
    }
}

impl fmt::Display for CiResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackInfo {
    pub number: u64,
    pub size: u64,
    pub position: u64,
    pub base: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Pending,
    Dismissed,
}

impl ReviewState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::ChangesRequested => "changes requested",
            Self::Commented => "commented",
            Self::Pending => "pending",
            Self::Dismissed => "dismissed",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "APPROVED" => Self::Approved,
            "CHANGES_REQUESTED" => Self::ChangesRequested,
            "COMMENTED" => Self::Commented,
            "DISMISSED" => Self::Dismissed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentData {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_bot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewData {
    pub id: String,
    pub author: String,
    pub state: ReviewState,
    pub body: String,
    pub submitted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_bot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCommentData {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_bot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestSnapshot {
    pub key: PullRequestKey,
    #[serde(default)]
    pub node_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub state: PullRequestState,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub merged_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub head_oid: Option<String>,
    pub ci: CiResult,
    pub author: String,
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub comments: Vec<CommentData>,
    #[serde(default)]
    pub reviews: Vec<ReviewData>,
    #[serde(default)]
    pub review_comments: Vec<ReviewCommentData>,
    #[serde(default)]
    pub stack: Option<StackInfo>,
    #[serde(default)]
    pub relations: Vec<FocusRelation>,
}

impl PullRequestSnapshot {
    pub fn author_in(&self, logins: &[String]) -> bool {
        logins
            .iter()
            .any(|login| login.eq_ignore_ascii_case(&self.author))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Comment,
    ReviewComment,
    Review { state: ReviewState },
    CiFailure,
    CiRecovered,
    Opened,
    Merged,
    Closed,
    Reopened,
}

impl EventKind {
    pub fn family_enabled(&self, events: &passuff_config::EventToggles) -> bool {
        match self {
            Self::Comment | Self::ReviewComment => events.comments,
            Self::Review { .. } => events.reviews,
            Self::CiFailure => events.ci_failure,
            Self::CiRecovered => events.ci_recovery,
            Self::Opened | Self::Merged | Self::Closed => events.state_changes,
            Self::Reopened => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub kind: EventKind,
    pub summary: String,
    #[serde(default)]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// True for own actions: recorded read and never notified.
    #[serde(default)]
    pub read: bool,
}

impl ActivityEvent {
    pub fn notification_summary(&self) -> &str {
        &self.summary
    }

    pub fn display_summary(&self) -> String {
        match &self.kind {
            EventKind::Comment => self
                .author
                .as_deref()
                .map(|author| format!("comment by @{author}"))
                .unwrap_or_else(|| self.summary.clone()),
            EventKind::ReviewComment => self
                .author
                .as_deref()
                .map(|author| format!("review comment by @{author}"))
                .unwrap_or_else(|| self.summary.clone()),
            EventKind::Review { state } => self
                .author
                .as_deref()
                .map(|author| format!("review {} by @{author}", state.label()))
                .unwrap_or_else(|| self.summary.clone()),
            EventKind::Opened => self
                .author
                .as_deref()
                .map(|author| format!("new pull request by @{author}"))
                .unwrap_or_else(|| self.summary.clone()),
            EventKind::CiFailure => self.summary.clone(),
            EventKind::CiRecovered => self.summary.clone(),
            EventKind::Merged => "merged".to_string(),
            EventKind::Closed => "closed".to_string(),
            EventKind::Reopened => "reopened".to_string(),
        }
    }

    pub fn body_label(&self) -> String {
        match &self.kind {
            EventKind::Comment => self
                .author
                .as_deref()
                .map(|author| format!("Comment by @{author}"))
                .unwrap_or_else(|| "Comment".to_string()),
            EventKind::ReviewComment => self
                .author
                .as_deref()
                .map(|author| format!("Review comment by @{author}"))
                .unwrap_or_else(|| "Review comment".to_string()),
            EventKind::Review { state } => self
                .author
                .as_deref()
                .map(|author| format!("Review {} by @{author}", state.label()))
                .unwrap_or_else(|| "Review".to_string()),
            _ => self.summary.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub key: PullRequestKey,
    pub title: String,
    pub events: Vec<ActivityEvent>,
}

impl Notification {
    pub fn activity_count(&self) -> usize {
        self.events.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbePullRequest {
    pub key: PullRequestKey,
    pub node_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub state: PullRequestState,
    pub head_oid: Option<String>,
    pub ci: CiResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub node_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub state: PullRequestState,
    pub head_oid: Option<String>,
    pub ci: CiResult,
}

impl From<&ProbePullRequest> for Fingerprint {
    fn from(probe: &ProbePullRequest) -> Self {
        Self {
            node_id: probe.node_id.clone(),
            created_at: probe.created_at,
            updated_at: probe.updated_at,
            state: probe.state,
            head_oid: probe.head_oid.clone(),
            ci: probe.ci,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Allowance {
    pub remaining: u32,
    pub reset_at: Option<DateTime<Utc>>,
    pub cost: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeReport {
    pub pull_requests: Vec<ProbePullRequest>,
    pub allowance: Allowance,
}
