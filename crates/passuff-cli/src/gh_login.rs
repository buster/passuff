use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use passuff_core::{CredentialError, GhLogin};

pub struct SystemGhLogin {
    program: PathBuf,
}

impl SystemGhLogin {
    pub fn new() -> Self {
        Self {
            program: PathBuf::from("gh"),
        }
    }

    pub fn with_program(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

#[async_trait]
impl GhLogin for SystemGhLogin {
    async fn login(&self, _login: &str, host: &str) -> Result<(), CredentialError> {
        let status = tokio::time::timeout(
            Duration::from_secs(300),
            tokio::process::Command::new(&self.program)
                .args(["auth", "login", "--web", "--hostname", host])
                .kill_on_drop(true)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status(),
        )
        .await
        .map_err(|_| CredentialError::Timeout)?
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                CredentialError::GhNotFound
            } else {
                CredentialError::CommandFailed
            }
        })?;
        if status.success() {
            Ok(())
        } else {
            Err(CredentialError::GhLoginFailed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    static SCRIPT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn fake_gh(exit: i32) -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().expect("temp dir");
        let script = dir.path().join("gh");
        let log = dir.path().join("gh.log");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\nexit {exit}\n",
                log.display()
            ),
        )
        .expect("fake gh");
        let mut permissions = std::fs::metadata(&script).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).expect("mode");
        (dir, script, log)
    }

    #[tokio::test]
    async fn login_runs_the_web_flow_for_the_host() {
        let _guard = SCRIPT_LOCK.lock().await;
        let (_dir, script, log) = fake_gh(0);
        let result = SystemGhLogin::with_program(script)
            .login("octo-work", "github.com")
            .await;
        assert!(result.is_ok(), "{result:?}");
        let logged = std::fs::read_to_string(log).expect("log");
        assert!(logged.contains("auth login --web --hostname github.com"));
    }

    #[tokio::test]
    async fn login_reports_a_missing_gh() {
        let dir = tempfile::tempdir().expect("temp dir");
        let result = SystemGhLogin::with_program(dir.path().join("missing"))
            .login("octo-work", "github.com")
            .await;
        assert!(matches!(result, Err(CredentialError::GhNotFound)));
    }

    #[tokio::test]
    async fn login_reports_a_gh_failure() {
        let _guard = SCRIPT_LOCK.lock().await;
        let (_dir, script, _log) = fake_gh(1);
        let result = SystemGhLogin::with_program(script)
            .login("octo-work", "github.com")
            .await;
        assert!(
            matches!(result, Err(CredentialError::GhLoginFailed)),
            "{result:?}"
        );
    }
}
