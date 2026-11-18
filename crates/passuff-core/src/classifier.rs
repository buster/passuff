//! Activity classification and deduplication.
//!
//! Classification compares one observation with a watched pull request's
//! recorded observation. It records one activity record per event and reports
//! at most one notification group per pull request per cycle.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use passuff_config::{EventToggles, FocusConfig, Relation};

use crate::github::{CommentObservation, PullRequestObservation, ReviewObservation};
use crate::model::{
    ActivityKind, ActivityRecord, CiResult, PrState, PullRequestKey, StackInfo, WatchedPullRequest,
    WatcherState,
};

pub struct ClassifyContext<'a> {
    pub now: DateTime<Utc>,
    /// Lower-cased resolved logins.
    pub login_set: &'a BTreeSet<String>,
    pub focus: &'a FocusConfig,
    pub events: &'a EventToggles,
    /// Lower-cased `owner/repo#number`.
    pub ignored: &'a BTreeSet<String>,
    /// Lower-cased `owner/repo` awaiting a silent catch-up.
    pub catch_up: &'a BTreeSet<String>,
}

#[derive(Debug, Default)]
pub struct ClassifyResult {
    pub baseline: bool,
    pub notified: Vec<String>,
}

struct EventDraft {
    kind: ActivityKind,
    summary: String,
    display: String,
    event_id: String,
    occurred_at: Option<DateTime<Utc>>,
    body: Option<String>,
    author: Option<String>,
    notify: bool,
    read: bool,
}

