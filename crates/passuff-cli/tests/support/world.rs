use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, TimeZone, Utc};
use passuff::args::TuiCli;
use passuff::runtime::{CliRuntime, CommandOutcome, Ports, RecordingBrowserOpener};
use passuff::tui::app::{TuiApp, UiPrefs};
use passuff::tui::{render, snapshot, TuiOptions};
use passuff::{ensure_bundled_sound, RecordingDelivery, RecordingSoundPlayer};
use passuff_config::{AccountConfig, AppConfig, AuthMode, FocusRelation, ScopeEditor, WatchMode};
use passuff_core::{
    ActivityRecord, CiResult, Clock, EventKind, InboxService, JsonStateStore, Notification,
    PullRequestKey, PullRequestSnapshot, PullRequestState, RefreshReport, ReviewState,
    SnapshotStore, StateError, WatchedPullRequest, WatcherState,
};
use tempfile::TempDir;

use super::clock::TestClock;
use super::fake::{FakeCredential, FakeCredentials, FakeData, FakeGatewayFactory};

pub const FIXED_NOW: &str = "2026-11-02T12:00:00Z";

#[derive(cucumber::World)]
#[world(init = Self::new)]
pub struct World {
    pub temp: TempDir,
    pub paths: passuff::PassuffPaths,
    pub config: AppConfig,
    pub data: Arc<Mutex<FakeData>>,
    pub clock: Arc<TestClock>,
    pub delivery: Arc<RecordingDelivery>,
    pub sound: Arc<RecordingSoundPlayer>,
    pub browser: Arc<RecordingBrowserOpener>,
    pub credentials: Arc<FakeCredentials>,
    pub transcripts: Vec<String>,
    pub last_exit: i32,
    pub last_report: Option<RefreshReport>,
    pub tui_output: String,
    pub tui_app: Option<TuiApp>,
    pub tui_exit: i32,
    pub tui_closed: bool,
    pub picker_output: String,
    pub interactive: bool,
    pub watcher_running: bool,
    pub last_reconciliation: DateTime<Utc>,
    pub planned_delay: Option<std::time::Duration>,
    pub token_file: Option<PathBuf>,
    pub sound_display: Option<String>,
    pub config_corrupted: bool,
    pub config_before: Option<String>,
    pub setup_output: String,
    pub home: PathBuf,
}

impl std::fmt::Debug for World {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("World")
            .field("paths", &self.paths)
            .field("transcripts", &self.transcripts.len())
            .field("last_exit", &self.last_exit)
            .finish()
    }
}

