use std::sync::Arc;

use passuff_config::CoreConfig;

use crate::classifier::classify;
use crate::credential::CredentialSource;
use crate::domain::{
    AccountFailure, AccountFailureKind, ActivityRecord, CoreError, LoginSet, Notification,
    RefreshReport, WatchedPullRequest,
};
use crate::github::{AccountObservation, GitHubGateway};
use crate::state::SnapshotStore;

pub struct PassuffCore {
    config: CoreConfig,
    gateway: Arc<dyn GitHubGateway>,
    store: Arc<dyn SnapshotStore>,
    credentials: Arc<dyn CredentialSource>,
}

impl PassuffCore {
    pub async fn with_ports(
        config: CoreConfig,
        gateway: Arc<dyn GitHubGateway>,
        store: Arc<dyn SnapshotStore>,
        credentials: Arc<dyn CredentialSource>,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            config,
            gateway,
            store,
            credentials,
        })
    }

    pub async fn refresh(&mut self) -> Result<RefreshReport, CoreError> {
        tracing::debug!(accounts = self.config.accounts.len(), "refreshing accounts");
        let mut state = self.store.load().await?;
        let mut report = RefreshReport::default();

        let mut observed: Vec<(String, AccountObservation)> = Vec::new();
        let mut failures: Vec<AccountFailure> = Vec::new();
        for (label, account) in &self.config.accounts {
            let token = match self.credentials.resolve(&account.credential).await {
                Ok(token) => token,
                Err(error) => {
                    tracing::debug!(account = %label, error = %error, "credential failed");
                    failures.push(AccountFailure {
                        label: label.clone(),
                        kind: AccountFailureKind::Credential,
                        detail: error.to_string(),
                    });
                    continue;
                }
            };
            let known: Vec<String> = state
                .accounts
                .get(label)
                .map(|account| {
                    account
                        .pull_requests
                        .values()
                        .map(|pull_request| pull_request.node_id.clone())
                        .collect()
                })
                .unwrap_or_default();
            match self
                .gateway
                .observe(&token, &account.graphql_url, &known)
                .await
            {
                Ok(observation) => {
                    tracing::debug!(
                        account = %label,
                        login = %observation.login,
                        pull_requests = observation.pull_requests.len(),
                        "account observed"
                    );
                    observed.push((label.clone(), observation));
                }
                Err(error) => {
                    tracing::debug!(account = %label, error = %error, "observation failed");
                    failures.push(AccountFailure {
                        label: label.clone(),
                        kind: error.account_failure_kind(),
                        detail: error.to_string(),
                    });
                }
            }
        }

        let mut accepted: Vec<(String, AccountObservation)> = Vec::new();
        for (label, observation) in observed {
            let expected = self
                .config
                .accounts
                .get(&label)
                .and_then(|account| account.expected_login.as_deref());
            if let Some(expected) = expected {
                if !observation.login.eq_ignore_ascii_case(expected) {
                    tracing::debug!(
                        account = %label,
                        expected = %expected,
                        actual = %observation.login,
                        "credential identity mismatch"
                    );
                    failures.push(AccountFailure {
                        label: label.clone(),
                        kind: AccountFailureKind::Identity,
                        detail: format!(
                            "credential belongs to @{}, expected @{}",
                            observation.login, expected
                        ),
                    });
                    continue;
                }
            }
            accepted.push((label, observation));
        }

        let mut logins: Vec<String> = state
            .accounts
            .values()
            .filter(|account| !account.login.is_empty())
            .map(|account| account.login.clone())
            .collect();
        logins.extend(
            accepted
                .iter()
                .map(|(_, observation)| observation.login.clone()),
        );
        let login_set = LoginSet::new(logins);

        for (label, observation) in accepted {
            let entry = state.accounts.entry(label.clone()).or_default();
            if !entry.login.is_empty() && !entry.login.eq_ignore_ascii_case(&observation.login) {
                entry.pull_requests.clear();
                entry.last_check_at = None;
            }
            let login = observation.login.clone();
            entry.login = observation.login;
            let has_prior_cycle = entry.last_check_at.is_some();
            let last_check_at = entry.last_check_at.clone();

            for pull_request in observation.pull_requests {
                if !self
                    .config
                    .watch
                    .allows_repository(&pull_request.key.repository)
                {
                    continue;
                }
                if !pull_request.author.eq_ignore_ascii_case(&login) {
                    continue;
                }
                let key = pull_request.key.clone();
                let key_string = key.to_string();
                let previous = entry.pull_requests.get(&key_string);
                let classification = classify(
                    previous,
                    &pull_request,
                    &self.config.events,
                    &login_set,
                    has_prior_cycle,
                    last_check_at.as_deref(),
                );
                let watched = match classification.baseline {
                    Some(_) => {
                        let mut watched = WatchedPullRequest::from_baseline(
                            &pull_request,
                            observation.checked_at.clone(),
                        );
                        watched.current = classification.current;
                        if classification.events.is_empty() {
                            report.baselines_recorded.push(key.clone());
                        }
                        watched
                    }
                    None => {
                        let mut watched = previous
                            .cloned()
                            .expect("previous observation for non-baseline classification");
                        watched.title = pull_request.title.clone();
                        watched.url = pull_request.url.clone();
                        watched.created_at = pull_request.created_at.clone();
                        watched.updated_at = pull_request.updated_at.clone();
                        watched.stack = pull_request.stack.clone();
                        watched.current = classification.current;
                        watched
                    }
                };
                entry.pull_requests.insert(key_string, watched);
                if !classification.events.is_empty() {
                    let notification = Notification {
                        key,
                        events: classification.events,
                    };
                    for event in &notification.events {
                        state.next_activity_id += 1;
                        state.activity.push(ActivityRecord {
                            id: format!("a{}", state.next_activity_id),
                            key: notification.key.clone(),
                            observed_at: observation.checked_at.clone(),
                            event: event.clone(),
                            read: false,
                        });
                    }
                    report.notifications.push(notification);
                }
            }
            entry
                .pull_requests
                .retain(|_, pr| self.config.watch.allows_repository(&pr.repository));
            entry.last_check_at = Some(observation.checked_at);
        }

        report.account_failures = failures;
        if state.activity.len() > 500 {
            let excess = state.activity.len() - 500;
            state.activity.drain(0..excess);
        }
        self.store.save(&state).await?;
        tracing::debug!(
            notifications = report.notifications.len(),
            baselines = report.baselines_recorded.len(),
            failures = report.account_failures.len(),
            "refresh complete"
        );
        Ok(report)
    }
}
