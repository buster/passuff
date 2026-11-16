//! UI-agnostic observation, classification, persistence and inbox read model.

pub mod classifier;
pub mod core;
pub mod credentials;
pub mod github;
pub mod inbox;
pub mod model;
pub mod probe;
pub mod state;

pub use classifier::{classify, Classification, ClassificationContext, ClassifiedEvent};
pub use core::{
    AccountFailure, AccountProbe, Clock, CoreError, FixedClock, PassuffCore, ProbeCycle,
    RefreshReport, SystemClock,
};
pub use credentials::{
    CredentialError, CredentialSource, ResolvedCredential, SystemCredentialSource,
};
pub use github::{GatewayError, GatewayFactory, GitHubGateway, SystemGatewayFactory};
pub use inbox::{
    AccountView, ActivityView, InboxService, InboxView, MarkReadResult, PullRequestView,
};
pub use model::{
    ActivityEvent, Allowance, CiResult, CommentData, EventKind, Fingerprint, Notification,
    ProbePullRequest, ProbeReport, PullRequestKey, PullRequestSnapshot, PullRequestState,
    ReviewCommentData, ReviewData, ReviewState, StackInfo,
};
pub use probe::{backoff, defer_pass, diff_probe, next_delay, open_targets, ProbeDiff};
pub use state::{
    AccountState, ActivityRecord, JsonStateStore, LoadOutcome, ScopeSnapshot, SnapshotStore,
    StateError, WatchedPullRequest, WatcherState, ACTIVITY_LIMIT, STATE_VERSION,
};
