use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use passuff_config::{AppConfig, FocusRelation, WatchMode};

use crate::classifier::{classify, ClassificationContext};
use crate::credentials::CredentialSource;
use crate::github::{GatewayError, GatewayFactory, GitHubGateway};
use crate::model::{
    Allowance, Fingerprint, Notification, ProbeReport, PullRequestKey, PullRequestSnapshot,
};
use crate::probe::diff_probe;
use crate::state::{
    ActivityRecord, ScopeSnapshot, SnapshotStore, StateError, WatchedPullRequest, WatcherState,
};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("state error: {0}")]
    State(#[from] StateError),
}

#[derive(Debug, Clone)]
pub struct AccountFailure {
    pub label: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    pub notifications: Vec<Notification>,
    pub account_failures: Vec<AccountFailure>,
    pub baselines: Vec<PullRequestKey>,
}

impl RefreshReport {
    pub fn failed(&self) -> bool {
        !self.account_failures.is_empty()
    }

    pub fn activity_count(&self) -> usize {
        self.notifications
            .iter()
            .map(Notification::activity_count)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct AccountProbe {
    pub label: String,
    pub login: String,
    pub report: ProbeReport,
    pub targets: Vec<PullRequestKey>,
}

#[derive(Debug, Clone, Default)]
pub struct ProbeCycle {
    pub accounts: Vec<AccountProbe>,
    pub failures: Vec<AccountFailure>,
    pub pending: BTreeMap<String, BTreeMap<String, Fingerprint>>,
}

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct FixedClock {
    now: DateTime<Utc>,
}

impl FixedClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}

#[derive(Clone)]
struct ResolvedAccount {
    label: String,
    login: String,
    gateway: Arc<Mutex<Box<dyn GitHubGateway>>>,
}

pub struct PassuffCore {
    config: AppConfig,
    gateway_factory: Arc<dyn GatewayFactory>,
    credentials: Arc<dyn CredentialSource>,
    store: Arc<dyn SnapshotStore>,
    clock: Arc<dyn Clock>,
    cached: Mutex<Option<Vec<ResolvedAccount>>>,
}

impl PassuffCore {
    pub fn new(
        config: AppConfig,
        gateway_factory: Arc<dyn GatewayFactory>,
        credentials: Arc<dyn CredentialSource>,
        store: Arc<dyn SnapshotStore>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            config,
            gateway_factory,
            credentials,
            store,
            clock,
            cached: Mutex::new(None),
        }
    }

