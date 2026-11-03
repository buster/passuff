#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use passuff_cli::delivery::{FailingDelivery, NotificationDelivery, RecordingDelivery};
use passuff_cli::inbox_cli::InboxCli;
use passuff_cli::live::{next_delay, ObservationSession, ObservationUpdate};
use passuff_cli::scope::ScopeEditor;
use passuff_cli::sound::{RecordingSoundPlayer, SoundPlayer, UnavailableSoundPlayer};
use passuff_cli::transcript::activity_summary;
use passuff_cli::tui::{load_view, TuiAction, TuiApp};
use passuff_cli::{CliPorts, CliRuntime, StopAfter};
use passuff_config::{
    AccountConfig, AppConfig, ConfigError, ConfigLoader, CoreConfig, CredentialSpec, EventToggles,
    NotificationsConfig, SoundFile,
};
use passuff_core::{
    AccountObservation, AccountProbe, ActivityEvent, ActivityRecord, ApiAllowance,
    AuthoredPullRequest, CiResult, CiState, Comment, CredentialError, CredentialSource, GhLogin,
    GitHubError, GitHubGateway, Inbox, JsonStateStore, PrState, ProbePullRequest, PullRequestKey,
    PullRequestStack, RefreshReport, Review, ReviewComment, SnapshotStore, StateError,
    SystemCredentialSource, WatchedPullRequest, WatcherState,
};
use serde_json::Value;
use tempfile::TempDir;
use url::Url;

#[derive(Debug, Clone)]
pub enum AccountAuth {
    Env { variable: String },
    Command { command: String },
    File { path: PathBuf, mode: u32 },
    GhOAuth { login: String, host: String },
}

