pub mod browser;
pub mod delivery;
pub mod gh_login;
pub mod inbox_cli;
pub mod live;
pub mod scope;
pub mod setup;
pub mod sound;
pub mod transcript;
pub mod tui;

use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

use passuff_config::{AppConfig, ConfigError};
use passuff_core::{
    CoreError, CredentialSource, GitHubGateway, PassuffCore, RefreshReport, SnapshotStore,
    SystemCredentialSource,
};

use crate::delivery::{DeliveryReport, NotificationDelivery};
use crate::sound::SoundOutcome;
use crate::transcript::TranscriptRenderer;

pub fn init_logging(verbose: bool) {
    if !verbose {
        return;
    }
    let filter = match std::env::var("RUST_LOG") {
        Ok(_) => tracing_subscriber::EnvFilter::from_default_env(),
        Err(_) => tracing_subscriber::EnvFilter::new(
            "passuff=debug,passuff_cli=debug,passuff_core=debug,passuff_config=debug",
        ),
    };
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .with_env_filter(filter)
        .try_init();
}

pub fn system_credentials(interactive: bool) -> Arc<dyn CredentialSource> {
    if interactive {
        Arc::new(SystemCredentialSource::interactive(Arc::new(
            gh_login::SystemGhLogin::new(),
        )))
    } else {
        Arc::new(SystemCredentialSource::headless())
    }
}

#[derive(Clone)]
pub struct CliPorts {
    pub gateway: Arc<dyn GitHubGateway>,
    pub store: Arc<dyn SnapshotStore>,
    pub credentials: Arc<dyn CredentialSource>,
    pub delivery: Arc<dyn NotificationDelivery>,
}

#[derive(Debug)]
pub struct RunOutcome {
    pub exit: ExitCode,
    pub status: i32,
    pub report: Option<RefreshReport>,
    pub transcript: String,
    pub emitted: bool,
}

#[async_trait::async_trait]
pub trait WatchControl: Send + Sync {
    fn should_continue(&self) -> bool;
    async fn wait(&self, interval: Duration);
    async fn interrupted(&self);
}

pub struct Interrupt {
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl Interrupt {
    pub fn install() -> Self {
        let cancelled = Arc::new(AtomicBool::new(false));
        let notify = Arc::new(Notify::new());
        let flag = cancelled.clone();
        let signal = notify.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            flag.store(true, Ordering::SeqCst);
            signal.notify_one();
        });
        Self { cancelled, notify }
    }
}

#[async_trait::async_trait]
impl WatchControl for Interrupt {
    fn should_continue(&self) -> bool {
        !self.cancelled.load(Ordering::SeqCst)
    }

    async fn wait(&self, interval: Duration) {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = self.interrupted() => {}
        }
    }

    async fn interrupted(&self) {
        if !self.should_continue() {
            return;
        }
        self.notify.notified().await;
    }
}

pub struct StopAfter {
    remaining: AtomicUsize,
}

impl StopAfter {
    pub fn new(cycles: usize) -> Self {
        Self {
            remaining: AtomicUsize::new(cycles),
        }
    }
}

#[async_trait::async_trait]
impl WatchControl for StopAfter {
    fn should_continue(&self) -> bool {
        self.remaining
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
    }

    async fn wait(&self, _interval: Duration) {}

    async fn interrupted(&self) {
        std::future::pending::<()>().await;
    }
}

pub struct CliRuntime {
    pub ports: CliPorts,
    renderer: TranscriptRenderer,
    config_path: Option<std::path::PathBuf>,
}

impl CliRuntime {
    pub fn new(ports: CliPorts) -> Self {
        Self {
            ports,
            renderer: TranscriptRenderer,
            config_path: None,
        }
    }

    pub fn with_config_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    pub async fn run_watch_cycle(&self, path: &std::path::Path) -> RunOutcome {
        let config = match passuff_config::ConfigLoader::load(path) {
            Ok(config) => config,
            Err(error) => {
                return RunOutcome {
                    exit: ExitCode::from(2),
                    status: 2,
                    report: None,
                    transcript: self.renderer.render_config_error(&error.to_string()),
                    emitted: false,
                };
            }
        };
        self.run_check(Ok(config)).await
    }

