use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use passuff_config::{expand_home, AccountConfig, AuthMode, FocusRelation};
use passuff_core::{
    Allowance, CiResult, CredentialError, CredentialSource, Fingerprint, GatewayError,
    GatewayFactory, GitHubGateway, ProbePullRequest, ProbeReport, PullRequestKey,
    PullRequestSnapshot, PullRequestState, ResolvedCredential, ReviewState,
};

#[derive(Debug, Clone)]
pub enum FakeCredential {
    Gh { login: String },
    Token { token: String },
    Invalid,
}

#[derive(Debug, Clone)]
pub struct FakeData {
    pub pull_requests: BTreeMap<PullRequestKey, PullRequestSnapshot>,
    pub tokens: BTreeMap<String, String>,
    pub credentials: BTreeMap<String, FakeCredential>,
    pub gh_authenticated: BTreeSet<String>,
    pub gh_login_attempts: Vec<String>,
    pub gh_login_fails: bool,
    pub full_history_calls: usize,
    pub probe_calls: usize,
    pub discovery_calls: usize,
    pub observed_keys: Vec<PullRequestKey>,
    pub last_observed_keys: Vec<PullRequestKey>,
    pub fail_next_observation: bool,
    pub unavailable_next: bool,
    pub allowance: Allowance,
    pub home: PathBuf,
    pub comment_sequence: u64,
    pub review_sequence: u64,
}

impl FakeData {
    pub fn new(home: PathBuf, now: DateTime<Utc>) -> Self {
        Self {
            pull_requests: BTreeMap::new(),
            tokens: BTreeMap::new(),
            credentials: BTreeMap::new(),
            gh_authenticated: BTreeSet::new(),
            gh_login_attempts: Vec::new(),
            gh_login_fails: false,
            full_history_calls: 0,
            probe_calls: 0,
            discovery_calls: 0,
            observed_keys: Vec::new(),
            last_observed_keys: Vec::new(),
            fail_next_observation: false,
            unavailable_next: false,
            allowance: Allowance {
                remaining: 5000,
                reset_at: Some(now + Duration::seconds(3600)),
                cost: 1,
            },
            home,
            comment_sequence: 0,
            review_sequence: 0,
        }
    }

    pub fn insert_pull_request(
        &mut self,
        key: PullRequestKey,
        author: &str,
        now: DateTime<Utc>,
    ) -> &mut PullRequestSnapshot {
        let snapshot = PullRequestSnapshot {
            key: key.clone(),
            node_id: Some(format!("node-{}", key)),
            title: format!("Pull request {key}"),
            body: String::new(),
            state: PullRequestState::Open,
            created_at: Some(now - Duration::hours(1)),
            updated_at: Some(now - Duration::hours(1)),
            merged_at: None,
            closed_at: None,
            head_oid: Some("head-1".to_string()),
            ci: CiResult::Passing,
            author: author.to_string(),
            assignees: Vec::new(),
            url: key.url(),
            comments: Vec::new(),
            reviews: Vec::new(),
            review_comments: Vec::new(),
            stack: None,
            relations: Vec::new(),
        };
        self.pull_requests.insert(key.clone(), snapshot);
        self.pull_requests.get_mut(&key).expect("inserted")
    }

    pub fn add_comment(
        &mut self,
        key: &PullRequestKey,
        author: &str,
        body: &str,
        occurred_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) {
        self.comment_sequence += 1;
        let id = format!("c{}", self.comment_sequence);
        let is_bot = author.ends_with("[bot]");
        if let Some(pull_request) = self.pull_requests.get_mut(key) {
            pull_request.comments.push(passuff_core::CommentData {
                id,
                author: author.to_string(),
                body: body.to_string(),
                created_at: occurred_at.or(Some(now)),
                is_bot,
            });
            pull_request.updated_at = Some(now + Duration::seconds(self.comment_sequence as i64));
        }
    }

    pub fn add_review_comment(
        &mut self,
        key: &PullRequestKey,
        author: &str,
        body: &str,
        now: DateTime<Utc>,
    ) {
        self.comment_sequence += 1;
        let id = format!("rc{}", self.comment_sequence);
        if let Some(pull_request) = self.pull_requests.get_mut(key) {
            pull_request
                .review_comments
                .push(passuff_core::ReviewCommentData {
                    id,
                    author: author.to_string(),
                    body: body.to_string(),
                    created_at: Some(now),
                    is_bot: author.ends_with("[bot]"),
                });
            pull_request.updated_at = Some(now + Duration::seconds(self.comment_sequence as i64));
        }
    }

