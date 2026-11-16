use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use passuff_config::{expand_home, AppConfig, ConfigError};
use passuff_core::state::JsonStateStore;
use passuff_core::{
    Allowance, Clock, CoreError, CredentialSource, GatewayFactory, InboxService, PassuffCore,
    RefreshReport, SnapshotStore,
};

use crate::args::{Cli, Command};
use crate::delivery::DesktopMessage;
use crate::paths::{ensure_bundled_sound, PassuffPaths};
use crate::render::{render_inbox, render_refresh, render_status};
use crate::setup::run_setup;
use crate::sound::SoundPlayer;

pub trait BrowserOpener: Send + Sync {
    fn open(&self, url: &str) -> Result<(), String>;
}

pub struct RecordingBrowserOpener {
    urls: std::sync::Mutex<Vec<String>>,
    fail: std::sync::atomic::AtomicBool,
}

impl Default for RecordingBrowserOpener {
    fn default() -> Self {
        Self {
            urls: std::sync::Mutex::new(Vec::new()),
            fail: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl RecordingBrowserOpener {
    pub fn failing() -> Self {
        Self {
            urls: std::sync::Mutex::new(Vec::new()),
            fail: std::sync::atomic::AtomicBool::new(true),
        }
    }

    pub fn set_fail(&self, fail: bool) {
        self.fail.store(fail, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn clear(&self) {
        if let Ok(mut urls) = self.urls.lock() {
            urls.clear();
        }
    }

    pub fn urls(&self) -> Vec<String> {
        self.urls
            .lock()
            .map(|urls| urls.clone())
            .unwrap_or_default()
    }
}

impl BrowserOpener for RecordingBrowserOpener {
    fn open(&self, url: &str) -> Result<(), String> {
        if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("could not open browser".to_string());
        }
        if let Ok(mut urls) = self.urls.lock() {
            urls.push(url.to_string());
        }
        Ok(())
    }
}

pub struct SystemBrowserOpener;

impl BrowserOpener for SystemBrowserOpener {
    fn open(&self, url: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        let command = "open";
        #[cfg(not(target_os = "macos"))]
        let command = "xdg-open";
        let status = std::process::Command::new(command)
            .arg(url)
            .status()
            .map_err(|error| format!("could not open browser: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("could not open browser".to_string())
        }
    }
}

pub struct Ports {
    pub gateway_factory: Arc<dyn GatewayFactory>,
    pub credentials: Arc<dyn CredentialSource>,
    pub delivery: Arc<dyn crate::delivery::Delivery>,
    pub sound: Arc<dyn SoundPlayer>,
    pub clock: Arc<dyn Clock>,
    pub browser: Arc<dyn BrowserOpener>,
    pub interactive: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub report: Option<RefreshReport>,
    pub allowance: Option<Allowance>,
}

pub struct CliRuntime {
    pub paths: PassuffPaths,
    pub ports: Arc<Ports>,
}

impl CliRuntime {
    pub fn new(paths: PassuffPaths, ports: Ports) -> Self {
        Self {
            paths,
            ports: Arc::new(ports),
        }
    }

    pub fn store(&self) -> Arc<dyn SnapshotStore> {
        Arc::new(JsonStateStore::new(self.paths.state.clone()))
    }

    pub fn load_config(&self) -> Result<AppConfig, ConfigError> {
        AppConfig::load(&self.paths.config)
    }

    pub fn run(&self, cli: &Cli) -> CommandOutcome {
        match cli.command.clone().unwrap_or(Command::Check) {
            Command::Check => self.run_check_with_flags(cli.no_desktop, cli.no_sound),
            Command::Status => self.run_status(),
            Command::Inbox { all } => self.run_inbox(all),
            Command::Read { id, all } => self.run_read(id.as_deref(), all),
            Command::NotifyTest => self.run_notify_test(cli.no_desktop, cli.no_sound),
            Command::Setup => CommandOutcome {
                exit_code: 2,
                stderr: "passuff: setup requires an input stream\n".to_string(),
                ..CommandOutcome::default()
            },
        }
    }

    pub fn run_check(&self) -> CommandOutcome {
        self.run_check_with_flags(false, false)
    }

    pub fn run_check_with_flags(&self, no_desktop: bool, no_sound: bool) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let core = self.core(&config);
        match core.refresh_reconciliation(false) {
            Ok(report) => self.finish_refresh(&config, report, no_desktop, no_sound),
            Err(error) => core_error(error),
        }
    }

    /// Start an interactive or non-interactive watch: print the watching line
    /// and run the first reconciliation cycle.
    pub fn run_watch_start(&self) -> CommandOutcome {
        self.run_watch_start_with_flags(false, false)
    }

    pub fn run_watch_start_with_flags(&self, no_desktop: bool, no_sound: bool) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let mut outcome = self.run_cycle_for(&config, self.ports.interactive, no_desktop, no_sound);
        outcome.stdout = format!("passuff: watching\n{}", outcome.stdout);
        outcome
    }

    /// Reload the configuration and run another cycle for a running watcher.
    pub fn run_watch_cycle_reload(&self, interactive: bool) -> CommandOutcome {
        self.run_watch_cycle_reload_with_flags(interactive, false, false)
    }

    pub fn run_watch_cycle_reload_with_flags(
        &self,
        interactive: bool,
        no_desktop: bool,
        no_sound: bool,
    ) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        self.run_cycle_for(&config, interactive, no_desktop, no_sound)
    }

    pub fn run_cycle_for(
        &self,
        config: &AppConfig,
        interactive: bool,
        no_desktop: bool,
        no_sound: bool,
    ) -> CommandOutcome {
        let core = self.core(config);
        match core.refresh_reconciliation(interactive) {
            Ok(report) => self.finish_refresh(config, report, no_desktop, no_sound),
            Err(error) => core_error(error),
        }
    }

    pub fn run_probe_and_targeted(&self, interactive: bool) -> CommandOutcome {
        self.run_probe_and_targeted_with_flags(interactive, false, false)
    }

    pub fn run_probe_and_targeted_with_flags(
        &self,
        interactive: bool,
        no_desktop: bool,
        no_sound: bool,
    ) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let core = self.core(&config);
        let probe = match core.probe(interactive) {
            Ok(probe) => probe,
            Err(error) => return core_error(error),
        };
        let allowance = probe
            .accounts
            .iter()
            .map(|account| account.report.allowance.clone())
            .min_by_key(|allowance| allowance.remaining);
        if probe
            .accounts
            .iter()
            .all(|account| account.targets.is_empty())
        {
            let mut outcome = CommandOutcome {
                allowance,
                ..CommandOutcome::default()
            };
            for failure in &probe.failures {
                outcome
                    .stderr
                    .push_str(&format!("passuff: {}\n", failure.message));
            }
            if outcome.stderr.is_empty() {
                outcome.stdout.push_str("passuff: no notifications\n");
            } else {
                outcome.exit_code = 1;
            }
            return outcome;
        }
        let mut targets = std::collections::BTreeMap::new();
        for account in &probe.accounts {
            if !account.targets.is_empty() {
                targets.insert(account.label.clone(), account.targets.clone());
            }
        }
        match core.refresh_targeted(interactive, &targets, &probe.pending) {
            Ok(mut report) => {
                report.account_failures.extend(probe.failures.clone());
                let mut outcome = self.finish_refresh(&config, report, no_desktop, no_sound);
                outcome.allowance = allowance;
                outcome
            }
            Err(error) => core_error(error),
        }
    }

    /// Probe without running a targeted or reconciliation pass. Used by the
    /// live inbox to keep the view fresh while a full pass is deferred.
    pub fn run_probe_only(&self) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let core = self.core(&config);
        let probe = match core.probe(false) {
            Ok(probe) => probe,
            Err(error) => return core_error(error),
        };
        let allowance = probe
            .accounts
            .iter()
            .map(|account| account.report.allowance.clone())
            .min_by_key(|allowance| allowance.remaining);
        let mut outcome = CommandOutcome {
            allowance,
            ..CommandOutcome::default()
        };
        for failure in &probe.failures {
            outcome
                .stderr
                .push_str(&format!("passuff: {}\n", failure.message));
        }
        if !probe.failures.is_empty() {
            outcome.exit_code = 1;
        }
        outcome
    }

