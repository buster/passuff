use std::collections::HashSet;
use std::fs;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use passuff_config::{expand_home, AccountConfig, AuthMode};

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("{0}")]
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub token: String,
}

pub trait CredentialSource: Send + Sync {
    /// Resolve one account's token. The token is never persisted or logged.
    fn resolve(
        &self,
        label: &str,
        account: &AccountConfig,
        interactive: bool,
    ) -> Result<ResolvedCredential, CredentialError>;
}

#[derive(Default)]
pub struct SystemCredentialSource {
    attempted_logins: Mutex<HashSet<String>>,
}

impl SystemCredentialSource {
    pub fn new() -> Self {
        Self::default()
    }

    fn resolve_env(&self, name: &str) -> Result<String, CredentialError> {
        let value = std::env::var(name).map_err(|_| {
            CredentialError::Failed(format!("credential environment variable {name} is not set"))
        })?;
        let value = value.trim().to_string();
        if value.is_empty() {
            return Err(CredentialError::Failed(format!(
                "credential environment variable {name} is empty"
            )));
        }
        Ok(value)
    }

    fn resolve_command(&self, command: &str) -> Result<String, CredentialError> {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                CredentialError::Failed(format!("credential command failed: {error}"))
            })?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        return Err(CredentialError::Failed(
                            "credential command failed".to_string(),
                        ));
                    }
                    let output = child.wait_with_output().map_err(|error| {
                        CredentialError::Failed(format!("credential command failed: {error}"))
                    })?;
                    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if token.is_empty() {
                        return Err(CredentialError::Failed(
                            "credential command produced an empty token".to_string(),
                        ));
                    }
                    return Ok(token);
                }
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    return Err(CredentialError::Failed(
                        "credential command timed out".to_string(),
                    ));
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(25)),
                Err(error) => {
                    return Err(CredentialError::Failed(format!(
                        "credential command failed: {error}"
                    )))
                }
            }
        }
    }

    fn resolve_file(&self, path: &str) -> Result<String, CredentialError> {
        let expanded = expand_home(path);
        let metadata = match fs::metadata(&expanded) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CredentialError::Failed(format!(
                    "token file is missing: {path}"
                )))
            }
            Err(error) => {
                return Err(CredentialError::Failed(format!(
                    "token file is not readable: {error}"
                )))
            }
        };
        if !metadata.is_file() {
            return Err(CredentialError::Failed(format!(
                "token file is not a regular file: {path}"
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode() & 0o777;
            if mode & 0o077 != 0 {
                return Err(CredentialError::Failed(format!(
                    "token_file is not private (insecure mode {mode:o})"
                )));
            }
        }
        let token = fs::read_to_string(&expanded)
            .map_err(|error| {
                CredentialError::Failed(format!("token file is not readable: {error}"))
            })?
            .trim()
            .to_string();
        if token.is_empty() {
            return Err(CredentialError::Failed(format!(
                "token file is empty: {path}"
            )));
        }
        Ok(token)
    }

    fn gh_token(&self, login: &str) -> Result<String, CredentialError> {
        let output = Command::new("gh")
            .args(["auth", "token", "--user", login])
            .stdin(Stdio::null())
            .output()
            .map_err(|_| CredentialError::Failed("gh is not available".to_string()))?;
        if !output.status.success() {
            return Err(CredentialError::Failed(format!(
                "gh auth token failed for @{login}"
            )));
        }
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            return Err(CredentialError::Failed(format!(
                "gh auth token failed for @{login}"
            )));
        }
        Ok(token)
    }

    fn gh_login(&self, _login: &str) -> Result<(), CredentialError> {
        let status = Command::new("gh")
            .args(["auth", "login", "--web", "--hostname", "github.com"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|_| CredentialError::Failed("gh auth login failed".to_string()))?;
        if status.success() {
            Ok(())
        } else {
            Err(CredentialError::Failed("gh auth login failed".to_string()))
        }
    }

    fn resolve_gh(&self, login: &str, interactive: bool) -> Result<String, CredentialError> {
        match self.gh_token(login) {
            Ok(token) => return Ok(token),
            Err(error) if !interactive => return Err(error),
            Err(_) => {}
        }
        let mut attempted = self
            .attempted_logins
            .lock()
            .map_err(|_| CredentialError::Failed("credential state is unavailable".to_string()))?;
        if !attempted.insert(login.to_string()) {
            return Err(CredentialError::Failed(format!(
                "gh auth login failed for @{login}"
            )));
        }
        drop(attempted);
        self.gh_login(login)?;
        self.gh_token(login)
    }
}

impl CredentialSource for SystemCredentialSource {
    fn resolve(
        &self,
        label: &str,
        account: &AccountConfig,
        interactive: bool,
    ) -> Result<ResolvedCredential, CredentialError> {
        match account.auth_mode() {
            AuthMode::GhOAuth => {
                let login = account.expected_login().ok_or_else(|| {
                    CredentialError::Failed(format!(
                        "account {label}: gh_oauth requires github_user"
                    ))
                })?;
                let token = self.resolve_gh(login, interactive)?;
                Ok(ResolvedCredential { token })
            }
            AuthMode::GithubPat => {
                if let Some(name) = account.token_env.as_deref() {
                    return Ok(ResolvedCredential {
                        token: self.resolve_env(name)?,
                    });
                }
                if let Some(command) = account.token_command.as_deref() {
                    return Ok(ResolvedCredential {
                        token: self.resolve_command(command)?,
                    });
                }
                if let Some(path) = account.token_file.as_deref() {
                    return Ok(ResolvedCredential {
                        token: self.resolve_file(path)?,
                    });
                }
                Err(CredentialError::Failed(format!(
                    "account {label}: must select exactly one credential source"
                )))
            }
        }
    }
}
