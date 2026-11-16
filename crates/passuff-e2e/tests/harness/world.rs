//! Per-scenario state: a temporary home and config, the GraphQL mock, the CLI
//! process outcome, and an optional pty session.

use std::path::PathBuf;
use std::time::Duration;

use crate::harness::bin::{self, ChildEnv};
use crate::harness::github::GitHubMock;
use crate::harness::pty::PtySession;

const SCREEN_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Default)]
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(cucumber::World)]
#[world(init = Self::new)]
pub struct World {
    pub dir: tempfile::TempDir,
    pub home: PathBuf,
    pub config: PathBuf,
    pub state: PathBuf,
    pub github: Option<GitHubMock>,
    pub graphql_url: String,
    pub desktop: bool,
    pub notification_log: Option<PathBuf>,
    pub last: Outcome,
    pub pty: Option<PtySession>,
    pub pty_exit: Option<u32>,
}

impl std::fmt::Debug for World {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("World")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("graphql_url", &self.graphql_url)
            .field("last", &self.last)
            .field("pty_exit", &self.pty_exit)
            .finish()
    }
}

impl World {
    async fn new() -> Result<Self, std::io::Error> {
        let dir = tempfile::tempdir()
            .map_err(|error| std::io::Error::other(format!("cannot create temp dir: {error}")))?;
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home)
            .map_err(|error| std::io::Error::other(format!("cannot create HOME: {error}")))?;
        let github = GitHubMock::start().await.map_err(std::io::Error::other)?;
        let world = Self {
            config: dir.path().join("config.toml"),
            state: dir.path().join("state.json"),
            graphql_url: github.graphql_url(),
            github: Some(github),
            home,
            dir,
            desktop: false,
            notification_log: None,
            last: Outcome::default(),
            pty: None,
            pty_exit: None,
        };
        world.write_config().map_err(std::io::Error::other)?;
        Ok(world)
    }
}

impl World {
    pub fn assert_ready(&self) {
        assert!(
            self.config.is_file(),
            "the scenario configuration was not written: {}",
            self.config.display()
        );
        assert!(
            self.github.is_some(),
            "the mock GitHub account was not started"
        );
    }

    pub fn add_comment(&self) {
        self.github
            .as_ref()
            .expect("mock GitHub account")
            .add_comment();
    }

    pub fn observe_only(&self) {
        self.github
            .as_ref()
            .expect("mock GitHub account")
            .observe_only();
    }

    pub fn fail_everything(&self) {
        self.github
            .as_ref()
            .expect("mock GitHub account")
            .fail_everything();
    }

