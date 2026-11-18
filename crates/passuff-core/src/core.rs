//! Check-cycle orchestration: account resolution, observation, classification,
//! and state updates. Delivery is deliberately outside this crate.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use chrono::Utc;
use passuff_config::{AccountConfig, AppConfig, AuthMode, Relation};

use crate::classifier::{self, ClassifyContext};
use crate::github::{DiscoveryObservation, GithubClient, PullRequestObservation, RateLimit};
use crate::model::{AccountFailure, Notification, PullRequestKey, RefreshReport, WatcherState};
use crate::view::{ignored_keys, scope_allows};

#[derive(Debug, Clone)]
struct Credential {
    token: String,
    login: String,
}

#[derive(Debug, Default)]
pub struct ProbeReport {
    pub failures: Vec<AccountFailure>,
    pub changed: Vec<(String, Vec<String>)>,
    pub logins: BTreeSet<String>,
    pub rate_limits: Vec<RateLimit>,
}

pub struct PassuffCore {
    client: GithubClient,
    gh_login_attempted: BTreeSet<String>,
    credentials: BTreeMap<String, Credential>,
    probe_fingerprints: BTreeMap<String, BTreeMap<String, String>>,
}

impl Default for PassuffCore {
    fn default() -> Self {
        Self::new()
    }
}

impl PassuffCore {
    pub fn new() -> Self {
        Self {
            client: GithubClient::new(),
            gh_login_attempted: BTreeSet::new(),
            credentials: BTreeMap::new(),
            probe_fingerprints: BTreeMap::new(),
        }
    }

    pub fn refresh_targeted(
        &mut self,
        config: &AppConfig,
        state: &mut WatcherState,
        interactive: bool,
    ) -> RefreshReport {
        self.refresh(config, state, interactive)
    }

    pub fn refresh_reconciliation(
        &mut self,
        config: &AppConfig,
        state: &mut WatcherState,
        interactive: bool,
    ) -> RefreshReport {
        self.refresh(config, state, interactive)
    }