    pub async fn run_check(&self, config: Result<AppConfig, ConfigError>) -> RunOutcome {
        let config = match config {
            Ok(config) => config,
            Err(error) => {
                return RunOutcome {
                    exit: ExitCode::from(2),
                    status: 2,
                    report: None,
                    transcript: self.renderer.render_config_error(&error.to_string()),
                    emitted: false,
                };
            }
        };

        let mut core = match PassuffCore::with_ports(
            config.core.clone(),
            self.ports.gateway.clone(),
            self.ports.store.clone(),
            self.ports.credentials.clone(),
        )
        .await
        {
            Ok(core) => core,
            Err(error) => return self.state_error(error),
        };

        let report = match core.refresh().await {
            Ok(report) => report,
            Err(error) => return self.state_error(error),
        };
        let (transcript, status) = self.process_report(&config, &report).await;
        RunOutcome {
            exit: ExitCode::from(status as u8),
            status,
            report: Some(report),
            transcript,
            emitted: false,
        }
    }

    pub async fn run_watch(&self, config: Result<AppConfig, ConfigError>) -> RunOutcome {
        let control = Interrupt::install();
        self.run_watch_with(config, &control).await
    }

    pub async fn run_watch_with(
        &self,
        config: Result<AppConfig, ConfigError>,
        control: &dyn WatchControl,
    ) -> RunOutcome {
        let config = match config {
            Ok(config) => config,
            Err(error) => {
                return RunOutcome {
                    exit: ExitCode::from(2),
                    status: 2,
                    report: None,
                    transcript: self.renderer.render_config_error(&error.to_string()),
                    emitted: false,
                };
            }
        };
        let mut config = config;
        let mut core = match self.build_core(&config).await {
            Ok(core) => core,
            Err(outcome) => return outcome,
        };

        let mut output: Vec<String> = Vec::new();
        let mut last_report = None;
        let mut first_success = false;
        let mut first_cycle = true;
        let mut last_reload_error: Option<String> = None;
        while control.should_continue() {
            if !first_cycle {
                if let Some(path) = self.config_path.as_ref() {
                    match passuff_config::ConfigLoader::load(path) {
                        Ok(reloaded) => {
                            config = reloaded;
                            if let Ok(rebuilt) = self.build_core(&config).await {
                                core = rebuilt;
                                last_reload_error = None;
                            }
                        }
                        Err(error) => {
                            let message = error.to_string();
                            if last_reload_error.as_deref() != Some(message.as_str()) {
                                let line = self.renderer.render_config_error(&message);
                                output.push(line.clone());
                                println!("{line}");
                                last_reload_error = Some(message);
                            }
                        }
                    }
                }
            }
            first_cycle = false;
            let refresh = tokio::select! {
                _ = control.interrupted() => break,
                refresh = core.refresh() => refresh,
            };
            match refresh {
                Ok(report) => {
                    tracing::debug!("watch cycle started");
                    let (transcript, _status) = self.process_report(&config, &report).await;
                    tracing::debug!(
                        interval_secs = config.poll_interval_secs,
                        "watch cycle complete"
                    );
                    if !transcript.is_empty() {
                        output.push(transcript.clone());
                        println!("{transcript}");
                    }
                    if !first_success && report.account_failures.is_empty() {
                        first_success = true;
                        let watching = self.renderer.render_watching();
                        output.push(watching.clone());
                        println!("{watching}");
                    }
                    last_report = Some(report);
                }
                Err(error) => {
                    let line = self.renderer.render_state_error(&error.to_string());
                    output.push(line.clone());
                    println!("{line}");
                }
            }
            use std::io::Write;
            let _ = std::io::stdout().flush();
            tokio::select! {
                _ = control.interrupted() => break,
                _ = control.wait(Duration::from_secs(config.poll_interval_secs.max(1))) => {}
            }
        }
        RunOutcome {
            exit: ExitCode::SUCCESS,
            status: 0,
            report: last_report,
            transcript: output.join("\n"),
            emitted: true,
        }
    }

    pub(crate) async fn build_core(&self, config: &AppConfig) -> Result<PassuffCore, RunOutcome> {
        PassuffCore::with_ports(
            config.core.clone(),
            self.ports.gateway.clone(),
            self.ports.store.clone(),
            self.ports.credentials.clone(),
        )
        .await
        .map_err(|error| self.state_error(error))
    }

    /// Deliver a refresh report's notifications and return its transcript.
    pub async fn deliver_report(&self, config: &AppConfig, report: &RefreshReport) -> String {
        self.process_report(config, report).await.0
    }

