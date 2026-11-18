//! Configuration model, validation, and persistence for passuff.
//!
//! The configuration file is TOML. Token values are never part of the
//! configuration: an account names an environment variable, a helper command,
//! or a permission-checked token file.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_GRAPHQL_URL: &str = "https://api.github.com/graphql";

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("configuration file is missing: {0}")]
    Missing(PathBuf),
    #[error("cannot read configuration {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("{0}")]
    Invalid(String),
    #[error("cannot write configuration {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    GhOauth,
    GithubPat,
}

impl AuthMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "gh_oauth" => Some(Self::GhOauth),
            "github_pat" => Some(Self::GithubPat),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::GhOauth => "gh_oauth",
            Self::GithubPat => "github_pat",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    Authored,
    Assigned,
    ReviewRequested,
    TeamReviewRequested,
}

impl Relation {
    pub const ALL: [Relation; 4] = [
        Relation::Authored,
        Relation::Assigned,
        Relation::ReviewRequested,
        Relation::TeamReviewRequested,
    ];

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "authored" => Some(Self::Authored),
            "assigned" => Some(Self::Assigned),
            "review-requested" => Some(Self::ReviewRequested),
            "team-review-requested" => Some(Self::TeamReviewRequested),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Authored => "authored",
            Self::Assigned => "assigned",
            Self::ReviewRequested => "review-requested",
            Self::TeamReviewRequested => "team-review-requested",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchMode {
    All,
    Selected,
}

impl WatchMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Selected => "selected",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl Urgency {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::Critical => "critical",
        }
    }
}

fn default_poll_interval() -> u64 {
    60
}

fn default_reconciliation_interval() -> u64 {
    3600
}

fn default_true() -> bool {
    true
}

fn default_sound_file() -> String {
    "default".to_string()
}

fn default_urgency() -> Urgency {
    Urgency::Normal
}

fn default_relations() -> Vec<Relation> {
    Relation::ALL.to_vec()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_reconciliation_interval")]
    pub reconciliation_interval_secs: u64,
    #[serde(default)]
    pub watch: WatchConfig,
    #[serde(default)]
    pub focus: FocusConfig,
    #[serde(default)]
    pub events: EventToggles,
    #[serde(default)]
    pub notifications: NotificationsConfig,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval(),
            reconciliation_interval_secs: default_reconciliation_interval(),
            watch: WatchConfig::default(),
            focus: FocusConfig::default(),
            events: EventToggles::default(),
            notifications: NotificationsConfig::default(),
            accounts: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default)]
    pub mode: Option<WatchMode>,
    #[serde(default)]
    pub repositories: Vec<String>,
    #[serde(default)]
    pub ignored_pull_requests: Vec<String>,
}

