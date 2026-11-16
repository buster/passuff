//! Shared configuration schema, validation and atomic editing for passuff.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml_edit::{Array, DocumentMut, Item, Value};

pub const DEFAULT_GRAPHQL_URL: &str = "https://api.github.com/graphql";
pub const DEFAULT_SOUND_FILE: &str = "default";

pub const DEFAULT_POLL_SECS: u64 = 60;
pub const DEFAULT_RECONCILIATION_SECS: u64 = 3600;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration file is missing: {path}")]
    Missing { path: PathBuf },
    #[error("cannot read configuration {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write configuration {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{0}")]
    Invalid(String),
}

impl ConfigError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchMode {
    #[default]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WatchConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<WatchMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_pull_requests: Vec<String>,
}

impl WatchConfig {
    /// A configuration without an explicit mode keeps the legacy meaning:
    /// an empty list watches all, a non-empty list watches the selected set.
    pub fn effective_mode(&self) -> WatchMode {
        self.mode.unwrap_or(if self.repositories.is_empty() {
            WatchMode::All
        } else {
            WatchMode::Selected
        })
    }

    pub fn includes_repository(&self, repository: &str) -> bool {
        match self.effective_mode() {
            WatchMode::All => true,
            WatchMode::Selected => self
                .repositories
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(repository)),
        }
    }

    pub fn is_empty_selected(&self) -> bool {
        self.effective_mode() == WatchMode::Selected && self.repositories.is_empty()
    }

    pub fn is_ignored(&self, key: &str) -> bool {
        self.ignored_pull_requests
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(key))
    }

    pub fn label(&self) -> String {
        match self.effective_mode() {
            WatchMode::All => "all authored pull requests".to_string(),
            WatchMode::Selected if self.repositories.is_empty() => "watching nothing".to_string(),
            WatchMode::Selected => self.repositories.join(", "),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum FocusRelation {
    Authored,
    Assigned,
    ReviewRequested,
    TeamReviewRequested,
}

impl FocusRelation {
    pub const ALL: [Self; 4] = [
        Self::Authored,
        Self::Assigned,
        Self::ReviewRequested,
        Self::TeamReviewRequested,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Authored => "authored",
            Self::Assigned => "assigned",
            Self::ReviewRequested => "review-requested",
            Self::TeamReviewRequested => "team-review-requested",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Authored => "authored",
            Self::Assigned => "assigned",
            Self::ReviewRequested => "review requested",
            Self::TeamReviewRequested => "team review",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "authored" => Some(Self::Authored),
            "assigned" => Some(Self::Assigned),
            "review-requested" | "review_requested" | "review requested" => {
                Some(Self::ReviewRequested)
            }
            "team-review-requested" | "team_review_requested" | "team review" => {
                Some(Self::TeamReviewRequested)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FocusConfig {
    #[serde(default = "all_focus_relations")]
    pub relations: Vec<FocusRelation>,
}

fn all_focus_relations() -> Vec<FocusRelation> {
    FocusRelation::ALL.to_vec()
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            relations: all_focus_relations(),
        }
    }
}

impl FocusConfig {
    /// An empty or absent relation set on a watched pull request is unknown and
    /// stays visible. An empty configured focus hides everything.
    pub fn allows(&self, relations: &[FocusRelation]) -> bool {
        if relations.is_empty() {
            return true;
        }
        relations
            .iter()
            .any(|relation| self.relations.contains(relation))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EventToggles {
    #[serde(default = "yes")]
    pub comments: bool,
    #[serde(default = "yes")]
    pub reviews: bool,
    #[serde(default = "yes")]
    pub ci_failure: bool,
    #[serde(default = "yes")]
    pub ci_recovery: bool,
    #[serde(default = "yes")]
    pub state_changes: bool,
    #[serde(default)]
    pub bot_comments: bool,
}

fn yes() -> bool {
    true
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    #[default]
    Normal,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NotificationsConfig {
    #[serde(default = "yes")]
    pub desktop: bool,
    #[serde(default = "yes")]
    pub sound: bool,
    #[serde(default = "default_sound_file")]
    pub sound_file: String,
    #[serde(default)]
    pub urgency: Urgency,
}

fn default_sound_file() -> String {
    DEFAULT_SOUND_FILE.to_string()
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            sound: true,
            sound_file: default_sound_file(),
            urgency: Urgency::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMode {
    #[serde(rename = "gh_oauth")]
    GhOAuth,
    #[serde(rename = "github_pat")]
    GithubPat,
}

impl AuthMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GhOAuth => "gh_oauth",
            Self::GithubPat => "github_pat",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AccountConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphql_url: Option<String>,
}

impl AccountConfig {
    pub fn auth_mode(&self) -> AuthMode {
        self.auth.unwrap_or(AuthMode::GithubPat)
    }

    pub fn expected_login(&self) -> Option<&str> {
        self.github_user.as_deref()
    }

    pub fn graphql_url(&self) -> &str {
        self.graphql_url.as_deref().unwrap_or(DEFAULT_GRAPHQL_URL)
    }

    pub fn credential_source_count(&self) -> usize {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "default_poll_secs")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_reconciliation_secs")]
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

fn default_poll_secs() -> u64 {
    DEFAULT_POLL_SECS
}

fn default_reconciliation_secs() -> u64 {
    DEFAULT_RECONCILIATION_SECS
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: DEFAULT_POLL_SECS,
            reconciliation_interval_secs: DEFAULT_RECONCILIATION_SECS,
            watch: WatchConfig::default(),
            focus: FocusConfig::default(),
            events: EventToggles::default(),
            notifications: NotificationsConfig::default(),
            accounts: BTreeMap::new(),
        }
    }
}

impl AppConfig {
    pub fn parse_str(text: &str) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(text).map_err(|error| {
            let message = error.to_string();
            if message.contains("unknown variant") && message.contains("gh_oauth") {
                ConfigError::invalid(format!("unknown auth mode: {message}"))
            } else {
                ConfigError::invalid(format!("invalid configuration: {message}"))
            }
        })?;
        validate(&config)?;
        Ok(config)
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ConfigError::Missing {
                    path: path.to_path_buf(),
                })
            }
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_path_buf(),
                    source,
                })
            }
        };
        Self::parse_str(&text)
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self)
            .map_err(|error| ConfigError::invalid(format!("cannot encode configuration: {error}")))
    }

    pub fn save_atomic(&self, path: &Path) -> Result<(), ConfigError> {
        let text = self.to_toml()?;
        write_atomic(path, text.as_bytes())
    }

    pub fn focus_allows(&self, relations: &[FocusRelation]) -> bool {
        self.focus.allows(relations)
    }

    pub fn watch_includes(&self, repository: &str) -> bool {
        self.watch.includes_repository(repository)
    }
}

