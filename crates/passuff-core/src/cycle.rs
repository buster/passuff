use std::sync::Arc;

use passuff_config::CoreConfig;

use crate::classifier::classify;
use crate::credential::CredentialSource;
use crate::domain::{
    AccountFailure, AccountFailureKind, CoreError, LoginSet, Notification, RefreshReport,
    WatchedPullRequest,
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
        let mut state = self.store.load().await?;
        let mut report = RefreshReport::default();

        let mut observed: Vec<(String, AccountObservation)> = Vec::new();
        let mut failures: Vec<AccountFailure> = Vec::new();
        for (label, account) in &self.config.accounts {
            let token = match self.credentials.resolve(&account.credential).await {
                Ok(token) => token,
                Err(_) => {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        kind: AccountFailureKind::Credential,
                        detail: "credential unavailable".to_string(),
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
                Ok(observation) => observed.push((label.clone(), observation)),
                Err(error) => failures.push(AccountFailure {
                    label: label.clone(),
                    kind: error.account_failure_kind(),
                    detail: error.to_string(),
                }),
            }
        }

        let mut logins: Vec<String> = state
            .accounts
            .values()
            .filter(|account| !account.login.is_empty())
            .map(|account| account.login.clone())
            .collect();
        logins.extend(
            observed
                .iter()
                .map(|(_, observation)| observation.login.clone()),
        );
        let login_set = LoginSet::new(logins);

        for (label, observation) in observed {
            let entry = state.accounts.entry(label.clone()).or_default();
            if !entry.login.is_empty() && entry.login != observation.login {
                entry.pull_requests.clear();
                entry.last_check_at = None;
            }
            let login = observation.login.clone();
            entry.login = observation.login;
            let has_prior_cycle = entry.last_check_at.is_some();
            let last_check_at = entry.last_check_at.clone();

            for pull_request in observation.pull_requests {
                if pull_request.author != login {
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
                        watched.current = classification.current;
                        watched
                    }
                };
                entry.pull_requests.insert(key_string, watched);
                if !classification.events.is_empty() {
                    report.notifications.push(Notification {
                        key,
                        events: classification.events,
                    });
                }
            }
            entry.last_check_at = Some(observation.checked_at);
        }

        report.account_failures = failures;
        self.store.save(&state).await?;
        Ok(report)
    }
}