    pub fn add_review(
        &mut self,
        key: &PullRequestKey,
        author: &str,
        state: ReviewState,
        body: &str,
        now: DateTime<Utc>,
    ) {
        self.review_sequence += 1;
        let id = format!("r{}", self.review_sequence);
        if let Some(pull_request) = self.pull_requests.get_mut(key) {
            pull_request.reviews.push(passuff_core::ReviewData {
                id,
                author: author.to_string(),
                state,
                body: body.to_string(),
                submitted_at: Some(now),
                is_bot: author.ends_with("[bot]"),
            });
            pull_request.updated_at = Some(now + Duration::seconds(self.review_sequence as i64));
        }
    }

    pub fn authored(&self, login: &str) -> Vec<PullRequestSnapshot> {
        self.pull_requests
            .values()
            .filter(|pull_request| pull_request.author.eq_ignore_ascii_case(login))
            .cloned()
            .collect()
    }

    pub fn involved(&self) -> Vec<PullRequestSnapshot> {
        self.pull_requests
            .values()
            .filter(|pull_request| {
                pull_request.relations.iter().any(|relation| {
                    matches!(
                        relation,
                        FocusRelation::Assigned
                            | FocusRelation::ReviewRequested
                            | FocusRelation::TeamReviewRequested
                    )
                })
            })
            .cloned()
            .collect()
    }

    pub fn snapshot_for(&self, key: &PullRequestKey) -> Option<PullRequestSnapshot> {
        self.pull_requests.get(key).cloned()
    }
}

pub struct FakeCredentials {
    pub data: Arc<Mutex<FakeData>>,
    pub home: PathBuf,
}

impl CredentialSource for FakeCredentials {
    fn resolve(
        &self,
        label: &str,
        account: &AccountConfig,
        interactive: bool,
    ) -> Result<ResolvedCredential, CredentialError> {
        let mut data = self.data.lock().map_err(|_| {
            CredentialError::Failed("fake credential state is unavailable".to_string())
        })?;
        match account.auth_mode() {
            AuthMode::GhOAuth => {
                let login = account.expected_login().unwrap_or(label).to_string();
                let token = format!("gho_{login}");
                let authenticated = data.gh_authenticated.contains(&login);
                if !authenticated {
                    if !interactive {
                        return Err(CredentialError::Failed(format!(
                            "gh auth token failed for @{login}"
                        )));
                    }
                    if data
                        .gh_login_attempts
                        .iter()
                        .any(|attempt| attempt == &login)
                    {
                        return Err(CredentialError::Failed(format!(
                            "gh auth login failed for @{login}"
                        )));
                    }
                    data.gh_login_attempts.push(login.clone());
                    if data.gh_login_fails {
                        return Err(CredentialError::Failed("gh auth login failed".to_string()));
                    }
                    data.gh_authenticated.insert(login.clone());
                }
                data.tokens.entry(token.clone()).or_insert(login);
                Ok(ResolvedCredential { token })
            }
            AuthMode::GithubPat => {
                let credential = data.credentials.get(label).cloned();
                match credential {
                    Some(FakeCredential::Invalid) => {
                        Err(CredentialError::Failed("invalid credential".to_string()))
                    }
                    Some(FakeCredential::Token { token }) => {
                        let login = account.expected_login().unwrap_or(label).to_string();
                        data.tokens.entry(token.clone()).or_insert(login);
                        Ok(ResolvedCredential { token })
                    }
                    Some(FakeCredential::Gh { login }) => {
                        let token = format!("gho_{login}");
                        data.tokens.insert(token.clone(), login);
                        Ok(ResolvedCredential { token })
                    }
                    None => {
                        if let Some(path) = account.token_file.as_deref() {
                            let path = expand_with_home(&self.home, path);
                            let metadata = match std::fs::metadata(&path) {
                                Ok(metadata) => metadata,
                                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                    return Err(CredentialError::Failed(format!(
                                        "token file is missing: {}",
                                        path.display()
                                    )))
                                }
                                Err(error) => {
                                    return Err(CredentialError::Failed(format!(
                                        "token file is not readable: {error}"
                                    )))
                                }
                            };
                            if !metadata.is_file() {
                                return Err(CredentialError::Failed(format!(
                                    "token file is not a regular file: {}",
                                    path.display()
                                )));
                            }
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let mode = metadata.permissions().mode() & 0o777;
                                if mode & 0o077 != 0 {
                                    return Err(CredentialError::Failed(format!(
                                        "token_file is not private (insecure mode {mode:o})"
                                    )));
                                }
                            }
                            let token = std::fs::read_to_string(&path)
                                .map_err(|error| {
                                    CredentialError::Failed(format!(
                                        "token file is not readable: {error}"
                                    ))
                                })?
                                .trim()
                                .to_string();
                            if token.is_empty() {
                                return Err(CredentialError::Failed(format!(
                                    "token file is empty: {}",
                                    path.display()
                                )));
                            }
                            let login = account.expected_login().unwrap_or(label).to_string();
                            data.tokens.entry(token.clone()).or_insert(login);
                            Ok(ResolvedCredential { token })
                        } else {
                            let token = format!("token-TOKEN_{}", label.to_ascii_uppercase());
                            let login = account.expected_login().unwrap_or(label).to_string();
                            data.tokens.entry(token.clone()).or_insert(login);
                            Ok(ResolvedCredential { token })
                        }
                    }
                }
            }
        }
    }
}

