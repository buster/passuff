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
    #[error("gh auth token failed for the requested login")]
    GhTokenFailed,
    #[error("gh auth login failed")]
    GhLoginFailed,
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

#[async_trait]
trait TokenSource: Send + Sync {
    async fn token(&self, spec: &CredentialSpec) -> Result<String, CredentialError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentCredential;

#[async_trait]
impl TokenSource for EnvironmentCredential {
    async fn token(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        let CredentialSpec::Env { variable } = spec else {
            return Err(CredentialError::CommandFailed);
        };
        std::env::var(variable)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.trim().to_string())
            .ok_or(CredentialError::MissingEnv)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CommandCredential;

#[async_trait]
impl TokenSource for CommandCredential {
    async fn token(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        let CredentialSpec::Command { command } = spec else {
            return Err(CredentialError::CommandFailed);
        };
        let output = tokio::time::timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(command)
                .kill_on_drop(true)
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

#[derive(Debug, Default, Clone, Copy)]
pub struct FileCredential;

impl FileCredential {
    fn resolve(&self, path: &PathBuf) -> Result<String, CredentialError> {
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

#[async_trait]
impl TokenSource for FileCredential {
    async fn token(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        let CredentialSpec::File { path } = spec else {
            return Err(CredentialError::CommandFailed);
        };
        self.resolve(path)
    }
}

pub struct GhOAuthCredential {
    interactive: bool,
    login: Option<Arc<dyn GhLogin>>,
    attempted: Mutex<HashSet<String>>,
}

impl GhOAuthCredential {
    fn new(interactive: bool, login: Option<Arc<dyn GhLogin>>) -> Self {
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
                .kill_on_drop(true)
                .output(),
        )
        .await
        .map_err(|_| CredentialError::Timeout)?;
        let output = match output {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CredentialError::GhNotFound);
            }
            Err(_) => return Err(CredentialError::GhTokenFailed),
        };
        if !output.status.success() {
            return Err(CredentialError::GhTokenFailed);
        }
        let token = String::from_utf8(output.stdout)
            .map_err(|_| CredentialError::GhTokenFailed)?
            .trim()
            .to_string();
        if token.is_empty() {
            Err(CredentialError::CommandEmpty)
        } else {
            Ok(token)
        }
    }
}

#[async_trait]
impl TokenSource for GhOAuthCredential {
    async fn token(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        let CredentialSpec::GhOAuth { login, host } = spec else {
            return Err(CredentialError::GhTokenFailed);
        };
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
}

pub struct SystemCredentialSource {
    env: EnvironmentCredential,
    command: CommandCredential,
    file: FileCredential,
    gh: GhOAuthCredential,
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
            env: EnvironmentCredential,
            command: CommandCredential,
            file: FileCredential,
            gh: GhOAuthCredential::new(interactive, login),
        }
    }
}

#[async_trait]
impl CredentialSource for SystemCredentialSource {
    async fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        match spec {
            CredentialSpec::Env { .. } => self.env.token(spec).await,
            CredentialSpec::Command { .. } => self.command.token(spec).await,
            CredentialSpec::File { .. } => self.file.token(spec).await,
            CredentialSpec::GhOAuth { .. } => self.gh.token(spec).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn private_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).expect("token file");
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&path, permissions).expect("mode");
        path
    }

    async fn file_token(path: PathBuf) -> Result<String, CredentialError> {
        FileCredential.token(&CredentialSpec::File { path }).await
    }

    #[tokio::test]
    async fn file_source_reads_a_private_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = private_file(dir.path(), "work.token", "token-value\n");
        assert_eq!(file_token(path).await.unwrap(), "token-value");
    }

    fn fake_gh_script(dir: &std::path::Path, body: &str) -> PathBuf {
        let script = dir.join("gh");
        std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();
        script
    }

    #[tokio::test]
    async fn gh_source_reports_missing_binaries_empty_output_and_failures() {
        let dir = tempfile::tempdir().unwrap();
        let spec = CredentialSpec::GhOAuth {
            login: "octo-work".to_string(),
            host: "github.com".to_string(),
        };
        let source = GhOAuthCredential::new(false, None);

        let old = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", dir.path());
        assert!(matches!(
            source.token(&spec).await,
            Err(CredentialError::GhNotFound)
        ));

        fake_gh_script(dir.path(), "exit 0");
        assert!(matches!(
            source.token(&spec).await,
            Err(CredentialError::CommandEmpty)
        ));

        fake_gh_script(dir.path(), "printf '\\377'");
        assert!(matches!(
            source.token(&spec).await,
            Err(CredentialError::GhTokenFailed)
        ));

        fake_gh_script(dir.path(), "exit 1");
        assert!(matches!(
            source.token(&spec).await,
            Err(CredentialError::GhTokenFailed)
        ));
        std::env::set_var("PATH", old);
    }

    #[tokio::test]
    async fn file_source_rejects_missing_empty_and_insecure_files() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.token");
        assert!(matches!(
            file_token(missing).await,
            Err(CredentialError::FileMissing { .. })
        ));

        let empty = private_file(dir.path(), "empty.token", "");
        assert!(matches!(
            file_token(empty).await,
            Err(CredentialError::FileEmpty { .. })
        ));

        let insecure = private_file(dir.path(), "insecure.token", "token");
        let mut permissions = std::fs::metadata(&insecure).unwrap().permissions();
        permissions.set_mode(0o644);
        std::fs::set_permissions(&insecure, permissions).unwrap();
        assert!(matches!(
            file_token(insecure).await,
            Err(CredentialError::FileInsecure { .. })
        ));
    }
}