    /// Point the configuration at a port with no listener. The listener is
    /// bound and dropped so the port is reserved and then closed.
    pub fn use_closed_port(&mut self) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind a closed port");
        let port = listener.local_addr().expect("local addr").port();
        drop(listener);
        self.graphql_url = format!("http://127.0.0.1:{port}/graphql");
        self.write_config().expect("rewrite configuration");
    }

    pub fn enable_desktop(&mut self) {
        self.desktop = true;
        self.notification_log = Some(self.dir.path().join("notifications.jsonl"));
        self.write_config().expect("rewrite configuration");
    }

    pub fn write_config(&self) -> Result<(), String> {
        let text = format!(
            "poll_interval_secs = 1\n\
             reconciliation_interval_secs = 600\n\
             \n\
             [notifications]\n\
             desktop = {desktop}\n\
             sound = false\n\
             \n\
             [accounts.work]\n\
             github_user = \"tester\"\n\
             token_env = \"PASSUFF_E2E_TOKEN\"\n\
             graphql_url = \"{url}\"\n",
            desktop = self.desktop,
            url = self.graphql_url,
        );
        std::fs::write(&self.config, text)
            .map_err(|error| format!("cannot write {}: {error}", self.config.display()))
    }

    fn child_env(&self) -> ChildEnv {
        let mut env = ChildEnv::new(self.home.clone());
        if let Some(path) = &self.notification_log {
            env = env.with_hook("PASSUFF_NOTIFICATION_LOG", path.display().to_string());
        }
        env
    }

    pub fn run(&mut self, args: &[&str]) -> Result<(), String> {
        let mut command = bin::product_command("passuff", &self.child_env())?;
        command
            .arg("--config")
            .arg(&self.config)
            .arg("--state")
            .arg(&self.state)
            .args(args);
        let output = command
            .output()
            .map_err(|error| format!("cannot run passuff {args:?}: {error}"))?;
        self.last = Outcome {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        };
        Ok(())
    }

    pub fn open_inbox(&mut self, cols: u16, rows: u16) -> Result<(), String> {
        let mut command = bin::pty_command("passuff-tui", &self.child_env())?;
        command.arg("--config");
        command.arg(&self.config);
        command.arg("--state");
        command.arg(&self.state);
        self.pty = Some(PtySession::spawn(command, cols, rows)?);
        self.pty_exit = None;
        Ok(())
    }

    pub fn press_enter(&mut self) -> Result<(), String> {
        self.pty
            .as_mut()
            .ok_or_else(|| "the inbox is not open".to_string())?
            .send(b"\r")
    }

    pub fn screen(&self) -> String {
        self.pty
            .as_ref()
            .map(PtySession::screen)
            .unwrap_or_default()
    }

    pub fn screen_shows(&self, needle: &str) {
        let session = self.pty.as_ref().expect("the inbox is not open");
        let screen = session.wait_for(needle, SCREEN_TIMEOUT);
        assert!(
            screen.contains(needle),
            "the screen never showed {needle:?}\n--- screen at {needle:?} deadline ---\n{screen}"
        );
    }

    /// Close the inbox. A first `q` may close an open detail overlay or open
    /// the save prompt; the step keeps pressing until the process exits.
    pub fn quit_inbox(&mut self) -> Result<(), String> {
        let session = self
            .pty
            .as_mut()
            .ok_or_else(|| "the inbox is not open".to_string())?;
        for _ in 0..3 {
            session.send(b"q")?;
            if let Some(code) = session.wait_exit(Duration::from_secs(3)) {
                self.pty_exit = Some(code);
                return Ok(());
            }
            if session
                .screen()
                .contains("Save view and notification settings?")
            {
                session.send(b"y")?;
                if let Some(code) = session.wait_exit(EXIT_TIMEOUT) {
                    self.pty_exit = Some(code);
                    return Ok(());
                }
            }
        }
        self.pty_exit = session.wait_exit(EXIT_TIMEOUT);
        Ok(())
    }

    pub fn assert_exit(&self, expected: i32) {
        assert_eq!(
            self.last.code, expected,
            "exit code was {}, expected {expected}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.last.code, self.last.stdout, self.last.stderr
        );
    }

    pub fn assert_tui_exit(&self, expected: i32) {
        let code = self
            .pty_exit
            .unwrap_or_else(|| panic!("the inbox did not exit\n--- screen ---\n{}", self.screen()));
        assert_eq!(
            code as i32,
            expected,
            "TUI exit code was {code}, expected {expected}\n--- screen ---\n{}",
            self.screen()
        );
    }

    pub fn assert_output(&self, needle: &str) {
        assert!(
            self.last.stdout.contains(needle),
            "stdout did not contain {needle:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.last.stdout,
            self.last.stderr
        );
    }

    pub fn assert_error_output(&self, needle: &str) {
        assert!(
            self.last.stderr.contains(needle),
            "stderr did not contain {needle:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.last.stdout,
            self.last.stderr
        );
    }

    pub fn assert_notification_line(&self, title: &str) {
        let path = self
            .notification_log
            .as_ref()
            .expect("desktop notifications are not enabled");
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let lines: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "the notification log must hold one line, found {}\n{text}",
            lines.len()
        );
        let value: serde_json::Value =
            serde_json::from_str(lines[0]).expect("the notification line is JSON");
        assert_eq!(
            value.get("title").and_then(|value| value.as_str()),
            Some(title),
            "unexpected notification title in {text}"
        );
    }
}
