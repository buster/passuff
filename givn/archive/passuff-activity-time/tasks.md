# Tasks: passuff-activity-time

### Setup: runner, strict mode, and the failing baseline

- [x] Setup: confirm the runner and strict mode.
  - `verify.command` and `verify.e2e_command` are unchanged
    (`./scripts/verify.sh`, `./scripts/verify.sh --e2e`).
  - Strict mode is enabled by `.fail_on_skipped()` on the `Cucumber` builder and
    by the runner's `total == 0` failure in
    `crates/passuff-cli/tests/features_runner.rs`.
  - Step files per capability: `crates/passuff-cli/tests/steps/activity_inbox.rs`
    (`activity-inbox`) and `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`
    (`inbox-navigation`).
  - Proof before implementation:
    `GIVN_FEATURES=$PWD/givn/changes/passuff-activity-time/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    exited 1 with `givn test-result: total 8, passed 0, failed 8, skipped 0`;
    the undefined phrases are `"alice" comments "..." on "..." at "..."`,
    `the inbox shows the occurrence time "..." for "..."`,
    `the inbox marks "..." on "..." as approximate`, and
    `the TUI marks "..." on "..." as approximate`.
  - Domain constraints carried from `review-activity-inbox` R1, T14, T28: the
    time shown is when the activity happened; a record without one shows the
    fetch time marked approximate; the order follows the shown time; the detail
    popup shows both times when they differ.

### Batch: `activity-time`

Core recording, the `passuff inbox` surface, and the terminal surfaces.
Contracts: `InboxService`; no consumer imports `Inbox` implementation types.

- [x] Core: record the occurrence time.
  - [x] `ActivityRecord.occurred_at: Option<String>` with `#[serde(default)]`.
  - [x] `AuthoredPullRequest.merged_at` / `closed_at`; `mergedAt` / `closedAt`
        in the node query and its mapping.
  - [x] `ClassifiedEvent { event, occurred_at }`; `Classification.events` and
        `.silent` carry it.
  - [x] Occurrence time per event: comment and review-comment `created_at`,
        review `submitted_at` (empty → `None`), opened = pull request
        `created_at`, merge → `merged_at`, close → `closed_at`, CI → `None`.
  - [x] The cycle records `occurred_at` and builds `Notification.events` from
        the same classified events.
  - [x] `effective_time` and `is_approximate` in `passuff-core::inbox`; activity
        sorting by the effective time, newest first, with the existing
        tiebreaks.
  - [x] `render_inbox` prints `occurred_at`, or `~` + `observed_at`.
  - [x] `record_lines` in `tests/steps/activity_inbox.rs` accepts the optional
        `~`.

- [x] Scenario: `Inbox lists unread activity newest first` (modified)
  - [x] **Design constraints:** the permanent steps stay identical except
        `the inbox shows an activity id and occurrence time`; the comment is
        created at 12:00:01 and the check observed at 12:00:02, so the CI
        failure (fetch-time estimate) still sorts before the comment; every
        record stays unread.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] Scenario: `Inbox shows when a comment was written`
  - [x] **Design constraints:** the comment's `created_at` is set from the
        step's `at` value; `passuff inbox` prints that value verbatim in the
        time field; the fetch time must not appear in its place.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] Scenario: `Inbox marks an event without a reported time as approximate`
  - [x] **Design constraints:** a CI failure has `occurred_at = None`; the line
        shows `~` followed by `observed_at`; the marker is part of the time
        field, not a separate column.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] Scenario: `Inbox orders activity by when it happened`
  - [x] **Design constraints:** two comments are recorded in one check with
        `created_at` 09:15 and 09:45; the read model orders newest first by the
        effective time, so the 09:45 comment is printed first.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] TUI: render the reported time.
  - [x] `short_record_time(record)`: `%m-%d %H:%M` of the effective time,
        prefixed `~` when approximate; used by the `When` cell and the
        `Changes on` list.
  - [x] `When` column stays `Constraint::Length(12)`.
  - [x] Popup line under the pull request line: `When: MM-DD HH:MM`, plus
        `Fetched: MM-DD HH:MM` only when the formatted fetch time differs;
        approximate renders `When: ~MM-DD HH:MM (fetch time)`.

- [x] Scenario: `The activity table shows when the activity happened`
  - [x] **Design constraints:** the comment is created at
        `2026-11-02T09:15:00Z`; the focused activity pane renders
        `11-02 09:15` in the `When` cell; the fetch time is not shown there.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] Scenario: `The activity table marks a CI failure as approximate`
  - [x] **Design constraints:** the CI row's `When` cell begins with `~`; the
        record's fetch time is what follows; no other row is marked.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] Scenario: `The activity detail shows when the activity happened and when it was fetched`
  - [x] **Design constraints:** Enter with the activity pane focused (`enter`
        from the default focus) opens the popup; it shows `When: 11-02 09:15`
        and `Fetched: 11-02 12:00`; the popup reads only the record and the
        pull request view.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

- [x] Scenario: `The activity detail marks an activity without a reported time as approximate`
  - [x] **Design constraints:** the CI record has no occurrence time; the popup
        shows `When: ~11-02 12:00 (fetch time)`.
  - [x] COMMIT: `9f3c9c5` -- Batch: `activity-time`

### Closeout

- [x] Full suite: `./scripts/verify.sh` and `./scripts/verify.sh --e2e` exit 0
      with no failed or skipped scenarios.
- [x] Delta step phrases are bound in the permanent step files; the permanent
      suite stays green before the merge through the `observation|occurrence`
      phrase alternation in `activity_inbox.rs`.
