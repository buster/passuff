use std::sync::Arc;

use passuff_config::CoreConfig;

use crate::classifier::classify;
use crate::credential::CredentialSource;
use crate::domain::{
    AccountFailure, AccountFailureKind, ActivityRecord, ApiAllowance, ChangeProbe, ChangeSignal,
    CoreError, LoginSet, Notification, PrState, ProbePullRequest, RefreshReport, ScopeSnapshot,
    WatchedPullRequest,
};
use crate::github::{AccountObservation, GitHubGateway};
use crate::state::SnapshotStore;

pub struct PassuffCore {
    config: CoreConfig,
    gateway: Arc<dyn GitHubGateway>,
    store: Arc<dyn SnapshotStore>,
    credentials: Arc<dyn CredentialSource>,
    current: Option<Vec<ChangeProbe>>,
    pending: Option<Vec<ChangeProbe>>,
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
        })
    }

    /// Ask GitHub for a cheap change signal without building observations.
    pub async fn probe(&mut self) -> Result<ChangeSignal, CoreError> {
        let scope = ScopeSnapshot::from_watch(&self.config.watch);
        let mut probes = Vec::new();
        let mut allowance: Option<ApiAllowance> = None;
        for (label, account) in &self.config.accounts {
            let token = self
                .credentials
                .resolve(&account.credential)
                .await
                .map_err(|error| CoreError::Probe(format!("{label}: {error}")))?;
            let probe = self
                .gateway
                .probe(&token, &account.graphql_url)
                .await
                .map_err(|error| CoreError::Probe(format!("{label}: {error}")))?;
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
                login: probe.login,
                pull_requests,
            });
        }
        probes.sort_by(|left, right| left.login.cmp(&right.login));
        let changed = match self.current.as_ref() {
            Some(current) => {
                current.len() != probes.len()
                    || current.iter().zip(probes.iter()).any(|(left, right)| {
                        left.login != right.login || left.pull_requests != right.pull_requests
                    })
            }
            None => true,
        };
        self.pending = Some(probes);
        Ok(ChangeSignal { changed, allowance })
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
                .map(observation_known_ids)
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
        let previous_scope = state.last_scope.clone();
        let current_scope = ScopeSnapshot::from_watch(&self.config.watch);

        for (label, observation) in accepted {
            let entry = state.accounts.entry(label.clone()).or_default();
            if !entry.login.is_empty() && !entry.login.eq_ignore_ascii_case(&observation.login) {
                entry.pull_requests.clear();
                entry.last_check_at = None;
            }
            let login = observation.login.clone();
            entry.login = observation.login;
            let involved = observation.involved_node_ids.clone();
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
                            observation.checked_at.clone(),
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
                            observed_at: observation.checked_at.clone(),
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
                        observed_at: observation.checked_at.clone(),
                        event,
                        read: true,
                    });
                }
            }
            entry.last_check_at = Some(observation.checked_at);
        }

        if failures.is_empty() {
            state.last_scope = Some(current_scope);
        }
        report.account_failures = failures;
        if state.activity.len() > 500 {
            let excess = state.activity.len() - 500;
            state.activity.drain(0..excess);
        }
        self.store.save(&state).await?;
        if let Some(pending) = self.pending.take() {
            self.current = Some(pending);
        }
        tracing::debug!(
            notifications = report.notifications.len(),
            baselines = report.baselines_recorded.len(),
            failures = report.account_failures.len(),
            "refresh complete"
        );
        Ok(report)
    }
}

const KNOWN_RECENT_PULL_REQUESTS: usize = 20;

/// Node ids to hydrate by id: every watched open pull request plus the most
/// recently updated watched pull requests, capped at the 100-id node limit.
fn observation_known_ids(account: &crate::domain::AccountState) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for pull_request in account
        .pull_requests
        .values()
        .filter(|pull_request| pull_request.current.state == PrState::Open)
    {
        if ids.len() >= 100 {
            return ids;
        }
        if !ids.contains(&pull_request.node_id) {
            ids.push(pull_request.node_id.clone());
        }
    }
    let mut recent: Vec<_> = account.pull_requests.values().collect();
    recent.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    for pull_request in recent.into_iter().take(KNOWN_RECENT_PULL_REQUESTS) {
        if ids.len() >= 100 {
            break;
        }
        if !ids.contains(&pull_request.node_id) {
            ids.push(pull_request.node_id.clone());
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::CredentialError;
    use crate::domain::{ApiAllowance, CiResult, CiState, Comment, ProbePullRequest, PullRequestKey, StateError, WatcherState};
    use crate::github::{AccountProbe, GitHubError};
    use async_trait::async_trait;
    use passuff_config::{
        AccountConfig, CredentialSpec, EventToggles, WatchConfig, WatchMode,
    };
    use std::collections::BTreeMap;
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
        assert!(core.refresh().await.is_err());
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
        assert!(core.refresh().await.expect("baseline").notifications.is_empty());
        gateway.set(observation(authored(
            crate::domain::PrState::Merged,
            vec![Comment {
                id: "c1".to_string(),
                author: "alice".to_string(),
                body: "post-merge".to_string(),
                created_at: "2026-11-03T12:30:00Z".to_string(),
            }],
        )));
        assert!(core.refresh().await.expect("retired").notifications.is_empty());
    }

}