pub fn apply(
    state: &mut WatcherState,
    account: &str,
    observation: &PullRequestObservation,
    relations: &[Relation],
    last_check: Option<DateTime<Utc>>,
    context: &ClassifyContext<'_>,
) -> ClassifyResult {
    let mut result = ClassifyResult::default();
    let repository = PullRequestKey::parse(&observation.key)
        .map(|key| key.repository_name().to_ascii_lowercase())
        .unwrap_or_default();
    let ignored = context
        .ignored
        .contains(&observation.key.to_ascii_lowercase());
    let catch_up = context.catch_up.contains(&repository);
    let now = context.now;

    let existing = state
        .accounts
        .entry(account.to_string())
        .or_default()
        .pull_requests
        .remove(&observation.key);

    let mut entry = match existing {
        Some(mut entry) => {
            if entry.state.is_terminal()
                && observation.state.is_terminal()
                && entry
                    .terminal_at
                    .is_some_and(|terminal| now.signed_duration_since(terminal).num_days() >= 30)
            {
                // Terminal retirement: a closed or merged pull request stops
                // being an observation target after 30 days.
                state
                    .accounts
                    .entry(account.to_string())
                    .or_default()
                    .pull_requests
                    .insert(observation.key.clone(), entry);
                return result;
            }

            let old_state = entry.state;
            let reopened = old_state.is_terminal() && observation.state == PrState::Open;
            let became_terminal = old_state == PrState::Open && observation.state.is_terminal();
            let was_terminal = old_state.is_terminal();

            update_fields(&mut entry, observation);
            merge_relations(&mut entry, relations);
            entry.last_observed = Some(now);

            if reopened {
                entry.state_change_reported = false;
                entry.terminal_at = None;
                entry.seen_comments = observation
                    .comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect();
                entry.seen_reviews = observation
                    .reviews
                    .iter()
                    .map(|review| review.id.clone())
                    .collect();
                entry.seen_review_comments = observation
                    .review_comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect();
            } else if became_terminal {
                entry.terminal_at = Some(now);
                if !entry.state_change_reported {
                    entry.state_change_reported = true;
                    let summary = match observation.state {
                        PrState::Merged => "Merged",
                        PrState::Closed => "Closed",
                        PrState::Open => unreachable!(),
                    };
                    let occurred = match observation.state {
                        PrState::Merged => observation.merged_at,
                        PrState::Closed => observation.closed_at,
                        PrState::Open => None,
                    };
                    let notify = !ignored
                        && !catch_up
                        && entry.in_focus(context.focus)
                        && context.events.state_changes;
                    let kind = if observation.state == PrState::Merged {
                        ActivityKind::Merged
                    } else {
                        ActivityKind::Closed
                    };
                    record(
                        state,
                        &entry,
                        now,
                        EventDraft {
                            kind,
                            summary: summary.to_string(),
                            display: summary.to_string(),
                            event_id: format!(
                                "state:{}:{}",
                                observation.state.human(),
                                now.timestamp_millis()
                            ),
                            occurred_at: occurred,
                            body: None,
                            author: None,
                            notify,
                            read: false,
                        },
                        &mut result,
                    );
                }
                // Comments and reviews that arrive with the terminal
                // transition are not activity: only the transition (or a
                // reopen) is reported.
                entry.seen_comments = observation
                    .comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect();
                entry.seen_reviews = observation
                    .reviews
                    .iter()
                    .map(|review| review.id.clone())
                    .collect();
                entry.seen_review_comments = observation
                    .review_comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect();
            } else if !was_terminal && !observation.state.is_terminal() {
                classify_comments(
                    state,
                    &mut entry,
                    &observation.comments,
                    ActivityKind::Comment,
                    context,
                    ignored,
                    catch_up,
                    now,
                    &mut result,
                );
                classify_comments(
                    state,
                    &mut entry,
                    &observation.review_comments,
                    ActivityKind::ReviewComment,
                    context,
                    ignored,
                    catch_up,
                    now,
                    &mut result,
                );
                classify_reviews(
                    state,
                    &mut entry,
                    &observation.reviews,
                    context,
                    ignored,
                    catch_up,
                    now,
                    &mut result,
                );
                classify_ci(
                    state,
                    &mut entry,
                    context,
                    ignored,
                    catch_up,
                    now,
                    &mut result,
                );
            } else {
                // Terminal pull request: keep the seen sets current so a
                // reopen starts from the present state.
                entry.seen_comments = observation
                    .comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect();
                entry.seen_reviews = observation
                    .reviews
                    .iter()
                    .map(|review| review.id.clone())
                    .collect();
                entry.seen_review_comments = observation
                    .review_comments
                    .iter()
                    .map(|comment| comment.id.clone())
                    .collect();
            }

            entry
        }
        None => {
            let created_after_last_check = last_check
                .zip(observation.created_at)
                .is_some_and(|(last_check, created)| created > last_check);
            let authored = observation
                .author
                .as_ref()
                .is_some_and(|author| context.login_set.contains(&author.to_ascii_lowercase()));

            let mut entry = WatchedPullRequest::base_entry(
                &observation.key,
                observation.state,
                observation.ci,
                &observation.title,
            );
            update_fields(&mut entry, observation);
            merge_relations(&mut entry, relations);
            entry.last_observed = Some(now);
            entry.seen_comments = observation
                .comments
                .iter()
                .map(|comment| comment.id.clone())
                .collect();
            entry.seen_reviews = observation
                .reviews
                .iter()
                .map(|review| review.id.clone())
                .collect();
            entry.seen_review_comments = observation
                .review_comments
                .iter()
                .map(|comment| comment.id.clone())
                .collect();
            entry.was_failing = observation.ci.is_failing();
            entry.state_change_reported = observation.state.is_terminal();
            if observation.state.is_terminal() {
                entry.terminal_at = Some(now);
            }

            result.baseline = true;

            if created_after_last_check && authored && !ignored {
                if observation.state == PrState::Open {
                    let notify = !catch_up && entry.in_focus(context.focus);
                    let author = observation.author.clone().unwrap_or_default();
                    record(
                        state,
                        &entry,
                        now,
                        EventDraft {
                            kind: ActivityKind::NewPullRequest,
                            summary: format!("New pull request by @{author}"),
                            display: format!("new pull request by @{author}"),
                            event_id: "new-pull-request".to_string(),
                            occurred_at: observation.created_at,
                            body: observation.body.clone(),
                            author: observation.author.clone(),
                            notify,
                            read: false,
                        },
                        &mut result,
                    );
                } else {
                    let summary = match observation.state {
                        PrState::Merged => "Merged",
                        PrState::Closed => "Closed",
                        PrState::Open => unreachable!(),
                    };
                    let kind = if observation.state == PrState::Merged {
                        ActivityKind::Merged
                    } else {
                        ActivityKind::Closed
                    };
                    let notify =
                        !catch_up && entry.in_focus(context.focus) && context.events.state_changes;
                    let occurred = match observation.state {
                        PrState::Merged => observation.merged_at,
                        PrState::Closed => observation.closed_at,
                        PrState::Open => None,
                    };
                    record(
                        state,
                        &entry,
                        now,
                        EventDraft {
                            kind,
                            summary: summary.to_string(),
                            display: summary.to_string(),
                            event_id: format!(
                                "short-lived:{}:{}",
                                observation.state.human(),
                                now.timestamp_millis()
                            ),
                            occurred_at: occurred,
                            body: None,
                            author: None,
                            notify,
                            read: false,
                        },
                        &mut result,
                    );
                    entry.state_change_reported = true;
                }
            }

            entry
        }
    };

    merge_relations(&mut entry, relations);
    state
        .accounts
        .entry(account.to_string())
        .or_default()
        .pull_requests
        .insert(observation.key.clone(), entry);
    state.cap_activity();
    result
}

