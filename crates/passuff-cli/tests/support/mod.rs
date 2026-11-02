#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use passuff_cli::delivery::{FailingDelivery, NotificationDelivery, RecordingDelivery};
use passuff_cli::sound::{RecordingSoundPlayer, SoundPlayer, UnavailableSoundPlayer};
use passuff_cli::{CliPorts, CliRuntime};
use passuff_config::{
    AccountConfig, AppConfig, ConfigError, CoreConfig, CredentialSpec, EventToggles,
    NotificationsConfig, SoundFile,
};
use passuff_core::{
    AccountObservation, AuthoredPullRequest, CiResult, CiState, Comment, CredentialError,
    CredentialSource, GitHubError, GitHubGateway, JsonStateStore, PrState, PullRequestKey,
    RefreshReport, Review, ReviewComment, SnapshotStore, StateError, WatcherState,
};
use serde_json::Value;
use tempfile::TempDir;
use url::Url;

#[derive(Debug, Clone)]
pub struct AccountFixture {
    pub label: String,
    pub login: String,
    pub token_var: String,
}

#[derive(Debug, Clone)]
pub struct PullRequestFixture {
    pub key: PullRequestKey,
    pub author: String,
    pub title: String,
    pub url: String,
    pub state: PrState,
    pub merged: bool,
    pub created_at: String,
    pub updated_at: String,
    pub head_oid: String,
    pub ci: CiResult,
    pub comments: Vec<Comment>,
    pub reviews: Vec<Review>,
    pub review_comments: Vec<ReviewComment>,
    pub is_draft: bool,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub base_ref: String,
}

impl PullRequestFixture {
    pub fn new(key: PullRequestKey, author: impl Into<String>) -> Self {
        let key_string = key.to_string();
        Self {
            key,
            author: author.into(),
            title: format!("Pull request {key_string}"),
            url: format!("https://github.com/{key_string}"),
            state: PrState::Open,
            merged: false,
            created_at: "2026-11-02T11:00:00Z".to_string(),
            updated_at: "2026-11-02T11:00:00Z".to_string(),
            head_oid: "abc123".to_string(),
            ci: CiResult {
                state: CiState::Success,
                failed_checks: Vec::new(),
            },
            comments: Vec::new(),
            reviews: Vec::new(),
            review_comments: Vec::new(),
            is_draft: false,
            assignees: Vec::new(),
            labels: Vec::new(),
            milestone: None,
            base_ref: "main".to_string(),
        }
    }

    pub fn to_authored(&self) -> AuthoredPullRequest {
        AuthoredPullRequest {
            key: self.key.clone(),
            node_id: format!("node-{}", self.key),
            author: self.author.clone(),
            title: self.title.clone(),
            url: self.url.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            state: self.state.clone(),
            merged: self.merged,
            is_draft: self.is_draft,
            assignees: self.assignees.clone(),
            labels: self.labels.clone(),
            milestone: self.milestone.clone(),
            base_ref: self.base_ref.clone(),
            head_oid: self.head_oid.clone(),
            ci: self.ci.clone(),
            comments: self.comments.clone(),
            reviews: self.reviews.clone(),
            review_comments: self.review_comments.clone(),
        }
    }
}