pub fn validate(config: &AppConfig) -> Result<(), ConfigError> {
    if config.poll_interval_secs < 1 {
        return Err(ConfigError::invalid(
            "poll_interval_secs must be at least 1",
        ));
    }
    if config.reconciliation_interval_secs < 1 {
        return Err(ConfigError::invalid(
            "reconciliation_interval_secs must be at least 1",
        ));
    }
    if config.accounts.is_empty() {
        return Err(ConfigError::invalid("at least one account is required"));
    }
    for (label, account) in &config.accounts {
        validate_account(label, account)?;
    }

    let mut repositories = Vec::new();
    for repository in &config.watch.repositories {
        validate_repository(repository)?;
        let lowered = repository.to_ascii_lowercase();
        if repositories.contains(&lowered) {
            return Err(ConfigError::invalid(format!(
                "duplicate watch repository: {repository}"
            )));
        }
        repositories.push(lowered);
    }

    let mut ignored = Vec::new();
    for pull_request in &config.watch.ignored_pull_requests {
        validate_ignored_pull_request(pull_request)?;
        let lowered = pull_request.to_ascii_lowercase();
        if ignored.contains(&lowered) {
            return Err(ConfigError::invalid(format!(
                "duplicate ignored pull request: {pull_request}"
            )));
        }
        ignored.push(lowered);
    }

    if config.notifications.sound_file != DEFAULT_SOUND_FILE {
        let path = expand_home(&config.notifications.sound_file);
        if !path.is_file() {
            return Err(ConfigError::invalid(format!(
                "sound_file is not a readable file: {}",
                config.notifications.sound_file
            )));
        }
    }

    Ok(())
}

