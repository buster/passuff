use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use passuff_core::{CredentialError, GhLogin};

pub struct SystemGhLogin;

#[async_trait]
impl GhLogin for SystemGhLogin {
    async fn login(&self, _login: &str, host: &str) -> Result<(), CredentialError> {
        let status = tokio::time::timeout(
            Duration::from_secs(300),
            tokio::process::Command::new("gh")
                .args(["auth", "login", "--web", "--hostname", host])
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

    fn fake_gh(exit: i32) -> (tempfile::TempDir, std::path::PathBuf) {
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
        (dir, log)
    }

    #[tokio::test]
    async fn login_runs_the_web_flow_for_the_host() {
        let (dir, log) = fake_gh(0);
        let old = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{}", dir.path().display(), old));
        let result = SystemGhLogin.login("octo-work", "github.com").await;
        std::env::set_var("PATH", old);
        assert!(result.is_ok());
        let logged = std::fs::read_to_string(log).expect("log");
        assert!(logged.contains("auth login --web --hostname github.com"));
    }

    #[tokio::test]
    async fn login_reports_a_missing_gh() {
        let old = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", "/nonexistent");
        let result = SystemGhLogin.login("octo-work", "github.com").await;
        std::env::set_var("PATH", old);
        assert!(matches!(result, Err(CredentialError::GhNotFound)));
    }

    #[tokio::test]
    async fn login_reports_a_gh_failure() {
        let (dir, _log) = fake_gh(1);
        let old = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{}", dir.path().display(), old));
        let result = SystemGhLogin.login("octo-work", "github.com").await;
        std::env::set_var("PATH", old);
        assert!(matches!(result, Err(CredentialError::GhLoginFailed)));
    }
}