    fn refresh(
        &mut self,
        config: &AppConfig,
        state: &mut WatcherState,
        interactive: bool,
    ) -> RefreshReport {
        let mut report = RefreshReport::default();
        let mut resolved = Vec::new();

        for (label, account) in &config.accounts {
            match self.resolve_and_verify(label, account, interactive) {
                Ok(credential) => resolved.push((label.clone(), account.clone(), credential)),
                Err(message) => report.failures.push(AccountFailure {
                    label: label.clone(),
                    message,
                }),
            }
        }

        let logins: BTreeSet<String> = resolved
            .iter()
            .map(|(_, _, credential)| credential.login.to_ascii_lowercase())
            .collect();
        let ignored = ignored_keys(config);
        let catch_up: BTreeSet<String> = state
            .catch_up
            .iter()
            .map(|repository| repository.to_ascii_lowercase())
            .collect();
        let now = Utc::now();
        let mut observed_repositories: BTreeSet<String> = BTreeSet::new();

        for (label, account, credential) in resolved {
            let url = account.graphql_url().to_string();
            let authored = match self.client.authored(&url, &credential.token) {
                Ok((nodes, _)) => nodes,
                Err(error) => {
                    report.failures.push(AccountFailure {
                        label,
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let discovery = match self.client.discovery(&url, &credential.token) {
                Ok(discovery) => discovery,
                Err(error) => {
                    report.failures.push(AccountFailure {
                        label,
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            self.seed_probe(&label, &authored);
            self.credentials.insert(label.clone(), credential.clone());

            let mut merged: BTreeMap<String, (PullRequestObservation, Vec<Relation>)> =
                BTreeMap::new();
            for observation in authored {
                // The viewer's pull request connection names the viewer as
                // author. A node whose author is outside the configured login
                // set is not an authored pull request (R2).
                let authored_by_login = observation
                    .author
                    .as_ref()
                    .is_some_and(|author| logins.contains(&author.to_ascii_lowercase()));
                if authored_by_login {
                    merged
                        .entry(observation.key.clone())
                        .or_insert_with(|| (observation, vec![Relation::Authored]));
                }
            }
            merge_discovery(&mut merged, &discovery);
            for (observation, relations) in merged.values_mut() {
                let authored_by_login = observation
                    .author
                    .as_ref()
                    .is_some_and(|author| logins.contains(&author.to_ascii_lowercase()));
                if authored_by_login && !relations.contains(&Relation::Authored) {
                    relations.push(Relation::Authored);
                }
            }

            let last_check = state
                .accounts
                .get(&label)
                .and_then(|account| account.last_check);
            let context = ClassifyContext {
                now,
                login_set: &logins,
                focus: &config.focus,
                events: &config.events,
                ignored: &ignored,
                catch_up: &catch_up,
            };

            for (key, (observation, relations)) in merged {
                if !scope_allows(config, &key) {
                    continue;
                }
                if let Some(repository) = PullRequestKey::parse(&key) {
                    observed_repositories.insert(repository.repository_name().to_ascii_lowercase());
                }
                let result = classifier::apply(
                    state,
                    &label,
                    &observation,
                    &relations,
                    last_check,
                    &context,
                );
                if result.baseline && !report.baselines.contains(&key) {
                    report.baselines.push(key.clone());
                }
                if !result.notified.is_empty() {
                    push_notification(&mut report.notifications, key, result.notified);
                }
            }

            let account_state = state.accounts.entry(label.clone()).or_default();
            account_state.login = Some(credential.login.clone());
            account_state.last_check = Some(now);
        }

        for repository in observed_repositories {
            state.catch_up.remove(&repository);
        }

        report
    }

    pub fn probe(
        &mut self,
        config: &AppConfig,
        state: &mut WatcherState,
        interactive: bool,
    ) -> ProbeReport {
        let mut report = ProbeReport::default();
        for (label, account) in &config.accounts {
            let credential = match self.credential_for(label, account, interactive) {
                Ok(credential) => credential,
                Err(message) => {
                    report.failures.push(AccountFailure {
                        label: label.clone(),
                        message,
                    });
                    continue;
                }
            };
            let url = account.graphql_url().to_string();
            let login = match self.client.viewer(&url, &credential.token) {
                Ok(login) => login,
                Err(error) => {
                    report.failures.push(AccountFailure {
                        label: label.clone(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            if let Some(expected) = &account.github_user {
                if !login.eq_ignore_ascii_case(expected) {
                    report.failures.push(AccountFailure {
                        label: label.clone(),
                        message: format!("credential belongs to @{login}, expected @{expected}"),
                    });
                    continue;
                }
            }
            let probe = match self.client.probe(&url, &credential.token) {
                Ok(probe) => probe,
                Err(error) => {
                    report.failures.push(AccountFailure {
                        label: label.clone(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            report.rate_limits.push(probe.rate_limit.clone());
            report.logins.insert(login.to_ascii_lowercase());

            let previous = self.probe_fingerprints.get(label);
            let mut next = BTreeMap::new();
            let mut changed = Vec::new();
            for node in probe.nodes {
                let key = node.key.clone();
                let fingerprint = fingerprint(&node);
                let was = previous.and_then(|previous| previous.get(&key));
                let state_changed = state
                    .accounts
                    .get(label)
                    .and_then(|account| account.pull_requests.get(&key))
                    .is_some_and(|watched| watched.state != node.state);
                if was != Some(&fingerprint) || state_changed {
                    if let Some(id) = &node.node_id {
                        changed.push(id.clone());
                    }
                }
                next.insert(key, fingerprint);
            }
            self.probe_fingerprints.insert(label.clone(), next);
            if !changed.is_empty() {
                report.changed.push((label.clone(), changed));
            }
            self.credentials.insert(
                label.clone(),
                Credential {
                    token: credential.token.clone(),
                    login,
                },
            );
        }
        report
    }

    pub fn observe_changed(
        &mut self,
        config: &AppConfig,
        state: &mut WatcherState,
        changed: &[(String, Vec<String>)],
        logins: &BTreeSet<String>,
        interactive: bool,
    ) -> RefreshReport {
        let mut report = RefreshReport::default();
        let ignored = ignored_keys(config);
        let catch_up: BTreeSet<String> = state
            .catch_up
            .iter()
            .map(|repository| repository.to_ascii_lowercase())
            .collect();
        let now = Utc::now();

        for (label, ids) in changed {
            let Some(account) = config.accounts.get(label) else {
                continue;
            };
            let credential = match self.credential_for(label, account, interactive) {
                Ok(credential) => credential,
                Err(message) => {
                    report.failures.push(AccountFailure {
                        label: label.clone(),
                        message,
                    });
                    continue;
                }
            };
            let url = account.graphql_url().to_string();
            let observations = match self.client.observe(&url, &credential.token, ids) {
                Ok(observations) => observations,
                Err(error) => {
                    report.failures.push(AccountFailure {
                        label: label.clone(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let last_check = state
                .accounts
                .get(label)
                .and_then(|account| account.last_check);
            let context = ClassifyContext {
                now,
                login_set: logins,
                focus: &config.focus,
                events: &config.events,
                ignored: &ignored,
                catch_up: &catch_up,
            };
            for observation in observations {
                if !scope_allows(config, &observation.key) {
                    continue;
                }
                let relations: Vec<Relation> = if observation
                    .author
                    .as_ref()
                    .is_some_and(|author| logins.contains(&author.to_ascii_lowercase()))
                {
                    vec![Relation::Authored]
                } else {
                    Vec::new()
                };
                let result =
                    classifier::apply(state, label, &observation, &relations, last_check, &context);
                if result.baseline && !report.baselines.contains(&observation.key) {
                    report.baselines.push(observation.key.clone());
                }
                if !result.notified.is_empty() {
                    push_notification(
                        &mut report.notifications,
                        observation.key.clone(),
                        result.notified,
                    );
                }
            }
            let account_state = state.accounts.entry(label.clone()).or_default();
            account_state.login = Some(credential.login.clone());
            account_state.last_check = Some(now);
            self.credentials.insert(label.clone(), credential);
        }
        report
    }

    fn seed_probe(&mut self, label: &str, nodes: &[PullRequestObservation]) {
        let mut fingerprints = BTreeMap::new();
        for node in nodes {
            fingerprints.insert(node.key.clone(), fingerprint(node));
        }
        self.probe_fingerprints
            .insert(label.to_string(), fingerprints);
    }

    fn credential_for(
        &mut self,
        label: &str,
        account: &AccountConfig,
        interactive: bool,
    ) -> Result<Credential, String> {
        if let Some(credential) = self.credentials.get(label) {
            return Ok(credential.clone());
        }
        let token = self.resolve_token(account, interactive)?;
        Ok(Credential {
            token,
            login: account.github_user.clone().unwrap_or_default(),
        })
    }

    fn resolve_and_verify(
        &mut self,
        label: &str,
        account: &AccountConfig,
        interactive: bool,
    ) -> Result<Credential, String> {
        let token = self.resolve_token(account, interactive)?;
        let url = account.graphql_url();
        let login = self
            .client
            .viewer(url, &token)
            .map_err(|error| error.to_string())?;
        if let Some(expected) = &account.github_user {
            if !login.eq_ignore_ascii_case(expected) {
                return Err(format!(
                    "credential belongs to @{login}, expected @{expected}"
                ));
            }
        }
        let credential = Credential { token, login };
        self.credentials
            .insert(label.to_string(), credential.clone());
        Ok(credential)
    }

    fn resolve_token(
        &mut self,
        account: &AccountConfig,
        interactive: bool,
    ) -> Result<String, String> {
        match account.auth_mode() {
            AuthMode::GhOauth => {
                let login = account
                    .github_user
                    .as_deref()
                    .ok_or_else(|| "gh_oauth requires github_user".to_string())?;
                match gh_auth_token(login) {
                    Ok(token) => Ok(token),
                    Err(_) => {
                        if interactive
                            && !self
                                .gh_login_attempted
                                .contains(&login.to_ascii_lowercase())
                        {
                            self.gh_login_attempted.insert(login.to_ascii_lowercase());
                            if gh_login(login) {
                                if let Ok(token) = gh_auth_token(login) {
                                    return Ok(token);
                                }
                                return Err(format!("gh auth token failed for @{login}"));
                            }
                            return Err(format!("gh auth login failed for @{login}"));
                        }
                        Err(format!("gh auth token failed for @{login}"))
                    }
                }
            }
            AuthMode::GithubPat => {
                if let Some(variable) = &account.token_env {
                    match std::env::var(variable) {
                        Ok(value) if value.trim().is_empty() => Err(format!(
                            "credential environment variable {variable} is empty"
                        )),
                        Ok(value) => Ok(value.trim().to_string()),
                        Err(std::env::VarError::NotPresent) => Err(format!(
                            "credential environment variable {variable} is not set"
                        )),
                        Err(std::env::VarError::NotUnicode(_)) => Err(format!(
                            "credential environment variable {variable} is empty"
                        )),
                    }
                } else if let Some(command) = &account.token_command {
                    let output = Command::new("/bin/sh")
                        .arg("-c")
                        .arg(command)
                        .output()
                        .map_err(|_| "credential command failed".to_string())?;
                    if !output.status.success() {
                        return Err("credential command failed".to_string());
                    }
                    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if token.is_empty() {
                        return Err("credential command produced an empty token".to_string());
                    }
                    Ok(token)
                } else if let Some(file) = &account.token_file {
                    let path = passuff_config::expand_home(file);
                    let text = std::fs::read_to_string(&path).map_err(|error| {
                        if error.kind() == std::io::ErrorKind::NotFound {
                            format!("token file is missing: {}", path.display())
                        } else {
                            format!("token file cannot be read: {}: {error}", path.display())
                        }
                    })?;
                    let token = text.trim().to_string();
                    if token.is_empty() {
                        return Err("token file is empty".to_string());
                    }
                    Ok(token)
                } else {
                    Err("must select exactly one credential source".to_string())
                }
            }
        }
    }
}

fn merge_discovery(
    merged: &mut BTreeMap<String, (PullRequestObservation, Vec<Relation>)>,
    discovery: &DiscoveryObservation,
) {
    for (nodes, relation) in [
        (&discovery.assigned, Relation::Assigned),
        (&discovery.review_requested, Relation::ReviewRequested),
        (
            &discovery.team_review_requested,
            Relation::TeamReviewRequested,
        ),
    ] {
        for observation in nodes {
            let entry = merged
                .entry(observation.key.clone())
                .or_insert_with(|| (observation.clone(), Vec::new()));
            if !entry.1.contains(&relation) {
                entry.1.push(relation);
            }
        }
    }
}

fn fingerprint(observation: &PullRequestObservation) -> String {
    format!(
        "{}|{}|{}|{}|{}|{:?}",
        observation.node_id.as_deref().unwrap_or_default(),
        observation.state.human(),
        observation
            .created_at
            .map(|time| time.to_rfc3339())
            .unwrap_or_default(),
        observation
            .updated_at
            .map(|time| time.to_rfc3339())
            .unwrap_or_default(),
        observation.head_ref_oid.as_deref().unwrap_or_default(),
        observation.ci
    )
}

fn push_notification(notifications: &mut Vec<Notification>, key: String, summaries: Vec<String>) {
    if let Some(existing) = notifications
        .iter_mut()
        .find(|notification| notification.key == key)
    {
        existing.summaries.extend(summaries);
    } else {
        notifications.push(Notification { key, summaries });
    }
}

fn gh_auth_token(login: &str) -> Result<String, String> {
    let output = Command::new("gh")
        .args(["auth", "token", "--user", login])
        .output()
        .map_err(|_| format!("gh auth token failed for @{login}"))?;
    if !output.status.success() {
        return Err(format!("gh auth token failed for @{login}"));
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Err(format!("gh auth token failed for @{login}"));
    }
    Ok(token)
}

fn gh_login(_login: &str) -> bool {
    Command::new("gh")
        .args(["auth", "login", "--web"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
