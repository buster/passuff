# 1. Introduction and Goals

## Requirements overview

passuff is a local watcher for GitHub pull requests, built as a reusable core
library with a command-line adapter. A developer with one or more GitHub
accounts configures those accounts and receives a desktop notification with
sound only when something they care about changes on a pull request they
authored. A terminal UI consumes the same core and read model.

The essential requirements are:

- Observe every pull request authored by any configured account, across all
  readable repositories.
- Notify on another person's comment or review, a review submission, a CI
  failure or recovery, and a merge or close.
- Stay silent for the developer's own actions, code pushes, bot comments by
  default, and every pull request the developer did not author.
- Group all new activity for one pull request into one notification, with one
  sound.
- Persist observations so a restart never replays notified activity.
- Show which accounts, watch scope, and pull requests are watched, and how much
  recorded activity is unread.
- Retain a bounded activity history and let the developer mark it read.
- Provide a full-screen terminal UI over the same inbox for browsing and acknowledging activity.
- Run on macOS and Linux with desktop notifications and configurable sound.
- Keep credentials out of configuration and persisted state.

The authoritative behaviour is in three use cases:
[`watch-pull-requests`](../../givn/specs/watch-pull-requests/usecase.md),
[`inspect-watch-status`](../../givn/specs/inspect-watch-status/usecase.md), and
[`review-activity-inbox`](../../givn/specs/review-activity-inbox/usecase.md),
with their feature files under `givn/specs/`.

## Quality goals

| Priority | Quality attribute | Motivation |
|---|---|---|
| 1 | Accuracy | A notification must mean eligible activity on an authored pull request. False positives destroy trust faster than missed edge cases. |
| 3 | Inspectability | The developer can see the watched accounts, scope, and pull requests and can review retained activity without contacting GitHub. |
| 2 | No replay | A restart must not repeat activity the developer already saw. The core persists observations before returning the report; the CLI delivers afterwards. |
| 3 | Reliability | One failed account, malformed response, or corrupt state must not stop the other accounts or crash the watcher. |
| 4 | Timeliness | A change should be visible within one configured observation interval plus a small processing margin. |
| 5 | Security | Tokens come from a gh session, an environment variable, a helper command, or a permission-checked token file, and never appear in config, state, logs, or transcripts. |
| 6 | Portability | The same binary runs on Linux and macOS, with platform notification and sound adapters. |
| 7 | Reusability | Observation, classification, configuration, and state live in a UI-agnostic core library so the CLI and the TUI share one implementation. |

## Stakeholders

| Role | Expectation |
|---|---|
| Developer (operator) | Sees only relevant activity on their own pull requests, with a clear notification and sound, without checking multiple GitHub accounts manually. |
| Maintainer (same person) | A small, testable Rust codebase with an executable specification, no hosted infrastructure, and a clear path to extend or debug. |
| GitHub | Receives read-only GraphQL queries and no mutation requests. |
| Desktop environment | Receives standard notifications and an audio-player invocation; no privileged integration. |
