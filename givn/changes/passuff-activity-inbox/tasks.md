# Tasks: passuff-activity-inbox

Behavior batches. Regular scenarios first, then the end-to-end proofs. One commit may serve several scenarios in a batch.

Legend: an unchecked box means pending; a checked box is done only when the batch commit is filled in.

---

## Strict-mode proof (do first)

- [ ] **Proof-of-strictness**: run the new delta against an undefined/`unimplemented!()` step. Confirm a non-zero exit and `failed > 0`.

      ```
      <!-- paste command + output -->
      ```

---

## Domain Constraints

| Rule | Constraint | Persona impact |
|---|---|---|
| S1-S7 | Status reports configured accounts (or `not resolved`), scope, watched PRs with human state/CI, unread/total; never contacts GitHub; missing state is empty; config 2 / state 3. | The operator can inspect the watch without guessing. |
| R1-R4 | Inbox lists unread/all newest first; read marks one or all, unknown id exits 2. | Activity can be reviewed and acknowledged. |
| R5-R6 | One record per event, bounded at 500, ids stable, read state persists across restart. | Bounded, durable history. |
| R7 | Version-1 state migrates to version 2, preserves watched PRs, empty history; rewrite failure is a state error. | Safe upgrade. |
| R8-R9 | Missing state is an empty inbox and no file is created; history never replays notifications. | No surprise state or duplicate alerts. |
| R39 | Global repository allowlist filters observation and prunes out-of-scope watched PRs. | Quiet accounts stay quiet. |

---

## Setup

- [ ] Create `crates/passuff-core/src/inbox.rs` (`InboxService`, `Inbox`, `InboxView`, `ReadOutcome`), the state-v2 types, the watch config, and the CLI `InboxCli`; add the two step files and generalize the e2e evidence path to the active change.
- [ ] Seed fixtures: version-1 state, 500-record state, a watched PR for scope pruning, and a failing-save store. Implement `GIVN_SCENARIO` already exists; record the strict proof and the e2e count (3 < full).

---

## Non-E2E Tasks

### Batch: `state-history`

- [ ] Scenario: `A version-1 state loads with empty history`
  - [ ] **Design constraints:** R7. v1 loads, watched PR preserved, version 2 written atomically; unknown version still quarantines; id counter recomputed.
        Files: `crates/passuff-core/src/{domain,state}.rs`, steps, support.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `state-history`

- [ ] Scenario: `Activity history keeps the 500 most recent records`
  - [ ] **Design constraints:** R5. Append after notification, drop oldest beyond 500, ids increase.
        Files: `crates/passuff-core/src/{cycle,state,domain}.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `state-history`

- [ ] Scenario: `Recording activity survives a delivery failure`
  - [ ] **Design constraints:** R5/R6. History is saved before delivery; a failing notification surface leaves the record and read/status queries work.
        Files: `crates/passuff-core/src/cycle.rs`, steps/support.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `state-history`

### Batch: `watch-scope`

- [ ] Scenario: `A repository outside the watch scope is not watched`
  - [ ] **Design constraints:** R39. Global allowlist validated; filter before classification; pruned from state; seeded watched PR + new comment proves no notification and status absence.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/cycle.rs`, steps/support.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `watch-scope`

### Batch: `inbox-service`

- [ ] Scenario: `Inbox lists unread activity newest first`
  - [ ] **Design constraints:** R1/R3. Newest-first, id/time/key/unread/summary, no enum names or body.
        Files: `crates/passuff-core/src/inbox.rs`, renderer, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Inbox with all includes read activity`
  - [ ] **Design constraints:** R2. `--all` includes read; read tag shown.
        Files: `inbox.rs`, `transcript.rs`, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Read marks one activity read`
  - [ ] **Design constraints:** R3. Newest id marked; 1 record read; 0 unread of 1.
        Files: `inbox.rs`, `InboxCli`, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Read marks only the named activity`
  - [ ] **Design constraints:** R3. Two events, newest read, 1 unread of 2; inbox shows one read and one unread.
        Files: `inbox.rs`, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Read all marks every activity read`
  - [ ] **Design constraints:** R4. Two records, all read, 0 unread of 2.
        Files: `inbox.rs`, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Read refuses an unknown activity id`
  - [ ] **Design constraints:** R3. `no activity with id a999`, exit 2, no write.
        Files: `inbox.rs`, `InboxCli`, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Read state survives a restart`
  - [ ] **Design constraints:** R6. Read flag persisted; new in-process runtime reads it.
        Files: `inbox.rs`, state, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `A state write failure is reported`
  - [ ] **Design constraints:** R7/S7. v1 migration save failure -> `state error`, exit 3, no partial view.
        Files: `state.rs`, support (seeded failing store), steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Inbox with no state shows no activity`
  - [ ] **Design constraints:** R8. Empty inbox, exit 0, no state file created.
        Files: `inbox.rs`, steps/support.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

### Batch: `cli-read`

- [ ] Scenario: `Status lists configured accounts before the first check`
  - [ ] **Design constraints:** S1/S6. Config accounts merged with empty state: `not resolved`, `last check never`.
        Files: `inbox.rs`, `InboxCli`, `transcript.rs`, steps.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status shows the login and last check after a check`
  - [ ] **Design constraints:** S1. State login/last check rendered.
        Files: `inbox.rs`, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status shows the watch scope when all repositories are watched`
  - [ ] **Design constraints:** S2. `all authored pull requests`.
        Files: `inbox.rs`, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status shows the watch scope when a repository allowlist is configured`
  - [ ] **Design constraints:** S2. Joined allowlist.
        Files: config, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status lists watched pull requests with their state and CI`
  - [ ] **Design constraints:** S3. Human `merged`/`CI failing`, title, no enum names.
        Files: `inbox.rs`, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status lists multiple accounts`
  - [ ] **Design constraints:** S1. Two accounts and logins.
        Files: `inbox.rs`, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status reports unread and total activity`
  - [ ] **Design constraints:** S4. `1 unread of 1`.
        Files: `inbox.rs`, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Status with no state shows empty sections`
  - [ ] **Design constraints:** S6. Empty sections, exit 0.
        Files: `inbox.rs`, `transcript.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] REFACTOR and re-run. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

## End-to-end proofs

- [ ] Scenario: `Developer sees the watch state` (status)
  - [ ] **Design constraints:** S1-S4; real `passuff status` after a real check; transcript evidence.
        Files: `tests/steps/watch_status.rs`, `tests/support/e2e.rs`.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `cli-read`

- [ ] Scenario: `Developer lists unread activity`
  - [ ] **Design constraints:** R1; real `passuff inbox` after a check; transcript evidence.
        Files: `tests/steps/activity_inbox.rs`, e2e support.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

- [ ] Scenario: `Developer marks activity read`
  - [ ] **Design constraints:** R3/R4; real `passuff read --all` then `inbox`; transcript evidence.
        Files: steps, e2e support.
  - [ ] Implement and run. Must pass. `<!-- evidence -->`
  - [ ] COMMIT: `pending` -- Batch: `inbox-service`

---

**Done when:** every box checked with a batch commit, the regular and end-to-end suites exit 0, and the setup proof records an end-to-end count (3) less than the full count.
