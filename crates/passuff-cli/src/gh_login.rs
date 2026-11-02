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
            Err(CredentialError::CommandFailed)
        }
    }
}