    async fn process_report(&self, config: &AppConfig, report: &RefreshReport) -> (String, i32) {
        let mut lines = Vec::new();
        for key in &report.baselines_recorded {
            lines.push(self.renderer.render_baseline(key));
        }
        for notification in &report.notifications {
            let rendered = self
                .renderer
                .render_notification(notification, config.notifications.sound);
            let delivery = self
                .ports
                .delivery
                .deliver(&rendered, &config.notifications)
                .await;
            tracing::debug!(
                desktop = delivery.desktop_delivered,
                sound = ?delivery.sound,
                errors = delivery.errors.len(),
                "delivery complete"
            );
            lines.push(rendered.transcript);
            lines.extend(render_delivery_errors(&self.renderer, &delivery));
        }
        for failure in &report.account_failures {
            lines.push(self.renderer.render_account_failure(failure));
        }
        if report.notifications.is_empty()
            && report.baselines_recorded.is_empty()
            && report.account_failures.is_empty()
        {
            lines.push(self.renderer.render_no_changes());
        }
        let status = if report.account_failures.is_empty() {
            0
        } else {
            1
        };
        (lines.join("\n"), status)
    }

    fn state_error(&self, error: CoreError) -> RunOutcome {
        RunOutcome {
            exit: ExitCode::from(3),
            status: 3,
            report: None,
            transcript: self.renderer.render_state_error(&error.to_string()),
            emitted: false,
        }
    }
}

