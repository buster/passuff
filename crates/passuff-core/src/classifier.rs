use chrono::DateTime;
use passuff_config::EventToggles;

use crate::domain::{
    ActivityEvent, AuthoredPullRequest, Baseline, CiState, CurrentObservation, LoginSet, PrState,
    WatchedPullRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedEvent {
    pub event: ActivityEvent,
    pub occurred_at: Option<String>,
}

impl From<ActivityEvent> for ClassifiedEvent {
    fn from(event: ActivityEvent) -> Self {
        Self {
            event,
            occurred_at: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Classification {
    pub events: Vec<ClassifiedEvent>,
    /// Events recorded as read and never notified, such as own comments.
    pub silent: Vec<ClassifiedEvent>,
    pub baseline: Option<Baseline>,
    pub current: CurrentObservation,
}

fn occurrence(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn state_change_time(pull_request: &AuthoredPullRequest) -> Option<String> {
    match pull_request.state {
        PrState::Merged => pull_request.merged_at.clone(),
        PrState::Closed => pull_request.closed_at.clone(),
        PrState::Open => None,
    }
}

pub fn classify(
    previous: Option<&WatchedPullRequest>,
    current: &AuthoredPullRequest,
    toggles: &EventToggles,
    login_set: &LoginSet,
    account_has_prior_cycle: bool,
    last_check_at: Option<&str>,
) -> Classification {
    let Some(previous) = previous else {
        let created_since_last_check = account_has_prior_cycle
            && last_check_at.is_some_and(|last| created_after(&current.created_at, last));
        let short_lived = created_since_last_check && current.state.is_terminal();
        let mut observation = current_observation(current);
        let mut events = Vec::new();
        if short_lived {
            observation.state_reported = true;
            if toggles.state_changes {
                events.push(ClassifiedEvent {
                    occurred_at: state_change_time(current),
                    event: ActivityEvent::StateChange {
                        state: current.state.clone(),
                        actor: current.merged_by.clone(),
                    },
                });
            }
        } else if created_since_last_check
            && current.state == PrState::Open
            && toggles.state_changes
        {
            events.push(ClassifiedEvent {
                occurred_at: occurrence(&current.created_at),
                event: ActivityEvent::Opened {
                    author: current.author.clone(),
                },
            });
        }
        return Classification {
            events,
            silent: Vec::new(),
            baseline: Some(baseline(current)),
            current: observation,
        };
    };

    let mut events = Vec::new();
    let mut silent = Vec::new();
    let seen_comments: std::collections::HashSet<&str> = previous
        .current
        .seen_comment_ids
        .iter()
        .map(String::as_str)
        .collect();
    let seen_reviews: std::collections::HashSet<&str> = previous
        .current
        .seen_review_ids
        .iter()
        .map(String::as_str)
        .collect();
    let seen_review_comments: std::collections::HashSet<&str> = previous
        .current
        .seen_review_comment_ids
        .iter()
        .map(String::as_str)
        .collect();

    if toggles.comments {
        for comment in &current.comments {
            if seen_comments.contains(comment.id.as_str()) {
                continue;
            }
            let event = ClassifiedEvent {
                event: ActivityEvent::Comment {
                    author: comment.author.clone(),
                    body: comment.body.clone(),
                },
                occurred_at: occurrence(&comment.created_at),
            };
            match action_filter(&comment.author, login_set, toggles) {
                ActionFilter::Include => events.push(event),
                ActionFilter::Silent => silent.push(event),
                ActionFilter::Skip => {}
            }
        }
        for comment in &current.review_comments {
            if seen_review_comments.contains(comment.id.as_str()) {
                continue;
            }
            let event = ClassifiedEvent {
                event: ActivityEvent::ReviewComment {
                    author: comment.author.clone(),
                    body: comment.body.clone(),
                },
                occurred_at: occurrence(&comment.created_at),
            };
            match action_filter(&comment.author, login_set, toggles) {
                ActionFilter::Include => events.push(event),
                ActionFilter::Silent => silent.push(event),
                ActionFilter::Skip => {}
            }
        }
    }

    if toggles.reviews {
        for review in &current.reviews {
            if seen_reviews.contains(review.id.as_str()) {
                continue;
            }
            let event = ClassifiedEvent {
                event: ActivityEvent::Review {
                    author: review.author.clone(),
                    state: normalize_review_state(&review.state),
                    body: review.body.clone(),
                },
                occurred_at: occurrence(&review.submitted_at),
            };
            match action_filter(&review.author, login_set, toggles) {
                ActionFilter::Include => events.push(event),
                ActionFilter::Silent => silent.push(event),
                ActionFilter::Skip => {}
            }
        }
    }

    let mut ci_failed = previous.current.ci_failed;
    if previous.current.head_oid != current.head_oid {
        ci_failed = false;
        if current.ci.state.failed() {
            ci_failed = true;
            if toggles.ci_failure {
                events.push(ClassifiedEvent {
                    event: ActivityEvent::CiFailure {
                        failed_checks: current.ci.failed_checks.clone(),
                    },
                    occurred_at: None,
                });
            }
        }
    } else if current.ci.state.failed() {
        if !ci_failed {
            ci_failed = true;
            if toggles.ci_failure {
                events.push(ClassifiedEvent {
                    event: ActivityEvent::CiFailure {
                        failed_checks: current.ci.failed_checks.clone(),
                    },
                    occurred_at: None,
                });
            }
        }
    } else if matches!(current.ci.state, CiState::Success) && ci_failed {
        ci_failed = false;
        if toggles.ci_recovery {
            events.push(ClassifiedEvent {
                event: ActivityEvent::CiRecovery,
                occurred_at: None,
            });
        }
    }

    let mut state_reported = previous.current.state_reported;
    if current.state == PrState::Open {
        state_reported = false;
    } else if !state_reported {
        state_reported = true;
        if toggles.state_changes {
            events.push(ClassifiedEvent {
                occurred_at: state_change_time(current),
                event: ActivityEvent::StateChange {
                    state: current.state.clone(),
                    actor: current.merged_by.clone(),
                },
            });
        }
    }

    Classification {
        events,
        silent,
        baseline: None,
        current: CurrentObservation {
            seen_comment_ids: current.comments.iter().map(|c| c.id.clone()).collect(),
            seen_review_ids: current.reviews.iter().map(|r| r.id.clone()).collect(),
            seen_review_comment_ids: current
                .review_comments
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            head_oid: current.head_oid.clone(),
            ci_state: current.ci.state.clone(),
            ci_failed,
            state: current.state.clone(),
            state_reported,
        },
    }
}

fn baseline(current: &AuthoredPullRequest) -> Baseline {
    Baseline {
        recorded_at: current.updated_at.clone(),
        seen_comment_ids: current.comments.iter().map(|c| c.id.clone()).collect(),
        seen_review_ids: current.reviews.iter().map(|r| r.id.clone()).collect(),
        seen_review_comment_ids: current
            .review_comments
            .iter()
            .map(|c| c.id.clone())
            .collect(),
        head_oid: current.head_oid.clone(),
        ci_state: current.ci.state.clone(),
        state: current.state.clone(),
    }
}

fn current_observation(current: &AuthoredPullRequest) -> CurrentObservation {
    CurrentObservation {
        seen_comment_ids: current.comments.iter().map(|c| c.id.clone()).collect(),
        seen_review_ids: current.reviews.iter().map(|r| r.id.clone()).collect(),
        seen_review_comment_ids: current
            .review_comments
            .iter()
            .map(|c| c.id.clone())
            .collect(),
        head_oid: current.head_oid.clone(),
        ci_state: current.ci.state.clone(),
        ci_failed: current.ci.state.failed(),
        state: current.state.clone(),
        state_reported: current.state.is_terminal(),
    }
}

pub(crate) fn created_after(created_at: &str, last_check_at: &str) -> bool {
    match (
        DateTime::parse_from_rfc3339(created_at),
        DateTime::parse_from_rfc3339(last_check_at),
    ) {
        (Ok(created), Ok(last)) => created > last,
        _ => false,
    }
}

enum ActionFilter {
    Include,
    Silent,
    Skip,
}

fn action_filter(author: &str, login_set: &LoginSet, toggles: &EventToggles) -> ActionFilter {
    if login_set.contains(author) {
        ActionFilter::Silent
    } else if is_bot(author) && !toggles.bot_comments {
        ActionFilter::Skip
    } else {
        ActionFilter::Include
    }
}

fn is_bot(author: &str) -> bool {
    author.ends_with("[bot]")
}

fn normalize_review_state(state: &str) -> String {
    match state.to_ascii_uppercase().as_str() {
        "APPROVED" => "approved".to_string(),
        "CHANGES_REQUESTED" => "changes requested".to_string(),
        "COMMENTED" => "commented".to_string(),
        other => other.to_ascii_lowercase().replace('_', " "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CiResult, Comment, PullRequestKey};

    fn plain_events(events: &[ClassifiedEvent]) -> Vec<ActivityEvent> {
        events.iter().map(|event| event.event.clone()).collect()
    }

    fn pull_request(state: PrState, created_at: &str) -> AuthoredPullRequest {
        let merged = state == PrState::Merged;
        AuthoredPullRequest {
            key: PullRequestKey::new("acme/api".to_string(), 42),
            node_id: "node-42".to_string(),
            author: "octo".to_string(),
            title: "Pull request".to_string(),
            body: String::new(),
            url: "https://github.com/acme/api/pull/42".to_string(),
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            merged_at: (state == PrState::Merged).then(|| created_at.to_string()),
            closed_at: (state == PrState::Closed).then(|| created_at.to_string()),
            state,
            merged,
            is_draft: false,
            assignees: Vec::new(),
            labels: Vec::new(),
            milestone: None,
            base_ref: "main".to_string(),
            merged_by: None,
            stack: None,
            head_oid: "abc123".to_string(),
            ci: CiResult::default(),
            comments: Vec::new(),
            reviews: Vec::new(),
            review_comments: Vec::new(),
        }
    }

    fn login_set() -> LoginSet {
        LoginSet::new(vec!["octo".to_string()])
    }

    #[test]
    fn an_already_merged_baseline_is_not_reported_on_the_next_cycle() {
        let toggles = EventToggles::default();
        let merged = pull_request(PrState::Merged, "2026-11-02T09:00:00Z");
        let first = classify(None, &merged, &toggles, &login_set(), false, None);
        assert!(first.events.is_empty());
        assert!(first.current.state_reported);

        let mut watched = WatchedPullRequest::from_baseline(&merged, "checked".to_string());
        watched.current = first.current;
        let second = classify(Some(&watched), &merged, &toggles, &login_set(), true, None);
        assert!(second.events.is_empty());
    }

    #[test]
    fn an_open_baseline_merged_later_is_reported() {
        let toggles = EventToggles::default();
        let open = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        let first = classify(None, &open, &toggles, &login_set(), false, None);
        assert!(!first.current.state_reported);

        let mut watched = WatchedPullRequest::from_baseline(&open, "checked".to_string());
        watched.current = first.current;
        let merged = pull_request(PrState::Merged, "2026-11-02T09:00:00Z");
        let second = classify(Some(&watched), &merged, &toggles, &login_set(), true, None);
        assert_eq!(
            plain_events(&second.events),
            vec![ActivityEvent::StateChange {
                state: PrState::Merged,
                actor: None
            }]
        );
    }

    #[test]
    fn own_comments_are_recorded_as_silent_events() {
        let toggles = EventToggles::default();
        let open = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        let first = classify(None, &open, &toggles, &login_set(), false, None);
        let mut watched = WatchedPullRequest::from_baseline(&open, "checked".to_string());
        watched.current = first.current;
        let mut commented = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        commented.comments.push(Comment {
            id: "own-comment".to_string(),
            author: "octo".to_string(),
            body: "my own note".to_string(),
            created_at: "2026-11-02T09:30:00Z".to_string(),
        });
        let second = classify(
            Some(&watched),
            &commented,
            &toggles,
            &login_set(),
            true,
            None,
        );
        assert!(second.events.is_empty());
        assert_eq!(
            plain_events(&second.silent),
            vec![ActivityEvent::Comment {
                author: "octo".to_string(),
                body: "my own note".to_string()
            }]
        );
    }

    #[test]
    fn a_pull_request_opened_since_the_last_check_is_reported() {
        let toggles = EventToggles::default();
        let last = "2026-11-02T10:00:00Z";
        let opened = pull_request(PrState::Open, "2026-11-02T10:00:01Z");
        let classification = classify(None, &opened, &toggles, &login_set(), true, Some(last));
        assert_eq!(
            plain_events(&classification.events),
            vec![ActivityEvent::Opened {
                author: "octo".to_string()
            }]
        );
    }

    #[test]
    fn a_pull_request_created_before_the_last_check_is_a_baseline() {
        let toggles = EventToggles::default();
        let last = "2026-11-02T10:00:00Z";
        let existing = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        let classification = classify(None, &existing, &toggles, &login_set(), true, Some(last));
        assert!(classification.events.is_empty());
    }

    #[test]
    fn short_lived_uses_rfc3339_instants_not_string_order() {
        let toggles = EventToggles::default();
        let created_same_second = pull_request(PrState::Merged, "2026-11-02T10:00:00Z");
        let last = "2026-11-02T10:00:00.500+00:00";
        let same_second = classify(
            None,
            &created_same_second,
            &toggles,
            &login_set(),
            true,
            Some(last),
        );
        assert!(same_second.events.is_empty());

        let created_later = pull_request(PrState::Merged, "2026-11-02T10:00:01Z");
        let later = classify(
            None,
            &created_later,
            &toggles,
            &login_set(),
            true,
            Some(last),
        );
        assert_eq!(
            plain_events(&later.events),
            vec![ActivityEvent::StateChange {
                state: PrState::Merged,
                actor: None
            }]
        );
    }

    #[test]
    fn a_comment_carries_its_creation_time() {
        let toggles = EventToggles::default();
        let open = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        let first = classify(None, &open, &toggles, &login_set(), false, None);
        let mut watched = WatchedPullRequest::from_baseline(&open, "checked".to_string());
        watched.current = first.current;
        let mut commented = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        commented.comments.push(Comment {
            id: "comment-1".to_string(),
            author: "alice".to_string(),
            body: "hello".to_string(),
            created_at: "2026-11-02T09:15:00Z".to_string(),
        });
        let second = classify(Some(&watched), &commented, &toggles, &login_set(), true, None);
        assert_eq!(second.events.len(), 1);
        assert_eq!(
            second.events[0].occurred_at.as_deref(),
            Some("2026-11-02T09:15:00Z")
        );
    }

    #[test]
    fn a_review_without_a_submitted_time_has_no_occurrence_time() {
        let toggles = EventToggles::default();
        let open = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        let first = classify(None, &open, &toggles, &login_set(), false, None);
        let mut watched = WatchedPullRequest::from_baseline(&open, "checked".to_string());
        watched.current = first.current;
        let mut reviewed = pull_request(PrState::Open, "2026-11-02T09:00:00Z");
        reviewed.reviews.push(crate::domain::Review {
            id: "review-1".to_string(),
            author: "alice".to_string(),
            state: "APPROVED".to_string(),
            body: String::new(),
            submitted_at: String::new(),
        });
        let second = classify(Some(&watched), &reviewed, &toggles, &login_set(), true, None);
        assert_eq!(second.events.len(), 1);
        assert_eq!(second.events[0].occurred_at, None);
    }

    #[test]
    fn invalid_timestamps_do_not_create_a_short_lived_notification() {
        assert!(!created_after("not-a-date", "2026-11-02T10:00:00Z"));
    }
}
