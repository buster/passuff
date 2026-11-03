# Glossary

`docs/glossary.md` is the sole vocabulary authority for this project. Structure,
boundaries, and relationships live in [the domain model](domain-model.md);
behaviour lives in the governing use-case contract and specifications.

Behavioural rules are owned by `watch-pull-requests` (`pr-activity-notification`),
`inspect-watch-status` (`watch-status`), `review-activity-inbox`
(`activity-inbox`), and `review-pull-request-stacks` (`stacked-inbox`). This
glossary defines terms; it does not restate the rules. The watch-scope mode,
known-repository, and catch-up terms are confirmed for the planned
`change-watch-scope` use case; its behavioural rules arrive with that change.

| Term | Definition | Code symbol | Anti-terms |
|---|---|---|---|
| Account | A GitHub identity configured for observation, labelled locally and resolved to a GitHub login. | `Account` | user, profile, credential — a credential authenticates an account; it is not the account. |
| Credential mode | The declared way an account obtains its token: a gh OAuth session or a personal access token. | `CredentialSpec` | auth type, login method — the mode names the source, not the identity. |
| Expected login | The GitHub login an account declares; a resolved credential must belong to it. | `AccountConfig.expected_login` | username, owner — the expected login is a validation target. |
| gh session | The OAuth token the GitHub CLI stores for a login on this machine. | `CredentialSpec::GhOAuth` | gh token, OAuth secret — the session belongs to the operator's gh installation. |
| Personal access token | A GitHub token read from an environment variable, a helper command, or a token file, independent of the gh session. | `CredentialSpec::{Env,Command,File}` | password, secret — a token authenticates; it is not an account. |
| Token file | A file containing a personal access token, accepted only when group and other users cannot read or write it. | `CredentialSpec::File` | credentials file, secret file — the file holds one token for one account. |
| Account failure | A failure to observe one configured account, reported in the refresh report while other accounts continue. | `AccountFailure` | error — an error can be a configuration or state error; an account failure is scoped to one account. |
| Activity record | A persisted, addressable record of one activity event with an observation time and a read flag. | `ActivityRecord` | notification — a notification groups events; a record preserves one event. |
| Activity event | A change on an authored pull request that is eligible for notification: a comment, a review, a CI result transition, or a pull request state change. | `ActivityEvent` | notification, update — a notification reports activity events; it is not one. |
| Authored pull request | A pull request whose author login belongs to the configured login set. | `AuthoredPullRequest` | my PR (informal), watched PR — a watched pull request is the observation record, not the GitHub object. |
| Repository | A GitHub resource identified by an owner/name pair that contains pull requests. | none (GitHub resource) | project, repo — project also names GitHub Projects boards and local work directories. |
| Baseline | The first recorded observation of an authored pull request, used to suppress notification of activity that already existed. | `Baseline` | snapshot — a snapshot is any observation; the baseline is the first one. |
| Bot comment | A comment authored by a GitHub bot account, identified by the bot marker on the author. | `ActivityClassifier` (bot filter) | automated comment — a human can automate a comment without being a bot account. |
| Check cycle | One observation pass over all configured accounts. | `PassuffCore::refresh` | poll — an implementation mechanism, not the domain pass. |
| CI result | The aggregate outcome of checks on the latest commit of an authored pull request. | `CiResult` | status, build — a build is one check; the CI result is the aggregate. |
| Comment | A written remark on a pull request, either a conversation comment or a review comment. | `ActivityEvent::Comment` | message, note — a notification is not a comment. |
| Configured login set | The union of GitHub logins resolved for all configured accounts whose credential matched their expected login. | `LoginSet` | owners, users — the set is identities, not people. |
| Developer | The person who owns the configured accounts and receives notifications. | none (actor) | user, customer |
| Event family | A configurable class of activity events: comments, reviews, CI failure, CI recovery, or state changes. | `EventToggles` | event type — a type is a value; a family is the configurable class. |
| Notification | A grouped report of activity events for one authored pull request. Delivery is performed by a UI adapter; the report itself has no delivery dependency. | `Notification` | alert, message — a message is a comment; an alert implies urgency the notification does not require. |
| Notification sound | An audible signal requested once per notification when sound is enabled. | `NotificationsConfig.sound` | alarm, beep |
| Observation | A snapshot of one authored pull request at one check cycle, including its comments, reviews, review comments, head commit, CI result, and state. | `AuthoredPullRequest` | poll — polling is the mechanism, not the observation. |
| Own action | A comment or review authored by a login in the configured login set. A state change is not an own action. | `ActivityClassifier` (own-action filter) | self comment — a merge or close is a state change and notifies. |
| Pull request key | The repository and number that identify a pull request. | `PullRequestKey` | PR id — the GitHub node ID is a different identifier. |
| Pull request stack | An ordered chain of pull requests in one repository, identified by a repository-scoped number and a total size, where each member has a 1-based position and the chain shares one ultimate base. On a watched member, passuff records the number, size, position, and base as `PullRequestStack`. | `PullRequestStack` | stack of commits — the members are pull requests, not commits. |
| Stack position | The 1-based ordinal of a pull request within its stack, counting from the bottom pull request closest to the ultimate base. | `PullRequestStack.position` | pull request number — number is repository identity, position is stack order. |
| Stack base | The branch that the whole stack ultimately targets. | `PullRequestStack.base` | pull request base — a pull request's base is its direct parent branch. |
| Reopen | A transition of a known closed or merged pull request back to open; it resets the state-change report flag without a notification. | `ActivityClassifier` | unmerge — a reopen is not a merge reversal. |
| Review | A submitted assessment of a pull request with one of the states approved, changes requested, or commented. | `ActivityEvent::Review` | approval — approval is one review state. |
| Short-lived pull request | An authored pull request created after the account's last successful observation and merged or closed before the next observation. | `ActivityClassifier` | fast merge — a short-lived pull request is defined by the observation boundary, not speed. |
| Activity history | The bounded set of retained activity records with their read state. | `WatcherState.activity` | log, feed — the history is reviewed, not delivered. |
| Inbox | The read model of configured accounts, the watch scope, watched pull requests, and activity records. | `Inbox` | list, feed, tracked — the inbox includes the watch summary and reviewed history. |
| Read state | Whether an activity record has been acknowledged by the developer. | `ActivityRecord.read` | seen — a record can be seen but unread, or read after review. |
| Catch-up | The recording of activity that happened while a repository was outside the watch scope, delivered to the inbox without a notification. | none (check-cycle policy) | notification backlog, replay — catch-up is silent and records new gap activity. |
| Terminal UI | The full-screen inbox adapter over `InboxService`; a second consumer of the read model. | `TuiApp` | dashboard, GUI — the terminal UI is a text interface. |
| Known repository | A repository that passuff has observed in an authored pull request or that appears in the watch scope. | none (read model) | full catalog — the known set is only what passuff has seen or the scope contains. |
| Watch scope | The set of repositories passuff observes: all repositories, or selected repositories, where an empty selected set observes none. | `WatchConfig` | filter, subscription — the scope is configuration, not a separate account. |
| Watched pull request | An authored pull request with a persisted observation record and a known baseline. | `WatchedPullRequest` | subscription, monitored repo |
| Watcher | The passuff process that performs check cycles: observation and classification live in the core; scheduling and delivery live in the UI adapter. | `passuff` process | poller, monitor — a watcher is the product's observation role. |
| Watcher state | The persisted record of observations and already-notified activity across check cycles. | `WatcherState` | cache, database — the state is the durable observation record, not an implementation store. |