fn validate_account(label: &str, account: &AccountConfig) -> Result<(), ConfigError> {
    if label.trim().is_empty() {
        return Err(ConfigError::invalid("account label must not be empty"));
    }
    if account.token.is_some() {
        return Err(ConfigError::invalid(format!(
            "account {label}: must not contain a literal token"
        )));
    }
    if let Some(login) = account.github_user.as_deref() {
        if login.chars().any(char::is_whitespace) {
            return Err(ConfigError::invalid(format!(
                "account {label}: github_user must not contain whitespace"
            )));
        }
    }

    match account.auth_mode() {
        AuthMode::GhOAuth => {
            if account.github_user.is_none() {
                return Err(ConfigError::invalid(format!(
                    "account {label}: gh_oauth requires github_user"
                )));
            }
            if account.credential_source_count() > 0 {
                return Err(ConfigError::invalid(format!(
                    "account {label}: gh_oauth must not set a personal access token source"
                )));
            }
        }
        AuthMode::GithubPat => {
            if account.credential_source_count() != 1 {
                return Err(ConfigError::invalid(format!(
                    "account {label}: must select exactly one credential source"
                )));
            }
            if let Some(path) = account.token_file.as_deref() {
                validate_token_file(label, path)?;
            }
        }
    }

    let url = account.graphql_url();
    validate_graphql_url(label, url)?;
    Ok(())
}

fn validate_graphql_url(label: &str, url: &str) -> Result<(), ConfigError> {
    let lowered = url.to_ascii_lowercase();
    if lowered.starts_with("https://") {
        return Ok(());
    }
    if let Some(rest) = lowered.strip_prefix("http://") {
        let host = rest
            .split('/')
            .next()
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default();
        if host == "localhost" || host == "127.0.0.1" {
            return Ok(());
        }
    }
    Err(ConfigError::invalid(format!(
        "account {label}: graphql_url must use https, or http only for localhost"
    )))
}

pub fn validate_repository(repository: &str) -> Result<(), ConfigError> {
    if repository.chars().any(char::is_whitespace)
        || repository.matches('/').count() != 1
        || repository.starts_with('/')
        || repository.ends_with('/')
    {
        return Err(ConfigError::invalid(format!(
            "watch repository must be owner/repo: {repository}"
        )));
    }
    Ok(())
}

pub fn validate_ignored_pull_request(value: &str) -> Result<(), ConfigError> {
    let (repository, number) = value.rsplit_once('#').ok_or_else(|| {
        ConfigError::invalid(format!(
            "ignored pull request must be owner/repo#number: {value}"
        ))
    })?;
    validate_repository(repository)?;
    let number: u64 = number.parse().map_err(|_| {
        ConfigError::invalid(format!(
            "ignored pull request must be owner/repo#number: {value}"
        ))
    })?;
    if number == 0 {
        return Err(ConfigError::invalid(format!(
            "ignored pull request number must be positive: {value}"
        )));
    }
    Ok(())
}

fn validate_token_file(label: &str, path: &str) -> Result<(), ConfigError> {
    if !(path.starts_with('/') || path.starts_with('~')) {
        return Err(ConfigError::invalid(format!(
            "account {label}: token_file must be absolute or start with ~"
        )));
    }
    let expanded = expand_home(path);
    match fs::metadata(&expanded) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(ConfigError::invalid(format!(
                    "account {label}: token_file is not a regular file: {path}"
                )));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode() & 0o777;
                if mode & 0o077 != 0 {
                    return Err(ConfigError::invalid(format!(
                        "account {label}: token_file is not private (insecure mode {mode:o})"
                    )));
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {
            return Err(ConfigError::invalid(format!(
                "account {label}: token_file is not readable: {path}"
            )))
        }
    }
    Ok(())
}

