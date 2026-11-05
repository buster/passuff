# Changelog

All notable changes to passuff are documented in this file.

The release sections are generated with [git-cliff](https://git-cliff.org/).
Versions are selected manually and use annotated `vX.Y.Z` Git tags.
## [Unreleased]

## [0.1.2] - 2026-11-05

### Changes

- Aktualisieren von README.md

- Fix formatting of coverage badge message

- Drop the brand emoji from the inbox header title

crates/passuff-cli/src/tui.rs: render the outer block title as " passuff — Inbox " instead of
" passuff (☞ﾟヮﾟ)☞ — Inbox ". Terminals without U+261E/U+FF9F coverage drew substitute glyphs
in the title. Added the_inbox_title_has_no_emoji to assert the header text and the absence of ☞.

- Show when activity happened and focus the inbox by involvement (#47)

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Scenarios: Inbox lists unread activity newest first; Inbox shows when a comment was written; Inbox marks an event without a reported time as approximate; Inbox orders activity by when it happened; The activity table shows when the activity happened; The activity table marks a CI failure as approximate; The activity detail shows when the activity happened and when it was fetched; The activity detail marks an activity without a reported time as approximate

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Records the task state and keeps the pre-merge permanent corpus green for the renamed step phrase.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Removes the wip tags now that every delta scenario is bound.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Records the tasks verification receipt.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Fixes the coverage and verify scripts so the e2e fallback checks .feature files only; prose mentions of @e2e in change artifacts no longer suppress the permanent corpus.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Preflight: lists the existing notify-test command action in the watch-pull-requests interaction inventory, clearing the corpus lint gate.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Records the review and its measured coverage.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Uses the required README-IMPACT value.

* The inbox now shows when each activity happened instead of when passuff fetched it.

Batch: activity-time
Refreshes the tasks verification receipt.

* chore(passuff-activity-time): publish archive candidate

* chore(passuff-activity-time): prove archive

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Scenarios: Developer toggles a repository in the watch scope; Adding a repository by owner/name; The picker shows the focus relations; Narrowing the focus persists the relation set; Developer narrows the focus in the watch scope; Inbox hides activity outside the focus; The watched table hides a pull request outside the focus; A pull request outside the focus produces no notification

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Records the task state.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Adds the change configuration lock.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Records the tasks verification receipt.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Records the review and its measured coverage.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Refreshes the tasks verification receipt.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Records the overlap dispositions.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Records the split-or-keep decisions.

* The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

Batch: involvement-focus
Keeps the task list parseable.

* chore(passuff-involvement-focus): publish archive candidate

* chore(passuff-involvement-focus): prove archive

* Drop the manual repository entry from the watch-scope corpus.

The archived involvement-focus change removed the entry path; this correction
points the invalid-entry scenario at the scripted selection and deletes the
compatibility step.

- Post-merge fixes: formatting, clippy in verify, focus filtering on existing state (#48)

* Fix formatting and restore the scope-filter test attribute.

cargo fmt --check failed on main. The inbox focus-filter test also lost its
#[tokio::test] attribute during insertion, so it never ran; clippy caught it.

* Run formatting and clippy in the verify script.

The regular scope now mirrors CI: cargo fmt --check, both clippy invocations
with -D warnings, then the locked library tests. A broken format fails the
script before the scenarios run. The e2e scope stays a single test pass.

* Make a focus change apply to existing state immediately.

State written before involvement relations existed left every watched pull
request visible, so narrowing the focus appeared to do nothing until a
reconciliation ran.

- Derive authored from the account login and the pull request author when no
  relation is recorded, so the developer's own pull requests filter at once.
- Refresh stored relations from every discovery pass, even for pull requests
  the pass does not observe, so involved pull requests need at most one cycle.
- Regression scenario: changing the focus in the picker filters the inbox
  immediately, starting from state with unknown relations.


### Features

- **pr-activity-notification:** Show the passuff brand mark in notifications and the CLI

## [0.1.1] - 2026-11-04

### Changes

- Deliver one digest notification per check pass and play a one-second three-beep chime.

Batch: notification-digest

- Deliver one digest notification per check pass and play a one-second three-beep chime.

Batch: sound-asset

## [0.0.0] - 2026-11-04

### Added

- Initial passuff capabilities: authored pull request watching, multi-account
  notifications, the terminal inbox, stacked pull requests, watch scope, and
  live observation.

[0.1.1]: https://github.com/buster/passuff/compare/v0.1.0...v0.1.1

[0.1.2]: https://github.com/buster/passuff/compare/v0.1.1...v0.1.2
