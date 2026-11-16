//! Per-scenario state: a temporary home and config, the GraphQL mock, the CLI
//! process outcome, optional pty and watch sessions, and the persisted files
//! a scenario inspects.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::harness::bin::{self, ChildEnv};
use crate::harness::child::ChildProcess;
use crate::harness::github::{GitHubMock, LOGIN, PERSONAL_LOGIN};
use crate::harness::pty::PtySession;

const SCREEN_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_TIMEOUT: Duration = Duration::from_secs(15);
const FILE_TIMEOUT: Duration = Duration::from_secs(20);

/// A state file written before the version counter reached 2. The declared
/// version-1 shape carries accounts and pull-request observations only; the
/// history arrived with version 2.
pub const VERSION_ONE_STATE: &str = r#"{
  "version": 1,
  "accounts": {
    "work": {
      "login": "tester",
      "last_check": "2026-11-01T09:00:00Z",
      "pull_requests": {
        "octo/repo#7": {"state": "open", "ci": "passing", "title": "Add retry loop"}
      }
    }
  }
}"#;

/// Truncated JSON: unparsable, so the loader has to quarantine it.
pub const CORRUPT_STATE: &str = "{\"version\":2,\"accounts\":";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    Env(String),
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Account {
    pub label: String,
    pub login: String,
    pub credential: Credential,
}

impl Account {
    fn environment(label: &str, login: &str, variable: &str) -> Self {
        Self {
            label: label.to_string(),
            login: login.to_string(),
            credential: Credential::Env(variable.to_string()),
        }
    }
}

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
    pub accounts: Vec<Account>,
    pub desktop: bool,
    pub bot_comments: bool,
    pub watch_scope: Option<(String, Vec<String>)>,
    pub notification_log: Option<PathBuf>,
    pub config_before: Option<Vec<u8>>,
    pub last: Outcome,
    pub pty: Option<PtySession>,
    pub pty_exit: Option<u32>,
    pub watch: Option<ChildProcess>,
    pub watch_exit: Option<i32>,
}

impl std::fmt::Debug for World {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("World")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("graphql_url", &self.graphql_url)
            .field("accounts", &self.accounts)
            .field("last", &self.last)
            .field("pty_exit", &self.pty_exit)
            .field("watch_exit", &self.watch_exit)
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
            accounts: vec![Account::environment("work", LOGIN, bin::TOKEN_ENV)],
            desktop: false,
            bot_comments: false,
            watch_scope: None,
            notification_log: None,
            config_before: None,
            last: Outcome::default(),
            pty: None,
            pty_exit: None,
            watch: None,
            watch_exit: None,
        };
        world.write_config().map_err(std::io::Error::other)?;
        Ok(world)
    }
}

impl World {
    pub fn assert_ready(&self) {
        assert!(
            self.github.is_some(),
            "the mock GitHub account was not started"
        );
    }

    fn github(&self) -> &GitHubMock {
        self.github.as_ref().expect("mock GitHub account")
    }

    pub fn add_comment(&self) {
        self.github().add_comment();
    }

    pub fn add_own_comment(&self) {
        self.github().add_own_comment();
    }

    pub fn add_bot_comment(&self) {
        self.github().add_bot_comment();
    }

    pub fn add_personal_comment(&self) {
        self.github().add_personal_comment();
    }

    pub fn fail_ci(&self) {
        self.github().fail_ci();
    }

    pub fn pass_ci(&self) {
        self.github().pass_ci();
    }

    pub fn observe_only(&self) {
        self.github().observe_only();
    }

    pub fn fail_everything(&self) {
        self.github().fail_everything();
    }