    pub fn run_status(&self) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let service = InboxService::new(config, self.store());
        match service.view() {
            Ok(view) => CommandOutcome {
                stdout: render_status(&view),
                ..CommandOutcome::default()
            },
            Err(error) => CommandOutcome {
                exit_code: 3,
                stderr: format!("passuff: state error: {error}\n"),
                ..CommandOutcome::default()
            },
        }
    }

    pub fn run_inbox(&self, all: bool) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let service = InboxService::new(config, self.store());
        match service.view() {
            Ok(view) => CommandOutcome {
                stdout: render_inbox(&view, all),
                ..CommandOutcome::default()
            },
            Err(error) => CommandOutcome {
                exit_code: 3,
                stderr: format!("passuff: state error: {error}\n"),
                ..CommandOutcome::default()
            },
        }
    }

    pub fn run_read(&self, id: Option<&str>, all: bool) -> CommandOutcome {
        if all && id.is_some() || !all && id.is_none() {
            return CommandOutcome {
                exit_code: 2,
                stderr: "passuff: read requires exactly one of <id> or --all\n".to_string(),
                ..CommandOutcome::default()
            };
        }
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let service = InboxService::new(config, self.store());
        let result = if all {
            service.mark_all_read()
        } else {
            service.mark_read(id.unwrap_or_default())
        };
        match result {
            Ok(result) => {
                if !all && !result.found {
                    return CommandOutcome {
                        exit_code: 2,
                        stderr: format!(
                            "passuff: no activity with id {}\n",
                            id.unwrap_or_default()
                        ),
                        ..CommandOutcome::default()
                    };
                }
                CommandOutcome {
                    stdout: format!(
                        "passuff: {} activity {} read\n",
                        result.changed,
                        if result.changed == 1 {
                            "record"
                        } else {
                            "records"
                        }
                    ),
                    ..CommandOutcome::default()
                }
            }
            Err(error) => CommandOutcome {
                exit_code: 3,
                stderr: format!("passuff: state error: {error}\n"),
                ..CommandOutcome::default()
            },
        }
    }

    pub fn run_notify_test(&self, no_desktop: bool, no_sound: bool) -> CommandOutcome {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(error) => return configuration_error(error),
        };
        let desktop_enabled = config.notifications.desktop && !no_desktop;
        let sound_enabled = config.notifications.sound && !no_sound;
        let mut outcome = CommandOutcome::default();
        if !desktop_enabled && !sound_enabled {
            outcome
                .stdout
                .push_str("passuff: desktop notifications are disabled\n");
            outcome.stdout.push_str("passuff: nothing to test\n");
            outcome.exit_code = 2;
            return outcome;
        }
        let mut failed = false;
        if desktop_enabled {
            let message = DesktopMessage {
                title: "passuff".to_string(),
                body: "If you can read this, passuff notifications are working.".to_string(),
                urgency: crate::delivery::urgency_label(config.notifications.urgency).to_string(),
            };
            match self.ports.delivery.deliver(&message) {
                Ok(()) => outcome
                    .stdout
                    .push_str("passuff: desktop notification sent\n"),
                Err(error) => {
                    failed = true;
                    outcome
                        .stdout
                        .push_str("passuff: desktop notification failed\n");
                    outcome.stderr.push_str(&format!("passuff: {error}\n"));
                }
            }
        } else {
            outcome
                .stdout
                .push_str("passuff: desktop notifications are disabled\n");
        }
        if sound_enabled {
            match self.play_sound(&config) {
                Ok(()) => outcome.stdout.push_str("passuff: sound played\n"),
                Err(error) => {
                    failed = true;
                    outcome.stdout.push_str("passuff: sound unavailable\n");
                    outcome.stderr.push_str(&format!("passuff: {error}\n"));
                }
            }
        } else {
            outcome.stdout.push_str("passuff: sound is disabled\n");
        }
        outcome.exit_code = if failed { 1 } else { 0 };
        outcome
    }

    pub fn run_setup_command(&self, input: &mut dyn BufRead) -> CommandOutcome {
        let mut stdout = std::io::stdout();
        let result = run_setup(&self.paths.config, input, &mut stdout);
        CommandOutcome {
            exit_code: result.exit_code,
            ..CommandOutcome::default()
        }
    }

    pub fn run_setup_capture(
        &self,
        input: &mut dyn BufRead,
        output: &mut dyn std::io::Write,
    ) -> CommandOutcome {
        let result = run_setup(&self.paths.config, input, output);
        CommandOutcome {
            exit_code: result.exit_code,
            stdout: result.stdout,
            ..CommandOutcome::default()
        }
    }

    fn finish_refresh(
        &self,
        config: &AppConfig,
        report: RefreshReport,
        no_desktop: bool,
        no_sound: bool,
    ) -> CommandOutcome {
        let rendered = render_refresh(&report, &config.notifications);
        let mut outcome = CommandOutcome {
            exit_code: if report.failed() { 1 } else { 0 },
            stdout: rendered.stdout,
            stderr: rendered.stderr,
            report: Some(report),
            allowance: None,
        };
        if let Some(digest) = rendered.digest {
            let desktop_enabled = config.notifications.desktop && !no_desktop;
            let sound_enabled = config.notifications.sound && !no_sound;
            if desktop_enabled {
                if let Err(error) = self.ports.delivery.deliver(&digest) {
                    outcome
                        .stderr
                        .push_str(&format!("passuff: desktop notification failed: {error}\n"));
                }
            }
            if sound_enabled {
                if let Err(error) = self.play_sound(config) {
                    outcome
                        .stderr
                        .push_str(&format!("passuff: sound unavailable: {error}\n"));
                }
            }
        }
        outcome
    }

    fn play_sound(&self, config: &AppConfig) -> Result<(), String> {
        let path = if config.notifications.sound_file == passuff_config::DEFAULT_SOUND_FILE {
            ensure_bundled_sound()?
        } else {
            expand_home(&config.notifications.sound_file)
        };
        if !path.is_file() {
            return Err(format!("sound file is missing: {}", path.display()));
        }
        self.ports.sound.play(&path)
    }

    pub fn core(&self, config: &AppConfig) -> PassuffCore {
        PassuffCore::new(
            config.clone(),
            self.ports.gateway_factory.clone(),
            self.ports.credentials.clone(),
            self.store(),
            self.ports.clock.clone(),
        )
    }
}

fn configuration_error(error: ConfigError) -> CommandOutcome {
    CommandOutcome {
        exit_code: 2,
        stderr: format!("passuff: configuration error: {error}\n"),
        ..CommandOutcome::default()
    }
}

fn core_error(error: CoreError) -> CommandOutcome {
    CommandOutcome {
        exit_code: 3,
        stderr: format!("passuff: {error}\n"),
        ..CommandOutcome::default()
    }
}

#[allow(dead_code)]
pub fn default_paths(config: Option<PathBuf>, state: Option<PathBuf>) -> PassuffPaths {
    PassuffPaths::resolve(config, state)
}

#[allow(dead_code)]
fn _path(_: &Path) {}
