#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::World;

#[cfg(target_os = "linux")]
pub struct DbustTwin {
    _dir: tempfile::TempDir,
    child: Child,
    pub address: String,
    _connection: zbus::Connection,
    pub notifications: Arc<Mutex<Vec<Value>>>,
}

#[cfg(target_os = "linux")]
impl Drop for DbustTwin {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(target_os = "linux")]
struct FakeNotifications {
    notifications: Arc<Mutex<Vec<Value>>>,
}

#[cfg(target_os = "linux")]
#[zbus::interface(name = "org.freedesktop.Notifications")]
impl FakeNotifications {
    #[zbus(name = "Notify")]
    async fn notify(
        &self,
        _app_name: String,
        _replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        _hints: HashMap<String, zbus::OwnedVariant>,
        _expire_timeout: i32,
    ) -> u32 {
        self.notifications
            .lock()
            .expect("notification log")
            .push(json!({"title": summary, "summary": summary, "body": body}));
        1
    }

    #[zbus(name = "GetCapabilities")]
    async fn get_capabilities(&self) -> Vec<String> {
        vec!["body".to_string(), "sound".to_string()]
    }

    #[zbus(name = "GetServerInformation")]
    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "passuff-test".to_string(),
            "passuff".to_string(),
            "0.1".to_string(),
            "1.2".to_string(),
        )
    }

    #[zbus(name = "CloseNotification")]
    async fn close_notification(&self, _id: u32) {}
}

