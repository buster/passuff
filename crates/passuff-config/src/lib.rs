//! Shared configuration schema for passuff.
//!
//! The crate owns the TOML document shape. It never executes a credential
//! command; `passuff-core` resolves the credential at observation time.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use url::Url;

pub const DEFAULT_GRAPHQL_URL: &str = "https://api.github.com/graphql";
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub poll_interval_secs: u64,
    pub core: CoreConfig,
    pub notifications: NotificationsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreConfig {
    pub accounts: BTreeMap<String, AccountConfig>,
    pub events: EventToggles,
    pub watch: WatchConfig,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchMode {
    #[default]
    All,
    Selected,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WatchConfig {
    pub mode: WatchMode,
    pub repositories: Vec<String>,
}

impl WatchConfig {
    pub fn is_all(&self) -> bool {
        self.mode == WatchMode::All
    }

    pub fn allows_repository(&self, repository: &str) -> bool {
        self.is_all()
            || self
                .repositories
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(repository))
    }

    pub fn label(&self) -> String {
        match self.mode {
            WatchMode::All => "all authored pull requests".to_string(),
            WatchMode::Selected if self.repositories.is_empty() => "watching nothing".to_string(),
            WatchMode::Selected => self.repositories.join(", "),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileWatchConfig {
    mode: Option<WatchMode>,
    repositories: Vec<String>,
}

impl FileWatchConfig {
    fn resolve(self) -> WatchConfig {
        let mode = self.mode.unwrap_or_else(|| {
            if self.repositories.is_empty() {
                WatchMode::All
            } else {
                WatchMode::Selected
            }
        });
        WatchConfig {
            mode,
            repositories: self.repositories,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountConfig {
    pub credential: CredentialSpec,
    pub graphql_url: Url,
    pub expected_login: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSpec {
    Env { variable: String },
    Command { command: String },
    File { path: PathBuf },
    GhOAuth { login: String, host: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EventToggles {
    pub comments: bool,
    pub reviews: bool,
    pub ci_failure: bool,
    pub ci_recovery: bool,
    pub state_changes: bool,
    pub bot_comments: bool,
}

impl Default for EventToggles {
    fn default() -> Self {
        Self {
            comments: true,
            reviews: true,
            ci_failure: true,
            ci_recovery: true,
            state_changes: true,
            bot_comments: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationsConfig {
    pub desktop: bool,
    pub sound: bool,
    pub sound_file: SoundFile,
    pub urgency: Urgency,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            sound: true,
            sound_file: SoundFile::Default,
            urgency: Urgency::Normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundFile {
    Default,
    Path(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    poll_interval_secs: u64,
    events: EventToggles,
    notifications: NotificationsConfig,
    watch: FileWatchConfig,
    accounts: BTreeMap<String, AccountToml>,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            events: EventToggles::default(),
            notifications: NotificationsConfig::default(),
            watch: FileWatchConfig::default(),
            accounts: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthMode {
    GhOAuth,
    Pat,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountToml {
    auth: Option<String>,
    github_user: Option<String>,
    token_env: Option<String>,
    token_command: Option<String>,
    token_file: Option<String>,
    token: Option<String>,
    graphql_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("read configuration {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parse configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid configuration: {0}")]
    Validation(String),
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load(path: impl AsRef<Path>) -> Result<AppConfig, ConfigError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<AppConfig, ConfigError> {
        let file: FileConfig = toml::from_str(text)?;
        file.into_app_config()
    }
}

impl FileConfig {
    fn into_app_config(self) -> Result<AppConfig, ConfigError> {
        if self.accounts.is_empty() {
            return Err(ConfigError::Validation(
                "at least one [accounts.<label>] entry is required".to_string(),
            ));
        }
        if self.poll_interval_secs < 1 {
            return Err(ConfigError::Validation(
                "poll_interval_secs must be at least 1".to_string(),
            ));
        }
        if let SoundFile::Path(path) = &self.notifications.sound_file {
            if !path.is_file() {
                return Err(ConfigError::Validation(format!(
                    "sound_file is not a readable file: {}",
                    path.display()
                )));
            }
        }

        validate_watch_scope(&self.watch.repositories)?;
        let mut accounts = BTreeMap::new();
        for (label, account) in self.accounts {
            if label.trim().is_empty() {
                return Err(ConfigError::Validation(
                    "account label must not be empty".to_string(),
                ));
            }
            if account.token.is_some() {
                return Err(ConfigError::Validation(format!(
                    "account '{label}' must not contain a literal token"
                )));
            }
            let github_user = account
                .github_user
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            if let Some(user) = github_user.as_deref() {
                if user.chars().any(char::is_whitespace) {
                    return Err(ConfigError::Validation(format!(
                        "account '{label}' github_user must be a login"
                    )));
                }
            }
            let source_count = [
                account.token_env.as_deref(),
                account.token_command.as_deref(),
                account.token_file.as_deref(),
            ]
            .into_iter()
            .filter(|value| value.is_some_and(|value| !value.trim().is_empty()))
            .count();
            let mode = match account.auth.as_deref().map(str::trim) {
                None | Some("github_pat") => AuthMode::Pat,
                Some("gh_oauth") => AuthMode::GhOAuth,
                Some(other) => {
                    return Err(ConfigError::Validation(format!(
                        "account '{label}' has unknown auth mode '{other}'"
                    )));
                }
            };
            let graphql_url = account
                .graphql_url
                .as_deref()
                .unwrap_or(DEFAULT_GRAPHQL_URL);
            let graphql_url = validate_graphql_url(label.as_str(), graphql_url)?;
            let (credential, expected_login) = match mode {
                AuthMode::GhOAuth => {
                    let Some(login) = github_user.clone() else {
                        return Err(ConfigError::Validation(format!(
                            "account '{label}' requires github_user when auth is gh_oauth"
                        )));
                    };
                    if source_count > 0 {
                        return Err(ConfigError::Validation(format!(
                            "account '{label}' must not set a personal access token source when auth is gh_oauth"
                        )));
                    }
                    (
                        CredentialSpec::GhOAuth {
                            login: login.clone(),
                            host: gh_host(&graphql_url),
                        },
                        Some(login),
                    )
                }
                AuthMode::Pat => {
                    if source_count != 1 {
                        return Err(ConfigError::Validation(format!(
                            "account '{label}' must select exactly one credential source"
                        )));
                    }
                    let credential = if let Some(variable) =
                        account.token_env.filter(|value| !value.trim().is_empty())
                    {
                        CredentialSpec::Env { variable }
                    } else if let Some(command) = account
                        .token_command
                        .filter(|value| !value.trim().is_empty())
                    {
                        CredentialSpec::Command { command }
                    } else {
                        let raw = account
                            .token_file
                            .filter(|value| !value.trim().is_empty())
                            .expect("one source counted");
                        CredentialSpec::File {
                            path: resolve_token_file(&label, &raw)?,
                        }
                    };
                    (credential, github_user)
                }
            };
            accounts.insert(
                label,
                AccountConfig {
                    credential,
                    graphql_url,
                    expected_login,
                },
            );
        }

        Ok(AppConfig {
            poll_interval_secs: self.poll_interval_secs,
            core: CoreConfig {
                accounts,
                events: self.events,
                watch: self.watch.resolve(),
            },
            notifications: self.notifications,
        })
    }
}

fn validate_watch_scope(repositories: &[String]) -> Result<(), ConfigError> {
    let mut seen: Vec<String> = Vec::new();
    for repository in repositories {
        let repository = repository.trim();
        let valid = !repository.is_empty()
            && !repository.chars().any(char::is_whitespace)
            && repository.matches('/').count() == 1
            && !repository.starts_with('/')
            && !repository.ends_with('/');
        if !valid {
            return Err(ConfigError::Validation(format!(
                "watch.repositories entry '{repository}' must be owner/repo"
            )));
        }
        if seen
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(repository))
        {
            return Err(ConfigError::Validation(format!(
                "watch.repositories contains duplicate '{repository}'"
            )));
        }
        seen.push(repository.to_string());
    }
    Ok(())
}

fn gh_host(url: &Url) -> String {
    match url.host_str() {
        Some("api.github.com") | None => "github.com".to_string(),
        Some(host) => host.to_string(),
    }
}

fn resolve_token_file(label: &str, raw: &str) -> Result<PathBuf, ConfigError> {
    let raw = raw.trim();
    let path = if raw == "~" || raw.starts_with("~/") {
        let home = std::env::var_os("HOME").ok_or_else(|| {
            ConfigError::Validation(format!(
                "account '{label}' token_file uses ~ but HOME is not set"
            ))
        })?;
        let mut path = PathBuf::from(home);
        if raw.len() > 2 {
            path.push(&raw[2..]);
        }
        path
    } else if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        return Err(ConfigError::Validation(format!(
            "account '{label}' token_file must be absolute or start with ~"
        )));
    };
    if path.exists() {
        let metadata = fs::metadata(&path).map_err(|error| {
            ConfigError::Validation(format!(
                "token_file cannot be inspected: {}: {error}",
                path.display()
            ))
        })?;
        if !metadata.is_file() {
            return Err(ConfigError::Validation(format!(
                "token_file is not a regular file: {}",
                path.display()
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            if mode & 0o077 != 0 || mode & 0o400 == 0 {
                return Err(ConfigError::Validation(format!(
                    "token_file is not private: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(path)
}

fn validate_graphql_url(label: &str, raw: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(raw).map_err(|error| {
        ConfigError::Validation(format!(
            "account '{label}' has an invalid graphql_url: {error}"
        ))
    })?;
    match url.scheme() {
        "https" => Ok(url),
        "http" if matches!(url.host_str(), Some("127.0.0.1" | "localhost")) => Ok(url),
        "http" => Err(ConfigError::Validation(format!(
            "account '{label}' uses plain HTTP for a non-loopback graphql_url"
        ))),
        other => Err(ConfigError::Validation(format!(
            "account '{label}' uses unsupported graphql_url scheme '{other}'"
        ))),
    }
}

impl<'de> Deserialize<'de> for SoundFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SoundFileVisitor;
        impl<'de> Visitor<'de> for SoundFileVisitor {
            type Value = SoundFile;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("\"default\" or a sound file path")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value.eq_ignore_ascii_case("default") {
                    Ok(SoundFile::Default)
                } else if value.trim().is_empty() {
                    Err(E::custom("sound_file must not be empty"))
                } else {
                    Ok(SoundFile::Path(PathBuf::from(value)))
                }
            }
        }
        deserializer.deserialize_str(SoundFileVisitor)
    }
}

impl<'de> Deserialize<'de> for Urgency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UrgencyVisitor;
        impl<'de> Visitor<'de> for UrgencyVisitor {
            type Value = Urgency;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("\"low\", \"normal\", or \"critical\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.to_ascii_lowercase().as_str() {
                    "low" => Ok(Urgency::Low),
                    "normal" => Ok(Urgency::Normal),
                    "critical" => Ok(Urgency::Critical),
                    other => Err(E::custom(format!("unknown urgency '{other}'"))),
                }
            }
        }
        deserializer.deserialize_str(UrgencyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<AppConfig, ConfigError> {
        ConfigLoader::parse(text)
    }

    #[test]
    fn parses_minimal_config_with_defaults() {
        let config = parse(
            r#"
            [accounts.personal]
            token_env = "PERSONAL_TOKEN"
            "#,
        )
        .unwrap();
        assert_eq!(config.poll_interval_secs, 60);
        assert_eq!(config.core.accounts.len(), 1);
        assert!(config.notifications.desktop);
        assert!(config.notifications.sound);
        assert_eq!(config.notifications.sound_file, SoundFile::Default);
        assert_eq!(config.notifications.urgency, Urgency::Normal);
    }

    #[test]
    fn rejects_unknown_keys() {
        let error = parse(
            r#"
            [accounts.personal]
            token_env = "PERSONAL_TOKEN"
            mystery = true
            "#,
        )
        .unwrap_err();
        assert!(matches!(error, ConfigError::Parse(_)));
    }

    #[test]
    fn rejects_missing_and_duplicate_credentials() {
        assert!(parse("[accounts.personal]\n").is_err());
        assert!(parse(
            r#"
            [accounts.personal]
            token_env = "A"
            token_command = "echo B"
            "#,
        )
        .is_err());
    }

    #[test]
    fn accepts_a_token_command_credential() {
        let config = parse(
            r#"
            [accounts.personal]
            token_command = "gh auth token"
            "#,
        )
        .unwrap();
        assert_eq!(
            config.core.accounts["personal"].credential,
            CredentialSpec::Command {
                command: "gh auth token".to_string()
            }
        );
    }

    #[test]
    fn rejects_empty_credentials_and_labels() {
        assert!(parse("[accounts.personal]\ntoken_env = \"  \"\n").is_err());
        assert!(parse("[accounts.\"\"]\ntoken_env = \"A\"\n").is_err());
        assert!(parse("poll_interval_secs = 0\n[accounts.personal]\ntoken_env = \"A\"\n").is_err());
    }

    #[test]
    fn rejects_a_missing_sound_file() {
        let error = parse(
            r#"
            [accounts.personal]
            token_env = "A"

            [notifications]
            sound_file = "/does/not/exist.wav"
            "#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("sound_file"));
    }

    #[test]
    fn parses_sound_paths_and_urgency_levels() {
        let config = parse(
            r#"
            [accounts.personal]
            token_env = "A"

            [notifications]
            sound_file = "../passuff-cli/assets/notification.wav"
            urgency = "critical"
            "#,
        )
        .unwrap();
        assert!(matches!(
            config.notifications.sound_file,
            SoundFile::Path(_)
        ));
        assert_eq!(config.notifications.urgency, Urgency::Critical);

        let config = parse(
            r#"
            [accounts.personal]
            token_env = "A"

            [notifications]
            urgency = "low"
            "#,
        )
        .unwrap();
        assert_eq!(config.notifications.urgency, Urgency::Low);
    }

    #[test]
    fn rejects_unknown_urgency_and_empty_sound_file() {
        assert!(parse(
            r#"
            [accounts.personal]
            token_env = "A"
            [notifications]
            urgency = "urgent"
            "#,
        )
        .is_err());
        assert!(parse(
            r#"
            [accounts.personal]
            token_env = "A"
            [notifications]
            sound_file = ""
            "#,
        )
        .is_err());
    }

    #[test]
    fn parses_gh_oauth_with_expected_login_and_host() {
        let config = parse(
            r#"
            [accounts.work]
            auth = "gh_oauth"
            github_user = "octo-work"
            "#,
        )
        .unwrap();
        let account = &config.core.accounts["work"];
        assert_eq!(account.expected_login.as_deref(), Some("octo-work"));
        assert!(matches!(
            &account.credential,
            CredentialSpec::GhOAuth { login, host } if login == "octo-work" && host == "github.com"
        ));

        let config = parse(
            r#"
            [accounts.work]
            auth = "gh_oauth"
            github_user = "octo-work"
            graphql_url = "https://ghe.example.com/api/graphql"
            "#,
        )
        .unwrap();
        assert!(matches!(
            &config.core.accounts["work"].credential,
            CredentialSpec::GhOAuth { host, .. } if host == "ghe.example.com"
        ));
    }

    #[test]
    fn rejects_missing_gh_login_and_literal_tokens() {
        let error = parse(
            r#"
            [accounts.work]
            auth = "gh_oauth"
            "#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("requires github_user"));

        let error = parse(
            r#"
            [accounts.work]
            token = "ghp_secret"
            "#,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("must not contain a literal token"));
    }

    #[test]
    fn rejects_a_tilde_token_file_without_home() {
        let saved = std::env::var_os("HOME");
        std::env::remove_var("HOME");
        let result = parse("[accounts.work]\ntoken_file = \"~/work.token\"\n");
        if let Some(home) = saved {
            std::env::set_var("HOME", home);
        }
        let error = result.unwrap_err();
        assert!(error.to_string().contains("HOME is not set"));
    }

    #[test]
    fn rejects_a_github_user_with_whitespace() {
        let error = parse(
            r#"
            [accounts.work]
            auth = "gh_oauth"
            github_user = "octo work"
            "#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("must be a login"));
    }

    #[test]
    fn rejects_relative_and_insecure_token_files() {
        assert!(parse("[accounts.work]\ntoken_file = \"relative.token\"\n").is_err());

        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("group.token");
        fs::write(&file, "token").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&file).unwrap().permissions();
            permissions.set_mode(0o640);
            fs::set_permissions(&file, permissions).unwrap();
        }
        let text = format!("[accounts.work]\ntoken_file = \"{}\"\n", file.display());
        let error = parse(&text).unwrap_err();
        assert!(error.to_string().contains("token_file is not private"));

        let directory = dir.path().join("token-dir");
        fs::create_dir(&directory).unwrap();
        let text = format!(
            "[accounts.work]\ntoken_file = \"{}\"\n",
            directory.display()
        );
        let error = parse(&text).unwrap_err();
        assert!(error.to_string().contains("not a regular file"));
    }

    #[test]
    fn parses_and_validates_the_watch_scope() {
        let config = parse(
            r#"
            [watch]
            repositories = ["Acme/API", "acme/web"]

            [accounts.personal]
            token_env = "A"
            "#,
        )
        .unwrap();
        assert_eq!(config.core.watch.mode, WatchMode::Selected);
        assert_eq!(config.core.watch.label(), "Acme/API, acme/web");
        assert!(config.core.watch.allows_repository("acme/api"));
        assert!(config.core.watch.allows_repository("ACME/WEB"));
        assert!(!config.core.watch.allows_repository("acme/other"));

        let empty = parse("[accounts.personal]\ntoken_env = \"A\"\n").unwrap();
        assert!(empty.core.watch.is_all());
        assert_eq!(empty.core.watch.label(), "all authored pull requests");
        assert!(empty.core.watch.allows_repository("anything/here"));

        let none = parse("[watch]\nmode = \"selected\"\n[accounts.personal]\ntoken_env = \"A\"\n")
            .unwrap();
        assert_eq!(none.core.watch.mode, WatchMode::Selected);
        assert!(!none.core.watch.allows_repository("anything/here"));
        assert_eq!(none.core.watch.label(), "watching nothing");

        let explicit_all = parse(
            "[watch]\nmode = \"all\"\nrepositories = [\"acme/api\"]\n[accounts.personal]\ntoken_env = \"A\"\n",
        )
        .unwrap();
        assert_eq!(explicit_all.core.watch.mode, WatchMode::All);
        assert!(explicit_all.core.watch.allows_repository("acme/web"));

        assert!(parse(
            "[watch]\nrepositories = [\"bad\"]\n[accounts.personal]\ntoken_env = \"A\"\n"
        )
        .is_err());
        assert!(parse(
            "[watch]\nrepositories = [\"acme/api\", \"ACME/API\"]\n[accounts.personal]\ntoken_env = \"A\"\n"
        )
        .is_err());
        assert!(parse(
            "[watch]\nrepositories = [\"acme/api/web\"]\n[accounts.personal]\ntoken_env = \"A\"\n"
        )
        .is_err());
    }

    #[test]
    fn rejects_remote_plain_http() {
        let error = parse(
            r#"
            [accounts.personal]
            token_env = "A"
            graphql_url = "http://example.com/graphql"
            "#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("plain HTTP"));
    }

    #[test]
    fn allows_loopback_http_for_tests() {
        let config = parse(
            r#"
            [accounts.personal]
            token_env = "A"
            graphql_url = "http://127.0.0.1:1234/graphql"
            "#,
        )
        .unwrap();
        assert_eq!(
            config.core.accounts["personal"].graphql_url.port(),
            Some(1234)
        );
    }
}
