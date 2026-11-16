use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use passuff_config::{AppConfig, FocusRelation};

use crate::model::{CiResult, EventKind, PullRequestKey, PullRequestState, StackInfo};
use crate::state::{ActivityRecord, SnapshotStore, StateError, WatchedPullRequest};

#[derive(Debug, Clone)]
pub struct AccountView {
    pub label: String,
    pub login: Option<String>,
    pub last_check: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PullRequestView {
    pub key: PullRequestKey,
    pub state: PullRequestState,
    pub ci: CiResult,
    pub title: String,
    pub body: String,
    pub author: String,
    pub assignees: Vec<String>,
    pub stack: Option<StackInfo>,
    pub relations: Vec<FocusRelation>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub ignored: bool,
    pub url: String,
}

impl PullRequestView {
    pub fn stack_label(&self) -> Option<String> {
        self.stack
            .as_ref()
            .map(|stack| format!("{}/{}", stack.position, stack.size))
    }
}

#[derive(Debug, Clone)]
pub struct ActivityView {
    pub id: String,
    pub observed_at: DateTime<Utc>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub pull_request: PullRequestKey,
    pub summary: String,
    pub display: String,
    pub body: Option<String>,
    pub read: bool,
    pub stack: Option<StackInfo>,
    pub kind: Option<EventKind>,
    pub author: Option<String>,
    pub ignored: bool,
}

impl ActivityView {
    pub fn effective_time(&self) -> DateTime<Utc> {
        self.occurred_at.unwrap_or(self.observed_at)
    }