    pub fn set_viewer_login(&self, login: &str) {
        self.github().set_viewer_login(login);
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

    pub fn enable_bot_comments(&mut self) {
        self.bot_comments = true;
        self.write_config().expect("rewrite configuration");
    }

    pub fn add_personal_account(&mut self) {
        self.accounts.push(Account::environment(
            "personal",
            PERSONAL_LOGIN,
            bin::PERSONAL_TOKEN_ENV,
        ));
        self.write_config().expect("rewrite configuration");
    }

    pub fn set_watch_scope(&mut self, mode: &str, repositories: &[&str]) {
        self.watch_scope = Some((
            mode.to_string(),
            repositories.iter().map(ToString::to_string).collect(),
        ));
        self.write_config().expect("rewrite configuration");
    }

    /// Replace the work account's credential with a token file the scenario
    /// writes at the given mode.
    pub fn use_token_file(&mut self, contents: &str, mode: u32) -> PathBuf {
        let path = self.dir.path().join("work.token");
        std::fs::write(&path, contents).expect("write token file");
        set_mode(&path, mode);
        if let Some(account) = self.accounts.first_mut() {
            account.credential = Credential::File(path.clone());
        }
        self.write_config().expect("rewrite configuration");
        path
    }

    pub fn remove_config(&mut self) {
        let _ = std::fs::remove_file(&self.config);
    }

    pub fn remember_config(&mut self) {
        self.config_before = Some(self.config_bytes());
    }

    pub fn assert_config_unchanged(&self) {
        let before = self
            .config_before
            .as_ref()
            .expect("the scenario did not remember the configuration");
        assert_eq!(
            before,
            &self.config_bytes(),
            "the configuration file changed"
        );
    }

    pub fn config_bytes(&self) -> Vec<u8> {
        std::fs::read(&self.config)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", self.config.display()))
    }

    pub fn config_text(&self) -> String {
        String::from_utf8_lossy(&self.config_bytes()).into_owned()
    }

    pub fn assert_config_contains(&self, needle: &str) {
        let text = self.config_text();
        assert!(
            text.contains(needle),
            "the configuration did not contain {needle:?}\n--- configuration ---\n{text}"
        );
    }

    pub fn write_state(&self, text: &str) {
        std::fs::write(&self.state, text)
            .unwrap_or_else(|error| panic!("cannot write {}: {error}", self.state.display()));
    }

    pub fn state_text(&self) -> String {
        std::fs::read_to_string(&self.state).unwrap_or_default()
    }

    pub fn state_json(&self) -> serde_json::Value {
        let text = self.state_text();
        serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("state file is not JSON: {error}\n{text}"))
    }

    pub fn assert_state_version(&self, expected: u64) {
        let value = self.state_json();
        assert_eq!(
            value.get("version").and_then(serde_json::Value::as_u64),
            Some(expected),
            "unexpected state version in\n{}",
            self.state_text()
        );
    }

    pub fn assert_state_contains(&self, needle: &str) {
        let text = self.state_text();
        assert!(
            text.contains(needle),
            "the state file did not contain {needle:?}\n--- state ---\n{text}"
        );
    }

    pub fn assert_state_absent(&self) {
        assert!(
            !self.state.exists(),
            "the state file still exists: {}",
            self.state.display()
        );
    }

    fn quarantine_files(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let Ok(entries) = std::fs::read_dir(self.dir.path()) else {
            return found;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("state.corrupt.") && name.ends_with(".json") {
                found.push(entry.path());
            }
        }
        found
    }

    pub fn assert_quarantined(&self, expected: &str) {
        let files = self.quarantine_files();
        assert_eq!(
            files.len(),
            1,
            "expected one quarantined state file, found {files:?}"
        );
        let text = std::fs::read_to_string(&files[0])
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", files[0].display()));
        assert_eq!(text, expected, "the quarantined bytes differ");
    }