impl WatchConfig {
    /// An absent mode keeps the historical meaning: a non-empty selected set
    /// means selected mode; otherwise every repository is watched.
    pub fn effective_mode(&self) -> WatchMode {
        self.mode.unwrap_or(if self.repositories.is_empty() {
            WatchMode::All
        } else {
            WatchMode::Selected
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusConfig {
    #[serde(default = "default_relations")]
    pub relations: Vec<Relation>,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            relations: default_relations(),
        }
    }
}

impl FocusConfig {
    pub fn contains(&self, relation: Relation) -> bool {
        self.relations.contains(&relation)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventToggles {
    #[serde(default = "default_true")]
    pub comments: bool,
    #[serde(default = "default_true")]
    pub reviews: bool,
    #[serde(default = "default_true")]
    pub ci_failure: bool,
    #[serde(default = "default_true")]
    pub ci_recovery: bool,
    #[serde(default = "default_true")]
    pub state_changes: bool,
    #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    #[serde(default = "default_true")]
    pub desktop: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
    #[serde(default = "default_sound_file")]
    pub sound_file: String,
    #[serde(default = "default_urgency")]
    pub urgency: Urgency,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            sound: true,
            sound_file: default_sound_file(),
            urgency: default_urgency(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountConfig {
    #[serde(default)]
    pub auth: Option<String>,
    #[serde(default)]
    pub github_user: Option<String>,
    #[serde(default)]
    pub token_env: Option<String>,
    #[serde(default)]
    pub token_command: Option<String>,
    #[serde(default)]
    pub token_file: Option<String>,
    #[serde(default)]
    pub graphql_url: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

impl AccountConfig {
    pub fn auth_mode(&self) -> AuthMode {
        match self.auth.as_deref() {
            None | Some("github_pat") => AuthMode::GithubPat,
            Some("gh_oauth") => AuthMode::GhOauth,
            Some(_) => AuthMode::GithubPat,
        }
    }

    pub fn graphql_url(&self) -> &str {
        self.graphql_url.as_deref().unwrap_or(DEFAULT_GRAPHQL_URL)
    }

    pub fn credential_sources(&self) -> usize {
        [
            self.token_env.is_some(),
            self.token_command.is_some(),
            self.token_file.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count()
    }
}

pub fn config_path() -> PathBuf {
    base_config_dir()
        .map(|dir| dir.join("passuff").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("passuff.toml"))
}

pub fn state_path() -> PathBuf {
    base_state_dir()
        .map(|dir| dir.join("passuff").join("state.json"))
        .unwrap_or_else(|| PathBuf::from("passuff-state.json"))
}

fn base_config_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| dirs.config_dir().to_path_buf())
}

fn base_state_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| {
        dirs.state_dir()
            .unwrap_or_else(|| dirs.data_dir())
            .to_path_buf()
    })
}

pub fn expand_home(value: &str) -> PathBuf {
    if value == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(value));
    }
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(value)
}

pub fn load(path: &Path) -> Result<AppConfig, LoadError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(LoadError::Missing(path.to_path_buf()));
        }
        Err(source) => {
            return Err(LoadError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let mut config: AppConfig = toml::from_str(&text)
        .map_err(|error| LoadError::Invalid(format!("cannot parse configuration: {error}")))?;
    if let Ok(url) = std::env::var("PASSUFF_GRAPHQL_URL") {
        if !url.is_empty() {
            for account in config.accounts.values_mut() {
                account.graphql_url = Some(url.clone());
            }
        }
    }
    validate(&config).map_err(LoadError::Invalid)?;
    Ok(config)
}

pub fn validate(config: &AppConfig) -> Result<(), String> {
    if config.poll_interval_secs < 1 {
        return Err("poll_interval_secs must be at least 1".to_string());
    }
    if config.reconciliation_interval_secs < 1 {
        return Err("reconciliation_interval_secs must be at least 1".to_string());
    }

    let mut seen_repositories = BTreeMap::new();
    for repository in &config.watch.repositories {
        validate_repository(repository)?;
        let key = repository.to_ascii_lowercase();
        if seen_repositories.insert(key, repository).is_some() {
            return Err(format!("duplicate watch repository: {repository}"));
        }
    }

    let mut seen_ignored = BTreeMap::new();
    for ignored in &config.watch.ignored_pull_requests {
        validate_ignored_pull_request(ignored)?;
        let key = ignored.to_ascii_lowercase();
        if seen_ignored.insert(key, ignored).is_some() {
            return Err(format!("duplicate ignored pull request: {ignored}"));
        }
    }

    if config.accounts.is_empty() {
        return Err("configuration has no accounts".to_string());
    }

    for (label, account) in &config.accounts {
        if label.is_empty() {
            return Err("account label must not be empty".to_string());
        }
        if account.extra.contains_key("token") {
            return Err(format!("account {label} must not contain a literal token"));
        }
        match account.auth.as_deref() {
            None | Some("github_pat") | Some("gh_oauth") => {}
            Some(other) => return Err(format!("unknown auth mode: {other}")),
        }
        if let Some(user) = &account.github_user {
            if user.is_empty() {
                return Err(format!("account {label} github_user must not be empty"));
            }
            if user.chars().any(char::is_whitespace) {
                return Err(format!(
                    "account {label} github_user must not contain whitespace"
                ));
            }
        }

        match account.auth_mode() {
            AuthMode::GhOauth => {
                if account.github_user.is_none() {
                    return Err(format!("account {label} gh_oauth requires github_user"));
                }
                if account.credential_sources() > 0 {
                    return Err(format!(
                        "account {label} gh_oauth must not set a personal access token source"
                    ));
                }
            }
            AuthMode::GithubPat => {
                if account.credential_sources() != 1 {
                    return Err(format!(
                        "account {label} must select exactly one credential source"
                    ));
                }
            }
        }

        if let Some(file) = &account.token_file {
            validate_token_file(file, label)?;
        }

        let url = account.graphql_url();
        if !is_valid_graphql_url(url) {
            return Err(format!(
                "account {label} graphql_url must use https, or http on localhost: {url}"
            ));
        }
    }

    if config.notifications.sound_file != "default" {
        let path = expand_home(&config.notifications.sound_file);
        if !path.is_file() {
            return Err(format!(
                "sound_file is not a readable file: {}",
                config.notifications.sound_file
            ));
        }
    }

    Ok(())
}

pub fn validate_repository(repository: &str) -> Result<(), String> {
    if repository.matches('/').count() != 1
        || repository.chars().any(char::is_whitespace)
        || repository.starts_with('/')
        || repository.ends_with('/')
        || repository.split('/').any(str::is_empty)
    {
        return Err(format!("repository must be owner/repo: {repository}"));
    }
    Ok(())
}

pub fn validate_ignored_pull_request(value: &str) -> Result<(), String> {
    let (repository, number) = value
        .split_once('#')
        .ok_or_else(|| format!("ignored pull request must be owner/repo#number: {value}"))?;
    validate_repository(repository)?;
    let number: u64 = number
        .parse()
        .map_err(|_| format!("ignored pull request must be owner/repo#number: {value}"))?;
    if number == 0 {
        return Err(format!(
            "ignored pull request must use a positive number: {value}"
        ));
    }
    Ok(())
}

fn validate_token_file(value: &str, label: &str) -> Result<(), String> {
    if !(Path::new(value).is_absolute() || value.starts_with('~')) {
        return Err(format!(
            "account {label} token_file must be absolute or start with ~: {value}"
        ));
    }
    let path = expand_home(value);
    if !path.exists() {
        // A missing token file is a runtime account failure, not a
        // configuration error.
        return Ok(());
    }
    let metadata = fs::metadata(&path)
        .map_err(|error| format!("account {label} token_file cannot be read: {value}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!(
            "account {label} token_file is not a regular file: {value}"
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o777;
        if mode != 0o600 && mode != 0o400 {
            return Err(format!(
                "account {label} token_file is not private (insecure permissions {mode:04o}): {value}"
            ));
        }
    }
    Ok(())
}

pub fn is_valid_graphql_url(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if rest.is_empty() {
        return false;
    }
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .rsplit('@')
        .next()
        .unwrap_or_default();
    let host = if let Some(stripped) = authority.strip_prefix('[') {
        stripped.split(']').next().unwrap_or_default().to_string()
    } else {
        authority.split(':').next().unwrap_or_default().to_string()
    };
    match scheme.to_ascii_lowercase().as_str() {
        "https" => !host.is_empty(),
        "http" => matches!(
            host.to_ascii_lowercase().as_str(),
            "localhost" | "127.0.0.1" | "::1"
        ),
        _ => false,
    }
}

pub fn save(path: &Path, config: &AppConfig) -> Result<(), SaveError> {
    validate(config).map_err(SaveError::Invalid)?;
    let text = toml::to_string_pretty(config)
        .map_err(|error| SaveError::Invalid(format!("cannot serialize configuration: {error}")))?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| SaveError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    let temporary = temporary_path(path);
    if let Err(source) = fs::write(&temporary, text) {
        let _ = fs::remove_file(&temporary);
        return Err(SaveError::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    if let Err(source) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(SaveError::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "passuff".to_string());
    path.with_file_name(format!(".{name}.tmp-{}", std::process::id()))
}