fn expand_with_home(home: &std::path::Path, path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home.join(rest)
    } else {
        expand_home(path)
    }
}

pub struct FakeGatewayFactory {
    pub data: Arc<Mutex<FakeData>>,
}

impl GatewayFactory for FakeGatewayFactory {
    fn create(
        &self,
        _label: &str,
        _account: &AccountConfig,
        token: &str,
    ) -> Result<Box<dyn GitHubGateway>, GatewayError> {
        let data = self
            .data
            .lock()
            .map_err(|_| GatewayError::Unavailable("fake gateway is unavailable".to_string()))?;
        let login = data
            .tokens
            .get(token)
            .cloned()
            .ok_or(GatewayError::Authentication)?;
        Ok(Box::new(FakeGateway {
            data: self.data.clone(),
            login,
        }))
    }
}

pub struct FakeGateway {
    data: Arc<Mutex<FakeData>>,
    login: String,
}

impl GitHubGateway for FakeGateway {
    fn viewer_login(&mut self) -> Result<String, GatewayError> {
        Ok(self.login.clone())
    }

    fn probe(&mut self) -> Result<ProbeReport, GatewayError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| GatewayError::Unavailable("fake gateway is unavailable".to_string()))?;
        data.probe_calls += 1;
        if data.unavailable_next {
            data.unavailable_next = false;
            return Err(GatewayError::Unavailable(
                "GitHub is unavailable".to_string(),
            ));
        }
        let pull_requests = data
            .authored(&self.login)
            .into_iter()
            .map(|pull_request| ProbePullRequest {
                key: pull_request.key,
                node_id: pull_request.node_id,
                created_at: pull_request.created_at,
                updated_at: pull_request.updated_at,
                state: pull_request.state,
                head_oid: pull_request.head_oid,
                ci: pull_request.ci,
            })
            .collect();
        let allowance = data.allowance.clone();
        Ok(ProbeReport {
            pull_requests,
            allowance,
        })
    }

    fn list_authored(&mut self) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| GatewayError::Unavailable("fake gateway is unavailable".to_string()))?;
        data.full_history_calls += 1;
        if data.unavailable_next || data.fail_next_observation {
            data.unavailable_next = false;
            data.fail_next_observation = false;
            return Err(GatewayError::Unavailable(
                "GitHub is unavailable".to_string(),
            ));
        }
        Ok(data.authored(&self.login))
    }

    fn discover_involved(&mut self) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| GatewayError::Unavailable("fake gateway is unavailable".to_string()))?;
        data.discovery_calls += 1;
        Ok(data.involved())
    }

    fn observe(
        &mut self,
        keys: &[PullRequestKey],
    ) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| GatewayError::Unavailable("fake gateway is unavailable".to_string()))?;
        if data.unavailable_next || data.fail_next_observation {
            data.unavailable_next = false;
            data.fail_next_observation = false;
            return Err(GatewayError::Unavailable(
                "GitHub is unavailable".to_string(),
            ));
        }
        data.last_observed_keys = keys.to_vec();
        data.observed_keys.extend(keys.iter().cloned());
        Ok(keys
            .iter()
            .filter_map(|key| data.pull_requests.get(key).cloned())
            .collect())
    }

    fn allowance(&self) -> Allowance {
        self.data
            .lock()
            .map(|data| data.allowance.clone())
            .unwrap_or_default()
    }
}

pub fn fingerprint_of(snapshot: &PullRequestSnapshot) -> Fingerprint {
    Fingerprint {
        node_id: snapshot.node_id.clone(),
        created_at: snapshot.created_at,
        updated_at: snapshot.updated_at,
        state: snapshot.state,
        head_oid: snapshot.head_oid.clone(),
        ci: snapshot.ci,
    }
}
