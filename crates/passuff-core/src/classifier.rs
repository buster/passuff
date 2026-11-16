use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use passuff_config::{EventToggles, FocusRelation};

use crate::model::{
    ActivityEvent, CiResult, EventKind, PullRequestSnapshot, PullRequestState, ReviewState,
};
use crate::state::WatchedPullRequest;

#[derive(Debug, Clone)]
pub struct ClassifiedEvent {
    pub event: ActivityEvent,
    pub notify: bool,
}

#[derive(Debug, Clone)]
pub struct Classification {
    pub events: Vec<ClassifiedEvent>,
    pub updated: WatchedPullRequest,
}

#[derive(Debug, Clone, Copy)]
pub struct ClassificationContext<'a> {
    pub last_check: Option<DateTime<Utc>>,
    pub logins: &'a [String],
    pub events: &'a EventToggles,
    pub focus_allows: bool,
    pub catch_up: bool,
    pub ignored: bool,
}

pub fn classify(
    previous: Option<&WatchedPullRequest>,
    snapshot: &PullRequestSnapshot,
    context: ClassificationContext<'_>,
) -> Classification {
    let mut updated = previous.cloned().unwrap_or_default();
    let first = previous.is_none() || !previous.is_some_and(|record| record.baseline);
    let mut events = Vec::new();

    let mut relations = snapshot.relations.clone();
    if context
        .logins
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
    updated.relations = relations.clone();
    updated.state = snapshot.state;
    updated.ci = snapshot.ci;
    if snapshot.ci.is_failing() {
        updated.ci_failure_seen = true;
    }

    for comment in &snapshot.comments {
        updated.seen_comments.insert(comment.id.clone());
    }
    for review in &snapshot.reviews {
        updated.seen_reviews.insert(review.id.clone());
    }
    for comment in &snapshot.review_comments {
        updated.seen_review_comments.insert(comment.id.clone());
    }

    if context.ignored {
        updated.terminal_reported = snapshot.state.is_terminal();
        return Classification { events, updated };
    }

    let push = |events: &mut Vec<ClassifiedEvent>, event: ActivityEvent, notify: bool| {
        events.push(ClassifiedEvent { event, notify });
    };

    if first {
        let created_after_last_check = match (snapshot.created_at, context.last_check) {
            (Some(created), Some(last_check)) => created > last_check,
            _ => false,
        };
        updated.terminal_reported = snapshot.state.is_terminal();
        if created_after_last_check && !context.catch_up {
            match snapshot.state {
                PullRequestState::Merged | PullRequestState::Closed => {
                    let (kind, summary, occurred_at) = terminal_event(snapshot);
                    if context.events.state_changes {
                        push(
                            &mut events,
                            ActivityEvent {
                                kind,
                                summary: summary.to_string(),
                                occurred_at,
                                body: None,
                                author: None,
                                read: false,
                            },
                            context.focus_allows && !context.catch_up,
                        );
                    }
                }
                PullRequestState::Open => {
                    if context.events.state_changes {
                        push(
                            &mut events,
                            ActivityEvent {
                                kind: EventKind::Opened,
                                summary: format!("New pull request by @{}", snapshot.author),
                                occurred_at: snapshot.created_at,
                                body: None,
                                author: Some(snapshot.author.clone()),
                                read: false,
                            },
                            context.focus_allows && !context.catch_up,
                        );
                    }
                }
            }
        }
        return Classification { events, updated };
    }

    let previous_state = previous
        .map(|record| record.state)
        .unwrap_or(PullRequestState::Open);
    if previous_state.is_terminal() {
        if snapshot.state == PullRequestState::Open {
            updated.terminal_reported = false;
        }
        return Classification { events, updated };
    }

    if snapshot.state.is_terminal() {
        let (kind, summary, occurred_at) = terminal_event(snapshot);
        if context.events.state_changes {
            push(
                &mut events,
                ActivityEvent {
                    kind,
                    summary: summary.to_string(),
                    occurred_at,
                    body: None,
                    author: None,
                    read: false,
                },
                context.focus_allows && !context.catch_up,
            );
        }
        updated.terminal_reported = true;
        return Classification { events, updated };
    }

    let previous = previous.expect("a non-first observation has a previous record");
    let focus = context.focus_allows;

    for comment in &snapshot.comments {
        if previous.seen_comments.contains(&comment.id) {
            continue;
        }
        if comment.is_bot && !context.events.bot_comments {
            continue;
        }
        if !context.events.comments {
            continue;
        }
        let own = context
            .logins
            .iter()
            .any(|login| login.eq_ignore_ascii_case(&comment.author));
        let notify = !own && focus && !context.catch_up;
        let event = ActivityEvent {
            kind: EventKind::Comment,
            summary: format!("New comment from @{}", comment.author),
            occurred_at: comment.created_at,
            body: Some(comment.body.clone()),
            author: Some(comment.author.clone()),
            read: own,
        };
        push(&mut events, event, notify);
    }

    for comment in &snapshot.review_comments {
        if previous.seen_review_comments.contains(&comment.id) {
            continue;
        }
        if comment.is_bot && !context.events.bot_comments {
            continue;
        }
        if !context.events.comments {
            continue;
        }
        let own = context
            .logins
            .iter()
            .any(|login| login.eq_ignore_ascii_case(&comment.author));
        let notify = !own && focus && !context.catch_up;
        let event = ActivityEvent {
            kind: EventKind::ReviewComment,
            summary: format!("Review comment from @{}", comment.author),
            occurred_at: comment.created_at,
            body: Some(comment.body.clone()),
            author: Some(comment.author.clone()),
            read: own,
        };
        push(&mut events, event, notify);
    }

    for review in &snapshot.reviews {
        if previous.seen_reviews.contains(&review.id) {
            continue;
        }
        if review.is_bot && !context.events.bot_comments {
            continue;
        }
        if !context.events.reviews {
            continue;
        }
        let own = context
            .logins
            .iter()
            .any(|login| login.eq_ignore_ascii_case(&review.author));
        let notify = !own && focus && !context.catch_up;
        let event = ActivityEvent {
            kind: EventKind::Review {
                state: review.state,
            },
            summary: format!("Review {} by @{}", review.state.label(), review.author),
            occurred_at: review.submitted_at,
            body: Some(review.body.clone()),
            author: Some(review.author.clone()),
            read: own,
        };
        push(&mut events, event, notify);
    }

    if snapshot.ci == CiResult::Failing && !previous.ci.is_failing() && context.events.ci_failure {
        push(
            &mut events,
            ActivityEvent {
                kind: EventKind::CiFailure,
                summary: "CI failing: build".to_string(),
                occurred_at: None,
                body: None,
                author: None,
                read: false,
            },
            focus && !context.catch_up,
        );
    }
    if snapshot.ci == CiResult::Passing && previous.ci_failure_seen {
        if context.events.ci_recovery {
            push(
                &mut events,
                ActivityEvent {
                    kind: EventKind::CiRecovered,
                    summary: "CI recovered".to_string(),
                    occurred_at: None,
                    body: None,
                    author: None,
                    read: false,
                },
                focus && !context.catch_up,
            );
        }
        updated.ci_failure_seen = false;
    }

    Classification { events, updated }
}

fn terminal_event(
    snapshot: &PullRequestSnapshot,
) -> (EventKind, &'static str, Option<DateTime<Utc>>) {
    match snapshot.state {
        PullRequestState::Merged => (
            EventKind::Merged,
            "Merged",
            snapshot
                .merged_at
                .or(snapshot.closed_at)
                .or(snapshot.created_at),
        ),
        PullRequestState::Closed => (
            EventKind::Closed,
            "Closed",
            snapshot
                .closed_at
                .or(snapshot.merged_at)
                .or(snapshot.created_at),
        ),
        PullRequestState::Open => (EventKind::Reopened, "", snapshot.created_at),
    }
}

pub fn review_state_label(state: ReviewState) -> &'static str {
    state.label()
}

#[allow(dead_code)]
fn _assert_sets_are_used(_: &BTreeSet<String>) {}
