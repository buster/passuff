use std::sync::Arc;

use async_trait::async_trait;
use chrono::DateTime;
use passuff_config::CoreConfig;

use crate::domain::{
    ActivityRecord, CiState, CoreError, PrState, PullRequestKey, PullRequestStack, WatcherState,
};
use crate::state::SnapshotStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountView {
    pub label: String,
    pub login: Option<String>,
    pub last_check_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestView {
    pub key: PullRequestKey,
    pub state: PrState,
    pub ci_state: CiState,
    pub title: String,
    pub url: String,
    pub updated_at: String,
    pub stack: Option<PullRequestStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxView {
    pub accounts: Vec<AccountView>,
    pub scope: Vec<String>,
    pub scope_label: String,
    pub pull_requests: Vec<PullRequestView>,
    pub activity: Vec<ActivityRecord>,
    pub unread: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOutcome {
    pub found: bool,
    pub changed: usize,
    pub unread: usize,
}

#[async_trait]
pub trait InboxService: Send + Sync {
    async fn view(&self, config: &CoreConfig) -> Result<InboxView, CoreError>;
    async fn mark_read(&self, id: &str) -> Result<ReadOutcome, CoreError>;
    async fn mark_all_read(&self) -> Result<ReadOutcome, CoreError>;
}

pub struct Inbox {
    store: Arc<dyn SnapshotStore>,
}

impl Inbox {
    pub fn new(store: Arc<dyn SnapshotStore>) -> Self {
        Self { store }
    }

    fn view_from_state(config: &CoreConfig, state: &WatcherState) -> InboxView {
        let accounts = config
            .accounts
            .keys()
            .map(|label| {
                let account = state.accounts.get(label);
                AccountView {
                    label: label.clone(),
                    login: account.map(|account| account.login.clone()),
                    last_check_at: account.and_then(|account| account.last_check_at.clone()),
                }
            })
            .collect();
        let mut pull_requests: Vec<PullRequestView> = state
            .accounts
            .values()
            .flat_map(|account| account.pull_requests.values())
            .map(|pr| PullRequestView {
                key: PullRequestKey::new(pr.repository.clone(), pr.number),
                state: pr.current.state.clone(),
                ci_state: pr.current.ci_state.clone(),
                title: pr.title.clone(),
                url: pr.url.clone(),
                updated_at: pr.updated_at.clone(),
                stack: pr.stack.clone(),
            })
            .collect();
        pull_requests.retain(|pr| {
            config.watch.allows_repository(&pr.key.repository)
                && !config
                    .watch
                    .ignores(&pr.key.repository, pr.key.number)
        });
        pull_requests.sort_by(|a, b| a.key.cmp(&b.key));
        let mut activity: Vec<(usize, ActivityRecord)> = state
            .activity
            .iter()
            .enumerate()
            .filter(|(_, record)| {
                config.watch.allows_repository(&record.key.repository)
                    && !config
                        .watch
                        .ignores(&record.key.repository, record.key.number)
            })
            .map(|(index, record)| (index, record.clone()))
            .collect();
        activity.sort_by(|(left_index, left), (right_index, right)| {
            let left_at = DateTime::parse_from_rfc3339(&left.observed_at).ok();
            let right_at = DateTime::parse_from_rfc3339(&right.observed_at).ok();
            right_at
                .cmp(&left_at)
                .then_with(|| right.observed_at.cmp(&left.observed_at))
                .then_with(|| right_index.cmp(left_index))
        });
        let activity: Vec<ActivityRecord> = activity
            .into_iter()
            .map(|(_, record)| record)
            .collect();
        let unread = activity.iter().filter(|record| !record.read).count();
        InboxView {
            accounts,
            scope: config.watch.repositories.clone(),
            scope_label: config.watch.label(),
            pull_requests,
            activity,
            unread,
        }
    }
}

#[async_trait]
impl InboxService for Inbox {
    async fn view(&self, config: &CoreConfig) -> Result<InboxView, CoreError> {
        let state = self.store.load().await?;
        Ok(Self::view_from_state(config, &state))
    }

    async fn mark_read(&self, id: &str) -> Result<ReadOutcome, CoreError> {
        let mut state = self.store.load().await?;
        let unread = state.activity.iter().filter(|record| !record.read).count();
        let Some(record) = state.activity.iter_mut().find(|record| record.id == id) else {
            return Ok(ReadOutcome {
                found: false,
                changed: 0,
                unread,
            });
        };
        if record.read {
            return Ok(ReadOutcome {
                found: true,
                changed: 0,
                unread,
            });
        }
        record.read = true;
        let unread = unread - 1;
        self.store.save(&state).await?;
        Ok(ReadOutcome {
            found: true,
            changed: 1,
            unread,
        })
    }

    async fn mark_all_read(&self) -> Result<ReadOutcome, CoreError> {
        let mut state = self.store.load().await?;
        let mut changed = 0;
        for record in &mut state.activity {
            if !record.read {
                record.read = true;
                changed += 1;
            }
        }
        if changed > 0 {
            self.store.save(&state).await?;
        }
        Ok(ReadOutcome {
            found: true,
            changed,
            unread: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AccountState, ActivityEvent, PrState, WatchedPullRequest, WatcherState};
    use crate::domain::{AuthoredPullRequest, CiResult, CiState, Comment};
    use crate::state::JsonStateStore;
    use passuff_config::{AccountConfig, CredentialSpec, EventToggles, WatchConfig};
    use std::collections::BTreeMap;

use chrono::DateTime;
    use std::sync::Arc;
    use url::Url;

    fn config() -> CoreConfig {
        CoreConfig {
            accounts: BTreeMap::from([(
                "personal".to_string(),
                AccountConfig {
                    credential: CredentialSpec::Env {
                        variable: "TOKEN".to_string(),
                    },
                    graphql_url: Url::parse("http://127.0.0.1/graphql").unwrap(),
                    expected_login: None,
                },
            )]),
            events: EventToggles::default(),
            watch: WatchConfig::default(),
        }
    }

    fn authored() -> AuthoredPullRequest {
        AuthoredPullRequest {
            key: PullRequestKey::new("acme/api".to_string(), 42),
            node_id: "node-42".to_string(),
            author: "octo".to_string(),
            title: "Pull request acme/api#42".to_string(),
            url: "https://github.com/acme/api/pull/42".to_string(),
            created_at: "2026-11-02T09:00:00Z".to_string(),
            updated_at: "2026-11-02T09:00:00Z".to_string(),
            state: PrState::Open,
            merged: false,
            is_draft: false,
            assignees: Vec::new(),
            labels: Vec::new(),
            milestone: None,
            base_ref: "main".to_string(),
            stack: None,
            head_oid: "abc123".to_string(),
            ci: CiResult {
                state: CiState::Success,
                failed_checks: Vec::new(),
            },
            comments: Vec::<Comment>::new(),
            reviews: Vec::new(),
            review_comments: Vec::new(),
        }
    }

    fn state() -> WatcherState {
        let watched = WatchedPullRequest::from_baseline(&authored(), "checked".to_string());
        let mut state = WatcherState {
            version: 2,
            accounts: BTreeMap::new(),
            activity: vec![
                ActivityRecord {
                    id: "a1".to_string(),
                    key: authored().key,
                    observed_at: "2026-11-02T10:00:00Z".to_string(),
                    event: ActivityEvent::Comment {
                        author: "alice".to_string(),
                        body_excerpt: "hi".to_string(),
                    },
                    read: false,
                },
                ActivityRecord {
                    id: "a2".to_string(),
                    key: authored().key,
                    observed_at: "2026-11-02T10:01:00Z".to_string(),
                    event: ActivityEvent::CiRecovery,
                    read: true,
                },
            ],
            next_activity_id: 2,
            last_scope: None,
        };
        state.accounts.insert(
            "personal".to_string(),
            AccountState {
                login: "octo-personal".to_string(),
                last_check_at: Some("2026-11-02T10:00:00Z".to_string()),
                pull_requests: [("acme/api#42".to_string(), watched)].into(),
            },
        );
        state
    }

    fn store(path: &std::path::Path) -> Arc<dyn SnapshotStore> {
        std::fs::write(path, serde_json::to_string(&state()).unwrap()).unwrap();
        Arc::new(JsonStateStore::new(path))
    }

    #[tokio::test]
    async fn view_merges_accounts_and_orders_activity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let inbox = Inbox::new(store(&path));
        let view = inbox.view(&config()).await.unwrap();
        assert_eq!(view.accounts[0].login.as_deref(), Some("octo-personal"));
        assert_eq!(view.activity[0].id, "a2");
        assert_eq!(view.activity[1].id, "a1");
        assert_eq!(view.unread, 1);
        assert_eq!(view.pull_requests[0].key.to_string(), "acme/api#42");
        assert_eq!(view.pull_requests[0].title, "Pull request acme/api#42");
    }

    #[tokio::test]
    async fn mark_read_is_per_event_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let inbox = Inbox::new(store(&path));
        let outcome = inbox.mark_read("a1").await.unwrap();
        assert!(outcome.found);
        assert_eq!(outcome.changed, 1);
        assert_eq!(outcome.unread, 0);
        let view = inbox.view(&config()).await.unwrap();
        assert_eq!(view.unread, 0);
        assert_eq!(view.activity.len(), 2);

        let unknown = inbox.mark_read("a999").await.unwrap();
        assert!(!unknown.found);
        assert_eq!(unknown.changed, 0);
    }

    #[tokio::test]
    async fn missing_state_is_empty_and_mark_all_does_not_create_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let inbox = Inbox::new(Arc::new(JsonStateStore::new(&path)));
        let view = inbox.view(&config()).await.unwrap();
        assert!(view.activity.is_empty());
        assert_eq!(view.unread, 0);
        let outcome = inbox.mark_all_read().await.unwrap();
        assert_eq!(outcome.changed, 0);
        assert!(!path.exists());
    }
    #[tokio::test]
    async fn view_hides_out_of_scope_records_and_counts_visible_unread() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonStateStore::new(dir.path().join("state.json"));
        let mut state = WatcherState::default();
        state.version = 2;
        let mut account = crate::domain::AccountState {
            login: "octo".to_string(),
            last_check_at: None,
            pull_requests: Default::default(),
        };
        let mut watched =
            WatchedPullRequest::from_baseline(&authored(), "2026-11-02T10:00:00Z".to_string());
        watched.repository = "acme/api".to_string();
        watched.number = 42;
        account
            .pull_requests
            .insert("acme/api#42".to_string(), watched.clone());
        let mut other = watched.clone();
        other.repository = "acme/web".to_string();
        other.number = 7;
        account
            .pull_requests
            .insert("acme/web#7".to_string(), other);
        state.accounts.insert("personal".to_string(), account);
        state.activity = vec![
            ActivityRecord {
                id: "a1".to_string(),
                key: PullRequestKey::new("acme/api".to_string(), 42),
                observed_at: "2026-11-02T10:00:00Z".to_string(),
                event: crate::domain::ActivityEvent::Comment {
                    author: "alice".to_string(),
                    body_excerpt: "hi".to_string(),
                },
                read: false,
            },
            ActivityRecord {
                id: "a2".to_string(),
                key: PullRequestKey::new("acme/web".to_string(), 7),
                observed_at: "2026-11-02T10:00:00Z".to_string(),
                event: crate::domain::ActivityEvent::Comment {
                    author: "bob".to_string(),
                    body_excerpt: "hi".to_string(),
                },
                read: false,
            },
        ];
        store.save(&state).await.unwrap();
        let config = CoreConfig {
            accounts: Default::default(),
            events: Default::default(),
            watch: passuff_config::WatchConfig {
                mode: passuff_config::WatchMode::Selected,
                repositories: vec!["acme/api".to_string()],
                ignored_pull_requests: Vec::new(),
            },
        };
        let view = Inbox::new(Arc::new(store))
            .view(&config)
            .await
            .expect("view");
        assert_eq!(view.pull_requests.len(), 1);
        assert_eq!(view.activity.len(), 1);
        assert_eq!(view.unread, 1);
        assert_eq!(view.scope_label, "acme/api");
    }
}
