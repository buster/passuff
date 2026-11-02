pub mod delivery;
pub mod sound;
pub mod transcript;

use std::process::ExitCode;
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
        }
    }

    pub async fn run_watch(&self, config: Result<AppConfig, ConfigError>) -> RunOutcome {
        let config = match config {
            Ok(config) => config,
            Err(error) => {
                return RunOutcome {
                    exit: ExitCode::from(2),
                    status: 2,
                    report: None,
                    transcript: self.renderer.render_config_error(&error.to_string()),
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

        let mut last_report = None;
        let mut last_transcript = String::new();
        let mut first_success = false;
        let mut ctrl_c = Box::pin(tokio::signal::ctrl_c());
        loop {
            match core.refresh().await {
                Ok(report) => {
                    let (transcript, _status) = self.process_report(&config, &report).await;
                    if !transcript.is_empty() {
                        println!("{transcript}");
                    }
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    if !first_success && report.account_failures.is_empty() {
                        first_success = true;
                        println!("{}", self.renderer.render_watching());
                        let _ = std::io::stdout().flush();
                    }
                    last_transcript = transcript;
                    last_report = Some(report);
                }
                Err(error) => {
                    println!("{}", self.renderer.render_state_error(&error.to_string()));
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(config.poll_interval_secs.max(1))) => {}
                _ = &mut ctrl_c => break,
            }
        }
        RunOutcome {
            exit: ExitCode::SUCCESS,
            status: 0,
            report: last_report,
            transcript: last_transcript,
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
