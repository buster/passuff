# Tasks: passuff-activity-inbox

Behavior batches. Regular scenarios first, then the end-to-end proofs. One commit may serve several scenarios in a batch.

Legend: an unchecked box means pending; a checked box is done only when the batch commit is filled in.

---

## Strict-mode proof (do first)

- [x] **Proof-of-strictness**: run the new delta against an undefined/`unimplemented!()` step. Confirm a non-zero exit and `failed > 0`.

      ```
./scripts/verify.sh with GIVN_FEATURES=/tmp/inbox-strict.feature
FAILED: Strict probe :: Undefined inbox step fails
  undefined Given  -> no inbox step binds this phrase
givn test-result: total 1, passed 0, failed 1, skipped 0
cargo exit status: 1
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

- [x] Create `crates/passuff-core/src/inbox.rs` (`InboxService`, `Inbox`, `InboxView`, `ReadOutcome`), the state-v2 types, the watch config, and the CLI `InboxCli`; add the two step files and generalize the e2e evidence path to the active change.

      COMMIT 1f9a3c8. Delta 32/32; permanent 80/80; unit 45 passed.
- [x] Seed fixtures: version-1 state, 500-record state, a watched PR for scope pruning, and a failing-save store. Implement `GIVN_SCENARIO` already exists; record the strict proof and the e2e count (3 < full).

      Delta regular 32/32; permanent 80/80; delta e2e 3/3; permanent e2e 4/4.
      Scenario-count proof: delta e2e 3 < full delta suite 32.

---

## Non-E2E Tasks

### Batch: `state-history`

- [x] Scenario: `A version-1 state loads with empty history`
  - [x] **Design constraints:** R7. v1 loads, watched PR preserved, version 2 written atomically; unknown version still quarantines; id counter recomputed.
        Files: `crates/passuff-core/src/{domain,state}.rs`, steps, support.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `state-history`

- [x] Scenario: `Activity history keeps the 500 most recent records`
  - [x] **Design constraints:** R5. Append after notification, drop oldest beyond 500, ids increase.
        Files: `crates/passuff-core/src/{cycle,state,domain}.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `state-history`

- [x] Scenario: `Recording activity survives a delivery failure`
  - [x] **Design constraints:** R5/R6. History is saved before delivery; a failing notification surface leaves the record and read/status queries work.
        Files: `crates/passuff-core/src/cycle.rs`, steps/support.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `state-history`

### Batch: `watch-scope`

- [x] Scenario: `A repository outside the watch scope is not watched`
  - [x] **Design constraints:** R39. Global allowlist validated; filter before classification; pruned from state; seeded watched PR + new comment proves no notification and status absence.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/cycle.rs`, steps/support.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `watch-scope`

### Batch: `inbox-service`

- [x] Scenario: `Inbox lists unread activity newest first`
  - [x] **Design constraints:** R1/R3. Newest-first, id/time/key/unread/summary, no enum names or body.
        Files: `crates/passuff-core/src/inbox.rs`, renderer, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Inbox with all includes read activity`
  - [x] **Design constraints:** R2. `--all` includes read; read tag shown.
        Files: `inbox.rs`, `transcript.rs`, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Read marks one activity read`
  - [x] **Design constraints:** R3. Newest id marked; 1 record read; 0 unread of 1.
        Files: `inbox.rs`, `InboxCli`, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Read marks only the named activity`
  - [x] **Design constraints:** R3. Two events, newest read, 1 unread of 2; inbox shows one read and one unread.
        Files: `inbox.rs`, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Read all marks every activity read`
  - [x] **Design constraints:** R4. Two records, all read, 0 unread of 2.
        Files: `inbox.rs`, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Read refuses an unknown activity id`
  - [x] **Design constraints:** R3. `no activity with id a999`, exit 2, no write.
        Files: `inbox.rs`, `InboxCli`, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Read state survives a restart`
  - [x] **Design constraints:** R6. Read flag persisted; new in-process runtime reads it.
        Files: `inbox.rs`, state, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `A state write failure is reported`
  - [x] **Design constraints:** R7/S7. v1 migration save failure -> `state error`, exit 3, no partial view.
        Files: `state.rs`, support (seeded failing store), steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Inbox with no state shows no activity`
  - [x] **Design constraints:** R8. Empty inbox, exit 0, no state file created.
        Files: `inbox.rs`, steps/support.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

### Batch: `cli-read`

- [x] Scenario: `Status lists configured accounts before the first check`
  - [x] **Design constraints:** S1/S6. Config accounts merged with empty state: `not resolved`, `last check never`.
        Files: `inbox.rs`, `InboxCli`, `transcript.rs`, steps.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status shows the login and last check after a check`
  - [x] **Design constraints:** S1. State login/last check rendered.
        Files: `inbox.rs`, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status shows the watch scope when all repositories are watched`
  - [x] **Design constraints:** S2. `all authored pull requests`.
        Files: `inbox.rs`, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status shows the watch scope when a repository allowlist is configured`
  - [x] **Design constraints:** S2. Joined allowlist.
        Files: config, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status lists watched pull requests with their state and CI`
  - [x] **Design constraints:** S3. Human `merged`/`CI failing`, title, no enum names.
        Files: `inbox.rs`, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status lists multiple accounts`
  - [x] **Design constraints:** S1. Two accounts and logins.
        Files: `inbox.rs`, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status reports unread and total activity`
  - [x] **Design constraints:** S4. `1 unread of 1`.
        Files: `inbox.rs`, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Status with no state shows empty sections`
  - [x] **Design constraints:** S6. Empty sections, exit 0.
        Files: `inbox.rs`, `transcript.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] REFACTOR and re-run. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