    pub fn store(&self) -> &Arc<dyn SnapshotStore> {
        &self.store
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn refresh_reconciliation(&self, interactive: bool) -> Result<RefreshReport, CoreError> {
        self.refresh_internal(
            interactive,
            RefreshKind::Reconciliation,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        )
    }

    /// Probe every account cheaply and cache its gateway so the following
    /// targeted pass can observe the selected node ids.
    pub fn probe(&self, interactive: bool) -> Result<ProbeCycle, CoreError> {
        let outcome = self.store.load()?;
        let state = outcome.state;
        if outcome.migrated {
            self.store.save(&state)?;
        }
        let (resolved, mut failures) = self.resolve_accounts(interactive)?;
        let cache = resolved.clone();
        let mut cycle = ProbeCycle {
            accounts: Vec::new(),
            failures: Vec::new(),
            pending: BTreeMap::new(),
        };
        for account in resolved {
            let report = match account.gateway.lock() {
                Ok(mut gateway) => gateway.probe(),
                Err(_) => Err(GatewayError::Unavailable(
                    "gateway is unavailable".to_string(),
                )),
            };
            let report = match report {
                Ok(report) => report,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: account.label.clone(),
                        message: format!("account {} failed: {error}", account.label),
                    });
                    continue;
                }
            };
            let account_state = state
                .accounts
                .get(&account.label)
                .cloned()
                .unwrap_or_default();
            let diff = diff_probe(&account_state.accepted_probe, &report, self.clock.now());
            let targets: Vec<PullRequestKey> = diff
                .targets
                .into_iter()
                .filter(|key| self.config.watch_includes(key.repository()))
                .collect();
            cycle.pending.insert(account.label.clone(), diff.pending);
            cycle.accounts.push(AccountProbe {
                label: account.label,
                login: account.login,
                report,
                targets,
            });
        }
        cycle.failures = failures;
        if let Ok(mut slot) = self.cached.lock() {
            *slot = Some(cache);
        }
        Ok(cycle)
    }

    pub fn refresh_targeted(
        &self,
        interactive: bool,
        targets: &BTreeMap<String, Vec<PullRequestKey>>,
        pending: &BTreeMap<String, BTreeMap<String, Fingerprint>>,
    ) -> Result<RefreshReport, CoreError> {
        self.refresh_internal(interactive, RefreshKind::Targeted, targets, pending, true)
    }

    fn refresh_internal(
        &self,
        interactive: bool,
        kind: RefreshKind,
        targets: &BTreeMap<String, Vec<PullRequestKey>>,
        pending: &BTreeMap<String, BTreeMap<String, Fingerprint>>,
        use_cache: bool,
    ) -> Result<RefreshReport, CoreError> {
        let outcome = self.store.load()?;
        let mut state = outcome.state;
        let cached = if use_cache {
            self.cached.lock().ok().and_then(|mut slot| slot.take())
        } else {
            None
        };
        let (resolved, mut failures) = match cached {
            Some(resolved) => (resolved, Vec::new()),
            None => self.resolve_accounts(interactive)?,
        };
        let login_set: Vec<String> = resolved
            .iter()
            .map(|account| account.login.clone())
            .collect();
        let catch_up = self.catch_up_repositories(&state);
        let now = self.clock.now();
        let mut report = RefreshReport::default();
        let mut notifications: BTreeMap<PullRequestKey, Vec<crate::model::ActivityEvent>> =
            BTreeMap::new();
        let mut pending_activity: Vec<ActivityRecord> = Vec::new();
        let mut next_id = state.next_activity_id.max(1);
        let mut baseline_set: BTreeSet<PullRequestKey> = BTreeSet::new();

        for account in resolved {
            let snapshots = match kind {
                RefreshKind::Reconciliation => self.fetch_reconciliation(&account),
                RefreshKind::Targeted => {
                    let keys = targets.get(&account.label).cloned().unwrap_or_default();
                    match account.gateway.lock() {
                        Ok(mut gateway) => {
                            let mut fetch = || -> Result<Vec<PullRequestSnapshot>, GatewayError> {
                                let mut snapshots = Vec::new();
                                if !keys.is_empty() {
                                    snapshots.extend(gateway.observe(&keys)?);
                                }
                                snapshots.extend(gateway.discover_involved()?);
                                Ok(merge_snapshots(snapshots))
                            };
                            fetch()
                        }
                        Err(_) => Err(GatewayError::Unavailable(
                            "gateway is unavailable".to_string(),
                        )),
                    }
                }
            };
            let snapshots = match snapshots {
                Ok(snapshots) => snapshots,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: account.label.clone(),
                        message: format!("account {} failed: {error}", account.label),
                    });
                    continue;
                }
            };
            let snapshots = self.merge_relations(snapshots, &login_set);
            let account_state = state.accounts.entry(account.label.clone()).or_default();
            account_state.login = Some(account.login.clone());
            let previous_last_check = account_state.last_check;

            for snapshot in &snapshots {
                if !self.config.watch_includes(snapshot.key.repository()) {
                    continue;
                }
                let key = snapshot.key.to_string();
                let previous = account_state.pull_requests.get(&key).cloned();
                let relations = self.relations_for(snapshot, previous.as_ref(), &login_set);
                let focus_allows = self.config.focus.allows(&relations);
                let catch_up_here = catch_up
                    .iter()
                    .any(|repo| repo.eq_ignore_ascii_case(snapshot.key.repository()));
                let context = ClassificationContext {
                    last_check: previous_last_check,
                    logins: &login_set,
                    events: &self.config.events,
                    focus_allows,
                    catch_up: catch_up_here,
                    ignored: self.config.watch.is_ignored(&key),
                };
                let first = previous.is_none();
                let terminal_known = previous
                    .as_ref()
                    .map(|record| record.state.is_terminal() && snapshot.state.is_terminal())
                    .unwrap_or(false);
                let classification = if terminal_known {
                    crate::classifier::Classification {
                        events: Vec::new(),
                        updated: refresh_terminal_metadata(
                            previous.as_ref().expect("previous"),
                            snapshot,
                            &relations,
                        ),
                    }
                } else {
                    classify(previous.as_ref(), snapshot, context)
                };
                for classified in classification.events {
                    let id = format!("a{next_id}");
                    next_id += 1;
                    let record = ActivityRecord::from_event(
                        id,
                        now,
                        &snapshot.key,
                        &classified.event,
                        snapshot.stack.clone(),
                    );
                    let notify = classified.notify && !record.read;
                    pending_activity.push(record);
                    if notify {
                        notifications
                            .entry(snapshot.key.clone())
                            .or_default()
                            .push(classified.event);
                    }
                }
                if first {
                    baseline_set.insert(snapshot.key.clone());
                }
                account_state
                    .pull_requests
                    .insert(key, classification.updated);
            }

            match kind {
                RefreshKind::Reconciliation => {
                    let mut fingerprints = BTreeMap::new();
                    for snapshot in &snapshots {
                        fingerprints.insert(snapshot.key.to_string(), fingerprint_of(snapshot));
                    }
                    account_state.accepted_probe = fingerprints;
                }
                RefreshKind::Targeted => {
                    if let Some(fingerprints) = pending.get(&account.label) {
                        account_state.accepted_probe = fingerprints.clone();
                    }
                }
            }
            account_state.last_check = Some(now);
        }

        state.next_activity_id = next_id;
        for record in pending_activity {
            state.push_activity(record);
        }

        for (key, events) in notifications {
            let title = state
                .accounts
                .values()
                .find_map(|account| account.pull_requests.get(&key.to_string()))
                .map(|watched| watched.title.clone())
                .filter(|title| !title.is_empty())
                .unwrap_or_else(|| key.to_string());
            report
                .notifications
                .push(Notification { key, title, events });
        }
        report.baselines = baseline_set.into_iter().collect();
        if failures.is_empty() {
            state.last_scope = Some(self.scope_snapshot());
        }
        report.account_failures = failures;
        self.store.save(&state)?;
        Ok(report)
    }

    fn fetch_reconciliation(
        &self,
        account: &ResolvedAccount,
    ) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        let mut gateway = match account.gateway.lock() {
            Ok(gateway) => gateway,
            Err(_) => {
                return Err(GatewayError::Unavailable(
                    "gateway is unavailable".to_string(),
                ))
            }
        };
        let authored = gateway.list_authored()?;
        let involved = gateway.discover_involved()?;
        let mut by_key: BTreeMap<PullRequestKey, PullRequestSnapshot> = BTreeMap::new();
        for snapshot in authored.into_iter().chain(involved) {
            if let Some(existing) = by_key.get_mut(&snapshot.key) {
                for relation in snapshot.relations {
                    if !existing.relations.contains(&relation) {
                        existing.relations.push(relation);
                    }
                }
            } else {
                by_key.insert(snapshot.key.clone(), snapshot);
            }
        }
        Ok(by_key.into_values().collect())
    }

    fn merge_relations(
        &self,
        snapshots: Vec<PullRequestSnapshot>,
        logins: &[String],
    ) -> Vec<PullRequestSnapshot> {
        snapshots
            .into_iter()
            .map(|mut snapshot| {
                if logins
                    .iter()
                    .any(|login| login.eq_ignore_ascii_case(&snapshot.author))
                    && !snapshot.relations.contains(&FocusRelation::Authored)
                {
                    snapshot.relations.push(FocusRelation::Authored);
                }
                snapshot.relations.sort();
                snapshot.relations.dedup();
                snapshot
            })
            .collect()
    }

    fn relations_for(
        &self,
        snapshot: &PullRequestSnapshot,
        previous: Option<&WatchedPullRequest>,
        logins: &[String],
    ) -> Vec<FocusRelation> {
        let mut relations = snapshot.relations.clone();
        if logins
            .iter()
            .any(|login| login.eq_ignore_ascii_case(&snapshot.author))
            && !relations.contains(&FocusRelation::Authored)
        {
            relations.push(FocusRelation::Authored);
        }
        if relations.is_empty() {
            if let Some(previous) = previous {
                relations = previous.relations.clone();
            }
        }
        relations.sort();
        relations.dedup();
        relations
    }

    fn catch_up_repositories(&self, state: &WatcherState) -> BTreeSet<String> {
        let Some(previous) = &state.last_scope else {
            return BTreeSet::new();
        };
        let current_mode = self.config.watch.effective_mode();
        let mut newly_included = BTreeSet::new();
        match (previous.mode, current_mode) {
            (WatchMode::All, _) => {}
            (_, WatchMode::All) => {
                for account in state.accounts.values() {
                    for key in account.pull_requests.keys() {
                        if let Ok(key) = key.parse::<PullRequestKey>() {
                            let repository = key.repository().to_string();
                            if !previous
                                .repositories
                                .iter()
                                .any(|candidate| candidate.eq_ignore_ascii_case(&repository))
                            {
                                newly_included.insert(repository);
                            }
                        }
                    }
                }
            }
            (WatchMode::Selected, WatchMode::Selected) => {
                for repository in &self.config.watch.repositories {
                    if !previous
                        .repositories
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(repository))
                    {
                        newly_included.insert(repository.clone());
                    }
                }
            }
        }
        let mut catch_up = BTreeSet::new();
        for repository in newly_included {
            let has_records = state.accounts.values().any(|account| {
                account.pull_requests.keys().any(|key| {
                    key.parse::<PullRequestKey>()
                        .map(|key| key.repository().eq_ignore_ascii_case(&repository))
                        .unwrap_or(false)
                })
            });
            if has_records {
                catch_up.insert(repository);
            }
        }
        catch_up
    }

    fn scope_snapshot(&self) -> ScopeSnapshot {
        ScopeSnapshot {
            mode: self.config.watch.effective_mode(),
            repositories: self.config.watch.repositories.clone(),
        }
    }

    fn resolve_accounts(
        &self,
        interactive: bool,
    ) -> Result<(Vec<ResolvedAccount>, Vec<AccountFailure>), CoreError> {
        let mut resolved = Vec::new();
        let mut failures = Vec::new();
        for (label, account) in &self.config.accounts {
            let credential = match self.credentials.resolve(label, account, interactive) {
                Ok(credential) => credential,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        message: format!("account {label} failed: {error}"),
                    });
                    continue;
                }
            };
            let gateway = match self
                .gateway_factory
                .create(label, account, &credential.token)
            {
                Ok(gateway) => gateway,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        message: format!("account {label} failed: {error}"),
                    });
                    continue;
                }
            };
            let gateway = Arc::new(Mutex::new(gateway));
            let viewer_login = match gateway.lock() {
                Ok(mut gateway) => gateway.viewer_login(),
                Err(_) => Err(GatewayError::Unavailable(
                    "gateway is unavailable".to_string(),
                )),
            };
            let viewer_login = match viewer_login {
                Ok(login) => login,
                Err(error) => {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        message: format!("account {label} failed: {error}"),
                    });
                    continue;
                }
            };
            if let Some(expected) = account.expected_login() {
                if !expected.eq_ignore_ascii_case(&viewer_login) {
                    failures.push(AccountFailure {
                        label: label.clone(),
                        message: format!(
                            "account {label} failed: credential belongs to @{viewer_login}, expected @{expected}"
                        ),
                    });
                    continue;
                }
            }
            resolved.push(ResolvedAccount {
                label: label.clone(),
                login: viewer_login,
                gateway,
            });
        }
        Ok((resolved, failures))
    }
}

