//! UI-agnostic core for passuff.

pub mod classifier;
pub mod credential;
pub mod cycle;
pub mod domain;
pub mod github;
pub mod inbox;
pub mod state;

pub use classifier::{classify, Classification, ClassifiedEvent};
pub use credential::{CredentialError, CredentialSource, GhLogin, SystemCredentialSource};
pub use cycle::PassuffCore;
pub use domain::{
    excerpt, Account, AccountFailure, AccountFailureKind, AccountState, ActivityEvent,
    ActivityRecord, ApiAllowance, AuthoredPullRequest, Baseline, ChangeProbe, ChangeSignal,
    CiResult, CiState, Comment, CoreError, CurrentObservation, LoginSet, Notification, PrState,
    ProbePullRequest, PullRequestKey, PullRequestStack, RefreshReport, Review, ReviewComment,
    ScopeSnapshot, StateError, WatchedPullRequest, WatcherState,
};
pub use github::{
    AccountObservation, AccountProbe, Discovery, GitHubApiAdapter, GitHubError, GitHubGateway,
};
pub use inbox::{
    effective_time, is_approximate, AccountView, Inbox, InboxService, InboxView, PullRequestView,
    ReadOutcome,
};
pub use state::{JsonStateStore, SnapshotStore};