fn render_delivery_errors(renderer: &TranscriptRenderer, delivery: &DeliveryReport) -> Vec<String> {
    let mut lines = Vec::new();
    for error in &delivery.errors {
        if let Some(detail) = error.strip_prefix("desktop notification failed: ") {
            lines.push(renderer.render_delivery_error(detail));
        } else {
            lines.push(renderer.render_delivery_error(error));
        }
    }
    match delivery.sound.as_ref() {
        Some(SoundOutcome::Unavailable) => {
            lines.push(renderer.render_sound_unavailable("no sound player found"));
        }
        Some(SoundOutcome::Failed(detail)) => {
            lines.push(renderer.render_sound_unavailable(detail));
        }
        _ => {}
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use passuff_config::CredentialSpec;

    #[tokio::test]
    async fn system_credentials_resolves_in_both_modes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("work.token");
        std::fs::write(&path, "token-value").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o600);
            std::fs::set_permissions(&path, permissions).unwrap();
        }
        let spec = CredentialSpec::File { path };
        assert_eq!(
            system_credentials(false).resolve(&spec).await.unwrap(),
            "token-value"
        );
        assert_eq!(
            system_credentials(true).resolve(&spec).await.unwrap(),
            "token-value"
        );
    }

    use crate::delivery::{DeliveryReport, NotificationDelivery};
    use crate::transcript::RenderedNotification;
    use passuff_config::{AccountConfig, CoreConfig, EventToggles, NotificationsConfig};
    use passuff_core::{
        AccountObservation, CredentialError, GitHubError, StateError, WatcherState,
    };
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct SlowGateway;

    #[async_trait::async_trait]
    impl GitHubGateway for SlowGateway {
        async fn observe(
            &self,
            _token: &str,
            _graphql_url: &url::Url,
            _known_node_ids: &[String],
        ) -> Result<AccountObservation, GitHubError> {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Err(GitHubError::Timeout)
        }

        async fn probe(
            &self,
            _token: &str,
            _graphql_url: &url::Url,
        ) -> Result<passuff_core::AccountProbe, GitHubError> {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Err(GitHubError::Timeout)
        }
    }

    struct FixedCredentials;

    #[async_trait::async_trait]
    impl CredentialSource for FixedCredentials {
        async fn resolve(&self, _spec: &CredentialSpec) -> Result<String, CredentialError> {
            Ok("token".to_string())
        }
    }

    struct EmptyStore;

    #[async_trait::async_trait]
    impl SnapshotStore for EmptyStore {
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
            Ok(())
        }
    }

    struct NoopDelivery;

    #[async_trait::async_trait]
    impl NotificationDelivery for NoopDelivery {
        async fn deliver(
            &self,
            _rendered: &RenderedNotification,
            _config: &NotificationsConfig,
        ) -> DeliveryReport {
            DeliveryReport {
                desktop_delivered: false,
                sound: None,
                errors: Vec::new(),
            }
        }
    }

    struct TestInterrupt {
        cancelled: AtomicBool,
        notify: Notify,
    }

    #[async_trait::async_trait]
    impl WatchControl for TestInterrupt {
        fn should_continue(&self) -> bool {
            !self.cancelled.load(Ordering::SeqCst)
        }

        async fn wait(&self, _interval: Duration) {
            self.interrupted().await;
        }

        async fn interrupted(&self) {
            if !self.should_continue() {
                return;
            }
            self.notify.notified().await;
        }
    }

    #[tokio::test]
    async fn watch_stops_while_a_cycle_is_in_flight() {
        let config = AppConfig {
            poll_interval_secs: 1,
            core: CoreConfig {
                watch: passuff_config::WatchConfig::default(),
                accounts: BTreeMap::from([(
                    "work".to_string(),
                    AccountConfig {
                        credential: CredentialSpec::Env {
                            variable: "TOKEN".to_string(),
                        },
                        graphql_url: url::Url::parse("http://127.0.0.1/graphql").unwrap(),
                        expected_login: None,
                    },
                )]),
                events: EventToggles::default(),
            },
            notifications: NotificationsConfig::default(),
        };
        let runtime = CliRuntime::new(CliPorts {
            gateway: Arc::new(SlowGateway),
            store: Arc::new(EmptyStore),
            credentials: Arc::new(FixedCredentials),
            delivery: Arc::new(NoopDelivery),
        });
        let control = Arc::new(TestInterrupt {
            cancelled: AtomicBool::new(false),
            notify: Notify::new(),
        });
        let trigger = control.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            trigger.cancelled.store(true, Ordering::SeqCst);
            trigger.notify.notify_one();
        });
        let outcome = tokio::time::timeout(
            Duration::from_secs(2),
            runtime.run_watch_with(Ok(config), control.as_ref()),
        )
        .await
        .expect("watch stopped while a cycle was in flight");
        assert_eq!(outcome.status, 0);
    }
    struct OnePrGateway;

    #[async_trait::async_trait]
    impl GitHubGateway for OnePrGateway {
        async fn observe(
            &self,
            _token: &str,
            _graphql_url: &url::Url,
            _known_node_ids: &[String],
        ) -> Result<AccountObservation, GitHubError> {
            Ok(AccountObservation {
                login: "octo".to_string(),
                involved_node_ids: Vec::new(),
                pull_requests: vec![passuff_core::AuthoredPullRequest {
                    key: passuff_core::PullRequestKey::new("acme/web".to_string(), 1),
                    node_id: "node-1".to_string(),
                    author: "octo".to_string(),
                    title: "One".to_string(),
                    body: String::new(),
                    url: "https://github.com/acme/web/pull/1".to_string(),
                    created_at: "2026-11-03T09:00:00Z".to_string(),
                    updated_at: "2026-11-03T09:00:00Z".to_string(),
                    state: passuff_core::PrState::Open,
                    merged: false,
                    is_draft: false,
                    assignees: Vec::new(),
                    labels: Vec::new(),
                    milestone: None,
                    base_ref: "main".to_string(),
                    merged_by: None,
                    stack: None,
                    head_oid: "abc".to_string(),
                    ci: passuff_core::CiResult::default(),
                    comments: Vec::new(),
                    reviews: Vec::new(),
                    review_comments: Vec::new(),
                }],
                checked_at: "2026-11-03T09:00:00Z".to_string(),
            })
        }

        async fn probe(
            &self,
            _token: &str,
            _graphql_url: &url::Url,
        ) -> Result<passuff_core::AccountProbe, GitHubError> {
            Ok(passuff_core::AccountProbe {
                login: "octo".to_string(),
                pull_requests: Vec::new(),
                allowance: None,
            })
        }
    }

    #[tokio::test]
    async fn runtime_reloads_scope_between_cycles() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let state_path = dir.path().join("state.json");
        let config = |repositories: &str| {
            format!(
                "[accounts.personal]\ntoken_env = \"TOKEN\"\n[watch]\nmode = \"selected\"\nrepositories = [{repositories}]\n"
            )
        };
        std::fs::write(&config_path, config("\"acme/api\"")).unwrap();
        let runtime = CliRuntime::new(CliPorts {
            gateway: Arc::new(OnePrGateway),
            store: Arc::new(passuff_core::JsonStateStore::new(state_path.clone())),
            credentials: Arc::new(FixedCredentials),
            delivery: Arc::new(NoopDelivery),
        });
        let first = runtime.run_watch_cycle(&config_path).await;
        assert!(!first
            .report
            .as_ref()
            .unwrap()
            .baselines_recorded
            .iter()
            .any(|key| key.to_string() == "acme/web#1"));
        std::fs::write(&config_path, config("\"acme/web\"")).unwrap();
        let second = runtime.run_watch_cycle(&config_path).await;
        assert!(second
            .report
            .as_ref()
            .unwrap()
            .baselines_recorded
            .iter()
            .any(|key| key.to_string() == "acme/web#1"));
    }

}

