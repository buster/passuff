use chrono::DateTime;
use passuff_config::EventToggles;

use crate::domain::{
    ActivityEvent, AuthoredPullRequest, Baseline, CiState, CurrentObservation, LoginSet, PrState,
    WatchedPullRequest,
};

#[derive(Debug, Clone)]
pub struct Classification {
    pub events: Vec<ActivityEvent>,
    pub baseline: Option<Baseline>,
    pub current: CurrentObservation,
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
                events.push(ActivityEvent::StateChange {
                    state: current.state.clone(),
                    actor: current.merged_by.clone(),
                });
            }
        } else if created_since_last_check
            && current.state == PrState::Open
            && toggles.state_changes
        {
            events.push(ActivityEvent::Opened {
                author: current.author.clone(),
            });
        }
        return Classification {
            events,
            baseline: Some(baseline(current)),
            current: observation,
        };
    };

    let mut events = Vec::new();
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
            if is_own_or_filtered(&comment.author, login_set, toggles) {
                continue;
            }
            events.push(ActivityEvent::Comment {
                author: comment.author.clone(),
                body_excerpt: excerpt(&comment.body),
            });
        }
        for comment in &current.review_comments {
            if seen_review_comments.contains(comment.id.as_str()) {
                continue;
            }
            if is_own_or_filtered(&comment.author, login_set, toggles) {
                continue;
            }
            events.push(ActivityEvent::ReviewComment {
                author: comment.author.clone(),
                body_excerpt: excerpt(&comment.body),
            });
        }
    }

    if toggles.reviews {
        for review in &current.reviews {
            if seen_reviews.contains(review.id.as_str()) {
                continue;
            }
            if is_own_or_filtered(&review.author, login_set, toggles) {
                continue;
            }
            events.push(ActivityEvent::Review {
                author: review.author.clone(),
                state: normalize_review_state(&review.state),
            });
        }
    }

    let mut ci_failed = previous.current.ci_failed;
    if previous.current.head_oid != current.head_oid {
        ci_failed = false;
        if current.ci.state.failed() {
            ci_failed = true;
            if toggles.ci_failure {
                events.push(ActivityEvent::CiFailure {
                    failed_checks: current.ci.failed_checks.clone(),
                });
            }
        }
    } else if current.ci.state.failed() {
        if !ci_failed {
            ci_failed = true;
            if toggles.ci_failure {
                events.push(ActivityEvent::CiFailure {
                    failed_checks: current.ci.failed_checks.clone(),
                });
            }
        }
    } else if matches!(current.ci.state, CiState::Success) && ci_failed {
        ci_failed = false;
        if toggles.ci_recovery {
            events.push(ActivityEvent::CiRecovery);
        }
    }

    let mut state_reported = previous.current.state_reported;
    if current.state == PrState::Open {
        state_reported = false;
    } else if !state_reported {
        state_reported = true;
        if toggles.state_changes {
            events.push(ActivityEvent::StateChange {
                state: current.state.clone(),
                actor: current.merged_by.clone(),
            });
        }
    }

    Classification {
        events,
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

fn created_after(created_at: &str, last_check_at: &str) -> bool {
    match (
        DateTime::parse_from_rfc3339(created_at),
        DateTime::parse_from_rfc3339(last_check_at),
    ) {
        (Ok(created), Ok(last)) => created > last,
        _ => false,
    }
}

fn is_own_or_filtered(author: &str, login_set: &LoginSet, toggles: &EventToggles) -> bool {
    login_set.contains(author) || (is_bot(author) && !toggles.bot_comments)
}

fn is_bot(author: &str) -> bool {
    author.ends_with("[bot]")
}

fn excerpt(body: &str) -> String {
    let single_line = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = single_line.chars();
    let excerpt: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        format!("{excerpt}...")
    } else {
        excerpt
    }
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
    use crate::domain::{CiResult, PullRequestKey};

    fn pull_request(state: PrState, created_at: &str) -> AuthoredPullRequest {
        let merged = state == PrState::Merged;
        AuthoredPullRequest {
            key: PullRequestKey::new("acme/api".to_string(), 42),
            node_id: "node-42".to_string(),
            author: "octo".to_string(),
            title: "Pull request".to_string(),
            url: "https://github.com/acme/api/pull/42".to_string(),
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
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
            second.events,
            vec![ActivityEvent::StateChange {
                state: PrState::Merged,
                actor: None
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
            classification.events,
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
            later.events,
            vec![ActivityEvent::StateChange {
                state: PrState::Merged,
                actor: None
            }]
        );
    }

    #[test]
    fn invalid_timestamps_do_not_create_a_short_lived_notification() {
        assert!(!created_after("not-a-date", "2026-11-02T10:00:00Z"));
    }
}