#[derive(cucumber::World)]
pub struct World {
    pub accounts: BTreeMap<String, AccountFixture>,
    pub pull_requests: BTreeMap<String, PullRequestFixture>,
    pub events: EventToggles,
    pub notifications: NotificationsConfig,
    pub invalid_config: bool,
    pub corrupt_state: bool,
    pub failing_store: bool,
    pub failing_delivery: bool,
    pub unavailable_sound: bool,
    pub fail_invalid_credential: bool,
    pub pending_baseline: bool,
    pub last_report: Option<RefreshReport>,
    pub last_transcript: String,
    pub last_status: i32,
    pub notification_log: Arc<Mutex<Vec<Value>>>,
    pub sound_log: Arc<Mutex<Vec<String>>>,
    pub check_clock: Arc<Mutex<u64>>,
    pub state_dir: Option<TempDir>,
    pub e2e: bool,
    pub e2e_scenario: String,
    pub e2e_pending_baseline: bool,
    pub e2e_config_dir: Option<TempDir>,
    pub e2e_state_path: Option<PathBuf>,
    pub e2e_label: String,
    pub e2e_login: String,
    pub e2e_server: Option<wiremock::MockServer>,
    pub e2e_prs: Vec<Value>,
    pub e2e_child: Option<std::process::Child>,
    pub e2e_stdout: Arc<Mutex<Vec<String>>>,
    pub e2e_sound_dir: Option<TempDir>,
    pub e2e_sound_log: Option<PathBuf>,
    pub e2e_sink: Option<PathBuf>,
    #[cfg(target_os = "linux")]
    pub e2e_dbus: Option<e2e::DbustTwin>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            accounts: BTreeMap::new(),
            pull_requests: BTreeMap::new(),
            events: EventToggles::default(),
            notifications: NotificationsConfig::default(),
            invalid_config: false,
            corrupt_state: false,
            failing_store: false,
            failing_delivery: false,
            unavailable_sound: false,
            fail_invalid_credential: false,
            pending_baseline: false,
            last_report: None,
            last_transcript: String::new(),
            last_status: 0,
            notification_log: Arc::new(Mutex::new(Vec::new())),
            sound_log: Arc::new(Mutex::new(Vec::new())),
            check_clock: Arc::new(Mutex::new(0)),
            state_dir: TempDir::new().ok(),
            e2e: false,
            e2e_scenario: String::new(),
            e2e_pending_baseline: false,
            e2e_config_dir: None,
            e2e_state_path: None,
            e2e_label: String::new(),
            e2e_login: String::new(),
            e2e_server: None,
            e2e_prs: Vec::new(),
            e2e_child: None,
            e2e_stdout: Arc::new(Mutex::new(Vec::new())),
            e2e_sound_dir: None,
            e2e_sound_log: None,
            e2e_sink: None,
            #[cfg(target_os = "linux")]
            e2e_dbus: None,
        }
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("World")
            .field("accounts", &self.accounts.len())
            .field("pull_requests", &self.pull_requests.len())
            .field("e2e", &self.e2e)
            .finish()
    }
}

impl World {
    pub fn state_path(&self) -> PathBuf {
        self.state_dir
            .as_ref()
            .expect("state dir")
            .path()
            .join("state.json")
    }

    pub fn add_account(&mut self, label: &str, login: &str, invalid: bool) {
        let token_var = if invalid {
            format!("INVALID_{}", label.to_ascii_uppercase())
        } else {
            format!("TOKEN_{}", label.to_ascii_uppercase())
        };
        if invalid {
            self.fail_invalid_credential = true;
        }
        self.accounts.insert(
            label.to_string(),
            AccountFixture {
                label: label.to_string(),
                login: login.to_string(),
                token_var,
            },
        );
    }

    pub fn add_pr(&mut self, key: &str, author: &str) -> &mut PullRequestFixture {
        let parsed = parse_key(key);
        self.pull_requests
            .entry(key.to_string())
            .or_insert_with(|| PullRequestFixture::new(parsed, author))
    }

    pub fn pr_mut(&mut self, key: &str) -> &mut PullRequestFixture {
        self.pull_requests.get_mut(key).expect("pull request")
    }

    pub fn build_config(&self) -> AppConfig {
        let accounts = self
            .accounts
            .values()
            .map(|account| {
                (
                    account.label.clone(),
                    AccountConfig {
                        credential: CredentialSpec::Env {
                            variable: account.token_var.clone(),
                        },
                        graphql_url: Url::parse("http://127.0.0.1/graphql").expect("url"),
                    },
                )
            })
            .collect();
        AppConfig {
            poll_interval_secs: 1,
            core: CoreConfig {
                accounts,
                events: self.events.clone(),
            },
            notifications: self.notifications.clone(),
        }
    }

    fn gateway_snapshot(&self) -> Vec<(String, String, Vec<AuthoredPullRequest>)> {
        self.accounts
            .values()
            .map(|account| {
                (
                    format!("token-{}", account.token_var),
                    account.login.clone(),
                    self.pull_requests
                        .values()
                        .map(PullRequestFixture::to_authored)
                        .collect(),
                )
            })
            .collect()
    }

    pub async fn run_check_internal(&mut self) {
        let config = if self.invalid_config {
            Err(ConfigError::Validation("invalid configuration".to_string()))
        } else {
            Ok(self.build_config())
        };
        let gateway = Arc::new(FakeGateway::new(
            self.gateway_snapshot(),
            self.check_clock.clone(),
        ));
        let credentials = Arc::new(FakeCredentials {
            fail_invalid: self.fail_invalid_credential,
        });
        let store: Arc<dyn SnapshotStore> = if self.failing_store {
            Arc::new(FailingStore)
        } else {
            Arc::new(JsonStateStore::new(self.state_path()))
        };
        let sound: Arc<dyn SoundPlayer> = if self.unavailable_sound {
            Arc::new(UnavailableSoundPlayer)
        } else {
            Arc::new(RecordingSoundPlayer::new(self.sound_log.clone()))
        };
        let delivery: Arc<dyn NotificationDelivery> = if self.failing_delivery {
            Arc::new(FailingDelivery)
        } else {
            Arc::new(RecordingDelivery::new(sound, self.notification_log.clone()))
        };
        let runtime = CliRuntime::new(CliPorts {
            gateway,
            store,
            credentials,
            delivery,
        });
        let outcome = runtime.run_check(config).await;
        self.last_report = outcome.report;
        self.last_transcript = outcome.transcript;
        self.last_status = outcome.status;
    }

