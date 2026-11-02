# Proposal: passuff-pr-watch

## Release note

passuff watches the GitHub pull requests authored by your configured accounts and sends a desktop notification with sound when someone else comments, reviews, changes CI status, or merges or closes one of them.

## Use-Case Context

- Use-case ID: `watch-pull-requests` (new in this change)
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — the developer who owns personal and work GitHub accounts, wants alerts only for pull requests they authored, and works on macOS and Linux. Confirmed by the operator's brief on 2026-11-02 and by the instruction to proceed with this change.

## Review Context

- Review kind: self-review
- Reason: single-operator greenfield CLI with no existing production behavior or external consumers. Self-review is proportionate; the design-review step still checks the full artifact set.

## Problem / Opportunity

GitHub's notification feed mixes the pull requests a developer authored with review requests, mentions, and activity on other people's pull requests. A developer with more than one GitHub account has to check several identities and many repositories to find the few changes that matter. Pull request comments, reviews, CI failures, and merges are easy to miss, and checking manually is slow.

The opportunity is a local, self-contained watcher that observes only the pull requests the developer authored and reports only the changes the developer asked for: someone else commenting or reviewing, CI failing or recovering, and the pull request being merged or closed. Code pushes, the developer's own actions, and other people's pull requests stay silent.

## Proposed Solution

### Reusable core

The watching rules, activity classification, configuration, and persisted
observations live in a reusable library. The command-line interface is the first
adapter. A future terminal interface can consume the same core without
duplicating the domain logic. The user-visible behaviour in this change remains
the CLI notifier.

### Accounts

The developer configures one or more GitHub accounts. Each account has a label, a credential source, and an optional GitHub API address. Credentials are never stored in the configuration file; they come from the environment or a configured credential command. The watcher discovers each account's GitHub login and uses the union of those logins as the developer's identity set.

### Observation

The watcher observes every pull request authored by a configured account across all repositories that account can read. It observes periodically at a configurable interval, with a default of 60 seconds. The first observation of a pull request establishes a baseline and does not notify about activity that already existed.

A one-shot check command performs exactly one observation pass and prints what it found. It is used to validate configuration and to run the watcher from an external scheduler.

### Notification rules

A notification is produced for an authored pull request when any of these occurs:

- Another person writes a conversation comment or a review comment.
- Another person submits a review: approved, changes requested, or commented.
- The CI result on the latest commit becomes failing or erroneous.
- The CI result recovers from failing or erroneous to successful.
- The pull request is merged or closed.

No notification is produced for:

- A comment or review written by any configured account.
- A commit or push to the pull request branch.
- Activity on a pull request authored by anyone outside the configured account set.
- Label, draft, assignee, milestone, or base-branch changes.
- A bot comment, unless the developer enables bot comments in configuration.

### Grouping and delivery

The watcher groups all new activity for one pull request in one observation pass into one notification. The notification names the repository, pull request number, and a concise list of what changed. The configured sound plays once for that notification. A notification is delivered as a desktop notification on macOS or Linux and is also printed in the terminal transcript.

The developer can enable or disable the desktop notification, enable or disable the sound, choose the sound, and toggle each event family. The default configuration uses desktop notifications and sound.

### Reliability

Observations persist across restarts. Activity that arrives while the watcher is stopped is reported on the next observation for pull requests the watcher already knows. A pull request merged or closed while the watcher was stopped is reported on the next observation. Already notified activity is never replayed.

Each account is observed independently. A failed account, an expired credential, or an unreachable GitHub endpoint is reported clearly and does not stop the remaining accounts. A corrupt persisted observation is set aside and replaced with a fresh baseline rather than crashing the watcher. Invalid configuration exits with a clear message and a nonzero status.

## Capability Routing

`givn spec route` reported no signal: no command, token, or lexical overlap with an existing capability.

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| pr-activity-notification | no signal | `NEW in watch-pull-requests` | This is the first capability in the project. There is no existing use case or specification to extend. |

## Out of Scope

- Webhooks, a public endpoint, or a hosted service. passuff is a local watcher.
- Notifications for review requests, mentions, or activity on pull requests the developer did not author.
- Notifications for code changes, diffs, or commits.
- Replying to comments, approving, merging, or otherwise changing GitHub state.
- Mobile push, email, Slack, Telegram, or other external delivery channels.
- Windows desktop support. The code may remain portable, but Windows is not a supported target for this change.
- A graphical interface, tray icon, or installer.
- An activity inbox (history, read/unread state, configurable watch scope) and
  a TUI. These are follow-up changes; this change only establishes the reusable
  core that lets them share the notification engine.
- Per-repository or per-organization allow/deny filters beyond the account boundary.
- Replaying historical activity that existed before the first observation of an
  account. The short-lived pull request state change in the use case is not a
  replay; it is a newly created pull request's terminal state.

## Open Questions

None. The event rules, account model, notification surface, and platform scope are settled above. Remaining choices are technical and are resolved in `design.md`: the GitHub observation mechanism, the persisted state format, the credential source forms, and the per-platform sound adapter.
