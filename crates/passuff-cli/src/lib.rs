pub mod delivery;
pub mod sound;
pub mod transcript;

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use passuff_config::{AppConfig, ConfigError};
use passuff_core::{
    CoreError, CredentialSource, GitHubGateway, PassuffCore, RefreshReport, SnapshotStore,
};

use crate::delivery::{DeliveryReport, NotificationDelivery};
use crate::sound::SoundOutcome;
use crate::transcript::TranscriptRenderer;

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
}

pub struct Interrupt {
    cancelled: Arc<AtomicBool>,
}

impl Interrupt {
    pub fn install() -> Self {
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            flag.store(true, Ordering::SeqCst);
        });
        Self { cancelled }
    }
}

#[async_trait::async_trait]
impl WatchControl for Interrupt {
    fn should_continue(&self) -> bool {
        !self.cancelled.load(Ordering::SeqCst)
    }

    async fn wait(&self, interval: Duration) {
        let deadline = tokio::time::Instant::now() + interval;
        while tokio::time::Instant::now() < deadline {
            if !self.should_continue() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
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
}

pub struct CliRuntime {
    pub ports: CliPorts,
    renderer: TranscriptRenderer,
}

impl CliRuntime {
    pub fn new(ports: CliPorts) -> Self {
        Self {
            ports,
            renderer: TranscriptRenderer,
        }
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

        let mut output: Vec<String> = Vec::new();
        let mut last_report = None;
        let mut first_success = false;
        while control.should_continue() {
            match core.refresh().await {
                Ok(report) => {
                    let (transcript, _status) = self.process_report(&config, &report).await;
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
            control
                .wait(Duration::from_secs(config.poll_interval_secs.max(1)))
                .await;
        }
        RunOutcome {
            exit: ExitCode::SUCCESS,
            status: 0,
            report: last_report,
            transcript: output.join("\n"),
            emitted: true,
        }
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
