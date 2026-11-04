use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use passuff_config::CoreConfig;

use crate::classifier::{classify, created_after};
use crate::credential::CredentialSource;
use crate::domain::{
    AccountFailure, AccountFailureKind, ActivityRecord, ApiAllowance, ChangeProbe, ChangeSignal,
    CoreError, LoginSet, Notification, PrState, ProbePullRequest, RefreshReport, ScopeSnapshot,
    WatchedPullRequest,
};
use crate::github::{Discovery, GitHubGateway};
use crate::state::SnapshotStore;

pub struct PassuffCore {
    config: CoreConfig,
    gateway: Arc<dyn GitHubGateway>,
    store: Arc<dyn SnapshotStore>,
    credentials: Arc<dyn CredentialSource>,
    current: Option<Vec<ChangeProbe>>,
    pending: Option<Vec<ChangeProbe>>,
    last_check: BTreeMap<String, Option<String>>,
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
            current: None,
            pending: None,
            last_check: BTreeMap::new(),
        })
    }

    /// Ask GitHub for the cheap change signal and the changed targets.
    pub async fn probe(&mut self) -> Result<ChangeSignal, CoreError> {
        let scope = ScopeSnapshot::from_watch(&self.config.watch);
        let mut probes = Vec::new();
        let mut allowance: Option<ApiAllowance> = None;
        let mut failures = Vec::new();
        for (label, account) in &self.config.accounts {
            let token = match self.credentials.resolve(&account.credential).await {
                Ok(token) => token,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        kind: AccountFailureKind::Credential,
                        detail: error.to_string(),
                    });
                    continue;
                }
            };
            let probe = match self.gateway.probe(&token, &account.graphql_url).await {
                Ok(probe) => probe,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        kind: error.account_failure_kind(),
                        detail: error.to_string(),
                    });
                    continue;
                }
            };
            let mut pull_requests: Vec<ProbePullRequest> = probe
                .pull_requests
                .into_iter()
                .filter(|pull_request| scope.allows_repository(&pull_request.repository))
                .filter(|pull_request| {
                    !self
                        .config
                        .watch
                        .ignores(&pull_request.repository, pull_request.number)
                })
                .collect();
            pull_requests.sort_by(|left, right| left.node_id.cmp(&right.node_id));
            if let Some(rate) = probe.allowance.clone() {
                let take = match allowance.as_ref() {
                    Some(current) => rate.remaining < current.remaining,
                    None => true,
                };
                if take {
                    allowance = Some(rate);
                }
            }
            probes.push(ChangeProbe {
                label: label.clone(),
                login: probe.login,
                pull_requests,
            });
        }
        probes.sort_by(|left, right| left.label.cmp(&right.label));
        let changed = probes.iter().any(|probe| {
            let current = self
                .current
                .as_ref()
                .and_then(|current| current.iter().find(|entry| entry.label == probe.label));
            let last_check = self
                .last_check
                .get(&probe.label)
                .and_then(|value| value.as_deref());
            !relevant_targets(current, probe, last_check).is_empty()
        });
        self.pending = Some(probes);
        Ok(ChangeSignal {
            changed,
            allowance,
            failures,
        })
    }

    /// Observe only the pull requests the probe reported as changed.
    pub async fn refresh_targeted(&mut self) -> Result<RefreshReport, CoreError> {
        let probe_failures = if self.pending.is_none() {
            self.probe().await?.failures
        } else {
            Vec::new()
        };
        let mut report = self.run_cycle(CycleKind::Targeted).await?;
        merge_probe_failures(&mut report, probe_failures);
        Ok(report)
    }

    /// Observe every watched open pull request and every open involved pull request.
    pub async fn refresh_reconciliation(&mut self) -> Result<RefreshReport, CoreError> {
        let probe_failures = if self.pending.is_none() {
            self.probe().await?.failures
        } else {
            Vec::new()
        };
        let mut report = self.run_cycle(CycleKind::Reconciliation).await?;
        merge_probe_failures(&mut report, probe_failures);
        Ok(report)
    }

    async fn run_cycle(&mut self, kind: CycleKind) -> Result<RefreshReport, CoreError> {
        tracing::debug!(accounts = self.config.accounts.len(), "refreshing accounts");
        let mut state = self.store.load().await?;
        let mut report = RefreshReport::default();
        let mut observed: BTreeMap<String, ObservedAccount> = BTreeMap::new();
        let mut failed: BTreeSet<String> = BTreeSet::new();
        let mut logins: Vec<String> = state
            .accounts
            .values()
            .filter(|account| !account.login.is_empty())
            .map(|account| account.login.clone())
            .collect();
        if let Some(pending) = &self.pending {
            for entry in pending {
                let expected = self
                    .config
                    .accounts
                    .get(&entry.label)
                    .and_then(|account| account.expected_login.as_deref());
                if expected.is_none_or(|expected| entry.login.eq_ignore_ascii_case(expected)) {
                    logins.push(entry.login.clone());
                }
            }
        }
        logins.extend(
            self.config
                .accounts
                .values()
                .filter_map(|account| account.expected_login.clone()),
        );
        let login_set = LoginSet::new(logins);
        let previous_scope = state.last_scope.clone();
        let current_scope = ScopeSnapshot::from_watch(&self.config.watch);

        for (label, account) in &self.config.accounts {
            let token = match self.credentials.resolve(&account.credential).await {
                Ok(token) => token,
                Err(error) => {
                    tracing::debug!(account = %label, error = %error, "credential failed");
                    failed.insert(label.clone());
                    report.account_failures.push(AccountFailure {
                        label: label.clone(),
                        kind: AccountFailureKind::Credential,
                        detail: error.to_string(),
                    });
                    continue;
                }
            };
            let pending = self
                .pending
                .as_ref()
                .and_then(|pending| pending.iter().find(|entry| &entry.label == label));
            let account_state = state.accounts.get(label);
            let last_check = account_state.and_then(|account| account.last_check_at.clone());
            let watched: Vec<&WatchedPullRequest> = account_state
                .map(|account| {
                    account
                        .pull_requests
                        .values()
                        .filter(|pull_request| {
                            self.config
                                .watch
                                .allows_repository(&pull_request.repository)
                                && !self
                                    .config
                                    .watch
                                    .ignores(&pull_request.repository, pull_request.number)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let known_ids: Vec<String> = watched
                .iter()
                .map(|pull_request| pull_request.node_id.clone())
                .collect();
            let current = self
                .current
                .as_ref()
                .and_then(|current| current.iter().find(|entry| &entry.label == label));
            let mut targets: Vec<String> = match kind {
                CycleKind::Targeted => pending
                    .map(|pending| relevant_targets(current, pending, last_check.as_deref()))
                    .unwrap_or_default(),
                CycleKind::Reconciliation => watched
                    .iter()
                    .filter(|pull_request| pull_request.current.state == PrState::Open)
                    .map(|pull_request| pull_request.node_id.clone())
                    .collect(),
            };
            if matches!(kind, CycleKind::Reconciliation) {
                if let Some(pending) = pending {
                    for id in relevant_targets(current, pending, last_check.as_deref()) {
                        if !targets.contains(&id) {
                            targets.push(id);
                        }
                    }
                }
            }
            targets.sort();
            targets.dedup();
            let discovery = match kind {
                CycleKind::Targeted => Discovery::New,
                CycleKind::Reconciliation => Discovery::All,
            };
            let observation = match self
                .gateway
                .observe(&token, &account.graphql_url, &targets, discovery, &known_ids)
                .await
            {
                Ok(observation) => observation,
                Err(error) => {
                    tracing::debug!(account = %label, error = %error, "observation failed");
                    failed.insert(label.clone());
                    report.account_failures.push(AccountFailure {
                        label: label.clone(),
                        kind: error.account_failure_kind(),
                        detail: error.to_string(),
                    });
                    continue;
                }
            };
            let expected = self
                .config
                .accounts
                .get(label)
                .and_then(|account| account.expected_login.as_deref());
            if let Some(expected) = expected {
                if !observation.login.eq_ignore_ascii_case(expected) {
                    tracing::debug!(account = %label, "credential identity mismatch");
                    failed.insert(label.clone());
                    report.account_failures.push(AccountFailure {
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

            let involved = observation.involved_node_ids.clone();
            let checked_at = observation.checked_at.clone();
            let seen: Vec<String> = observation
                .pull_requests
                .iter()
                .map(|pull_request| pull_request.node_id.clone())
                .collect();
            let entry = state.accounts.entry(label.clone()).or_default();
            let login = observation.login.clone();
            entry.login = observation.login;
            let has_prior_cycle = entry.last_check_at.is_some();
            let last_check_at = entry.last_check_at.clone();

            for pull_request in observation.pull_requests {
                if !self
                    .config
                    .watch
                    .allows_repository(&pull_request.key.repository)
                    || self
                        .config
                        .watch
                        .ignores(&pull_request.key.repository, pull_request.key.number)
                {
                    continue;
                }
                if !pull_request.author.eq_ignore_ascii_case(&login)
                    && !involved.contains(&pull_request.node_id)
                {
                    continue;
                }
                let key = pull_request.key.clone();
                let key_string = key.to_string();
                let previous = entry.pull_requests.get(&key_string);
                let retired = previous
                    .is_some_and(|watched| watched.current.state.is_terminal())
                    && pull_request.state.is_terminal();
                let catch_up = previous_scope.as_ref().is_some_and(|scope| {
                    !scope.allows_repository(&pull_request.key.repository)
                        && current_scope.allows_repository(&pull_request.key.repository)
                });
                let classification = classify(
                    previous,
                    &pull_request,
                    &self.config.events,
                    &login_set,
                    has_prior_cycle,
                    last_check_at.as_deref(),
                );
                let events = if retired {
                    Vec::new()
                } else {
                    classification.events
                };
                let silent = if retired {
                    Vec::new()
                } else {
                    classification.silent
                };
                let baseline = classification.baseline;
                let current = classification.current;
                let watched = match baseline {
                    Some(_) => {
                        let mut watched = WatchedPullRequest::from_baseline(
                            &pull_request,
                            checked_at.clone(),
                        );
                        watched.current = current;
                        if events.is_empty() {
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
                        watched.description = pull_request.body.clone();
                        watched.assignees = pull_request.assignees.clone();
                        watched.is_draft = pull_request.is_draft;
                        watched.merged_by = pull_request.merged_by.clone();
                        watched.stack = pull_request.stack.clone();
                        watched.current = current;
                        watched
                    }
                };
                entry.pull_requests.insert(key_string, watched);
                if !events.is_empty() {
                    let notification = Notification {
                        key: key.clone(),
                        events,
                    };
                    for event in &notification.events {
                        state.next_activity_id += 1;
                        state.activity.push(ActivityRecord {
                            id: format!("a{}", state.next_activity_id),
                            key: notification.key.clone(),
                            observed_at: checked_at.clone(),
                            event: event.clone(),
                            read: false,
                        });
                    }
                    if !catch_up {
                        report.notifications.push(notification);
                    }
                }
                for event in silent {
                    state.next_activity_id += 1;
                    state.activity.push(ActivityRecord {
                        id: format!("a{}", state.next_activity_id),
                        key: key.clone(),
                        observed_at: checked_at.clone(),
                        event,
                        read: true,
                    });
                }
            }
            entry.last_check_at = Some(checked_at);
            observed.insert(
                label.clone(),
                ObservedAccount {
                    targets,
                    seen,
                },
            );
        }

        if report.account_failures.is_empty() {
            state.last_scope = Some(current_scope);
        }
        if state.activity.len() > 500 {
            let excess = state.activity.len() - 500;
            state.activity.drain(0..excess);
        }
        self.store.save(&state).await?;
        self.commit_probe(observed, failed);
        self.last_check = self
            .config
            .accounts
            .keys()
            .map(|label| {
                (
                    label.clone(),
                    state.accounts.get(label).and_then(|account| account.last_check_at.clone()),
                )
            })
            .collect();
        tracing::debug!(
            notifications = report.notifications.len(),
            baselines = report.baselines_recorded.len(),
            failures = report.account_failures.len(),
            "refresh complete"
        );
        Ok(report)
    }

    fn commit_probe(
        &mut self,
        observed: BTreeMap<String, ObservedAccount>,
        failed: BTreeSet<String>,
    ) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let previous = self.current.take().unwrap_or_default();
        let mut merged = Vec::new();
        let mut handled = BTreeSet::new();
        for entry in pending {
            handled.insert(entry.label.clone());
            if failed.contains(&entry.label) {
                if let Some(old) = previous.iter().find(|old| old.label == entry.label) {
                    merged.push(old.clone());
                }
                continue;
            }
            let Some(seen) = observed.get(&entry.label) else {
                if let Some(old) = previous.iter().find(|old| old.label == entry.label) {
                    merged.push(old.clone());
                }
                continue;
            };
            let mut pull_requests = Vec::new();
            for pull_request in entry.pull_requests {
                if seen.targets.contains(&pull_request.node_id)
                    && !seen.seen.contains(&pull_request.node_id)
                {
                    if let Some(old) = previous
                        .iter()
                        .find(|old| old.label == entry.label)
                        .and_then(|old| {
                            old.pull_requests
                                .iter()
                                .find(|old| old.node_id == pull_request.node_id)
                        })
                    {
                        pull_requests.push(old.clone());
                    }
                } else {
                    pull_requests.push(pull_request);
                }
            }
            merged.push(ChangeProbe {
                label: entry.label,
                login: entry.login,
                pull_requests,
            });
        }
        for old in previous {
            if !handled.contains(&old.label) {
                merged.push(old);
            }
        }
        merged.sort_by(|left, right| left.label.cmp(&right.label));
        self.current = Some(merged);
    }
}

fn merge_probe_failures(report: &mut RefreshReport, failures: Vec<AccountFailure>) {
    let labels: BTreeSet<String> = failures.iter().map(|failure| failure.label.clone()).collect();
    report
        .account_failures
        .retain(|failure| !labels.contains(&failure.label));
    report.account_failures.extend(failures);
}

#[derive(Debug, Clone, Copy)]
enum CycleKind {
    Targeted,
    Reconciliation,
}

struct ObservedAccount {
    targets: Vec<String>,
    seen: Vec<String>,
}

/// Ids the probe reported as relevant: changed or new open pull requests,
/// state transitions, and short-lived pull requests.
fn relevant_targets(
    current: Option<&ChangeProbe>,
    pending: &ChangeProbe,
    last_check_at: Option<&str>,
) -> Vec<String> {
    let mut targets = Vec::new();
    for pull_request in &pending.pull_requests {
        let previous = current.and_then(|current| {
            current
                .pull_requests
                .iter()
                .find(|previous| previous.node_id == pull_request.node_id)
        });
        let relevant = match previous {
            None => {
                pull_request.state == PrState::Open
                    || (pull_request.state.is_terminal()
                        && last_check_at.is_some_and(|last| {
                            created_after(&pull_request.created_at, last)
                        }))
            }
            Some(previous) => {
                if previous.state != pull_request.state {
                    true
                } else if pull_request.state == PrState::Open {
                    previous != pull_request
                } else {
                    false
                }
            }
        };
        if relevant && !targets.contains(&pull_request.node_id) {
            targets.push(pull_request.node_id.clone());
        }
    }
    targets
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::CredentialError;
    use crate::domain::{ApiAllowance, CiResult, CiState, Comment, ProbePullRequest, PullRequestKey, StateError, WatcherState};
    use crate::github::{AccountObservation, AccountProbe, GitHubError};
    use async_trait::async_trait;
    use passuff_config::{
        AccountConfig, CredentialSpec, EventToggles, WatchConfig, WatchMode,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Mutex;
    use url::Url;

    struct EnvCredentials;

    #[async_trait]
    impl CredentialSource for EnvCredentials {
        async fn resolve(&self, _spec: &CredentialSpec) -> Result<String, CredentialError> {
            Ok("token".to_string())
        }
    }

    struct MemoryStore(Arc<Mutex<WatcherState>>);

    #[async_trait]
    impl SnapshotStore for MemoryStore {
        async fn load(&self) -> Result<WatcherState, StateError> {
            Ok(self.0.lock().expect("state").clone())
        }

        async fn save(&self, state: &WatcherState) -> Result<(), StateError> {
            *self.0.lock().expect("state") = state.clone();
            Ok(())
        }
    }

    struct BrokenStore;

    #[async_trait]
    impl SnapshotStore for BrokenStore {
        async fn load(&self) -> Result<WatcherState, StateError> {
            Err(StateError::Read {
                path: "state.json".into(),
                message: "broken".to_string(),
            })
        }

        async fn save(&self, _state: &WatcherState) -> Result<(), StateError> {
            Err(StateError::Write {
                path: "state.json".into(),
                message: "broken".to_string(),
            })
        }
    }

    struct ProbeGateway {
        pull_requests: Mutex<Vec<ProbePullRequest>>,
    }

    impl ProbeGateway {
        fn new(pull_requests: Vec<ProbePullRequest>) -> Self {
            Self {
                pull_requests: Mutex::new(pull_requests),
            }
        }

        fn set(&self, pull_requests: Vec<ProbePullRequest>) {
            *self.pull_requests.lock().expect("probe data") = pull_requests;
        }
    }

    #[async_trait]
    impl GitHubGateway for ProbeGateway {
        async fn observe(
            &self,
            _token: &str,
            _graphql_url: &Url,
            _target_ids: &[String],
            _discovery: Discovery,
            _known_node_ids: &[String],
        ) -> Result<AccountObservation, GitHubError> {
            Err(GitHubError::Transport("unused".to_string()))
        }

        async fn probe(
            &self,
            _token: &str,
            _graphql_url: &Url,
        ) -> Result<AccountProbe, GitHubError> {
            Ok(AccountProbe {
                login: "octo".to_string(),
                pull_requests: self.pull_requests.lock().expect("probe data").clone(),
                allowance: Some(ApiAllowance {
                    cost: 1,
                    limit: 5_000,
                    remaining: 5_000,
                    reset_at: "2026-11-03T13:00:00Z".to_string(),
                }),
            })
        }
    }

    fn pull_request(node_id: &str, updated_at: &str) -> ProbePullRequest {
        ProbePullRequest {
            repository: "acme/api".to_string(),
            number: 42,
            node_id: node_id.to_string(),
            updated_at: updated_at.to_string(),
            created_at: "2026-11-03T10:00:00Z".to_string(),
            head_oid: "abc".to_string(),
            ci_state: Some(CiState::Success),
            state: crate::domain::PrState::Open,
        }
    }

    fn config(mode: WatchMode, repositories: Vec<String>) -> CoreConfig {
        CoreConfig {
            accounts: BTreeMap::from([(
                "personal".to_string(),
                AccountConfig {
                    credential: CredentialSpec::Env {
                        variable: "TOKEN".to_string(),
                    },
                    graphql_url: Url::parse("https://example.test/graphql").unwrap(),
                    expected_login: Some("octo".to_string()),
                },
            )]),
            events: EventToggles::default(),
            watch: WatchConfig {
                mode,
                repositories,
                ignored_pull_requests: Vec::new(),
            },
        }
    }

    fn core(
        config: CoreConfig,
        gateway: Arc<dyn GitHubGateway>,
        store: Arc<dyn SnapshotStore>,
    ) -> PassuffCore {
        PassuffCore {
            config,
            gateway,
            store,
            credentials: Arc::new(EnvCredentials),
            current: None,
            pending: None,
            last_check: BTreeMap::new(),
        }
    }

    #[tokio::test]
    async fn the_first_probe_reports_a_change() {
        let gateway = Arc::new(ProbeGateway::new(vec![pull_request("n1", "t1")]));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway, store);
        let signal = core.probe().await.expect("probe");
        assert!(signal.changed);
        assert_eq!(signal.allowance.expect("allowance").remaining, 5_000);
    }

    #[tokio::test]
    async fn an_unchanged_probe_reports_no_change() {
        let gateway = Arc::new(ProbeGateway::new(vec![pull_request("n1", "t1")]));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway, store);
        core.probe().await.expect("first probe");
        let pending = core.pending.clone();
        core.current = pending;
        let signal = core.probe().await.expect("second probe");
        assert!(!signal.changed);
    }

    #[tokio::test]
    async fn a_modified_pull_request_reports_a_change() {
        let gateway = Arc::new(ProbeGateway::new(vec![pull_request("n1", "t1")]));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway.clone(), store);
        core.probe().await.expect("first probe");
        let pending = core.pending.clone();
        core.current = pending;
        gateway.set(vec![pull_request("n1", "t2")]);
        assert!(core.probe().await.expect("second probe").changed);
    }

    #[tokio::test]
    async fn an_out_of_scope_change_reports_no_change() {
        let gateway = Arc::new(ProbeGateway::new(vec![pull_request("n1", "t1")]));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(
            config(WatchMode::Selected, vec!["other/repo".to_string()]),
            gateway.clone(),
            store,
        );
        core.probe().await.expect("first probe");
        let pending = core.pending.clone();
        core.current = pending;
        gateway.set(vec![pull_request("n1", "t2")]);
        assert!(!core.probe().await.expect("second probe").changed);
    }

    #[tokio::test]
    async fn a_failed_check_cycle_keeps_the_change_pending() {
        let gateway = Arc::new(ProbeGateway::new(vec![pull_request("n1", "t1")]));
        let mut core = core(
            config(WatchMode::All, Vec::new()),
            gateway,
            Arc::new(BrokenStore),
        );
        assert!(core.probe().await.expect("first probe").changed);
        assert!(core.refresh_reconciliation().await.is_err());
        assert!(core.probe().await.expect("second probe").changed);
    }
    struct ObservationGateway {
        observation: Mutex<AccountObservation>,
    }

    impl ObservationGateway {
        fn new(observation: AccountObservation) -> Self {
            Self {
                observation: Mutex::new(observation),
            }
        }

        fn set(&self, observation: AccountObservation) {
            *self.observation.lock().expect("observation") = observation;
        }
    }

    #[async_trait]
    impl GitHubGateway for ObservationGateway {
        async fn observe(
            &self,
            _token: &str,
            _graphql_url: &Url,
            _target_ids: &[String],
            _discovery: Discovery,
            _known_node_ids: &[String],
        ) -> Result<AccountObservation, GitHubError> {
            Ok(self.observation.lock().expect("observation").clone())
        }

        async fn probe(
            &self,
            _token: &str,
            _graphql_url: &Url,
        ) -> Result<AccountProbe, GitHubError> {
            Err(GitHubError::Transport("unused".to_string()))
        }
    }

    fn authored(state: crate::domain::PrState, comments: Vec<Comment>) -> crate::domain::AuthoredPullRequest {
        let merged = matches!(state, crate::domain::PrState::Merged);
        crate::domain::AuthoredPullRequest {
            key: PullRequestKey::new("acme/api".to_string(), 42),
            node_id: "n1".to_string(),
            author: "octo".to_string(),
            title: "Pull request".to_string(),
            body: String::new(),
            url: "https://github.com/acme/api/pull/42".to_string(),
            created_at: "2026-11-03T10:00:00Z".to_string(),
            updated_at: "2026-11-03T11:00:00Z".to_string(),
            state,
            merged,
            is_draft: false,
            assignees: Vec::new(),
            labels: Vec::new(),
            milestone: None,
            base_ref: "main".to_string(),
            merged_by: None,
            stack: None,
            head_oid: "abc".to_string(),
            ci: CiResult::default(),
            comments,
            reviews: Vec::new(),
            review_comments: Vec::new(),
        }
    }

    fn observation(pull_request: crate::domain::AuthoredPullRequest) -> AccountObservation {
        AccountObservation {
            login: "octo".to_string(),
            pull_requests: vec![pull_request],
            involved_node_ids: Vec::new(),
            checked_at: "2026-11-03T12:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn a_comment_on_a_terminal_pull_request_produces_no_notification() {
        let gateway = Arc::new(ObservationGateway::new(observation(authored(
            crate::domain::PrState::Merged,
            Vec::new(),
        ))));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway.clone(), store);
        assert!(core.refresh_reconciliation().await.expect("baseline").notifications.is_empty());
        gateway.set(observation(authored(
            crate::domain::PrState::Merged,
            vec![Comment {
                id: "c1".to_string(),
                author: "alice".to_string(),
                body: "post-merge".to_string(),
                created_at: "2026-11-03T12:30:00Z".to_string(),
            }],
        )));
        assert!(core.refresh_reconciliation().await.expect("retired").notifications.is_empty());
    }

    fn probe(pull_requests: Vec<ProbePullRequest>) -> ChangeProbe {
        ChangeProbe {
            label: "personal".to_string(),
            login: "octo".to_string(),
            pull_requests,
        }
    }

    fn terminal(node_id: &str, updated_at: &str, created_at: &str) -> ProbePullRequest {
        let mut pull_request = pull_request(node_id, updated_at);
        pull_request.state = crate::domain::PrState::Merged;
        pull_request.created_at = created_at.to_string();
        pull_request
    }

    #[test]
    fn a_new_open_pull_request_is_relevant() {
        let pending = probe(vec![pull_request("n1", "t1")]);
        assert_eq!(relevant_targets(None, &pending, None), vec!["n1"]);
    }

    #[test]
    fn an_unchanged_open_pull_request_is_not_relevant() {
        let current = probe(vec![pull_request("n1", "t1")]);
        let pending = current.clone();
        assert!(relevant_targets(Some(&current), &pending, None).is_empty());
    }

    #[test]
    fn a_changed_open_pull_request_is_relevant() {
        let current = probe(vec![pull_request("n1", "t1")]);
        let pending = probe(vec![pull_request("n1", "t2")]);
        assert_eq!(relevant_targets(Some(&current), &pending, None), vec!["n1"]);
    }

    #[test]
    fn a_terminal_update_time_change_is_not_relevant() {
        let current = probe(vec![terminal("n1", "t1", "2026-11-03T10:00:00Z")]);
        let pending = probe(vec![terminal("n1", "t2", "2026-11-03T10:00:00Z")]);
        assert!(relevant_targets(Some(&current), &pending, None).is_empty());
    }

    #[test]
    fn a_terminal_transition_is_relevant() {
        let current = probe(vec![pull_request("n1", "t1")]);
        let pending = probe(vec![terminal("n1", "t2", "2026-11-03T10:00:00Z")]);
        assert_eq!(relevant_targets(Some(&current), &pending, None), vec!["n1"]);
    }

    #[test]
    fn a_reopen_is_relevant() {
        let current = probe(vec![terminal("n1", "t1", "2026-11-03T10:00:00Z")]);
        let pending = probe(vec![pull_request("n1", "t2")]);
        assert_eq!(relevant_targets(Some(&current), &pending, None), vec!["n1"]);
    }

    #[test]
    fn a_new_short_lived_pull_request_is_relevant() {
        let pending = probe(vec![terminal("n1", "t2", "2026-11-03T12:00:00Z")]);
        assert_eq!(
            relevant_targets(None, &pending, Some("2026-11-03T10:00:00Z")),
            vec!["n1"]
        );
        assert!(
            relevant_targets(None, &pending, Some("2026-11-03T13:00:00Z")).is_empty()
        );
    }

    #[test]
    fn a_failed_account_keeps_its_previous_baseline() {
        let gateway = Arc::new(ProbeGateway::new(Vec::new()));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway, store);
        core.current = Some(vec![probe(vec![pull_request("n1", "t1")])]);
        core.pending = Some(vec![probe(vec![pull_request("n1", "t2")])]);
        core.commit_probe(
            BTreeMap::<String, ObservedAccount>::new(),
            BTreeSet::from(["personal".to_string()]),
        );
        assert_eq!(
            core.current.expect("current")[0].pull_requests[0].updated_at,
            "t1"
        );
    }

    #[test]
    fn a_missing_observed_target_keeps_its_previous_baseline() {
        let gateway = Arc::new(ProbeGateway::new(Vec::new()));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway, store);
        core.current = Some(vec![probe(vec![pull_request("n1", "t1")])]);
        core.pending = Some(vec![probe(vec![pull_request("n1", "t2")])]);
        let mut observed = BTreeMap::new();
        observed.insert(
            "personal".to_string(),
            ObservedAccount {
                targets: vec!["n1".to_string()],
                seen: Vec::new(),
            },
        );
        core.commit_probe(observed, BTreeSet::new());
        assert_eq!(
            core.current.expect("current")[0].pull_requests[0].updated_at,
            "t1"
        );
    }

    #[test]
    fn an_observed_target_commits_the_pending_baseline() {
        let gateway = Arc::new(ProbeGateway::new(Vec::new()));
        let store = Arc::new(MemoryStore(Arc::new(Mutex::new(WatcherState::default()))));
        let mut core = core(config(WatchMode::All, Vec::new()), gateway, store);
        core.current = Some(vec![probe(vec![pull_request("n1", "t1")])]);
        core.pending = Some(vec![probe(vec![pull_request("n1", "t2")])]);
        let mut observed = BTreeMap::new();
        observed.insert(
            "personal".to_string(),
            ObservedAccount {
                targets: vec!["n1".to_string()],
                seen: vec!["n1".to_string()],
            },
        );
        core.commit_probe(observed, BTreeSet::new());
        assert_eq!(
            core.current.expect("current")[0].pull_requests[0].updated_at,
            "t2"
        );
    }

}