#[derive(Debug, Clone)]
pub struct AccountFixture {
    pub label: String,
    pub login: String,
    pub token: String,
    pub token_var: String,
    pub expected_login: Option<String>,
    pub auth: AccountAuth,
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
    pub stack: Option<PullRequestStack>,
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
            stack: None,
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
            stack: self.stack.clone(),
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
    pub auth_dir: Option<TempDir>,
    pub home_dir: Option<TempDir>,
    pub gh_login_attempts: Arc<Mutex<Vec<String>>>,
    pub gh_login_fails: bool,
    pub interactive: bool,
    pub real_credentials: bool,
    pub config_text: Option<String>,
    pub watch_scope: Vec<String>,
    pub fail_state_write: bool,
    pub tui_app: Option<TuiApp>,
    pub tui_closed: bool,
    pub observation: Option<ObservationSession>,
    observation_gateway: Option<FakeGateway>,
    observation_allowance: Option<ApiAllowance>,
    pub observation_update: Option<ObservationUpdate>,
    pub planned_delay: Option<Duration>,
    pub e2e_pending_comment: Option<(String, String, String, String)>,
    pub ignored_pull_requests: Vec<String>,
    pub browser_log: Arc<Mutex<Vec<String>>>,
    pub browser_fails: bool,
    pub e2e_browser_log: Option<PathBuf>,
    pub watcher_config_path: Option<PathBuf>,
    pub setup_original: Option<String>,
    pub setup_token_file: Option<PathBuf>,
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
            auth_dir: None,
            home_dir: None,
            gh_login_attempts: Arc::new(Mutex::new(Vec::new())),
            gh_login_fails: false,
            interactive: false,
            real_credentials: false,
            config_text: None,
            watch_scope: Vec::new(),
            fail_state_write: false,
            tui_app: None,
            tui_closed: false,
            observation: None,
            observation_gateway: None,
            observation_allowance: None,
            observation_update: None,
            planned_delay: None,
            e2e_pending_comment: None,
            ignored_pull_requests: Vec::new(),
            browser_log: Arc::new(Mutex::new(Vec::new())),
            browser_fails: false,
            e2e_browser_log: None,
            watcher_config_path: None,
            setup_original: None,
            setup_token_file: None,
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
        let token = format!("token-{token_var}");
        self.accounts.insert(
            label.to_string(),
            AccountFixture {
                label: label.to_string(),
                login: login.to_string(),
                token,
                token_var: token_var.clone(),
                expected_login: None,
                auth: AccountAuth::Env {
                    variable: token_var,
                },
            },
        );
    }

    fn auth_root(&mut self) -> PathBuf {
        if self.auth_dir.is_none() {
            let dir = TempDir::new().expect("auth dir");
            let bin = dir.path().join("bin");
            std::fs::create_dir_all(&bin).expect("auth bin");
            let script = bin.join("gh");
            std::fs::write(
                &script,
                "#!/bin/sh
if [ \"$1\" = \"auth\" ] && [ \"$2\" = \"token\" ]; then\n  shift 2\n  user=\"\"\n  while [ $# -gt 0 ]; do\n    case \"$1\" in\n      --user) user=\"$2\"; shift 2 ;;\n      *) shift ;;\n    esac\n  done\n  if [ -n \"$user\" ] && [ -f \"$FAKE_GH_DIR/token-$user\" ]; then\n    cat \"$FAKE_GH_DIR/token-$user\"\n    exit 0\n  fi\nfi\nexit 1\n",
            )
            .expect("fake gh");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut permissions = std::fs::metadata(&script)
                    .expect("fake gh metadata")
                    .permissions();
                permissions.set_mode(0o755);
                std::fs::set_permissions(&script, permissions).expect("fake gh mode");
            }
            self.auth_dir = Some(dir);
        }
        self.auth_dir
            .as_ref()
            .expect("auth dir")
            .path()
            .to_path_buf()
    }

    pub fn add_gh_account(&mut self, label: &str, login: &str, authenticated: bool) {
        self.add_account(label, login, false);
        if let Some(account) = self.accounts.get_mut(label) {
            account.auth = AccountAuth::GhOAuth {
                login: login.to_string(),
                host: "github.com".to_string(),
            };
            account.expected_login = Some(login.to_string());
            account.token = format!("token-{login}");
        }
        self.real_credentials = true;
        let root = self.auth_root();
        let token_file = root.join(format!("token-{login}"));
        if authenticated {
            std::fs::write(token_file, format!("token-{login}")).expect("gh token");
        } else {
            let _ = std::fs::remove_file(token_file);
        }
    }

    pub fn add_file_account(
        &mut self,
        label: &str,
        login: &str,
        expected: Option<&str>,
        mode: u32,
    ) {
        self.add_account(label, login, false);
        let root = self.auth_root();
        let path = root.join(format!("{label}.token"));
        let token = self.accounts[label].token.clone();
        std::fs::write(&path, token).expect("token file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&path)
                .expect("token metadata")
                .permissions();
            permissions.set_mode(mode);
            std::fs::set_permissions(&path, permissions).expect("token mode");
        }
        if let Some(account) = self.accounts.get_mut(label) {
            account.auth = AccountAuth::File { path, mode };
            account.expected_login = expected.map(str::to_string);
        }
        self.real_credentials = true;
    }

    pub fn set_gh_authenticated(&mut self, login: &str, authenticated: bool) {
        let root = self.auth_root();
        let token_file = root.join(format!("token-{login}"));
        if authenticated {
            std::fs::write(token_file, format!("token-{login}")).expect("gh token");
        } else {
            let _ = std::fs::remove_file(token_file);
        }
    }

    pub fn set_expected_login(&mut self, label: &str, login: &str) {
        if let Some(account) = self.accounts.get_mut(label) {
            account.expected_login = Some(login.to_string());
        }
    }

    pub fn rotate_credential(&mut self, label: &str, new_login: &str) {
        if let Some(account) = self.accounts.get_mut(label) {
            account.login = new_login.to_string();
            account.token = format!("token-{new_login}");
            if let AccountAuth::GhOAuth { login, .. } = &mut account.auth {
                *login = new_login.to_string();
            }
        }
        let root = self.auth_root();
        let _ = std::fs::write(
            root.join(format!("token-{new_login}")),
            format!("token-{new_login}"),
        );
    }

    pub fn set_config_text(&mut self, text: String) {
        self.config_text = Some(text);
    }

    fn config_text_from_accounts(&self) -> String {
        let mut text = String::new();
        if !self.ignored_pull_requests.is_empty() {
            let entries = self
                .ignored_pull_requests
                .iter()
                .map(|entry| format!("\"{entry}\""))
                .collect::<Vec<_>>()
                .join(", ");
            text.push_str(&format!("[watch]\nignored_pull_requests = [{entries}]\n\n"));
        }
        for account in self.accounts.values() {
            text.push_str(&format!("[accounts.{}]\n", account.label));
            match &account.auth {
                AccountAuth::Env { variable } => {
                    text.push_str(&format!("token_env = \"{variable}\"\n"));
                }
                AccountAuth::Command { command } => {
                    text.push_str(&format!("token_command = \"{command}\"\n"));
                }
                AccountAuth::File { path, .. } => {
                    text.push_str(&format!("token_file = \"{}\"\n", path.display()));
                }
                AccountAuth::GhOAuth { login, .. } => {
                    text.push_str("auth = \"gh_oauth\"\n");
                    text.push_str(&format!("github_user = \"{login}\"\n"));
                }
            }
            if !matches!(account.auth, AccountAuth::GhOAuth { .. }) {
                if let Some(expected) = account.expected_login.as_ref() {
                    text.push_str(&format!("github_user = \"{expected}\"\n"));
                }
            }
        }
        text
    }

    pub fn enable_real_credentials(&mut self) {
        self.real_credentials = true;
        self.auth_root();
    }

    pub fn account_token_file_path(&self, label: &str) -> PathBuf {
        match &self.accounts[label].auth {
            AccountAuth::File { path, .. } => path.clone(),
            other => panic!("account {label} is not a token-file account: {other:?}"),
        }
    }

    pub fn set_config_token_file(&mut self, label: &str, path: &str) {
        self.set_config_text(format!("[accounts.{label}]\ntoken_file = \"{path}\"\n"));
    }

    pub fn set_config_missing_token_file(&mut self, label: &str) {
        let path = self.state_path().with_file_name("missing.token");
        let _ = std::fs::remove_file(&path);
        self.set_config_token_file(label, &path.display().to_string());
    }

    pub fn set_config_invalid(&mut self, label: &str, invalid: &str) {
        let text = match invalid {
            "no credential source" => format!("[accounts.{label}]\n"),
            "an unknown auth mode" => {
                format!("[accounts.{label}]\nauth = \"magic\"\ntoken_env = \"X\"\n")
            }
            "two personal access token sources" => format!(
                "[accounts.{label}]\ntoken_env = \"A\"\ntoken_command = \"echo B\"\n"
            ),
            "gh OAuth but no expected login" => {
                format!("[accounts.{label}]\nauth = \"gh_oauth\"\n")
            }
            "gh OAuth and a personal access token source" => format!(
                "[accounts.{label}]\nauth = \"gh_oauth\"\ngithub_user = \"octo-work\"\ntoken_env = \"A\"\n"
            ),
            "a literal personal access token" => {
                format!("[accounts.{label}]\ntoken = \"ghp_x\"\n")
            }
            other => panic!("unknown invalid configuration {other}"),
        };
        self.set_config_text(text);
    }

    pub fn prepare_home_token_file(&mut self, name: &str, token: &str) {
        let home = TempDir::new().expect("home dir");
        let path = home.path().join(name);
        std::fs::write(&path, token).expect("home token file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&path)
                .expect("home token metadata")
                .permissions();
            permissions.set_mode(0o600);
            std::fs::set_permissions(&path, permissions).expect("home token mode");
        }
        self.home_dir = Some(home);
    }

    fn apply_auth_env(&self) {
        if let Some(dir) = self.auth_dir.as_ref() {
            let bin = dir.path().join("bin");
            let path = format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            );
            std::env::set_var("PATH", path);
            std::env::set_var("FAKE_GH_DIR", dir.path());
        }
        for account in self.accounts.values() {
            if let AccountAuth::Env { variable } = &account.auth {
                std::env::set_var(variable, &account.token);
            }
        }
        if let Some(home) = self.home_dir.as_ref() {
            std::env::set_var("HOME", home.path());
        }
    }

    fn credentials(&self, interactive: bool) -> Arc<dyn CredentialSource> {
        if self.real_credentials {
            let dir = self
                .auth_dir
                .as_ref()
                .expect("auth dir")
                .path()
                .to_path_buf();
            Arc::new(SystemCredentialSource::with_login(
                interactive,
                Some(Arc::new(RecordingGhLogin {
                    attempts: self.gh_login_attempts.clone(),
                    fails: self.gh_login_fails,
                    dir,
                })),
            ))
        } else {
            Arc::new(FakeCredentials {
                fail_invalid: self.fail_invalid_credential,
            })
        }
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
                let credential = match &account.auth {
                    AccountAuth::Env { variable } => CredentialSpec::Env {
                        variable: variable.clone(),
                    },
                    AccountAuth::Command { command } => CredentialSpec::Command {
                        command: command.clone(),
                    },
                    AccountAuth::File { path, .. } => CredentialSpec::File { path: path.clone() },
                    AccountAuth::GhOAuth { login, host } => CredentialSpec::GhOAuth {
                        login: login.clone(),
                        host: host.clone(),
                    },
                };
                (
                    account.label.clone(),
                    AccountConfig {
                        credential,
                        graphql_url: Url::parse("http://127.0.0.1/graphql").expect("url"),
                        expected_login: account.expected_login.clone(),
                    },
                )
            })
            .collect();
        AppConfig {
            poll_interval_secs: 1,
            core: CoreConfig {
                accounts,
                events: self.events.clone(),
                watch: passuff_config::WatchConfig {
                    mode: if self.watch_scope.is_empty() {
                        passuff_config::WatchMode::All
                    } else {
                        passuff_config::WatchMode::Selected
                    },
                    repositories: self.watch_scope.clone(),
                    ignored_pull_requests: self.ignored_pull_requests.clone(),
                },
            },
            notifications: self.notifications.clone(),
        }
    }

    fn gateway_snapshot(&self) -> Vec<(String, String, Vec<AuthoredPullRequest>)> {
        self.accounts
            .values()
            .map(|account| {
                (
                    account.token.clone(),
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
        self.apply_auth_env();
        let config = if let Some(text) = self.config_text.clone() {
            ConfigLoader::parse(&text)
        } else if self.invalid_config {
            Err(ConfigError::Validation("invalid configuration".to_string()))
        } else if self.real_credentials {
            ConfigLoader::parse(&self.config_text_from_accounts())
        } else {
            Ok(self.build_config())
        };
        let gateway = Arc::new(FakeGateway::new(
            self.gateway_snapshot(),
            self.check_clock.clone(),
        ));
        let credentials = self.credentials(false);
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

    pub async fn run_watch_cycles(&mut self, cycles: usize) {
        if self.pending_baseline {
            self.ensure_baseline().await;
            self.last_report = None;
        }
        self.apply_auth_env();
        let config = if let Some(text) = self.config_text.clone() {
            ConfigLoader::parse(&text)
        } else if self.real_credentials {
            ConfigLoader::parse(&self.config_text_from_accounts())
        } else {
            Ok(self.build_config())
        };
        let config = match config {
            Ok(config) => config,
            Err(error) => {
                let runtime = CliRuntime::new(CliPorts {
                    gateway: Arc::new(FakeGateway::new(
                        self.gateway_snapshot(),
                        self.check_clock.clone(),
                    )),
                    store: Arc::new(JsonStateStore::new(self.state_path())),
                    credentials: self.credentials(false),
                    delivery: Arc::new(RecordingDelivery::new(
                        Arc::new(RecordingSoundPlayer::new(self.sound_log.clone())),
                        self.notification_log.clone(),
                    )),
                });
                let outcome = runtime.run_check(Err(error)).await;
                self.last_report = outcome.report;
                self.last_transcript = outcome.transcript;
                self.last_status = outcome.status;
                return;
            }
        };
        let gateway = Arc::new(FakeGateway::new(
            self.gateway_snapshot(),
            self.check_clock.clone(),
        ));
        let credentials = self.credentials(self.interactive);
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
        let outcome = runtime
            .run_watch_with(Ok(config), &StopAfter::new(cycles))
            .await;
        self.last_report = outcome.report;
        self.last_transcript = outcome.transcript;
        self.last_status = outcome.status;
    }

    pub fn state_store(&self) -> Arc<dyn SnapshotStore> {
        Arc::new(JsonStateStore::new(self.state_path()))
    }

    pub async fn observe_account(&mut self) {
        if self.observation.is_none() {
            self.start_observation().await;
        }
        if let Some(gateway) = self.observation_gateway.clone() {
            gateway.replace(self.gateway_snapshot());
        }
        let session = self.observation.as_mut().expect("observation session");
        let update = session.tick().await;
        let inbox = Inbox::new(self.state_store());
        if let Some(app) = self.tui_app.as_mut() {
            app.reload(&inbox, &update.config.core)
                .await
                .expect("reload after observation");
            app.set_status(update.failure.clone());
            self.last_transcript = app.snapshot();
        }
    }

    async fn start_observation(&mut self) {
        self.apply_auth_env();
        let text = self
            .config_text
            .clone()
            .unwrap_or_else(|| self.config_text_from_accounts());
        let config = ConfigLoader::parse(&text).expect("observation configuration");
        let path = self.scope_config_path();
        std::fs::write(&path, &text).expect("observation configuration file");
        let gateway = FakeGateway::new(self.gateway_snapshot(), self.check_clock.clone());
        let credentials = self.credentials(false);
        let store: Arc<dyn SnapshotStore> = Arc::new(JsonStateStore::new(self.state_path()));
        let sound: Arc<dyn SoundPlayer> =
            Arc::new(RecordingSoundPlayer::new(self.sound_log.clone()));
        let delivery: Arc<dyn NotificationDelivery> =
            Arc::new(RecordingDelivery::new(sound, self.notification_log.clone()));
        let runtime = CliRuntime::new(CliPorts {
            gateway: Arc::new(gateway.clone()),
            store,
            credentials,
            delivery,
        });
        let session = ObservationSession::open(runtime, config, path)
            .await
            .expect("observation session");
        self.observation = Some(session);
        self.observation_gateway = Some(gateway);
    }

    pub fn observation_probe_calls(&self) -> usize {
        self.observation_gateway
            .as_ref()
            .map(FakeGateway::probe_calls)
            .unwrap_or(0)
    }

    pub fn browser_urls(&self) -> Vec<String> {
        if self.e2e {
            if let Some(path) = self.e2e_browser_log.as_ref() {
                if let Ok(text) = std::fs::read_to_string(path) {
                    return text.lines().map(str::to_string).collect();
                }
            }
            Vec::new()
        } else {
            self.browser_log.lock().expect("browser log").clone()
        }
    }

    pub fn fail_browser(&mut self) {
        self.browser_fails = true;
    }

    pub fn observation_observe_calls(&self) -> usize {
        self.observation_gateway
            .as_ref()
            .map(FakeGateway::observe_calls)
            .unwrap_or(0)
    }

    pub fn fail_next_observation(&self) {
        if let Some(gateway) = self.observation_gateway.as_ref() {
            gateway.fail_next_probe();
        }
    }

    pub fn set_observation_allowance(&mut self, remaining: u32, reset_in_secs: i64) {
        self.observation_allowance = Some(ApiAllowance {
            cost: 1,
            remaining,
            reset_at: (chrono::Utc::now() + chrono::Duration::seconds(reset_in_secs))
                .to_rfc3339(),
        });
    }

    pub fn plan_observation_delay(&mut self) {
        self.planned_delay = Some(next_delay(
            self.observation_allowance.as_ref(),
            chrono::Utc::now(),
        ));
    }

    pub fn planned_delay(&self) -> Option<Duration> {
        self.planned_delay
    }

    async fn with_state_write_guard<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        if !self.fail_state_write {
            return future.await;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir = self.state_path().parent().unwrap().to_path_buf();
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500));
            let result = future.await;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
            result
        }
        #[cfg(not(unix))]
        {
            future.await
        }
    }

    pub async fn run_status(&mut self) {
        let inbox = Inbox::new(self.state_store());
        let cli = InboxCli::new(Arc::new(inbox));
        let config = match self.config_text.clone() {
            Some(text) => ConfigLoader::parse(&text),
            None => Ok(self.build_config()),
        };
        let outcome = self.with_state_write_guard(cli.status(config)).await;
        self.last_report = outcome.report;
        self.last_transcript = outcome.transcript;
        self.last_status = outcome.status;
    }

    pub async fn run_inbox(&mut self, all: bool) {
        let inbox = Inbox::new(self.state_store());
        let cli = InboxCli::new(Arc::new(inbox));
        let config = match self.config_text.clone() {
            Some(text) => ConfigLoader::parse(&text),
            None => Ok(self.build_config()),
        };
        let outcome = self.with_state_write_guard(cli.inbox(config, all)).await;
        self.last_report = outcome.report;
        self.last_transcript = outcome.transcript;
        self.last_status = outcome.status;
    }

    pub async fn run_read(&mut self, id: Option<String>, all: bool) {
        let inbox = Inbox::new(self.state_store());
        let cli = InboxCli::new(Arc::new(inbox));
        let config = match self.config_text.clone() {
            Some(text) => ConfigLoader::parse(&text),
            None => Ok(self.build_config()),
        };
        let outcome = self.with_state_write_guard(cli.read(config, id, all)).await;
        self.last_report = outcome.report;
        self.last_transcript = outcome.transcript;
        self.last_status = outcome.status;
    }

    pub async fn run_tui_snapshot(&mut self) {
        if self.e2e {
            self.e2e_run_tui_snapshot().await;
            return;
        }
        let inbox = Inbox::new(self.state_store());
        let config = match self.config_text.clone() {
            Some(text) => ConfigLoader::parse(&text),
            None => Ok(self.build_config()),
        };
        let loaded = self
            .with_state_write_guard(load_view(&config, &inbox))
            .await;
        match loaded {
            Ok(view) => {
                let app = TuiApp::new(view);
                self.last_transcript = app.snapshot();
                self.last_status = 0;
                self.tui_app = Some(app);
            }
            Err(error) => {
                self.last_transcript = error.message;
                self.last_status = error.exit;
                self.tui_app = None;
            }
        }
    }

    pub async fn run_tui_press(&mut self, key: &str) {
        let inbox = Inbox::new(self.state_store());
        let config = self.build_config();
        let mut app = self.tui_app.take().expect("tui opened");
        let code = match key {
            "q" => crossterm::event::KeyCode::Char('q'),
            "esc" => crossterm::event::KeyCode::Esc,
            "j" => crossterm::event::KeyCode::Char('j'),
            "k" => crossterm::event::KeyCode::Char('k'),
            "down" => crossterm::event::KeyCode::Down,
            "up" => crossterm::event::KeyCode::Up,
            "r" => crossterm::event::KeyCode::Char('r'),
            "a" => crossterm::event::KeyCode::Char('a'),
            "tab" => crossterm::event::KeyCode::Tab,
            "enter" => crossterm::event::KeyCode::Enter,
            other => panic!("unknown key {other}"),
        };
        let event = crossterm::event::KeyEvent::new(code, crossterm::event::KeyModifiers::NONE);
        match app.handle_key(event, &inbox, &config.core).await {
            Ok(TuiAction::Quit) => self.tui_closed = true,
            Ok(TuiAction::Continue) => self.last_transcript = app.snapshot(),
            Ok(TuiAction::OpenUrl(url)) => {
                if self.browser_fails {
                    app.set_status(Some("could not open browser: fake failure".to_string()));
                } else {
                    self.browser_log.lock().expect("browser log").push(url);
                    app.close_popup();
                }
                self.last_transcript = app.snapshot();
            }
            Err(error) => {
                self.last_status = 3;
                self.last_transcript = format!("passuff: state error: {error}");
            }
        }
        self.tui_app = Some(app);
    }

    pub fn run_tui_binary(&mut self) {
        self.run_tui_binary_with(&["--snapshot"]);
    }

    pub fn run_tui_binary_without_snapshot(&mut self) {
        self.run_tui_binary_with(&[]);
    }

    fn run_tui_binary_with(&mut self, args: &[&str]) {
        let dir = self
            .state_dir
            .as_ref()
            .expect("state dir")
            .path()
            .to_path_buf();
        let config_path = dir.join("config.toml");
        let text = self
            .config_text
            .clone()
            .unwrap_or_else(|| self.config_text_from_accounts());
        std::fs::write(&config_path, text).expect("config write");
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_passuff-tui"))
            .args(args)
            .arg("--config")
            .arg(&config_path)
            .arg("--state")
            .arg(self.state_path())
            .output()
            .expect("run passuff-tui");
        self.last_status = output.status.code().unwrap_or(-1);
        self.last_transcript = String::from_utf8_lossy(&output.stderr).to_string();
    }

    pub fn assert_focus(&self, expected: &str) {
        let app = self.tui_app.as_ref().expect("tui opened");
        let actual = match app.focus() {
            passuff_cli::tui::Pane::Watched => "PRs",
            passuff_cli::tui::Pane::Activity => "activity",
        };
        assert_eq!(actual, expected, "focus");
    }

    pub fn assert_selected_pull_request(&self, key: &str) {
        let app = self.tui_app.as_ref().expect("tui opened");
        let actual = app
            .selected_pull_request()
            .map(|pull_request| pull_request.key.to_string());
        assert_eq!(actual.as_deref(), Some(key), "selected pull request");
    }

    pub async fn press_many(&mut self, key: &str, count: usize) {
        for _ in 0..count {
            self.run_tui_press(key).await;
        }
    }

    pub async fn refresh_tui(&mut self) {
        let inbox = Inbox::new(self.state_store());
        let config = self.build_config();
        let mut app = self.tui_app.take().expect("tui opened");
        app.reload(&inbox, &config.core).await.expect("tui refresh");
        self.last_transcript = app.snapshot();
        self.tui_app = Some(app);
    }

    pub fn add_many_pull_requests(&mut self, repository: &str, author: &str, count: u64) {
        for number in 1..=count {
            self.add_pr(&format!("{repository}#{number}"), author);
        }
    }

    pub fn assert_selected_activity(&self, summary: &str) {
        let app = self.tui_app.as_ref().expect("tui opened");
        assert_eq!(
            app.selected_summary().as_deref(),
            Some(summary),
            "selected activity differs"
        );
    }

    pub async fn assert_activity_read(&self, summary: &str, expected: bool) {
        let state = self.state_store().load().await.expect("state");
        let record = state
            .activity
            .iter()
            .find(|record| activity_summary(&record.event) == summary)
            .unwrap_or_else(|| panic!("no activity {summary}"));
        assert_eq!(record.read, expected, "activity {summary}");
    }

    pub async fn newest_activity_id(&self) -> String {
        let state = self.state_store().load().await.expect("state");
        state.activity.last().expect("activity").id.clone()
    }

    pub fn write_state(&self, state: &WatcherState) {
        let text = serde_json::to_string_pretty(state).expect("state json");
        std::fs::write(self.state_path(), text).expect("state write");
    }

    pub fn seed_watched_pull_request(&mut self, key: &str) {
        let label = self
            .accounts
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "personal".to_string());
        let account = self
            .accounts
            .get(&label)
            .cloned()
            .expect("configured account");
        let fixture = self.pull_requests.get(key).expect("pull request").clone();
        let authored = fixture.to_authored();
        let watched =
            WatchedPullRequest::from_baseline(&authored, "2026-11-02T10:00:00Z".to_string());
        let mut state = WatcherState {
            version: 2,
            accounts: BTreeMap::new(),
            activity: Vec::new(),
            next_activity_id: 0,
            last_scope: None,
        };
        state.accounts.insert(
            label,
            passuff_core::AccountState {
                login: account.login,
                last_check_at: Some("2026-11-02T10:00:00Z".to_string()),
                pull_requests: [(key.to_string(), watched)].into(),
            },
        );
        self.write_state(&state);
    }

    pub fn seed_version_one_state(&mut self) {
        let label = self
            .accounts
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "personal".to_string());
        let account = self
            .accounts
            .get(&label)
            .cloned()
            .expect("configured account");
        let fixture = PullRequestFixture::new(parse_key("acme/api#42"), account.login.clone());
        let watched = WatchedPullRequest::from_baseline(
            &fixture.to_authored(),
            "2026-11-02T10:00:00Z".to_string(),
        );
        let mut state = WatcherState {
            version: 1,
            accounts: BTreeMap::new(),
            activity: Vec::new(),
            next_activity_id: 0,
            last_scope: None,
        };
        state.accounts.insert(
            label,
            passuff_core::AccountState {
                login: account.login,
                last_check_at: Some("2026-11-02T10:00:00Z".to_string()),
                pull_requests: [("acme/api#42".to_string(), watched)].into(),
            },
        );
        self.write_state(&state);
    }

    pub fn seed_activity_records(&mut self, count: u64) {
        let label = self
            .accounts
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "personal".to_string());
        let login = self
            .accounts
            .get(&label)
            .map(|account| account.login.clone())
            .unwrap_or_else(|| "octo-personal".to_string());
        let mut state = WatcherState {
            version: 2,
            accounts: BTreeMap::new(),
            activity: Vec::new(),
            next_activity_id: count,
            last_scope: None,
        };
        state.accounts.insert(
            label,
            passuff_core::AccountState {
                login,
                last_check_at: None,
                pull_requests: BTreeMap::new(),
            },
        );
        for id in 1..=count {
            state.activity.push(ActivityRecord {
                id: format!("a{id}"),
                key: parse_key("acme/api#42"),
                observed_at: "2026-11-02T10:00:00Z".to_string(),
                event: ActivityEvent::Comment {
                    author: "seed".to_string(),
                    body_excerpt: "seed".to_string(),
                },
                read: false,
            });
        }
        self.write_state(&state);
    }

    pub fn setup_config_path(&self) -> PathBuf {
        if self.e2e {
            self.e2e_config_dir
                .as_ref()
                .expect("e2e configured")
                .path()
                .join("config.toml")
        } else {
            self.scope_config_path()
        }
    }

    pub fn setup_config_text(&self) -> String {
        std::fs::read_to_string(self.setup_config_path()).unwrap_or_default()
    }

    pub async fn run_setup(&mut self, answers: Vec<String>) {
        let path = self.setup_config_path();
        if self.e2e {
            let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_passuff"));
            command
                .arg("--config")
                .arg(&path)
                .arg("setup")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            self.configure_child(&mut command);
            let mut child = command.spawn().expect("run setup");
            {
                use std::io::Write;
                let stdin = child.stdin.as_mut().expect("stdin");
                for answer in &answers {
                    writeln!(stdin, "{answer}").expect("setup stdin");
                }
            }
            let output = child.wait_with_output().expect("setup output");
            self.last_status = output.status.code().unwrap_or(3);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            self.last_transcript = if stdout.is_empty() { stderr } else { stdout };
        } else {
            let mut output = Vec::new();
            let refs: Vec<&str> = answers.iter().map(String::as_str).collect();
            let result = passuff_cli::setup::run_scripted(&path, &refs, &mut output);
            self.last_transcript = String::from_utf8_lossy(&output).to_string();
            self.last_status = match result {
                Ok(()) => 0,
                Err(error) => error.exit,
            };
            if self.last_status == 0 {
                self.config_text = Some(self.setup_config_text());
            }
        }
    }

    pub fn scope_config_path(&self) -> PathBuf {
        self.state_dir
            .as_ref()
            .expect("state dir")
            .path()
            .join("config.toml")
    }

    pub fn ensure_scope_config(&self) {
        let path = self.scope_config_path();
        let text = self
            .config_text
            .clone()
            .unwrap_or_else(|| self.config_text_from_accounts());
        std::fs::write(path, text).expect("scope config write");
    }

    pub fn scope_editor(&self) -> ScopeEditor {
        self.ensure_scope_config();
        ScopeEditor::new(self.scope_config_path())
    }

    pub fn sync_scope_config(&mut self) {
        self.config_text =
            Some(std::fs::read_to_string(self.scope_config_path()).expect("scope config read"));
    }

    pub fn scope_config(&self) -> AppConfig {
        ConfigLoader::parse(
            &std::fs::read_to_string(self.scope_config_path()).expect("scope config read"),
        )
        .expect("valid scope config")
    }

    pub fn e2e_config_text(&self) -> String {
        let path = self
            .e2e_config_dir
            .as_ref()
            .expect("e2e configured")
            .path()
            .join("config.toml");
        std::fs::read_to_string(path).unwrap_or_default()
    }

    pub fn e2e_state_json(&self) -> serde_json::Value {
        let path = self.e2e_state_path.as_ref().expect("e2e state");
        let text = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }

    pub async fn run_watch_cycle_from_path(&mut self, path: PathBuf) {
        self.apply_auth_env();
        let gateway = Arc::new(FakeGateway::new(
            self.gateway_snapshot(),
            self.check_clock.clone(),
        ));
        let credentials = self.credentials(false);
        let store: Arc<dyn SnapshotStore> = Arc::new(JsonStateStore::new(self.state_path()));
        let sound: Arc<dyn SoundPlayer> =
            Arc::new(RecordingSoundPlayer::new(self.sound_log.clone()));
        let delivery: Arc<dyn NotificationDelivery> =
            Arc::new(RecordingDelivery::new(sound, self.notification_log.clone()));
        let runtime = CliRuntime::new(CliPorts {
            gateway,
            store,
            credentials,
            delivery,
        })
        .with_config_path(path.clone());
        let outcome = runtime.run_watch_cycle(&path).await;
        self.last_report = outcome.report;
        self.last_transcript = outcome.transcript;
        self.last_status = outcome.status;
    }

    pub fn state_json(&self) -> serde_json::Value {
        let text = std::fs::read_to_string(self.state_path()).unwrap_or_default();
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
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
    live: Arc<Mutex<BTreeMap<String, (String, Vec<AuthoredPullRequest>)>>>,
    clock: Arc<Mutex<u64>>,
    probe_calls: Arc<AtomicUsize>,
    observe_calls: Arc<AtomicUsize>,
    fail_probe: Arc<AtomicBool>,
    allowance: Arc<Mutex<Option<ApiAllowance>>>,
}

