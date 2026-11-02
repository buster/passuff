use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
    #[error("gh is not available")]
    GhNotFound,
    #[error("token file is missing: {path}")]
    FileMissing { path: PathBuf },
    #[error("token file cannot be read: {path}")]
    FileUnreadable { path: PathBuf },
    #[error("token file is not private: {path}")]
    FileInsecure { path: PathBuf },
    #[error("token file is empty: {path}")]
    FileEmpty { path: PathBuf },
}

#[async_trait]
pub trait CredentialSource: Send + Sync {
    async fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError>;
}

#[async_trait]
pub trait GhLogin: Send + Sync {
    async fn login(&self, login: &str, host: &str) -> Result<(), CredentialError>;
}

pub struct SystemCredentialSource {
    interactive: bool,
    login: Option<Arc<dyn GhLogin>>,
    attempted: Mutex<HashSet<String>>,
}

impl SystemCredentialSource {
    pub fn headless() -> Self {
        Self::with_login(false, None)
    }

    pub fn interactive(login: Arc<dyn GhLogin>) -> Self {
        Self::with_login(true, Some(login))
    }

    pub fn with_login(interactive: bool, login: Option<Arc<dyn GhLogin>>) -> Self {
        Self {
            interactive,
            login,
            attempted: Mutex::new(HashSet::new()),
        }
    }

    async fn gh_token(&self, login: &str, host: &str) -> Result<String, CredentialError> {
        let output = tokio::time::timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("gh")
                .args(["auth", "token", "--user", login, "--hostname", host])
                .output(),
        )
        .await
        .map_err(|_| CredentialError::Timeout)?;
        let output = match output {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CredentialError::GhNotFound);
            }
            Err(_) => return Err(CredentialError::CommandFailed),
        };
        if !output.status.success() {
            return Err(CredentialError::CommandFailed);
        }
        let token = String::from_utf8(output.stdout)
            .map_err(|_| CredentialError::CommandFailed)?
            .trim()
            .to_string();
        if token.is_empty() {
            Err(CredentialError::CommandEmpty)
        } else {
            Ok(token)
        }
    }

    async fn resolve_gh_oauth(&self, login: &str, host: &str) -> Result<String, CredentialError> {
        let first = self.gh_token(login, host).await;
        if first.is_ok() {
            return first;
        }
        let should_login = self.interactive
            && self.login.is_some()
            && self
                .attempted
                .lock()
                .expect("login attempts")
                .insert(login.to_string());
        if !should_login {
            return first;
        }
        self.login
            .as_ref()
            .expect("interactive login")
            .login(login, host)
            .await?;
        self.gh_token(login, host).await
    }

    fn resolve_file(&self, path: &PathBuf) -> Result<String, CredentialError> {
        let metadata = std::fs::metadata(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                CredentialError::FileMissing { path: path.clone() }
            } else {
                CredentialError::FileUnreadable { path: path.clone() }
            }
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            if mode & 0o077 != 0 || mode & 0o400 == 0 {
                return Err(CredentialError::FileInsecure { path: path.clone() });
            }
        }
        let token = std::fs::read_to_string(path)
            .map_err(|_| CredentialError::FileUnreadable { path: path.clone() })?
            .trim()
            .to_string();
        if token.is_empty() {
            Err(CredentialError::FileEmpty { path: path.clone() })
        } else {
            Ok(token)
        }
    }
}

impl Default for SystemCredentialSource {
    fn default() -> Self {
        Self::headless()
    }
}

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
            CredentialSpec::File { path } => self.resolve_file(path),
            CredentialSpec::GhOAuth { login, host } => self.resolve_gh_oauth(login, host).await,
        }
    }
}