    pub fn is_approximate(&self) -> bool {
        self.occurred_at.is_none()
    }
}

impl From<&ActivityRecord> for ActivityView {
    fn from(record: &ActivityRecord) -> Self {
        Self {
            id: record.id.clone(),
            observed_at: record.observed_at,
            occurred_at: record.occurred_at,
            pull_request: record.pull_request.clone(),
            summary: record.summary.clone(),
            display: if record.display.is_empty() {
                record.summary.clone()
            } else {
                record.display.clone()
            },
            body: record.body.clone(),
            read: record.read,
            stack: record.stack.clone(),
            kind: record.kind.clone(),
            author: record.author.clone(),
            ignored: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InboxView {
    pub accounts: Vec<AccountView>,
    pub scope_label: String,
    pub scope_mode: passuff_config::WatchMode,
    pub scope_repositories: Vec<String>,
    pub watched: Vec<PullRequestView>,
    pub activity: Vec<ActivityView>,
    pub known_repositories: Vec<String>,
    pub unread: usize,
    pub total: usize,
}

impl InboxView {
    pub fn pull_request(&self, key: &PullRequestKey) -> Option<&PullRequestView> {
        self.watched.iter().find(|view| &view.key == key)
    }

    pub fn unread_activity(&self) -> impl Iterator<Item = &ActivityView> {
        self.activity.iter().filter(|activity| !activity.read)
    }
}

#[derive(Debug, Clone)]
pub struct MarkReadResult {
    pub changed: usize,
    pub found: bool,
}

pub struct InboxService {
    config: AppConfig,
    store: Arc<dyn SnapshotStore>,
}

impl InboxService {
    pub fn new(config: AppConfig, store: Arc<dyn SnapshotStore>) -> Self {
        Self { config, store }
    }

    pub fn view(&self) -> Result<InboxView, StateError> {
        let outcome = self.store.load()?;
        let state = outcome.state;
        if outcome.migrated {
            self.store.save(&state)?;
        }

        let mut accounts = Vec::new();
        for label in self.config.accounts.keys() {
            let account = state.accounts.get(label);
            accounts.push(AccountView {
                label: label.clone(),
                login: account.and_then(|account| account.login.clone()),
                last_check: account.and_then(|account| account.last_check),
            });
        }

        let mut watched_map: BTreeMap<PullRequestKey, PullRequestView> = BTreeMap::new();
        let mut relations_by_key: BTreeMap<PullRequestKey, Vec<FocusRelation>> = BTreeMap::new();
        let mut known_repositories: BTreeSet<String> = BTreeSet::new();
        for repository in &self.config.watch.repositories {
            known_repositories.insert(repository.to_ascii_lowercase());
        }
        for account in state.accounts.values() {
            for (key, watched) in &account.pull_requests {
                if let Ok(key) = key.parse::<PullRequestKey>() {
                    known_repositories.insert(key.repository().to_string());
                }
                let Ok(key) = key.parse::<PullRequestKey>() else {
                    continue;
                };
                if !self.config.watch_includes(key.repository()) {
                    continue;
                }
                relations_by_key.insert(key.clone(), watched.relations.clone());
                if !self.config.focus.allows(&watched.relations) {
                    continue;
                }
                let ignored = self.config.watch.is_ignored(&key.to_string());
                watched_map.insert(
                    key.clone(),
                    PullRequestView {
                        key,
                        state: watched.state,
                        ci: watched.ci,
                        title: watched.title.clone(),
                        body: watched.body.clone(),
                        author: watched.author.clone(),
                        assignees: watched.assignees.clone(),
                        stack: watched.stack.clone(),
                        relations: watched.relations.clone(),
                        created_at: watched.created_at,
                        updated_at: watched.updated_at,
                        ignored,
                        url: watched.url.clone(),
                    },
                );
            }
        }

        let watched_keys: BTreeSet<PullRequestKey> = watched_map.keys().cloned().collect();
        let mut activity: Vec<ActivityView> = state
            .activity
            .iter()
            .filter(|record| self.config.watch_includes(record.pull_request.repository()))
            .filter(|record| {
                let relations = relations_by_key
                    .get(&record.pull_request)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                self.config.focus.allows(relations)
            })
            .map(|record| {
                let mut view = ActivityView::from(record);
                view.ignored = watched_map
                    .get(&view.pull_request)
                    .map(|pr| pr.ignored)
                    .unwrap_or(false);
                view
            })
            .collect();
        activity.sort_by(|left, right| {
            right
                .effective_time()
                .cmp(&left.effective_time())
                .then_with(|| id_number(&right.id).cmp(&id_number(&left.id)))
        });

        let unread = activity
            .iter()
            .filter(|activity| !activity.read && !activity.ignored)
            .count();
        let total = activity.iter().filter(|activity| !activity.ignored).count();
        let _ = watched_keys;

        Ok(InboxView {
            accounts,
            scope_label: self.config.watch.label(),
            scope_mode: self.config.watch.effective_mode(),
            scope_repositories: self.config.watch.repositories.clone(),
            watched: watched_map.into_values().collect(),
            activity,
            known_repositories: known_repositories.into_iter().collect(),
            unread,
            total,
        })
    }

    pub fn mark_read(&self, id: &str) -> Result<MarkReadResult, StateError> {
        let outcome = self.store.load()?;
        let mut state = outcome.state;
        let mut changed = 0;
        let mut found = false;
        for record in &mut state.activity {
            if record.id == id {
                found = true;
                if !record.read {
                    record.read = true;
                    changed = 1;
                }
                break;
            }
        }
        if found || outcome.migrated {
            self.store.save(&state)?;
        }
        Ok(MarkReadResult { changed, found })
    }

    pub fn mark_many_read<'a>(
        &self,
        ids: impl IntoIterator<Item = &'a str>,
    ) -> Result<MarkReadResult, StateError> {
        let wanted: BTreeSet<&str> = ids.into_iter().collect();
        let outcome = self.store.load()?;
        let mut state = outcome.state;
        let mut changed = 0;
        for record in &mut state.activity {
            if wanted.contains(record.id.as_str()) && !record.read {
                record.read = true;
                changed += 1;
            }
        }
        if changed > 0 || outcome.migrated {
            self.store.save(&state)?;
        }
        Ok(MarkReadResult {
            changed,
            found: true,
        })
    }

    pub fn mark_all_read(&self) -> Result<MarkReadResult, StateError> {
        let outcome = self.store.load()?;
        let mut state = outcome.state;
        let mut changed = 0;
        for record in &mut state.activity {
            if !record.read {
                record.read = true;
                changed += 1;
            }
        }
        if changed > 0 || outcome.migrated {
            self.store.save(&state)?;
        }
        Ok(MarkReadResult {
            changed,
            found: true,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn store(&self) -> &Arc<dyn SnapshotStore> {
        &self.store
    }
}

fn id_number(id: &str) -> u64 {
    id.strip_prefix('a')
        .and_then(|number| number.parse().ok())
        .unwrap_or(0)
}

#[allow(dead_code)]
fn _watched(_: &WatchedPullRequest) {}
