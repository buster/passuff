use std::time::Duration;

use async_trait::async_trait;
use passuff_config::CredentialSpec;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CredentialError {
    #[error("environment variable is not set")]
    MissingEnv,
    #[error("credential command failed")]
    CommandFailed,
    #[error("credential command returned no value")]
    CommandEmpty,
    #[error("credential command timed out")]
    Timeout,
}

#[async_trait]
pub trait CredentialSource: Send + Sync {
    async fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemCredentialSource;

#[async_trait]
impl CredentialSource for SystemCredentialSource {
    async fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        match spec {
            CredentialSpec::Env { variable } => std::env::var(variable)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .map(|value| value.trim().to_string())
                .ok_or(CredentialError::MissingEnv),
            CredentialSpec::Command { command } => {
                let output = tokio::time::timeout(
                    Duration::from_secs(10),
                    tokio::process::Command::new("/bin/sh")
                        .arg("-c")
                        .arg(command)
                        .output(),
                )
                .await
                .map_err(|_| CredentialError::Timeout)?
                .map_err(|_| CredentialError::CommandFailed)?;
                if !output.status.success() {
                    return Err(CredentialError::CommandFailed);
                }
                let value = String::from_utf8(output.stdout)
                    .map_err(|_| CredentialError::CommandFailed)?
                    .trim()
                    .to_string();
                if value.is_empty() {
                    Err(CredentialError::CommandEmpty)
                } else {
                    Ok(value)
                }
            }
        }
    }
}