#[cfg(target_os = "linux")]
impl DbustTwin {
    async fn start() -> Result<Self, String> {
        let dir = tempfile::TempDir::new().map_err(|error| error.to_string())?;
        let config = dir.path().join("bus.conf");
        fs::write(&config, BUS_CONFIG).map_err(|error| error.to_string())?;
        let mut child = Command::new("dbus-daemon")
            .args([
                "--config-file",
                config
                    .to_str()
                    .ok_or_else(|| "bus config path".to_string())?,
                "--nofork",
                "--print-address=1",
                "--print-pid=1",
            ])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| format!("start dbus-daemon: {error}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "dbus-daemon stdout".to_string())?;
        let mut reader = BufReader::new(stdout);
        let mut address = String::new();
        let mut pid = String::new();
        reader
            .read_line(&mut address)
            .map_err(|error| error.to_string())?;
        reader
            .read_line(&mut pid)
            .map_err(|error| error.to_string())?;
        let address = address.trim().to_string();
        if address.is_empty() {
            return Err("dbus-daemon printed no address".to_string());
        }
        let connection = zbus::connection::Builder::address(address.as_str())
            .build()
            .await
            .map_err(|error| error.to_string())?;
        let notifications = Arc::new(Mutex::new(Vec::new()));
        connection
            .object_server()
            .at(
                "/org/freedesktop/Notifications",
                FakeNotifications {
                    notifications: notifications.clone(),
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        connection
            .request_name("org.freedesktop.Notifications")
            .await
            .map_err(|error| error.to_string())?;
        Ok(Self {
            _dir: dir,
            child,
            address,
            _connection: connection,
            notifications,
        })
    }
}

#[cfg(target_os = "linux")]
const BUS_CONFIG: &str = r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:tmpdir=/tmp</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
"#;

impl World {
    pub async fn e2e_configure_account(&mut self, label: &str, login: &str) {
        if self.e2e_config_dir.is_some() {
            return;
        }
        let dir = tempfile::TempDir::new().expect("e2e config dir");
        let server = MockServer::start().await;
        let sound_dir = tempfile::TempDir::new().expect("sound dir");
        let sound_log = sound_dir.path().join("sound.log");
        let script = sound_dir.path().join("paplay");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$1\" >> \"{}\"\n",
                sound_log.display()
            ),
        )
        .expect("sound script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&script)
                .expect("sound script metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&script, permissions).expect("sound script mode");
        }

        let graphql_url = format!("{}/graphql", server.uri());
        let config = format!(
            "poll_interval_secs = 1\n\n[notifications]\ndesktop = true\nsound = true\nsound_file = \"default\"\nurgency = \"normal\"\n\n[events]\ncomments = true\nreviews = true\nci_failure = true\nci_recovery = true\nstate_changes = true\nbot_comments = false\n\n[accounts.{label}]\ntoken_env = \"PASSUFF_E2E_TOKEN\"\ngraphql_url = \"{graphql_url}\"\n"
        );
        fs::write(dir.path().join("config.toml"), config).expect("e2e config");

        #[cfg(target_os = "linux")]
        {
            let dbus = DbustTwin::start()
                .await
                .unwrap_or_else(|error| panic!("start D-Bus fake: {error}"));
            self.e2e_dbus = Some(dbus);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let sink = dir.path().join("notifications.jsonl");
            fs::write(&sink, "").expect("sink");
            self.e2e_sink = Some(sink);
        }

        self.e2e_label = label.to_string();
        self.e2e_login = login.to_string();
        self.e2e_server = Some(server);
        self.e2e_config_dir = Some(dir);
        self.e2e_sound_dir = Some(sound_dir);
        self.e2e_sound_log = Some(sound_log);
        self.e2e_state_path = Some(
            self.e2e_config_dir
                .as_ref()
                .unwrap()
                .path()
                .join("state.json"),
        );
        self.e2e_prs.clear();
        self.e2e_refresh_mock().await;
    }

    async fn e2e_refresh_mock(&self) {
        let Some(server) = self.e2e_server.as_ref() else {
            return;
        };
        let body = json!({
            "data": {
                "viewer": {
                    "login": self.e2e_login,
                    "pullRequests": {
                        "pageInfo": { "hasNextPage": false, "endCursor": Value::Null },
                        "nodes": self.e2e_prs,
                    }
                }
            }
        });
        server.reset().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(server)
            .await;
    }

    pub async fn e2e_add_pull_request(&mut self, key: &str, author: &str) {
        if self.e2e_node(key).is_some() {
            return;
        }
        let parsed = super::parse_key(key);
        let node = json!({
            "id": format!("node-{key}"),
            "number": parsed.number,
            "title": format!("Pull request {key}"),
            "url": format!("https://github.com/{key}"),
            "state": "OPEN",
            "merged": false,
            "isDraft": false,
            "createdAt": "2026-11-02T11:00:00Z",
            "updatedAt": "2026-11-02T11:00:00Z",
            "baseRefName": "main",
            "author": { "login": author },
            "repository": { "nameWithOwner": parsed.repository },
            "assignees": { "nodes": [] },
            "labels": { "nodes": [] },
            "milestone": Value::Null,
            "comments": { "nodes": [] },
            "reviews": { "nodes": [] },
            "reviewThreads": { "nodes": [] },
            "commits": { "nodes": [{ "commit": { "oid": "abc123", "statusCheckRollup": { "state": "SUCCESS", "contexts": { "nodes": [] } } } }] },
        });
        self.e2e_prs.push(node);
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_clear_pull_requests(&mut self) {
        self.e2e_prs.clear();
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_set_comment(&mut self, key: &str, id: &str, author: &str, body: &str) {
        let node = self.e2e_node_mut(key);
        node["comments"]["nodes"]
            .as_array_mut()
            .expect("comments")
            .push(json!({
                "id": id,
                "author": { "login": author },
                "body": body,
                "createdAt": "2026-11-02T12:00:01Z",
            }));
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_set_review_comment(&mut self, key: &str, id: &str, author: &str, body: &str) {
        let node = self.e2e_node_mut(key);
        node["reviewThreads"]["nodes"]
            .as_array_mut()
            .expect("review threads")
            .push(json!({
                "comments": { "nodes": [{
                    "id": id,
                    "author": { "login": author },
                    "body": body,
                    "createdAt": "2026-11-02T12:00:01Z",
                }]}
            }));
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_set_review(&mut self, key: &str, id: &str, author: &str, state: &str) {
        let state = match state {
            "approved" => "APPROVED",
            "changes requested" => "CHANGES_REQUESTED",
            _ => "COMMENTED",
        };
        let node = self.e2e_node_mut(key);
        node["reviews"]["nodes"]
            .as_array_mut()
            .expect("reviews")
            .push(json!({
                "id": id,
                "author": { "login": author },
                "state": state,
                "body": "",
                "submittedAt": "2026-11-02T12:00:01Z",
            }));
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_set_ci(&mut self, key: &str, state: &str) {
        let state = match state {
            "failing" | "failure" => "FAILURE",
            "error" => "ERROR",
            "successful" | "success" => "SUCCESS",
            "pending" => "PENDING",
            other => panic!("unknown CI state {other}"),
        };
        let node = self.e2e_node_mut(key);
        node["commits"]["nodes"][0]["commit"]["statusCheckRollup"]["state"] = json!(state);
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_set_state(&mut self, key: &str, state: &str) {
        let (state, merged) = match state {
            "merged" => ("MERGED", true),
            "closed" => ("CLOSED", false),
            "open" => ("OPEN", false),
            other => panic!("unknown PR state {other}"),
        };
        let node = self.e2e_node_mut(key);
        node["state"] = json!(state);
        node["merged"] = json!(merged);
        self.e2e_refresh_mock().await;
    }

    pub async fn e2e_run_check(&mut self) {
        let config = self
            .e2e_config_dir
            .as_ref()
            .expect("e2e configured")
            .path()
            .join("config.toml");
        let state = self.e2e_state_path.clone().expect("e2e state");
        let mut command = Command::new(env!("CARGO_BIN_EXE_passuff"));
        command
            .arg("check")
            .arg("--config")
            .arg(config)
            .arg("--state")
            .arg(state);
        self.configure_child(&mut command);
        let output = command.output().expect("run passuff check");
        self.last_status = output.status.code().unwrap_or(3);
        self.last_transcript = String::from_utf8_lossy(&output.stdout).to_string();
        self.e2e_write_evidence();
    }

    pub async fn e2e_run_with(&mut self, flag: &str) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_passuff"));
        command.arg(flag);
        let output = command.output().expect("run passuff");
        self.last_status = output.status.code().unwrap_or(3);
        self.last_transcript = String::from_utf8_lossy(&output.stdout).to_string();
        self.e2e_write_evidence();
    }

    pub async fn e2e_start_watching(&mut self) {
        let config = self
            .e2e_config_dir
            .as_ref()
            .expect("e2e configured")
            .path()
            .join("config.toml");
        let state = self.e2e_state_path.clone().expect("e2e state");
        let mut command = Command::new(env!("CARGO_BIN_EXE_passuff"));
        command
            .arg("--config")
            .arg(config)
            .arg("--state")
            .arg(state)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        self.configure_child(&mut command);
        let mut child = command.spawn().expect("start passuff");
        let stdout = child.stdout.take().expect("child stdout");
        let lines = self.e2e_stdout.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                lines.lock().expect("stdout log").push(line);
            }
        });
        self.e2e_child = Some(child);
        self.e2e_wait_for("passuff: watching", Duration::from_secs(15))
            .await;
    }

    pub async fn e2e_stop(&mut self) {
        let mut child = self.e2e_child.take().expect("running child");
        let pid = child.id().to_string();
        let _ = Command::new("kill").args(["-INT", &pid]).status();
        let status = child.wait().expect("wait for child");
        self.last_status = status.code().unwrap_or(1);
        self.e2e_write_evidence();
    }

    pub async fn e2e_wait_terminal_notification(&self, key: &str) {
        let needle = format!("passuff: {key}");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        loop {
            if self
                .e2e_stdout
                .lock()
                .expect("stdout log")
                .iter()
                .any(|line| line.contains(&needle))
            {
                return;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!(
                    "terminal did not show {needle}: {:?}",
                    self.e2e_stdout.lock().expect("stdout log")
                );
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn e2e_wait_notification(&self) -> Option<Value> {
        for _ in 0..100 {
            if let Some(notification) = self.e2e_last_notification() {
                return Some(notification);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        self.e2e_last_notification()
    }

    pub async fn e2e_wait_sound(&self) {
        for _ in 0..100 {
            if !self.sound_calls().is_empty() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub fn e2e_last_notification(&self) -> Option<Value> {
        #[cfg(target_os = "linux")]
        {
            if let Some(dbus) = self.e2e_dbus.as_ref() {
                if let Some(notification) = dbus
                    .notifications
                    .lock()
                    .expect("notification log")
                    .last()
                    .cloned()
                {
                    return Some(notification);
                }
            }
        }
        if let Some(sink) = self.e2e_sink.as_ref() {
            if let Ok(text) = fs::read_to_string(sink) {
                if let Some(line) = text.lines().last() {
                    return serde_json::from_str(line).ok();
                }
            }
        }
        None
    }

    fn e2e_write_evidence(&self) {
        if self.e2e_scenario.is_empty() {
            return;
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let base = if root.join("givn/changes/passuff-pr-watch/specs").is_dir() {
            root.join("givn/changes/passuff-pr-watch/evidence/visual")
        } else {
            root.join("target/visual")
        };
        let directory = base.join(&self.e2e_scenario);
        if fs::create_dir_all(&directory).is_err() {
            return;
        }
        let stdout = self.e2e_stdout.lock().expect("stdout log").join("\n");
        let mut transcript = if stdout.trim().is_empty() {
            self.last_transcript.clone()
        } else {
            let mut text = stdout;
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&self.last_transcript);
            text
        };
        if !transcript.ends_with('\n') {
            transcript.push('\n');
        }
        transcript.push_str(&format!("exit status: {}\n", self.last_status));
        let _ = fs::write(directory.join("transcript.txt"), transcript);
    }

    fn configure_child(&self, command: &mut Command) {
        command.env("PASSUFF_E2E_TOKEN", "e2e-token");
        if let Ok(parent) = std::env::var("LLVM_PROFILE_FILE") {
            let directory = std::path::Path::new(&parent)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            command.env(
                "LLVM_PROFILE_FILE",
                directory.join("e2e-child-%p-%m.profraw"),
            );
        }
        if let Some(dir) = self.e2e_sound_dir.as_ref() {
            let path = format!(
                "{}:{}",
                dir.path().display(),
                std::env::var("PATH").unwrap_or_default()
            );
            command.env("PATH", path);
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(dbus) = self.e2e_dbus.as_ref() {
                command.env("DBUS_SESSION_BUS_ADDRESS", &dbus.address);
            }
        }
        if let Some(sink) = self.e2e_sink.as_ref() {
            command.env("PASSUFF_NOTIFICATION_SINK", sink);
        }
    }

    async fn e2e_wait_for(&self, needle: &str, timeout: Duration) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self
                .e2e_stdout
                .lock()
                .expect("stdout log")
                .iter()
                .any(|line| line.contains(needle))
            {
                return;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!(
                    "timed out waiting for {needle}: {:?}",
                    self.e2e_stdout.lock().expect("stdout log")
                );
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn e2e_node(&self, key: &str) -> Option<&Value> {
        let parsed = super::parse_key(key);
        self.e2e_prs.iter().find(|node| {
            node["repository"]["nameWithOwner"] == parsed.repository
                && node["number"] == parsed.number
        })
    }

    fn e2e_node_mut(&mut self, key: &str) -> &mut Value {
        let parsed = super::parse_key(key);
        self.e2e_prs
            .iter_mut()
            .find(|node| {
                node["repository"]["nameWithOwner"] == parsed.repository
                    && node["number"] == parsed.number
            })
            .expect("e2e pull request")
    }
}