    pub async fn ensure_baseline(&mut self) {
        if self.e2e && self.e2e_pending_baseline {
            self.e2e_pending_baseline = false;
            self.e2e_run_check().await;
            return;
        }
        if self.pending_baseline {
            self.pending_baseline = false;
            self.run_check_internal().await;
        }
    }

    pub fn last_report(&self) -> &RefreshReport {
        self.last_report.as_ref().expect("run a check first")
    }

    pub fn last_notification(&self) -> Option<Value> {
        self.notification_log
            .lock()
            .expect("notification log")
            .last()
            .cloned()
    }

    pub fn sound_calls(&self) -> Vec<String> {
        if self.e2e {
            if let Some(path) = self.e2e_sound_log.as_ref() {
                if let Ok(text) = std::fs::read_to_string(path) {
                    return text.lines().map(str::to_string).collect();
                }
            }
        }
        self.sound_log.lock().expect("sound log").clone()
    }
}

mod e2e;

#[derive(Debug, Clone)]
struct FakeGateway {
    by_token: BTreeMap<String, (String, Vec<AuthoredPullRequest>)>,
    clock: Arc<Mutex<u64>>,
}

impl FakeGateway {
    fn new(
        snapshot: Vec<(String, String, Vec<AuthoredPullRequest>)>,
        clock: Arc<Mutex<u64>>,
    ) -> Self {
        let by_token = snapshot
            .into_iter()
            .map(|(token, login, pull_requests)| (token, (login, pull_requests)))
            .collect();
        Self { by_token, clock }
    }
}

#[async_trait]
impl GitHubGateway for FakeGateway {
    async fn observe(
        &self,
        token: &str,
        _graphql_url: &Url,
        _known_node_ids: &[String],
    ) -> Result<AccountObservation, GitHubError> {
        let Some((login, pull_requests)) = self.by_token.get(token) else {
            return Err(GitHubError::Authentication);
        };
        let mut clock = self.clock.lock().expect("check clock");
        *clock += 1;
        let checked_at = format!("2026-11-02T12:00:{:02}Z", *clock);
        Ok(AccountObservation {
            login: login.clone(),
            pull_requests: pull_requests.clone(),
            checked_at,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct FakeCredentials {
    fail_invalid: bool,
}

#[async_trait]
impl CredentialSource for FakeCredentials {
    async fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        match spec {
            CredentialSpec::Env { variable } => {
                if self.fail_invalid && variable.starts_with("INVALID_") {
                    Err(CredentialError::MissingEnv)
                } else {
                    Ok(format!("token-{variable}"))
                }
            }
            CredentialSpec::Command { command } => Ok(format!("token-{command}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FailingStore;

#[async_trait]
impl SnapshotStore for FailingStore {
    async fn load(&self) -> Result<WatcherState, StateError> {
        Ok(WatcherState {
            version: 1,
            accounts: BTreeMap::new(),
        })
    }

    async fn save(&self, _state: &WatcherState) -> Result<(), StateError> {
        Err(StateError::Write {
            path: PathBuf::from("state.json"),
            message: "injected state write failure".to_string(),
        })
    }
}

fn parse_key(key: &str) -> PullRequestKey {
    let (repository, number) = key.rsplit_once('#').expect("key with #");
    PullRequestKey::new(repository, number.parse().expect("PR number"))
}

pub fn ci_failing() -> CiResult {
    CiResult {
        state: CiState::Failure,
        failed_checks: vec!["build".to_string()],
    }
}

pub fn ci_success() -> CiResult {
    CiResult {
        state: CiState::Success,
        failed_checks: Vec::new(),
    }
}

pub fn ci_pending() -> CiResult {
    CiResult {
        state: CiState::Pending,
        failed_checks: Vec::new(),
    }
}

pub fn sound_path() -> SoundFile {
    SoundFile::Default
}
