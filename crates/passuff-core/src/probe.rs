use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::model::{Allowance, Fingerprint, ProbePullRequest, ProbeReport, PullRequestKey};
use crate::state::WatchedPullRequest;

#[derive(Debug, Clone, Default)]
pub struct ProbeDiff {
    pub targets: Vec<PullRequestKey>,
    pub pending: BTreeMap<String, Fingerprint>,
    pub changed: bool,
}

pub fn diff_probe(
    accepted: &BTreeMap<String, Fingerprint>,
    current: &ProbeReport,
    now: DateTime<Utc>,
) -> ProbeDiff {
    let _ = now;
    let mut targets = Vec::new();
    let mut pending = BTreeMap::new();
    for pull_request in &current.pull_requests {
        let key = pull_request.key.to_string();
        let fingerprint = Fingerprint::from(pull_request);
        pending.insert(key.clone(), fingerprint.clone());
        let Some(previous) = accepted.get(&key) else {
            targets.push(pull_request.key.clone());
            continue;
        };
        if is_relevant_change(previous, pull_request) {
            targets.push(pull_request.key.clone());
        }
    }
    ProbeDiff {
        changed: !targets.is_empty(),
        targets,
        pending,
    }
}

fn is_relevant_change(previous: &Fingerprint, current: &ProbePullRequest) -> bool {
    // A known terminal pull request whose update time moved alone is not an
    // observation target: activity after a terminal state is not reported.
    if previous.state.is_terminal() && current.state.is_terminal() {
        return previous.created_at != current.created_at
            || previous.state != current.state
            || previous.head_oid != current.head_oid
            || previous.ci != current.ci;
    }
    previous.created_at != current.created_at
        || previous.state != current.state
        || previous.head_oid != current.head_oid
        || previous.ci != current.ci
        || previous.updated_at != current.updated_at
}

/// The live inbox spends at most half of the remaining allowance across the
/// window to the reset, and never schedules past the reset.
pub fn next_delay(allowance: &Allowance, now: DateTime<Utc>) -> Duration {
    let time_to_reset = allowance
        .reset_at
        .map(|reset| (reset - now).num_seconds().max(0) as u64)
        .unwrap_or(60);
    if time_to_reset == 0 {
        return Duration::from_secs(1);
    }
    let divisor = (allowance.remaining as u64 / 2).max(1);
    let seconds = (time_to_reset / divisor).clamp(1, time_to_reset);
    Duration::from_secs(seconds)
}

pub fn defer_pass(allowance: &Allowance) -> bool {
    allowance.remaining < 100
}

pub fn backoff(failures: u32) -> Duration {
    let seconds = 1u64.checked_shl(failures.min(6)).unwrap_or(60).min(60);
    Duration::from_secs(seconds)
}

pub fn open_targets(
    watched: &BTreeMap<String, WatchedPullRequest>,
    scope_includes: impl Fn(&str) -> bool,
) -> Vec<PullRequestKey> {
    let mut targets = Vec::new();
    for (key, watched) in watched {
        if !watched.state.is_terminal() && scope_includes(key) {
            if let Ok(key) = key.parse() {
                targets.push(key);
            }
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CiResult;
    use chrono::TimeZone;

    #[test]
    fn pacing_spends_at_most_half_the_allowance() {
        let now = Utc.with_ymd_and_hms(2026, 11, 2, 12, 0, 0).unwrap();
        let allowance = Allowance {
            remaining: 100,
            reset_at: Some(now + chrono::Duration::seconds(3600)),
            cost: 1,
        };
        assert!(next_delay(&allowance, now) >= Duration::from_secs(30));
    }

    #[test]
    fn terminal_update_time_alone_is_ignored() {
        let key = PullRequestKey::new("acme/api", 42);
        let now = Utc.with_ymd_and_hms(2026, 11, 2, 12, 0, 0).unwrap();
        let previous = Fingerprint {
            node_id: None,
            created_at: Some(now),
            updated_at: Some(now),
            state: crate::model::PullRequestState::Merged,
            head_oid: None,
            ci: CiResult::Passing,
        };
        let current = ProbePullRequest {
            key,
            node_id: None,
            created_at: Some(now),
            updated_at: Some(now + chrono::Duration::seconds(10)),
            state: crate::model::PullRequestState::Merged,
            head_oid: None,
            ci: CiResult::Passing,
        };
        assert!(!is_relevant_change(&previous, &current));
    }
}