pub fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    } else if path == "~" {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(path)
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    if std::env::var_os("PASSUFF_TEST_FAIL_CONFIG_WRITE").is_some() {
        return Err(ConfigError::Write {
            path: path.to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "test-injected write failure",
            ),
        });
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.toml");
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    fs::write(&temporary, bytes).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    fs::rename(&temporary, path).map_err(|source| {
        let _ = fs::remove_file(&temporary);
        ConfigError::Write {
            path: path.to_path_buf(),
            source,
        }
    })?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SetupAccount {
    pub label: String,
    pub auth: AuthMode,
    pub login: String,
    pub token_source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SetupAnswers {
    pub accounts: Vec<SetupAccount>,
    pub watch: WatchConfig,
}

/// Build and validate a configuration without reading or storing a token.
pub fn build_setup_config(answers: &SetupAnswers) -> Result<AppConfig, ConfigError> {
    let mut config = AppConfig {
        watch: answers.watch.clone(),
        ..AppConfig::default()
    };
    if config.watch.mode.is_none() {
        config.watch.mode = Some(if config.watch.repositories.is_empty() {
            WatchMode::All
        } else {
            WatchMode::Selected
        });
    }
    for account in &answers.accounts {
        let mut entry = AccountConfig {
            auth: Some(account.auth),
            github_user: Some(account.login.clone()),
            ..AccountConfig::default()
        };
        match account.auth {
            AuthMode::GhOAuth => {}
            AuthMode::GithubPat => {
                let source = account.token_source.as_deref().unwrap_or_default();
                match source.split_once(':') {
                    Some(("env", value)) => entry.token_env = Some(value.to_string()),
                    Some(("command", value)) => entry.token_command = Some(value.to_string()),
                    Some(("file", value)) => entry.token_file = Some(value.to_string()),
                    _ => {
                        return Err(ConfigError::invalid(format!(
                            "account {}: must select exactly one credential source",
                            account.label
                        )))
                    }
                }
            }
        }
        config.accounts.insert(account.label.clone(), entry);
    }
    validate(&config)?;
    Ok(config)
}

pub fn create_config(path: &Path, answers: &SetupAnswers) -> Result<AppConfig, ConfigError> {
    let config = build_setup_config(answers)?;
    config.save_atomic(path)?;
    Ok(config)
}

fn load_document(path: &Path) -> Result<DocumentMut, ConfigError> {
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    // Validate the document through the typed schema before editing it.
    AppConfig::parse_str(&text)?;
    text.parse::<DocumentMut>()
        .map_err(|error| ConfigError::invalid(format!("invalid configuration: {error}")))
}

fn watch_table(document: &mut DocumentMut) -> &mut toml_edit::Table {
    if !document.contains_key("watch") {
        document["watch"] = Item::Table(toml_edit::Table::new());
    }
    document["watch"].as_table_mut().expect("watch is a table")
}

fn focus_table(document: &mut DocumentMut) -> &mut toml_edit::Table {
    if !document.contains_key("focus") {
        document["focus"] = Item::Table(toml_edit::Table::new());
    }
    document["focus"].as_table_mut().expect("focus is a table")
}

pub struct ScopeEditor;

impl ScopeEditor {
    pub fn select_repository(path: &Path, repository: &str) -> Result<(), ConfigError> {
        validate_repository(repository)?;
        let mut document = load_document(path)?;
        let config = AppConfig::parse_str(&document.to_string())?;
        let mode = config.watch.effective_mode();
        let mut repositories = config.watch.repositories.clone();
        if !repositories
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(repository))
        {
            repositories.push(repository.to_string());
        }
        let mode = if mode == WatchMode::All {
            WatchMode::Selected
        } else {
            mode
        };
        write_watch(
            &mut document,
            mode,
            &repositories,
            &config.watch.ignored_pull_requests,
        )?;
        write_atomic(path, document.to_string().as_bytes())
    }

    pub fn deselect_repository(path: &Path, repository: &str) -> Result<(), ConfigError> {
        validate_repository(repository)?;
        let mut document = load_document(path)?;
        let config = AppConfig::parse_str(&document.to_string())?;
        let repositories: Vec<String> = config
            .watch
            .repositories
            .iter()
            .filter(|candidate| !candidate.eq_ignore_ascii_case(repository))
            .cloned()
            .collect();
        write_watch(
            &mut document,
            WatchMode::Selected,
            &repositories,
            &config.watch.ignored_pull_requests,
        )?;
        write_atomic(path, document.to_string().as_bytes())
    }

    pub fn set_mode(path: &Path, mode: WatchMode) -> Result<(), ConfigError> {
        let mut document = load_document(path)?;
        let config = AppConfig::parse_str(&document.to_string())?;
        write_watch(
            &mut document,
            mode,
            &config.watch.repositories,
            &config.watch.ignored_pull_requests,
        )?;
        write_atomic(path, document.to_string().as_bytes())
    }

    pub fn set_focus(path: &Path, relations: &[FocusRelation]) -> Result<(), ConfigError> {
        let mut document = load_document(path)?;
        let config = AppConfig::parse_str(&document.to_string())?;
        let table = focus_table(&mut document);
        let array: Array = relations
            .iter()
            .map(|relation| Value::from(relation.as_str()))
            .collect();
        table["relations"] = toml_edit::value(array);
        // Validate the edited document before persisting it.
        AppConfig::parse_str(&document.to_string())?;
        let _ = config;
        write_atomic(path, document.to_string().as_bytes())
    }

    pub fn set_ignored(path: &Path, ignored: &[String]) -> Result<(), ConfigError> {
        let mut document = load_document(path)?;
        let config = AppConfig::parse_str(&document.to_string())?;
        write_watch(
            &mut document,
            config.watch.effective_mode(),
            &config.watch.repositories,
            ignored,
        )?;
        write_atomic(path, document.to_string().as_bytes())
    }
}

fn write_watch(
    document: &mut DocumentMut,
    mode: WatchMode,
    repositories: &[String],
    ignored: &[String],
) -> Result<(), ConfigError> {
    let table = watch_table(document);
    table["mode"] = toml_edit::value(mode.as_str());
    let mut repo_array = Array::new();
    for repository in repositories {
        repo_array.push(repository.as_str());
    }
    table["repositories"] = toml_edit::value(repo_array);
    if ignored.is_empty() {
        table.remove("ignored_pull_requests");
    } else {
        let mut ignored_array = Array::new();
        for value in ignored {
            ignored_array.push(value.as_str());
        }
        table["ignored_pull_requests"] = toml_edit::value(ignored_array);
    }
    // The edited document must still satisfy the schema and the format rules.
    let config = AppConfig::parse_str(&document.to_string())?;
    let _ = config;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(label: &str) -> (String, AccountConfig) {
        (
            label.to_string(),
            AccountConfig {
                auth: Some(AuthMode::GhOAuth),
                github_user: Some("octo".to_string()),
                ..AccountConfig::default()
            },
        )
    }

    #[test]
    fn legacy_empty_scope_means_all() {
        let watch = WatchConfig::default();
        assert_eq!(watch.effective_mode(), WatchMode::All);
        assert!(watch.includes_repository("acme/api"));
    }

    #[test]
    fn explicit_empty_selected_scope_means_none() {
        let watch = WatchConfig {
            mode: Some(WatchMode::Selected),
            ..WatchConfig::default()
        };
        assert!(!watch.includes_repository("acme/api"));
        assert_eq!(watch.label(), "watching nothing");
    }

    #[test]
    fn rejects_two_credential_sources() {
        let mut config = AppConfig::default();
        let (label, mut entry) = account("work");
        entry.auth = Some(AuthMode::GithubPat);
        entry.token_env = Some("A".into());
        entry.token_file = Some("/tmp/token".into());
        config.accounts.insert(label, entry);
        let error = validate(&config).unwrap_err().to_string();
        assert!(error.contains("must select exactly one credential source"));
    }
}
