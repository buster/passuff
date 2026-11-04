//! Live observation for the terminal inbox.
//!
//! The session owns the check cycle while the inbox is open. A cheap change
//! probe decides whether the expensive cycle runs, and the delay between probes
//! is derived from the GraphQL allowance GitHub reports.

use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use passuff_config::{AppConfig, ConfigLoader};
use passuff_core::{ApiAllowance, PassuffCore};

use crate::CliRuntime;

/// A check cycle is skipped below this many remaining GraphQL points.
pub const MIN_FULL_OBSERVATION_REMAINING: u32 = 100;

/// The delay used when GitHub reports no allowance.
pub const DEFAULT_DELAY: Duration = Duration::from_secs(5);

const INITIAL_BACKOFF: Duration = Duration::from_secs(2);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// One observation tick handed to the terminal loop.
#[derive(Debug, Clone)]
pub struct ObservationUpdate {
    pub config: AppConfig,
    pub changed: bool,
    pub failure: Option<String>,
    pub allowance: Option<ApiAllowance>,
    pub next_delay: Duration,
}

/// Owns the check cycle while the terminal inbox is open.
pub struct ObservationSession {
    runtime: CliRuntime,
    core: PassuffCore,
    config: AppConfig,
    config_path: PathBuf,
    backoff: Duration,
    last_full: Option<std::time::Instant>,
}

impl ObservationSession {
    pub async fn open(
        runtime: CliRuntime,
        config: AppConfig,
        config_path: PathBuf,
    ) -> Result<Self, String> {
        let core = runtime
            .build_core(&config)
            .await
            .map_err(|outcome| outcome.transcript)?;
        Ok(Self {
            runtime,
            core,
            config,
            config_path,
            backoff: INITIAL_BACKOFF,
            last_full: None,
        })
    }

    /// Reload configuration, probe GitHub, and run one check cycle on a change.
    pub async fn tick(&mut self) -> ObservationUpdate {
        match ConfigLoader::load(&self.config_path) {
            Ok(config) if config != self.config => {
                match self.runtime.build_core(&config).await {
                    Ok(core) => {
                        self.core = core;
                        self.config = config;
                    }
                    Err(outcome) => {
                        return self.failed(outcome.transcript);
                    }
                }
            }
            Ok(_) => {}
            Err(error) => {
                return self.failed(format!("configuration error: {error}"));
            }
        }

        let signal = match self.core.probe().await {
            Ok(signal) => signal,
            Err(error) => return self.failed(error.to_string()),
        };
        self.backoff = INITIAL_BACKOFF;

        let due = self
            .last_full
            .is_none_or(|last| last.elapsed() >= Duration::from_secs(300));
        let mut changed = false;
        let mut failure = None;
        if (signal.changed || due) && allowance_allows_cycle(signal.allowance.as_ref()) {
            match self.core.refresh().await {
                Ok(refreshed) => {
                    self.runtime
                        .deliver_report(&self.config, &refreshed)
                        .await;
                    if let Some(account_failure) = refreshed.account_failures.first() {
                        let extra = refreshed.account_failures.len().saturating_sub(1);
                        failure = Some(if extra > 0 {
                            format!(
                                "account {} failed: {} (+{extra})",
                                account_failure.label, account_failure.detail
                            )
                        } else {
                            format!(
                                "account {} failed: {}",
                                account_failure.label, account_failure.detail
                            )
                        });
                    }
                    changed = true;
                    self.last_full = Some(std::time::Instant::now());
                }
                Err(error) => {
                    failure = Some(error.to_string());
                }
            }
        }
        let next_delay = next_delay(signal.allowance.as_ref(), Utc::now());
        ObservationUpdate {
            config: self.config.clone(),
            changed,
            failure,
            allowance: signal.allowance.clone(),
            next_delay,
        }
    }

    fn failed(&mut self, message: String) -> ObservationUpdate {
        let delay = self.backoff.min(MAX_BACKOFF);
        self.backoff = (self.backoff * 2).min(MAX_BACKOFF);
        ObservationUpdate {
            config: self.config.clone(),
            changed: false,
            failure: Some(message),
            allowance: None,
            next_delay: delay,
        }
    }
}

fn allowance_allows_cycle(allowance: Option<&ApiAllowance>) -> bool {
    match allowance {
        Some(allowance) => allowance.remaining >= MIN_FULL_OBSERVATION_REMAINING,
        None => true,
    }
}

/// Delay before the next probe: spend at most half the remaining allowance
/// across the window, never faster than once per second, never past the reset.
pub fn next_delay(allowance: Option<&ApiAllowance>, now: DateTime<Utc>) -> Duration {
    let Some(allowance) = allowance else {
        return DEFAULT_DELAY;
    };
    let Ok(reset_at) = DateTime::parse_from_rfc3339(&allowance.reset_at) else {
        return DEFAULT_DELAY;
    };
    let seconds_to_reset = (reset_at.with_timezone(&Utc) - now).num_seconds().max(1) as u64;
    let budget = u64::from(allowance.remaining / 2).max(1);
    let millis = (seconds_to_reset * 1_000 / budget).max(1_000);
    Duration::from_millis(millis.min(seconds_to_reset * 1_000))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn allowance(remaining: u32, reset_at: &str) -> ApiAllowance {
        ApiAllowance {
            cost: 1,
            limit: 5_000,
            remaining,
            reset_at: reset_at.to_string(),
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 3, 12, 0, 0).unwrap()
    }

    #[test]
    fn five_thousand_points_probe_about_every_second_and_a_half() {
        let reset = "2026-11-03T13:00:00Z";
        assert_eq!(
            next_delay(Some(&allowance(5_000, reset)), now()),
            Duration::from_millis(1_440)
        );
    }

    #[test]
    fn one_hundred_points_wait_seventy_two_seconds() {
        let reset = "2026-11-03T13:00:00Z";
        assert_eq!(
            next_delay(Some(&allowance(100, reset)), now()),
            Duration::from_secs(72)
        );
    }

    #[test]
    fn ten_points_wait_until_the_reset() {
        let reset = "2026-11-03T13:00:00Z";
        assert_eq!(
            next_delay(Some(&allowance(10, reset)), now()),
            Duration::from_secs(720)
        );
    }

    #[test]
    fn a_missing_allowance_uses_the_default_delay() {
        assert_eq!(next_delay(None, now()), DEFAULT_DELAY);
        assert_eq!(
            next_delay(
                Some(&allowance(10, "not-a-date")),
                now()
            ),
            DEFAULT_DELAY
        );
    }

    #[test]
    fn a_reset_in_the_past_still_waits_one_second() {
        assert_eq!(
            next_delay(Some(&allowance(10, "2026-11-03T11:00:00Z")), now()),
            Duration::from_secs(1)
        );
    }
}