### Batch: `cli-read` (read hardening)

- [x] Scenario: `Status uses human labels for closed and pending`
  - [x] **Design constraints:** S3. Closed + pending labels, no enum names.
        Files: `transcript.rs`, `watch_status.feature`, steps.
  - [x] Implement and run. Must pass.
        ```
        ./scripts/verify.sh (delta) -> 32/32; permanent -> 80/80; scenario "Status uses human labels for closed and pending" green. COMMIT 8ffbe0b.
        ```
  - [x] REFACTOR and re-run.
        ```
        cargo fmt --all; delta 32/32, permanent 80/80. COMMIT 8ffbe0b.
        ```
  - [x] COMMIT: `8ffbe0b` -- Batch: `cli-read`

- [x] Scenario: `A read command rejects invalid configuration`
  - [x] **Design constraints:** S7/R1-R3. status/inbox/read config errors exit 2.
        Files: `inbox_cli.rs`, support, feature, steps.
  - [x] Implement and run. Must pass.
        ```
        ./scripts/verify.sh (delta) -> 32/32; permanent -> 80/80; scenario "A read command rejects invalid configuration" green. COMMIT 8ffbe0b.
        ```
  - [x] REFACTOR and re-run.
        ```
        cargo fmt --all; delta 32/32, permanent 80/80. COMMIT 8ffbe0b.
        ```
  - [x] COMMIT: `8ffbe0b` -- Batch: `cli-read`

### Batch: `inbox-service` (read hardening)

- [x] Scenario: `Inbox reports a state error`
  - [x] **Design constraints:** R7/S7. v1 migration save failure exits 3.
        Files: state, feature, steps.
  - [x] Implement and run. Must pass.
        ```
        ./scripts/verify.sh (delta) -> 32/32; permanent -> 80/80; scenario "Inbox reports a state error" green. COMMIT 8ffbe0b.
        ```
  - [x] REFACTOR and re-run.
        ```
        cargo fmt --all; delta 32/32, permanent 80/80. COMMIT 8ffbe0b.
        ```
  - [x] COMMIT: `8ffbe0b` -- Batch: `inbox-service`

- [x] Scenario: `Read reports a state error`
  - [x] **Design constraints:** R7. Read save failure exits 3.
        Files: support, feature, steps.
  - [x] Implement and run. Must pass.
        ```
        ./scripts/verify.sh (delta) -> 32/32; permanent -> 80/80; scenario "Read reports a state error" green. COMMIT 8ffbe0b.
        ```
  - [x] REFACTOR and re-run.
        ```
        cargo fmt --all; delta 32/32, permanent 80/80. COMMIT 8ffbe0b.
        ```
  - [x] COMMIT: `8ffbe0b` -- Batch: `inbox-service`

- [x] Scenario: `Read requires an activity id or all`
  - [x] **Design constraints:** R3. Neither target exits 2 with a usage line.
        Files: `inbox_cli.rs`, feature, steps.
  - [x] Implement and run. Must pass.
        ```
        ./scripts/verify.sh (delta) -> 32/32; permanent -> 80/80; scenario "Read requires an activity id or all" green. COMMIT 8ffbe0b.
        ```
  - [x] REFACTOR and re-run.
        ```
        cargo fmt --all; delta 32/32, permanent 80/80. COMMIT 8ffbe0b.
        ```
  - [x] COMMIT: `8ffbe0b` -- Batch: `inbox-service`

## End-to-end proofs

- [x] Scenario: `Developer sees the watch state` (status)
  - [x] **Design constraints:** S1-S4; real `passuff status` after a real check; transcript evidence.
        Files: `tests/steps/watch_status.rs`, `tests/support/e2e.rs`.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `cli-read`

- [x] Scenario: `Developer lists unread activity`
  - [x] **Design constraints:** R1; real `passuff inbox` after a check; transcript evidence.
        Files: `tests/steps/activity_inbox.rs`, e2e support.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

- [x] Scenario: `Developer marks activity read`
  - [x] **Design constraints:** R3/R4; real `passuff read --all` then `inbox`; transcript evidence.
        Files: steps, e2e support.
  - [x] Implement and run. Must pass. `<!-- evidence -->`
  - [x] COMMIT: `1f9a3c8` -- Batch: `inbox-service`

---

**Done when:** every box checked with a batch commit, the regular and end-to-end suites exit 0, and the setup proof records an end-to-end count (3) less than the full count.
