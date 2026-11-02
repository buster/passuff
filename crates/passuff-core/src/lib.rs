//! UI-agnostic core for passuff.

pub mod classifier;
pub mod credential;
pub mod cycle;
pub mod domain;
pub mod github;
pub mod inbox;
pub mod state;

pub use classifier::{classify, Classification};
pub use credential::{CredentialError, CredentialSource, GhLogin, SystemCredentialSource};
pub use cycle::PassuffCore;
pub use domain::{
    Account, AccountFailure, AccountFailureKind, AccountState, ActivityEvent, ActivityRecord,
    AuthoredPullRequest, Baseline, CiResult, CiState, Comment, CoreError, CurrentObservation,
    LoginSet, Notification, PrState, PullRequestKey, RefreshReport, Review, ReviewComment,
    StateError, WatchedPullRequest, WatcherState,
};
pub use github::{AccountObservation, GitHubApiAdapter, GitHubError, GitHubGateway};
pub use inbox::{AccountView, Inbox, InboxService, InboxView, PullRequestView, ReadOutcome};
pub use state::{JsonStateStore, SnapshotStore};
