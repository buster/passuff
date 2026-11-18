//! Read-model helpers shared by the CLI and the terminal inbox.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{ActivityRecord, PullRequestKey, WatchedPullRequest, WatcherState};
use passuff_config::{AppConfig, Relation, WatchMode};

pub fn ignored_keys(config: &AppConfig) -> BTreeSet<String> {
    config
        .watch
        .ignored_pull_requests
        .iter()
        .map(|key| key.to_ascii_lowercase())
        .collect()
}

pub fn is_ignored(config: &AppConfig, key: &str) -> bool {
    config
        .watch
        .ignored_pull_requests
        .iter()
        .any(|ignored| ignored.eq_ignore_ascii_case(key))
}

pub fn scope_allows(config: &AppConfig, key: &str) -> bool {
    let Some(key) = PullRequestKey::parse(key) else {
        return false;
    };
    match config.watch.effective_mode() {
        WatchMode::All => true,
        WatchMode::Selected => {
            let repository = key.repository_name();
            config
                .watch
                .repositories
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(&repository))
        }
    }
}

pub fn watched_in_focus(
    config: &AppConfig,
    watched: &WatchedPullRequest,
    login_set: &BTreeSet<String>,
) -> bool {
    if watched.relations_known && !watched.relations.is_empty() {
        return watched
            .relations
            .iter()
            .any(|relation| config.focus.contains(*relation));
    }
    // A relation that is not recorded but derivable from the author still
    // filters; a completely unknown relation leaves the row visible.
    if let Some(author) = &watched.author {
        if login_set.contains(&author.to_ascii_lowercase()) {
            return config.focus.contains(Relation::Authored);
        }
    }
    true
}

pub fn watched_visible(
    config: &AppConfig,
    watched: &WatchedPullRequest,
    login_set: &BTreeSet<String>,
) -> bool {
    scope_allows(config, &watched.key)
        && watched_in_focus(config, watched, login_set)
        && !is_ignored(config, &watched.key)
}

pub fn known_repositories(config: &AppConfig, state: &WatcherState) -> Vec<String> {
    let mut repositories: BTreeMap<String, String> = BTreeMap::new();
    for repository in &config.watch.repositories {
        repositories
            .entry(repository.to_ascii_lowercase())
            .or_insert_with(|| repository.clone());
    }
    for repository in state.known_repositories() {
        repositories
            .entry(repository.to_ascii_lowercase())
            .or_insert(repository);
    }
    repositories.into_values().collect()
}

pub fn scope_label(config: &AppConfig) -> String {
    match config.watch.effective_mode() {
        WatchMode::All => "all authored pull requests".to_string(),
        WatchMode::Selected if config.watch.repositories.is_empty() => {
            "watching nothing".to_string()
        }
        WatchMode::Selected => config.watch.repositories.join(", "),
    }
}

pub fn relation_label(relation: Relation) -> &'static str {
    match relation {
        Relation::Authored => "authored",
        Relation::Assigned => "assigned",
        Relation::ReviewRequested => "review requested",
        Relation::TeamReviewRequested => "team review",
    }
}

pub fn activity_visible(
    config: &AppConfig,
    watched: &BTreeMap<String, &WatchedPullRequest>,
    login_set: &BTreeSet<String>,
    record: &ActivityRecord,
) -> bool {
    if is_ignored(config, &record.key) {
        return false;
    }
    if !scope_allows(config, &record.key) {
        return false;
    }
    match watched.get(&record.key) {
        Some(pull_request) => watched_in_focus(config, pull_request, login_set),
        // A retained record without its pull request is kept when the scope
        // permits it; relations are unknown and therefore visible.
        None => true,
    }
}

/// Visible activity records, newest first. The order follows the displayed
/// time: occurrence time when reported, fetch time otherwise.
pub fn visible_activity<'a>(
    config: &AppConfig,
    state: &'a WatcherState,
) -> Vec<&'a ActivityRecord> {
    let watched = state.watched_by_key();
    let login_set = state.login_set();
    let mut records: Vec<&ActivityRecord> = state
        .activity
        .iter()
        .filter(|record| activity_visible(config, &watched, &login_set, record))
        .collect();
    records.sort_by(|left, right| {
        right
            .shown_time()
            .cmp(&left.shown_time())
            .then_with(|| right.observed_at.cmp(&left.observed_at))
            .then_with(|| activity_number(&right.id).cmp(&activity_number(&left.id)))
    });
    records
}

fn activity_number(id: &str) -> u64 {
    id.strip_prefix('a')
        .and_then(|number| number.parse().ok())
        .unwrap_or(0)
}

pub fn unread_counts(config: &AppConfig, state: &WatcherState) -> (usize, usize) {
    let records = visible_activity(config, state);
    let unread = records.iter().filter(|record| !record.read).count();
    (unread, records.len())
}