#[derive(Debug, Clone, Copy)]
enum RefreshKind {
    Reconciliation,
    Targeted,
}

fn refresh_terminal_metadata(
    previous: &WatchedPullRequest,
    snapshot: &PullRequestSnapshot,
    relations: &[FocusRelation],
) -> WatchedPullRequest {
    let mut updated = previous.clone();
    updated.baseline = true;
    updated.title = snapshot.title.clone();
    updated.body = snapshot.body.clone();
    updated.author = snapshot.author.clone();
    updated.assignees = snapshot.assignees.clone();
    updated.url = snapshot.url.clone();
    updated.head_oid = snapshot.head_oid.clone();
    updated.created_at = snapshot.created_at;
    updated.updated_at = snapshot.updated_at;
    updated.merged_at = snapshot.merged_at;
    updated.closed_at = snapshot.closed_at;
    updated.stack = snapshot.stack.clone();
    updated.relations = relations.to_vec();
    for comment in &snapshot.comments {
        updated.seen_comments.insert(comment.id.clone());
    }
    for review in &snapshot.reviews {
        updated.seen_reviews.insert(review.id.clone());
    }
    for comment in &snapshot.review_comments {
        updated.seen_review_comments.insert(comment.id.clone());
    }
    updated
}

fn merge_snapshots(snapshots: Vec<PullRequestSnapshot>) -> Vec<PullRequestSnapshot> {
    let mut by_key: BTreeMap<PullRequestKey, PullRequestSnapshot> = BTreeMap::new();
    for snapshot in snapshots {
        if let Some(existing) = by_key.get_mut(&snapshot.key) {
            for relation in snapshot.relations {
                if !existing.relations.contains(&relation) {
                    existing.relations.push(relation);
                }
            }
        } else {
            by_key.insert(snapshot.key.clone(), snapshot);
        }
    }
    by_key.into_values().collect()
}

fn fingerprint_of(snapshot: &PullRequestSnapshot) -> Fingerprint {
    Fingerprint {
        node_id: snapshot.node_id.clone(),
        created_at: snapshot.created_at,
        updated_at: snapshot.updated_at,
        state: snapshot.state,
        head_oid: snapshot.head_oid.clone(),
        ci: snapshot.ci,
    }
}

#[allow(dead_code)]
pub fn allowance_label(allowance: &Allowance) -> String {
    match allowance.reset_at {
        Some(reset) => format!("{} points left until {}", allowance.remaining, reset),
        None => format!("{} points left", allowance.remaining),
    }
}