impl FakeGateway {
    fn new(
        snapshot: Vec<(String, String, Vec<AuthoredPullRequest>)>,
        clock: Arc<Mutex<u64>>,
    ) -> Self {
        let live = snapshot
            .into_iter()
            .map(|(token, login, pull_requests)| (token, (login, pull_requests)))
            .collect();
        Self {
            live: Arc::new(Mutex::new(live)),
            clock,
            probe_calls: Arc::new(AtomicUsize::new(0)),
            observe_calls: Arc::new(AtomicUsize::new(0)),
            fail_probe: Arc::new(AtomicBool::new(false)),
            allowance: Arc::new(Mutex::new(None)),
        }
    }

    fn replace(&self, snapshot: Vec<(String, String, Vec<AuthoredPullRequest>)>) {
        let live = snapshot
            .into_iter()
            .map(|(token, login, pull_requests)| (token, (login, pull_requests)))
            .collect();
        *self.live.lock().expect("live gateway") = live;
    }

    fn set_allowance(&self, allowance: Option<ApiAllowance>) {
        *self.allowance.lock().expect("allowance") = allowance;
    }

    fn fail_next_probe(&self) {
        self.fail_probe.store(true, Ordering::SeqCst);
    }

    fn probe_calls(&self) -> usize {
        self.probe_calls.load(Ordering::SeqCst)
    }

