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
        let short_lived = account_has_prior_cycle
            && current.state.is_terminal()
            && last_check_at.is_some_and(|last| current.created_at.as_str() > last);
        let mut observation = current_observation(current);
        let mut events = Vec::new();
        if short_lived {
            observation.state_reported = true;
            if toggles.state_changes {
                events.push(ActivityEvent::StateChange {
                    state: current.state.clone(),
                });
            }
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
        state_reported: false,
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