impl World {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp dir");
        let home = temp.path().join("home");
        std::fs::create_dir_all(&home).expect("home dir");
        let paths = passuff::PassuffPaths {
            config: temp.path().join("config.toml"),
            state: temp.path().join("state.json"),
            ui: temp.path().join("ui.json"),
        };
        let now = Utc
            .with_ymd_and_hms(2026, 11, 2, 12, 0, 0)
            .single()
            .expect("valid fixed time");
        let config = AppConfig {
            poll_interval_secs: 1,
            ..AppConfig::default()
        };
        let data = Arc::new(Mutex::new(FakeData::new(home.clone(), now)));
        let credentials = Arc::new(FakeCredentials {
            data: data.clone(),
            home: home.clone(),
        });
        std::env::remove_var("PASSUFF_TEST_FAIL_CONFIG_WRITE");
        std::env::remove_var("PASSUFF_TEST_FAIL_STATE_WRITE");
        std::env::remove_var("PASSUFF_GRAPHQL_URL");
        std::env::remove_var("PASSUFF_NOTIFICATION_LOG");
        std::env::remove_var("PASSUFF_SOUND_PLAYER");
        Self {
            temp,
            paths,
            config,
            data,
            clock: Arc::new(TestClock::new(now)),
            delivery: Arc::new(RecordingDelivery::available()),
            sound: Arc::new(RecordingSoundPlayer::available()),
            browser: Arc::new(RecordingBrowserOpener::default()),
            credentials,
            transcripts: Vec::new(),
            last_exit: 0,
            last_report: None,
            tui_output: String::new(),
            tui_app: None,
            tui_exit: 0,
            tui_closed: false,
            picker_output: String::new(),
            interactive: false,
            watcher_running: false,
            last_reconciliation: now,
            planned_delay: None,
            token_file: None,
            sound_display: None,
            config_corrupted: false,
            config_before: None,
            setup_output: String::new(),
            home,
        }
    }

    pub fn store(&self) -> Arc<JsonStateStore> {
        Arc::new(JsonStateStore::new(self.paths.state.clone()))
    }

    pub fn ports(&self) -> Ports {
        Ports {
            gateway_factory: Arc::new(FakeGatewayFactory {
                data: self.data.clone(),
            }),
            credentials: self.credentials.clone(),
            delivery: self.delivery.clone(),
            sound: self.sound.clone(),
            clock: self.clock.clone(),
            browser: self.browser.clone(),
            interactive: self.interactive,
        }
    }

    pub fn runtime(&self) -> CliRuntime {
        CliRuntime::new(self.paths.clone(), self.ports())
    }

    pub fn write_config(&mut self) {
        let text = self.config.to_toml().expect("encode config");
        if let Some(parent) = self.paths.config.parent() {
            std::fs::create_dir_all(parent).expect("config dir");
        }
        std::fs::write(&self.paths.config, text).expect("write config");
        self.config_corrupted = false;
    }

    pub fn reload_config(&mut self) {
        if let Ok(config) = AppConfig::load(&self.paths.config) {
            self.config = config;
        }
    }

    pub fn latest(&self) -> String {
        self.transcripts.last().cloned().unwrap_or_default()
    }

    pub fn record(&mut self, outcome: CommandOutcome) {
        if let Some(report) = outcome.report {
            self.last_report = Some(report);
        }
        self.last_exit = outcome.exit_code;
        self.transcripts
            .push(format!("{}{}", outcome.stdout, outcome.stderr));
    }

    pub fn clear_delivery(&self) {
        self.delivery.clear();
        self.sound.clear();
    }

    pub fn run_check(&mut self) {
        self.clear_delivery();
        let outcome = self.runtime().run_check();
        self.record(outcome);
    }

    pub fn run_status(&mut self) {
        let outcome = self.runtime().run_status();
        self.record(outcome);
    }

    pub fn run_inbox(&mut self, all: bool) {
        let outcome = self.runtime().run_inbox(all);
        self.record(outcome);
    }

    pub fn run_read(&mut self, id: Option<&str>, all: bool) {
        let outcome = self.runtime().run_read(id, all);
        self.record(outcome);
    }

    pub fn run_notify_test(&mut self, no_desktop: bool, no_sound: bool) {
        self.clear_delivery();
        let outcome = self.runtime().run_notify_test(no_desktop, no_sound);
        self.record(outcome);
    }

    pub fn run_watch_start(&mut self) {
        self.clear_delivery();
        let outcome = self.runtime().run_watch_start();
        let exit_code = outcome.exit_code;
        self.record(outcome);
        self.watcher_running = exit_code == 0;
    }

    pub fn run_watch_cycle(&mut self) {
        self.clear_delivery();
        let outcome = self.runtime().run_watch_cycle_reload(self.interactive);
        self.record(outcome);
    }

    pub fn run_watch_cycle_twice(&mut self) {
        self.clear_delivery();
        let runtime = self.runtime();
        let first = runtime.run_watch_cycle_reload(self.interactive);
        let second = runtime.run_watch_cycle_reload(self.interactive);
        let exit_code = first.exit_code.max(second.exit_code);
        let stdout = format!("{}{}", first.stdout, second.stdout);
        let stderr = format!("{}{}", first.stderr, second.stderr);
        self.last_exit = exit_code;
        self.transcripts.push(format!("{stdout}{stderr}"));
    }

    pub fn run_setup(&mut self, input: &str) {
        let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
        let mut output = Vec::new();
        let outcome = self.runtime().run_setup_capture(&mut cursor, &mut output);
        self.setup_output = outcome.stdout.clone();
        self.record(outcome);
        if self.paths.config.exists() {
            self.reload_config();
        }
    }

    pub fn state(&self) -> WatcherState {
        self.store()
            .load()
            .map(|outcome| outcome.state)
            .unwrap_or_else(|_| WatcherState::new())
    }

    pub fn save_state(&self, state: &WatcherState) {
        self.store().save(state).expect("save state");
    }

    pub fn write_raw_state(&self, text: &str) {
        std::fs::write(&self.paths.state, text).expect("write raw state");
    }

    pub fn write_raw_config(&self, text: &str) {
        if let Some(parent) = self.paths.config.parent() {
            std::fs::create_dir_all(parent).expect("config dir");
        }
        std::fs::write(&self.paths.config, text).expect("write raw config");
    }

    pub fn state_text(&self) -> String {
        std::fs::read_to_string(&self.paths.state).unwrap_or_default()
    }

    pub fn config_text(&self) -> String {
        std::fs::read_to_string(&self.paths.config).unwrap_or_default()
    }

    pub fn add_account(
        &mut self,
        label: &str,
        account: AccountConfig,
        credential: Option<FakeCredential>,
    ) {
        self.config.accounts.insert(label.to_string(), account);
        if let Some(credential) = credential {
            if let Ok(mut data) = self.data.lock() {
                data.credentials.insert(label.to_string(), credential);
            }
        }
        self.write_config();
    }

    pub fn add_gh_account(&mut self, label: &str, login: &str) {
        self.add_account(
            label,
            AccountConfig {
                auth: Some(AuthMode::GhOAuth),
                github_user: Some(login.to_string()),
                ..AccountConfig::default()
            },
            Some(FakeCredential::Gh {
                login: login.to_string(),
            }),
        );
    }

    pub fn add_pat_account(&mut self, label: &str, login: &str, token: &str) {
        self.add_account(
            label,
            AccountConfig {
                auth: Some(AuthMode::GithubPat),
                github_user: Some(login.to_string()),
                token_env: Some(format!("{}_TOKEN", label.to_ascii_uppercase())),
                ..AccountConfig::default()
            },
            Some(FakeCredential::Token {
                token: token.to_string(),
            }),
        );
    }

    pub fn account_label_for_login(&self, login: &str) -> Option<String> {
        self.config
            .accounts
            .iter()
            .find(|(_, account)| {
                account
                    .github_user
                    .as_deref()
                    .map(|expected| expected.eq_ignore_ascii_case(login))
                    .unwrap_or(false)
            })
            .map(|(label, _)| label.clone())
    }

    pub fn first_account_label(&self) -> Option<String> {
        self.config.accounts.keys().next().cloned()
    }

    pub fn add_pull_request(&mut self, key: &PullRequestKey, author: &str) -> PullRequestSnapshot {
        let now = self.clock.now();
        let snapshot = {
            let mut data = self.data.lock().expect("fake data");
            let snapshot = data.insert_pull_request(key.clone(), author, now);
            snapshot.created_at = Some(now - Duration::hours(1));
            snapshot.updated_at = Some(now - Duration::hours(1));
            snapshot.clone()
        };
        snapshot
    }

    pub fn add_comment(
        &mut self,
        key: &PullRequestKey,
        author: &str,
        body: &str,
        occurred_at: Option<DateTime<Utc>>,
    ) {
        let now = self.clock.now();
        if let Ok(mut data) = self.data.lock() {
            data.add_comment(key, author, body, occurred_at, now);
        }
        if self.watcher_running {
            let outcome = self.runtime().run_watch_cycle_reload(self.interactive);
            self.record(outcome);
        }
    }

    pub fn add_review_comment(&mut self, key: &PullRequestKey, author: &str, body: &str) {
        let now = self.clock.now();
        if let Ok(mut data) = self.data.lock() {
            data.add_review_comment(key, author, body, now);
        }
        if self.watcher_running {
            let outcome = self.runtime().run_watch_cycle_reload(self.interactive);
            self.record(outcome);
        }
    }

    pub fn add_review(
        &mut self,
        key: &PullRequestKey,
        author: &str,
        state: ReviewState,
        body: &str,
    ) {
        let now = self.clock.now();
        if let Ok(mut data) = self.data.lock() {
            data.add_review(key, author, state, body, now);
        }
        if self.watcher_running {
            let outcome = self.runtime().run_watch_cycle_reload(self.interactive);
            self.record(outcome);
        }
    }

    pub fn set_ci(&mut self, key: &PullRequestKey, ci: CiResult) {
        let now = self.clock.now();
        if let Ok(mut data) = self.data.lock() {
            if let Some(pull_request) = data.pull_requests.get_mut(key) {
                pull_request.ci = ci;
                pull_request.updated_at = Some(now + Duration::seconds(5));
            }
        }
    }

    pub fn set_state(&mut self, key: &PullRequestKey, state: PullRequestState) {
        let now = self.clock.now();
        if let Ok(mut data) = self.data.lock() {
            if let Some(pull_request) = data.pull_requests.get_mut(key) {
                pull_request.state = state;
                if state == PullRequestState::Merged {
                    pull_request.merged_at = Some(now + Duration::seconds(10));
                }
                if state == PullRequestState::Closed {
                    pull_request.closed_at = Some(now + Duration::seconds(10));
                }
                pull_request.updated_at = Some(now + Duration::seconds(10));
            }
        }
    }

    pub fn snapshot_for(&self, key: &PullRequestKey) -> Option<PullRequestSnapshot> {
        self.data
            .lock()
            .ok()
            .and_then(|data| data.snapshot_for(key))
    }

    pub fn seed_observed(&mut self, keys: &[PullRequestKey], ci_failing: bool) {
        let now = self.clock.now();
        let mut state = self.state();
        for key in keys {
            let Some(snapshot) = self.snapshot_for(key) else {
                continue;
            };
            let label = self
                .account_label_for_login(&snapshot.author)
                .or_else(|| self.first_account_label())
                .expect("configured account");
            let login = self
                .config
                .accounts
                .get(&label)
                .and_then(|account| account.github_user.clone())
                .unwrap_or_else(|| snapshot.author.clone());
            let account = state.accounts.entry(label.clone()).or_default();
            account.login = Some(login.clone());
            account.last_check = Some(now);
            let mut relations = snapshot.relations.clone();
            if login.eq_ignore_ascii_case(&snapshot.author)
                && !relations.contains(&FocusRelation::Authored)
            {
                relations.push(FocusRelation::Authored);
            }
            let watched = WatchedPullRequest {
                baseline: true,
                state: snapshot.state,
                ci: if ci_failing {
                    CiResult::Failing
                } else {
                    snapshot.ci
                },
                terminal_reported: snapshot.state.is_terminal(),
                ci_failure_seen: ci_failing || snapshot.ci.is_failing(),
                title: snapshot.title.clone(),
                body: snapshot.body.clone(),
                author: snapshot.author.clone(),
                assignees: snapshot.assignees.clone(),
                url: snapshot.url.clone(),
                head_oid: snapshot.head_oid.clone(),
                created_at: snapshot.created_at,
                updated_at: snapshot.updated_at,
                merged_at: snapshot.merged_at,
                closed_at: snapshot.closed_at,
                seen_comments: snapshot
                    .comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect(),
                seen_reviews: snapshot
                    .reviews
                    .iter()
                    .map(|review| review.id.clone())
                    .collect(),
                seen_review_comments: snapshot
                    .review_comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect(),
                stack: snapshot.stack.clone(),
                relations,
            };
            account.pull_requests.insert(key.to_string(), watched);
        }
        self.save_state(&state);
    }

    pub fn seed_unknown_relations(&mut self) {
        let mut state = self.state();
        for account in state.accounts.values_mut() {
            for watched in account.pull_requests.values_mut() {
                watched.relations.clear();
            }
        }
        self.save_state(&state);
    }

    pub fn write_version_one_state(&self) {
        let now = self.clock.now().to_rfc3339();
        let text = format!(
            r#"{{
  "version": 1,
  "accounts": {{
    "personal": {{
      "login": "octo-personal",
      "last_check": "{now}",
      "pull_requests": {{
        "acme/api#42": {{
          "state": "open",
          "ci": "passing",
          "title": "Pull request acme/api#42",
          "author": "octo-personal",
          "seen_comments": [],
          "seen_reviews": [],
          "seen_review_comments": []
        }}
      }}
    }}
  }}
}}"#
        );
        self.write_raw_state(&text);
    }

    pub fn write_activity_state(&self, count: usize) {
        let mut state = WatcherState::new();
        state.version = 2;
        state.next_activity_id = count as u64 + 1;
        let now = self.clock.now();
        for index in 1..=count {
            state.push_activity(ActivityRecord {
                id: format!("a{index}"),
                observed_at: now,
                occurred_at: Some(now),
                pull_request: PullRequestKey::new("acme/api", 42),
                summary: format!("old event {index}"),
                display: format!("old event {index}"),
                body: None,
                read: true,
                stack: None,
                kind: Some(EventKind::Comment),
                author: Some("alice".to_string()),
            });
        }
        self.save_state(&state);
    }

    pub fn newest_activity_id(&self) -> Option<String> {
        let view = InboxService::new(self.config.clone(), self.store())
            .view()
            .ok()?;
        view.activity.first().map(|activity| activity.id.clone())
    }

    pub fn activity_is_read(&self, summary: &str) -> Option<bool> {
        let state = self.state();
        state
            .activity
            .iter()
            .find(|record| record.summary == summary)
            .map(|record| record.read)
    }

    pub fn tui_open(&mut self, keys: &[passuff::tui::keys::Key], observe: bool) {
        self.browser.clear();
        if observe {
            let config = match AppConfig::load(&self.paths.config) {
                Ok(config) => config,
                Err(_) => self.config.clone(),
            };
            let runtime = self.runtime();
            let outcome = runtime.run_cycle_for(&config, self.interactive, false, false);
            if let Some(report) = outcome.report {
                self.last_report = Some(report);
            }
            self.last_reconciliation = self.clock.now();
        }
        self.build_tui(keys);
    }

    pub fn build_tui(&mut self, keys: &[passuff::tui::keys::Key]) {
        self.reload_config();
        let store = self.store();
        let view = match InboxService::new(self.config.clone(), store.clone()).view() {
            Ok(view) => view,
            Err(error) => {
                self.tui_exit = 3;
                self.tui_output = format!("passuff-tui: state error: {error}\n");
                return;
            }
        };
        let prefs = UiPrefs::load(&self.paths.ui);
        let mut app = TuiApp::new(
            self.config.clone(),
            self.paths.config.clone(),
            self.paths.ui.clone(),
            store,
            view,
            prefs,
            self.browser.clone(),
        );
        for key in keys {
            app.handle_key(*key);
        }
        self.tui_closed = app.should_quit;
        self.tui_output = snapshot(&app);
        self.tui_exit = 0;
        self.last_exit = 0;
        self.tui_app = Some(app);
    }

    pub fn tui_press(&mut self, key: passuff::tui::keys::Key) {
        let Some(mut app) = self.tui_app.take() else {
            return;
        };
        app.handle_key(key);
        if app.should_quit {
            self.tui_closed = true;
        }
        self.tui_output = snapshot(&app);
        self.tui_exit = 0;
        self.tui_app = Some(app);
    }

    pub fn tui_refresh(&mut self) {
        let runtime = self.runtime();
        let config = self.config.clone();
        let outcome = runtime.run_cycle_for(&config, self.interactive, false, false);
        if let Some(report) = outcome.report {
            self.last_report = Some(report);
        }
        if let Some(mut app) = self.tui_app.take() {
            let _ = app.reload_view();
            self.tui_output = snapshot(&app);
            self.tui_app = Some(app);
        } else {
            self.build_tui(&[]);
        }
    }

    pub fn open_picker(&mut self) {
        self.reload_config();
        let store = self.store();
        let view = match InboxService::new(self.config.clone(), store.clone()).view() {
            Ok(view) => view,
            Err(error) => {
                self.tui_exit = 3;
                self.tui_output = format!("passuff-tui: state error: {error}\n");
                return;
            }
        };
        let prefs = UiPrefs::load(&self.paths.ui);
        let app = TuiApp::new(
            self.config.clone(),
            self.paths.config.clone(),
            self.paths.ui.clone(),
            store,
            view,
            prefs,
            self.browser.clone(),
        );
        self.picker_output = render::picker_text(&app);
        self.tui_output = self.picker_output.clone();
        self.tui_exit = 0;
        self.tui_app = Some(app);
    }

    pub fn tui_observe(&mut self) {
        let now = self.clock.now();
        let interval = self.config.reconciliation_interval_secs.max(1) as i64;
        let due = (now - self.last_reconciliation).num_seconds() >= interval;
        let runtime = self.runtime();
        let outcome = if due {
            self.last_reconciliation = now;
            runtime.run_cycle_for(&self.config.clone(), self.interactive, false, false)
        } else {
            runtime.run_probe_and_targeted(self.interactive)
        };
        if let Some(report) = outcome.report {
            self.last_report = Some(report);
        }
        if let Some(mut app) = self.tui_app.take() {
            let _ = app.reload_view();
            self.tui_output = snapshot(&app);
            self.tui_app = Some(app);
        } else {
            self.build_tui(&[]);
        }
    }

    pub fn run_tui_command(&mut self, snapshot: bool) {
        let cli = TuiCli {
            config: Some(self.paths.config.clone()),
            state: Some(self.paths.state.clone()),
            snapshot,
            observe: false,
            observe_for: None,
            scope_select: None,
            scope_deselect: None,
            scope_mode: None,
            focus_set: None,
            picker: false,
            keys: None,
            no_desktop: false,
            no_sound: false,
        };
        let options = TuiOptions::from_cli(&cli).expect("tui options");
        let outcome = passuff::tui::run_tui(options, self.ports());
        self.tui_exit = outcome.exit_code;
        self.last_exit = outcome.exit_code;
        self.tui_output = format!("{}{}", outcome.stdout, outcome.stderr);
    }

    pub fn selected_activity_summary(&self) -> Option<String> {
        let app = self.tui_app.as_ref()?;
        let activity = app.visible_activity();
        activity
            .get(app.activity_selection)
            .map(|activity| activity.summary.clone())
    }

    pub fn selected_pr_key(&self) -> Option<PullRequestKey> {
        let app = self.tui_app.as_ref()?;
        app.selected_pull_request_owned().map(|pr| pr.key)
    }

    pub fn set_allowance(&mut self, remaining: u32, reset_seconds: i64) {
        if let Ok(mut data) = self.data.lock() {
            data.allowance.remaining = remaining;
            data.allowance.reset_at = Some(self.clock.now() + Duration::seconds(reset_seconds));
        }
    }

    pub fn plan_next_delay(&mut self) {
        if let Ok(data) = self.data.lock() {
            self.planned_delay = Some(passuff_core::next_delay(&data.allowance, self.clock.now()));
        }
    }

    pub fn full_history_calls(&self) -> usize {
        self.data
            .lock()
            .map(|data| data.full_history_calls)
            .unwrap_or(0)
    }

    pub fn probe_calls(&self) -> usize {
        self.data.lock().map(|data| data.probe_calls).unwrap_or(0)
    }

    pub fn last_observed_keys(&self) -> Vec<PullRequestKey> {
        self.data
            .lock()
            .map(|data| data.last_observed_keys.clone())
            .unwrap_or_default()
    }

    pub fn ui_sound_enabled(&self) -> bool {
        std::fs::read_to_string(&self.paths.ui)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|value| value.get("sound").and_then(serde_json::Value::as_bool))
            .unwrap_or(true)
    }

    pub fn ui_desktop_enabled(&self) -> bool {
        std::fs::read_to_string(&self.paths.ui)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|value| value.get("desktop").and_then(serde_json::Value::as_bool))
            .unwrap_or(true)
    }

    pub fn set_focus(&mut self, relations: &[FocusRelation]) {
        ScopeEditor::set_focus(&self.paths.config, relations).expect("focus edit");
        self.reload_config();
    }

    pub fn set_scope(&mut self, mode: WatchMode, repositories: &[String]) {
        self.config.watch.mode = Some(mode);
        self.config.watch.repositories = repositories.to_vec();
        self.write_config();
    }

    pub fn select_repository(&mut self, repository: &str) -> Result<(), String> {
        ScopeEditor::select_repository(&self.paths.config, repository)
            .map_err(|error| error.to_string())?;
        self.reload_config();
        Ok(())
    }

    pub fn deselect_repository(&mut self, repository: &str) -> Result<(), String> {
        ScopeEditor::deselect_repository(&self.paths.config, repository)
            .map_err(|error| error.to_string())?;
        self.reload_config();
        Ok(())
    }

    pub fn set_scope_mode(&mut self, mode: WatchMode) -> Result<(), String> {
        ScopeEditor::set_mode(&self.paths.config, mode).map_err(|error| error.to_string())?;
        self.reload_config();
        Ok(())
    }

    pub fn last_ci(&self, key: &PullRequestKey) -> Option<CiResult> {
        self.snapshot_for(key).map(|snapshot| snapshot.ci)
    }

    pub fn notification_summaries(&self) -> Vec<String> {
        self.last_report
            .as_ref()
            .map(|report| {
                report
                    .notifications
                    .iter()
                    .flat_map(|notification| notification.events.iter())
                    .map(|event| event.summary.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn notification_count_for(&self, key: &PullRequestKey) -> usize {
        self.last_report
            .as_ref()
            .map(|report| {
                report
                    .notifications
                    .iter()
                    .filter(|notification| &notification.key == key)
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn baseline_recorded_for(&self, key: &PullRequestKey) -> bool {
        self.last_report
            .as_ref()
            .map(|report| report.baselines.contains(key))
            .unwrap_or(false)
    }

    pub fn add_watched_state_for(&mut self, key: &PullRequestKey) {
        self.seed_observed(std::slice::from_ref(key), false);
    }

    pub fn bundle_sound_path(&self) -> PathBuf {
        ensure_bundled_sound().unwrap_or_else(|_| PathBuf::from("notification.wav"))
    }

    pub fn add_world_readable_token_file(&mut self) -> PathBuf {
        let path = self.temp.path().join("world-readable.token");
        std::fs::write(&path, "token-TOKEN_WORK\n").expect("token file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");
        }
        self.token_file = Some(path.clone());
        path
    }

    pub fn add_private_token_file(&mut self, mode: u32, token: &str) -> PathBuf {
        let path = self.temp.path().join(format!("work-{mode}.token"));
        std::fs::write(&path, format!("{token}\n")).expect("token file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("chmod");
        }
        self.token_file = Some(path.clone());
        path
    }

    pub fn account_config(&self, label: &str) -> AccountConfig {
        self.config.accounts.get(label).cloned().unwrap_or_default()
    }

    pub fn all_pull_requests(&self) -> Vec<PullRequestSnapshot> {
        self.data
            .lock()
            .map(|data| data.pull_requests.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn token_values(&self) -> Vec<String> {
        self.data
            .lock()
            .map(|data| data.tokens.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn notifications(&self) -> Vec<Notification> {
        self.last_report
            .as_ref()
            .map(|report| report.notifications.clone())
            .unwrap_or_default()
    }
}

pub fn key(value: &str) -> PullRequestKey {
    value.parse().expect("pull request key")
}

pub fn review_state(value: &str) -> ReviewState {
    match value.to_ascii_lowercase().as_str() {
        "approved" => ReviewState::Approved,
        "changes requested" => ReviewState::ChangesRequested,
        _ => ReviewState::Commented,
    }
}

pub fn ci_result(value: &str) -> CiResult {
    match value.to_ascii_lowercase().as_str() {
        "failing" | "error" => CiResult::Failing,
        "successful" | "passing" => CiResult::Passing,
        "pending" => CiResult::Pending,
        _ => CiResult::NotStarted,
    }
}

pub fn pr_state(value: &str) -> PullRequestState {
    PullRequestState::parse(value)
}

pub fn parse_time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("rfc3339")
        .with_timezone(&Utc)
}

pub fn empty_state_error() -> StateError {
    StateError::Decode("unused".to_string())
}

pub fn _unused(_: BTreeMap<String, String>, _: BTreeSet<String>) {}