    pub fn write_config(&self) -> Result<(), String> {
        let mut text = format!(
            "poll_interval_secs = 1\n\
             reconciliation_interval_secs = 600\n\
             \n\
             [events]\n\
             bot_comments = {bot}\n\
             \n\
             [notifications]\n\
             desktop = {desktop}\n\
             sound = false\n",
            bot = self.bot_comments,
            desktop = self.desktop,
        );
        if let Some((mode, repositories)) = &self.watch_scope {
            let list = repositories
                .iter()
                .map(|repository| format!("\"{repository}\""))
                .collect::<Vec<_>>()
                .join(", ");
            text.push_str(&format!(
                "\n[watch]\nmode = \"{mode}\"\nrepositories = [{list}]\n"
            ));
        }
        for account in &self.accounts {
            text.push_str(&format!(
                "\n[accounts.{}]\ngithub_user = \"{}\"\n",
                account.label, account.login
            ));
            match &account.credential {
                Credential::Env(variable) => {
                    text.push_str(&format!("token_env = \"{variable}\"\n"));
                }
                Credential::File(path) => {
                    text.push_str(&format!("token_file = \"{}\"\n", path.display()));
                }
            }
            text.push_str(&format!("graphql_url = \"{}\"\n", self.graphql_url));
        }
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

    fn base_command(&self) -> Result<std::process::Command, String> {
        let mut command = bin::product_command("passuff", &self.child_env())?;
        command
            .arg("--config")
            .arg(&self.config)
            .arg("--state")
            .arg(&self.state);
        Ok(command)
    }

    pub fn run(&mut self, args: &[&str]) -> Result<(), String> {
        let mut command = self.base_command()?;
        command.args(args);
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

    /// Run a command with a prepared stdin. Setup prompts read line by line.
    pub fn run_with_input(&mut self, args: &[&str], input: &str) -> Result<(), String> {
        use std::io::Write;
        use std::process::Stdio;
        let mut command = self.base_command()?;
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|error| format!("cannot start passuff {args:?}: {error}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .map_err(|error| format!("cannot write setup input: {error}"))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|error| format!("cannot wait for passuff {args:?}: {error}"))?;
        self.last = Outcome {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        };
        Ok(())
    }

    pub fn start_watch(&mut self) -> Result<(), String> {
        use std::process::Stdio;
        let mut command = self.base_command()?;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        self.watch = Some(ChildProcess::spawn(&mut command)?);
        self.watch_exit = None;
        Ok(())
    }

    /// Wait until the first watch pass stored an account baseline.
    pub fn wait_for_watch_baseline(&self) {
        let deadline = Instant::now() + FILE_TIMEOUT;
        loop {
            let ready = serde_json::from_str::<serde_json::Value>(&self.state_text())
                .ok()
                .and_then(|value| {
                    value
                        .pointer("/accounts/work/last_check")
                        .and_then(serde_json::Value::as_str)
                        .map(ToString::to_string)
                })
                .is_some();
            if ready {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "the watch never recorded a baseline\n--- state ---\n{}",
                self.state_text()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn interrupt_watch(&mut self) {
        let watch = self.watch.as_ref().expect("the watch is not running");
        watch.signal("INT").expect("send SIGINT");
        let code = self
            .watch
            .as_mut()
            .expect("the watch is not running")
            .wait_exit(EXIT_TIMEOUT);
        self.watch_exit = code;
    }

    pub fn assert_watch_exit(&self, expected: i32) {
        let code = self
            .watch_exit
            .unwrap_or_else(|| panic!("the watch did not exit"));
        assert_eq!(code, expected, "the watch exited with {code}");
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

    fn send_keys(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.pty
            .as_mut()
            .ok_or_else(|| "the inbox is not open".to_string())?
            .send(bytes)
    }

    pub fn press_enter(&mut self) -> Result<(), String> {
        self.send_keys(b"\r")
    }

    pub fn press_tab(&mut self) -> Result<(), String> {
        self.send_keys(b"\t")
    }

    pub fn press_backtab(&mut self) -> Result<(), String> {
        self.send_keys(b"\x1b[Z")
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

    pub fn assert_output_lacks(&self, needle: &str) {
        assert!(
            !self.last.stdout.contains(needle),
            "stdout unexpectedly contained {needle:?}\n--- stdout ---\n{}",
            self.last.stdout
        );
    }

    pub fn assert_output_lines(&self, needle: &str, expected: usize) {
        let found = self
            .last
            .stdout
            .lines()
            .filter(|line| line.contains(needle))
            .count();
        assert_eq!(
            found, expected,
            "expected {expected} output lines containing {needle:?}, found {found}\n--- stdout ---\n{}",
            self.last.stdout
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

    fn notification_lines(&self) -> Vec<serde_json::Value> {
        let Some(path) = &self.notification_log else {
            return Vec::new();
        };
        let text = std::fs::read_to_string(path).unwrap_or_default();
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line)
                    .unwrap_or_else(|error| panic!("notification line is not JSON: {error}"))
            })
            .collect()
    }

    pub fn assert_notification_line(&self, title: &str) {
        assert!(
            self.notification_log.is_some(),
            "desktop notifications are not enabled"
        );
        let deadline = Instant::now() + FILE_TIMEOUT;
        while self.notification_lines().is_empty() {
            assert!(
                Instant::now() < deadline,
                "no notification line was written"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        let lines = self.notification_lines();
        assert_eq!(
            lines.len(),
            1,
            "the notification log must hold one line, found {}",
            lines.len()
        );
        let value = &lines[0];
        assert_eq!(
            value.get("title").and_then(|value| value.as_str()),
            Some(title),
            "unexpected notification title in {value}"
        );
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let permissions = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, permissions).expect("set token file mode");
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) {}