    fn observe_calls(&self) -> usize {
        self.observe_calls.load(Ordering::SeqCst)
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
        self.observe_calls.fetch_add(1, Ordering::SeqCst);
        let live = self.live.lock().expect("live gateway");
        let Some((login, pull_requests)) = live.get(token) else {
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

    async fn probe(&self, token: &str, _graphql_url: &Url) -> Result<AccountProbe, GitHubError> {
        self.probe_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_probe.swap(false, Ordering::SeqCst) {
            return Err(GitHubError::Transport("unavailable".to_string()));
        }
        let live = self.live.lock().expect("live gateway");
        let Some((login, pull_requests)) = live.get(token) else {
            return Err(GitHubError::Authentication);
        };
        let allowance = self.allowance.lock().expect("allowance").clone();
        Ok(AccountProbe {
            login: login.clone(),
            pull_requests: pull_requests
                .iter()
                .map(|pull_request| ProbePullRequest {
                    repository: pull_request.key.repository.clone(),
                    number: pull_request.key.number,
                    node_id: pull_request.node_id.clone(),
                    updated_at: pull_request.updated_at.clone(),
                    head_oid: pull_request.head_oid.clone(),
                    ci_state: Some(pull_request.ci.state.clone()),
                })
                .collect(),
            allowance,
        })
    }
}

struct RecordingGhLogin {
    attempts: Arc<Mutex<Vec<String>>>,
    fails: bool,
    dir: PathBuf,
}

#[async_trait]
impl GhLogin for RecordingGhLogin {
    async fn login(&self, login: &str, _host: &str) -> Result<(), CredentialError> {
        self.attempts
            .lock()
            .expect("login attempts")
            .push(login.to_string());
        if self.fails {
            return Err(CredentialError::GhLoginFailed);
        }
        std::fs::write(
            self.dir.join(format!("token-{login}")),
            format!("token-{login}"),
        )
        .map_err(|_| CredentialError::CommandFailed)?;
        Ok(())
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
            CredentialSpec::File { .. } | CredentialSpec::GhOAuth { .. } => {
                Err(CredentialError::CommandFailed)
            }
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
            activity: Vec::new(),
            next_activity_id: 0,
            last_scope: None,
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
