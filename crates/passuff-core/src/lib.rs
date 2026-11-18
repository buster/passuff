//! UI-agnostic core for passuff: observation, classification, and watcher
//! state. The core does not depend on a terminal, a notification surface, or a
//! sound player.

pub mod classifier;
pub mod core;
pub mod github;
pub mod model;
pub mod state;
pub mod view;

pub use core::{PassuffCore, ProbeReport};
pub use github::GithubClient;
pub use model::{
    AccountFailure, AccountState, ActivityKind, ActivityRecord, CiResult, Notification, PrState,
    PullRequestKey, RefreshReport, StackInfo, WatchedPullRequest, WatcherState,
};
pub use state::{LoadedState, StateError};
