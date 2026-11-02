# Glossary

`docs/glossary.md` is the sole vocabulary authority for this project. Structure,
boundaries, and relationships live in [the domain model](domain-model.md);
behaviour lives in the governing use-case contract and specifications.

Behavioural rules are owned by the `watch-pull-requests` use case and its
`pr-activity-notification` capability. This glossary defines terms; it does not
restate the rules.

| Term | Definition | Code symbol | Anti-terms |
|---|---|---|---|
| Account | A GitHub identity configured for observation, labelled locally and resolved to a GitHub login. | `Account` | user, profile, credential — a credential authenticates an account; it is not the account. |
| Credential mode | The declared way an account obtains its token: a gh OAuth session or a personal access token. | `AuthMode`, `CredentialSpec` | auth type, login method — the mode names the source, not the identity. |
| Expected login | The GitHub login an account declares; a resolved credential must belong to it. | `AccountConfig.expected_login` | username, owner — the expected login is a validation target. |
| gh session | The OAuth token the GitHub CLI stores for a login on this machine. | `CredentialSpec::GhOAuth` | gh token, OAuth secret — the session belongs to the operator's gh installation. |
| Personal access token | A GitHub token read from an environment variable, a helper command, or a token file, independent of the gh session. | `CredentialSpec::{Env,Command,File}` | password, secret — a token authenticates; it is not an account. |
| Token file | A file containing a personal access token, accepted only when group and other users cannot read or write it. | `CredentialSpec::File` | credentials file, secret file — the file holds one token for one account. |
| Account failure | A failure to observe one configured account, reported in the refresh report while other accounts continue. | `AccountFailure` | error — an error can be a configuration or state error; an account failure is scoped to one account. |
| Activity event | A change on an authored pull request that is eligible for notification: a comment, a review, a CI result transition, or a pull request state change. | `ActivityEvent` | notification, update — a notification reports activity events; it is not one. |
| Authored pull request | A pull request whose author login belongs to the configured login set. | `AuthoredPullRequest` | my PR (informal), watched PR — a watched pull request is the observation record, not the GitHub object. |
| Baseline | The first recorded observation of an authored pull request, used to suppress notification of activity that already existed. | `Baseline` | snapshot — a snapshot is any observation; the baseline is the first one. |
| Bot comment | A comment authored by a GitHub bot account, identified by the bot marker on the author. | `ActivityClassifier` (bot filter) | automated comment — a human can automate a comment without being a bot account. |
| Check cycle | One observation pass over all configured accounts. | `PassuffCore::refresh` | poll — an implementation mechanism, not the domain pass. |
| CI result | The aggregate outcome of checks on the latest commit of an authored pull request. | `CiResult` | status, build — a build is one check; the CI result is the aggregate. |
| Comment | A written remark on a pull request, either a conversation comment or a review comment. | `ActivityEvent::Comment` | message, note — a notification is not a comment. |
| Configured login set | The union of GitHub logins resolved for all configured accounts. | `LoginSet` | owners, users — the set is identities, not people. |
| Developer | The person who owns the configured accounts and receives notifications. | none (actor) | user, customer |
| Event family | A configurable class of activity events: comments, reviews, CI failure, CI recovery, or state changes. | `EventToggles` | event type — a type is a value; a family is the configurable class. |
| Notification | A grouped report of activity events for one authored pull request. Delivery is performed by a UI adapter; the report itself has no delivery dependency. | `Notification` | alert, message — a message is a comment; an alert implies urgency the notification does not require. |
| Notification sound | An audible signal requested once per notification when sound is enabled. | `NotificationsConfig.sound` | alarm, beep |
| Observation | A snapshot of one authored pull request at one check cycle, including its comments, reviews, review comments, head commit, CI result, and state. | `AuthoredPullRequest` | poll — polling is the mechanism, not the observation. |
| Own action | A comment or review authored by a login in the configured login set. A state change is not an own action. | `ActivityClassifier` (own-action filter) | self comment — a merge or close is a state change and notifies. |
| Pull request key | The repository and number that identify a pull request. | `PullRequestKey` | PR id — the GitHub node ID is a different identifier. |
| Reopen | A transition of a known closed or merged pull request back to open; it resets the state-change report flag without a notification. | `ActivityClassifier` | unmerge — a reopen is not a merge reversal. |
| Review | A submitted assessment of a pull request with one of the states approved, changes requested, or commented. | `ActivityEvent::Review` | approval — approval is one review state. |
| Short-lived pull request | An authored pull request created after the account's last successful observation and merged or closed before the next observation. | `ActivityClassifier` | fast merge — a short-lived pull request is defined by the observation boundary, not speed. |
| Watched pull request | An authored pull request with a persisted observation record and a known baseline. | `WatchedPullRequest` | subscription, monitored repo |
| Watcher | The passuff process that performs check cycles: observation and classification live in the core; scheduling and delivery live in the UI adapter. | `passuff` process | poller, monitor — a watcher is the product's observation role. |
| Watcher state | The persisted record of observations and already-notified activity across check cycles. | `WatcherState` | cache, database — the state is the durable observation record, not an implementation store. |