fn update_fields(entry: &mut WatchedPullRequest, observation: &PullRequestObservation) {
    entry.state = observation.state;
    entry.ci = observation.ci;
    entry.title = observation.title.clone();
    entry.body = observation.body.clone();
    entry.url = observation.url.clone();
    entry.author = observation.author.clone();
    entry.author_is_bot = observation.author_is_bot;
    entry.assignees = observation.assignees.clone();
    entry.is_draft = observation.is_draft;
    entry.created_at = observation.created_at;
    entry.updated_at = observation.updated_at;
    entry.merged_at = observation.merged_at;
    entry.closed_at = observation.closed_at;
    entry.stack = observation.stack.clone();
    entry.node_id = observation.node_id.clone();
}

fn merge_relations(entry: &mut WatchedPullRequest, relations: &[Relation]) {
    if !relations.is_empty() {
        entry.relations_known = true;
        for relation in relations {
            if !entry.relations.contains(relation) {
                entry.relations.push(*relation);
            }
        }
        entry.relations.sort_by_key(|relation| match relation {
            Relation::Authored => 0,
            Relation::Assigned => 1,
            Relation::ReviewRequested => 2,
            Relation::TeamReviewRequested => 3,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_comments(
    state: &mut WatcherState,
    entry: &mut WatchedPullRequest,
    comments: &[CommentObservation],
    kind: ActivityKind,
    context: &ClassifyContext<'_>,
    ignored: bool,
    catch_up: bool,
    now: DateTime<Utc>,
    result: &mut ClassifyResult,
) {
    for comment in comments {
        let already_seen = match kind {
            ActivityKind::Comment => entry.seen_comments.contains(&comment.id),
            _ => entry.seen_review_comments.contains(&comment.id),
        };
        if already_seen {
            continue;
        }
        match kind {
            ActivityKind::Comment => {
                entry.seen_comments.insert(comment.id.clone());
            }
            _ => {
                entry.seen_review_comments.insert(comment.id.clone());
            }
        }
        if ignored {
            continue;
        }
        if comment.author_is_bot && !context.events.bot_comments {
            continue;
        }
        let author = comment
            .author
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let own = context.login_set.contains(&author.to_ascii_lowercase());
        let (summary, display) = match kind {
            ActivityKind::Comment => (
                format!("New comment from @{author}"),
                format!("comment by @{author}"),
            ),
            _ => (
                format!("Review comment from @{author}"),
                format!("review comment by @{author}"),
            ),
        };
        let family_enabled = context.events.comments;
        let notify = !own && family_enabled && !catch_up && entry.in_focus(context.focus);
        record(
            state,
            entry,
            now,
            EventDraft {
                kind: match kind {
                    ActivityKind::Comment => ActivityKind::Comment,
                    _ => ActivityKind::ReviewComment,
                },
                summary,
                display,
                event_id: comment.id.clone(),
                occurred_at: comment.created_at,
                body: comment.body.clone(),
                author: Some(author),
                notify,
                read: own,
            },
            result,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_reviews(
    state: &mut WatcherState,
    entry: &mut WatchedPullRequest,
    reviews: &[ReviewObservation],
    context: &ClassifyContext<'_>,
    ignored: bool,
    catch_up: bool,
    now: DateTime<Utc>,
    result: &mut ClassifyResult,
) {
    for review in reviews {
        if entry.seen_reviews.contains(&review.id) {
            continue;
        }
        entry.seen_reviews.insert(review.id.clone());
        if ignored {
            continue;
        }
        if review.author_is_bot && !context.events.bot_comments {
            continue;
        }
        let author = review
            .author
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let state_label = review_state(&review.state);
        let own = context.login_set.contains(&author.to_ascii_lowercase());
        let notify = !own && context.events.reviews && !catch_up && entry.in_focus(context.focus);
        record(
            state,
            entry,
            now,
            EventDraft {
                kind: ActivityKind::Review,
                summary: format!("Review {state_label} by @{author}"),
                display: format!("review {state_label} by @{author}"),
                event_id: review.id.clone(),
                occurred_at: review.submitted_at,
                body: review.body.clone(),
                author: Some(author),
                notify,
                read: own,
            },
            result,
        );
    }
}

fn classify_ci(
    state: &mut WatcherState,
    entry: &mut WatchedPullRequest,
    context: &ClassifyContext<'_>,
    ignored: bool,
    catch_up: bool,
    now: DateTime<Utc>,
    result: &mut ClassifyResult,
) {
    if ignored {
        entry.was_failing = entry.ci.is_failing();
        return;
    }
    match entry.ci {
        CiResult::Failing if !entry.was_failing => {
            entry.was_failing = true;
            let notify = !catch_up && entry.in_focus(context.focus) && context.events.ci_failure;
            record(
                state,
                entry,
                now,
                EventDraft {
                    kind: ActivityKind::CiFailure,
                    summary: "CI failing: build".to_string(),
                    display: "CI failing: build".to_string(),
                    event_id: format!("ci-failure:{}", now.timestamp_millis()),
                    occurred_at: None,
                    body: None,
                    author: None,
                    notify,
                    read: false,
                },
                result,
            );
        }
        CiResult::Passing if entry.was_failing => {
            entry.was_failing = false;
            let notify = !catch_up && entry.in_focus(context.focus) && context.events.ci_recovery;
            record(
                state,
                entry,
                now,
                EventDraft {
                    kind: ActivityKind::CiRecovery,
                    summary: "CI recovered".to_string(),
                    display: "CI recovered".to_string(),
                    event_id: format!("ci-recovery:{}", now.timestamp_millis()),
                    occurred_at: None,
                    body: None,
                    author: None,
                    notify,
                    read: false,
                },
                result,
            );
        }
        CiResult::Failing => {
            entry.was_failing = true;
        }
        CiResult::Passing | CiResult::Pending | CiResult::NotStarted => {}
    }
}

fn record(
    state: &mut WatcherState,
    entry: &WatchedPullRequest,
    observed_at: DateTime<Utc>,
    draft: EventDraft,
    result: &mut ClassifyResult,
) {
    if state.activity_contains(&entry.key, &draft.event_id) {
        return;
    }
    let id = state.allocate_activity_id();
    let stack: Option<StackInfo> = entry.stack.clone();
    state.activity.push(ActivityRecord {
        id,
        key: entry.key.clone(),
        kind: draft.kind,
        summary: draft.summary.clone(),
        display: draft.display,
        event_id: draft.event_id,
        occurred_at: draft.occurred_at,
        observed_at,
        read: draft.read,
        stack,
        body: draft.body,
        author: draft.author,
    });
    if draft.notify {
        result.notified.push(draft.summary);
    }
}

pub fn review_state(value: &str) -> &'static str {
    match value {
        "APPROVED" => "approved",
        "CHANGES_REQUESTED" => "changes requested",
        "COMMENTED" => "commented",
        "PENDING" => "pending",
        "DISMISSED" => "dismissed",
        _ => "commented",
    }
}
