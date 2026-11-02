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
use thiserror::Error;
use url::Url;

pub const DEFAULT_GRAPHQL_URL: &str = "https://api.github.com/graphql";
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub poll_interval_secs: u64,
    pub core: CoreConfig,
    pub notifications: NotificationsConfig,
}

#[derive(Debug, Clone)]
pub struct CoreConfig {
    pub accounts: BTreeMap<String, AccountConfig>,
    pub events: EventToggles,
}

#[derive(Debug, Clone)]
pub struct AccountConfig {
    pub credential: CredentialSpec,
    pub graphql_url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSpec {
    Env { variable: String },
    Command { command: String },
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
    accounts: BTreeMap<String, AccountToml>,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            events: EventToggles::default(),
            notifications: NotificationsConfig::default(),
            accounts: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountToml {
    token_env: Option<String>,
    token_command: Option<String>,
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

        let mut accounts = BTreeMap::new();
        for (label, account) in self.accounts {
            if label.trim().is_empty() {
                return Err(ConfigError::Validation(
                    "account label must not be empty".to_string(),
                ));
            }
            let credential = match (account.token_env, account.token_command) {
                (Some(variable), None) if !variable.trim().is_empty() => {
                    CredentialSpec::Env { variable }
                }
                (None, Some(command)) if !command.trim().is_empty() => {
                    CredentialSpec::Command { command }
                }
                (Some(_), Some(_)) => {
                    return Err(ConfigError::Validation(format!(
                        "account '{label}' sets both token_env and token_command"
                    )));
                }
                _ => {
                    return Err(ConfigError::Validation(format!(
                        "account '{label}' must set exactly one of token_env or token_command"
                    )));
                }
            };
            let graphql_url = account
                .graphql_url
                .as_deref()
                .unwrap_or(DEFAULT_GRAPHQL_URL);
            let graphql_url = validate_graphql_url(label.as_str(), graphql_url)?;
            accounts.insert(
                label,
                AccountConfig {
                    credential,
                    graphql_url,
                },
            );
        }

        Ok(AppConfig {
            poll_interval_secs: self.poll_interval_secs,
            core: CoreConfig {
                accounts,
                events: self.events,
            },
            notifications: self.notifications,
        })
    }
}

fn validate_graphql_url(label: &str, raw: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(raw).map_err(|error| {
        ConfigError::Validation(format!("account '{label}' has an invalid graphql_url: {error}"))
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
        assert_eq!(config.core.accounts["personal"].graphql_url.port(), Some(1234));
    }
}
