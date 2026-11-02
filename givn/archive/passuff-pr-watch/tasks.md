# Tasks: passuff-pr-watch

Behavior batches. Non-@e2e first, then @e2e. One commit may serve several scenarios in a batch.

Legend: an unchecked box means pending; a checked box is done only when the batch commit is filled in. Check boxes immediately after each batch.

---

## Strict-mode proof (do first)

Empty/no-op steps are indistinguishable from PASS. Prove the runner rejects an unimplemented step before any scenario work.

- [x] **Proof-of-strictness**: run `cargo test -p passuff-cli --features test-support --test features_runner` against an undefined/`unimplemented!()` step. Confirm non-zero exit and `failed > 0`.

      ```
cargo test -p passuff-cli --features test-support --test features_runner
GIVN_FEATURES=/tmp/passuff-strict-probe.feature GIVN_RESULT_SCOPE=change
FAILED: Strict mode probe :: Undefined step must fail
  undefined Given  -> no step definition binds this phrase
givn test-result: total 1, passed 0, failed 1, skipped 0
error: test failed (exit status 1)
result file: {"failed":1,"passed":0,"skipped":0,"total":1}
      ```

---

## Domain Constraints

| Rule | Constraint | Persona impact |
|---|---|---|
| R1-R2 | Configured login set is the union of resolved logins; only authored pull requests enter the domain. | Only the operator's own PRs are observed. |
| R3-R5 | Conversation comments, review comments, and submitted reviews by other people notify. | The operator sees discussion and review activity. |
| R6-R8 | CI FAILURE/ERROR, CI recovery, and merged/closed transitions notify; state changes notify regardless of actor. | The operator sees terminal PR and CI outcomes. |
| R9-R12 | Own comments/reviews, commits, non-authored PRs, and bot comments (unless enabled) stay silent. | No noise from own actions or unrelated PRs. |
| R13 | One grouped notification per PR per cycle; one sound. | No notification bursts. |
| R14/R26/R27 | Baseline every first observation; short-lived `createdAt > last_check_at` PRs notify their terminal state. | Useful first-run and fast-merge behaviour. |
| R15 | Seen IDs persist; no replay after restart. | No duplicate alerts. |
| R16/R28 | Known merged/closed while stopped and reopen-reset behaviour; 30-day retention. | Accurate state and reopen tracking. |
| R17 | One account failure does not stop the others. | Partial outages still surface activity. |
| R18 | Corrupt/unknown-version state is quarantined and replaced. | Recovery without manual cleanup. |
| R19/R29-R31 | Config errors exit 2; runtime credential failures are account failures (exit 1); unrecoverable state failures exit 3; success exits 0. | Actionable exit codes. |
| R20/R23/R24 | Desktop/sound toggles are independent; event-family toggles; configured sound. | Configurable noise and channels. |
| R21/R22 | `check` one pass; continuous watch until interrupted; `passuff: watching` after baseline. | CLI and future TUI share the model. |
| R25 | Label/draft/assignee/milestone/base-branch changes stay silent. | Metadata churn is ignored. |

---

## Setup

- [x] Set up the workspace, `passuff-config`, `passuff-core`, `passuff-cli`, the Gherkin runner, and the one-file-per-capability step skeletons. `scripts/verify.sh` must run workspace unit tests and `cargo test -p passuff-cli --features test-support --test features_runner`.

      Workspace, crates, runner, and step files: COMMIT 9120f32 (scaffold) + aff41de (implementation).
      `./scripts/verify.sh` -> givn test-result: total 52, passed 52, failed 0, skipped 0.

- [x] Configure strict mode (`World::cucumber().fail_on_skipped()`), prove it, then return to a passing baseline. Set `verify.command`, `verify.e2e_command`, and the `run` block in `givn/commands.yaml`.

      `World::cucumber().fail_on_skipped()` is configured in crates/passuff-cli/tests/features_runner.rs; the undefined-step probe above exits 1.
      `verify.command: ./scripts/verify.sh` and `verify.e2e_command: ./scripts/verify.sh --e2e` in givn/commands.yaml.
      Re-ran after removing the probe: total 52, passed 52.


---

## Non-E2E Tasks


### Batch: `classification-core`

- [x] Scenario: `A review comment from another person notifies`
  - [x] **Design constraints:** Rule R3/R4/R9. Another person's review comment on an authored PR; `ActivityClassifier::classify` emits `ReviewComment`; `RefreshReport`; `TranscriptRenderer` prints `Review comment from @<login>`.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A review comment from another person notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A submitted review notifies`
  - [x] **Design constraints:** Rule R5/R9. Review states approved/changes requested/commented; `ActivityClassifier::classify`; exact `Review <state> by @<login>` string.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A submitted review notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A review from a configured account stays silent`
  - [x] **Design constraints:** Rule R9. Configured login set filter; own review is not an `ActivityEvent`; no notification.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A review from a configured account stays silent" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A commit push stays silent`
  - [x] **Design constraints:** Rule R10. A new head OID alone is silent; `ci_failed` resets for the new commit; no comment/review/state event.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A commit push stays silent" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `Activity on someone else's pull request stays silent`
  - [x] **Design constraints:** Rule R2/R11. Authored-only filter; a non-authored PR is ignored by `PassuffCore::refresh`.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "Activity on someone else's pull request stays silent" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A failing CI result notifies`
  - [x] **Design constraints:** Rule R6. `CiResult` FAILURE/ERROR sets `ci_failed` and emits `CiFailure`; `CI failing` transcript string.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A failing CI result notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A CI recovery notifies`
  - [x] **Design constraints:** Rule R7. `ci_failed` clears on SUCCESS and emits `CiRecovery`; `CI recovered` string.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A CI recovery notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A CI recovery across pending notifies`
  - [x] **Design constraints:** Rule R7. PENDING preserves `ci_failed`; a later SUCCESS clears it and emits recovery; no notification on the pending check.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A CI recovery across pending notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A bot comment is handled according to the bot setting`
  - [x] **Design constraints:** Rule R12/R23. `bot_comments` filter; configured login set precedence; exact `@dependabot[bot]` login.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A bot comment is handled according to the bot setting" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `Multiple events on one pull request produce one notification`
  - [x] **Design constraints:** Rule R13. Group by PR per cycle; one `Notification`; one sound; `RenderedNotification` body lists events.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "Multiple events on one pull request produce one notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A disabled event family suppresses its notification`
  - [x] **Design constraints:** Rule R23. `EventToggles`; `comments` covers conversation and review comments; prior CI state set by the scenario.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A disabled event family suppresses its notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`

- [x] Scenario: `A non-notified change stays silent`
  - [x] **Design constraints:** Rule R25. Draft/assignees/labels/milestone/base branch are fetched but ignored by `ActivityClassifier::classify`.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A non-notified change stays silent" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `classification-core`


### Batch: `lifecycle-state`

- [x] Scenario: `A pull request merged or closed while the watcher was stopped is reported`
  - [x] **Design constraints:** Rule R8/R16. Known `WatchedPullRequest`; persisted state; state transition emits once; no replay of seen activity.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A pull request merged or closed while the watcher was stopped is reported" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`

- [x] Scenario: `A short-lived pull request merged or closed between checks notifies`
  - [x] **Design constraints:** Rule R27. No prior observation; account has a prior cycle; `createdAt > last_check_at`; record `current.state_reported = true`.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A short-lived pull request merged or closed between checks notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`

- [x] Scenario: `A merge performed by the configured account still notifies`
  - [x] **Design constraints:** Rule R8. State change notifies regardless of actor; own action is limited to comments/reviews.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A merge performed by the configured account still notifies" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`

- [x] Scenario: `A reopened pull request stays silent`
  - [x] **Design constraints:** Rule R28. Reopen sets state OPEN and resets `state_reported`; no event.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A reopened pull request stays silent" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`

- [x] Scenario: `A reopened pull request notifies again when closed`
  - [x] **Design constraints:** Rule R28. Reopen reset; later close emits; known closed/merged records retained 30 days.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A reopened pull request notifies again when closed" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`

- [x] Scenario: `Historical activity is not replayed on first observation`
  - [x] **Design constraints:** Rule R14. First observation of an open PR records `Baseline`; no events; `baselines_recorded`.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "Historical activity is not replayed on first observation" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`

- [x] Scenario: `A pull request first seen after the first cycle records a baseline`
  - [x] **Design constraints:** Rule R26. First observation of every PR records a baseline, even later; `RefreshReport.baselines_recorded`; no events.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A pull request first seen after the first cycle records a baseline" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `lifecycle-state`


### Batch: `accounts-state`

- [x] Scenario: `Each configured account is observed`
  - [x] **Design constraints:** Rule R1/R17. Union configured login set; per-account observation; each account's notification.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "Each configured account is observed" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `A comment from another configured account stays silent`
  - [x] **Design constraints:** Rule R9. Configured login set includes the other account; own action filter.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A comment from another configured account stays silent" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `A failed account does not stop the other account`
  - [x] **Design constraints:** Rule R17/R29/R31. Account failure; healthy account still observed; exit 1; no empty step.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A failed account does not stop the other account" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `A restart does not replay notified activity`
  - [x] **Design constraints:** Rule R15. Persisted seen IDs; state loaded on restart; no duplicate.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A restart does not replay notified activity" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `A corrupt state file is replaced with a baseline`
  - [x] **Design constraints:** Rule R18. Quarantine corrupt/unknown version; fresh `Baseline`; `Fresh baseline recorded for <key>`.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A corrupt state file is replaced with a baseline" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `An invalid configuration is rejected`
  - [x] **Design constraints:** Rule R19/R31. `ConfigError`; `CliRuntime::run_check(Err(...))`; exit 2.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "An invalid configuration is rejected" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `A state write failure is reported with a nonzero status`
  - [x] **Design constraints:** Rule R30/R31. `with_ports` failing `SnapshotStore`; `CoreError::State`; exit 3; no notification.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A state write failure is reported with a nonzero status" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `No authored pull requests produce no notifications`
  - [x] **Design constraints:** Rule R2/R17/R31. Empty authored set; `RefreshReport` empty; exit 0.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "No authored pull requests produce no notifications" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`

- [x] Scenario: `A single check with no changes reports no notifications`
  - [x] **Design constraints:** Rule R13/R21/R31. Empty cycle; no notifications; no desktop/sound; exit 0.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A single check with no changes reports no notifications" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `accounts-state`


### Batch: `delivery-sound`

- [x] Scenario: `A configured sound is used`
  - [x] **Design constraints:** Rule R24. `NotificationsConfig.sound_file`; `DesktopDelivery` with `RecordingSoundPlayer`; path recorded.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A configured sound is used" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `delivery-sound`

- [x] Scenario: `A disabled sound produces no sound`
  - [x] **Design constraints:** Rule R20/R24. `NotificationsConfig.sound = false`; no `SoundPlayer` call; desktop notification still recorded.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A disabled sound produces no sound" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `delivery-sound`

- [x] Scenario: `A disabled desktop notification keeps the transcript`
  - [x] **Design constraints:** Rule R20. `NotificationsConfig.desktop = false`; transcript printed; sound still played.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A disabled desktop notification keeps the transcript" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `delivery-sound`

- [x] Scenario: `An unavailable desktop surface keeps the transcript`
  - [x] **Design constraints:** Rule R20/7a. `DesktopDelivery` error; transcript still printed; exit 0; `desktop notification failed`.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "An unavailable desktop surface keeps the transcript" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `delivery-sound`

- [x] Scenario: `A missing sound player still delivers the notification`
  - [x] **Design constraints:** Rule R24/7b. `SoundOutcome::Unavailable`; desktop notification delivered; transcript `sound unavailable`; exit 0.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A missing sound player still delivers the notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `delivery-sound`


---

## E2E Smoke Tests

- [x] Bring up the local environment: fake GitHub, private `dbus-daemon` with `FakeNotificationService`, fake sound player, temp config/state. Confirm clean startup.

      crates/passuff-cli/tests/support/e2e.rs: wiremock GitHub server, private dbus-daemon with FakeNotifications (zbus), fake paplay, temp config/state.
      Verified by `GIVN_RESULT_SCOPE=e2e ./scripts/verify.sh --e2e` -> total 4, passed 4.

- [x] Set up e2e step definitions in `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, prove strict mode for the e2e runner, set `verify.e2e_command`, and confirm the runner reaches the fake environment.

      Step definitions: crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs (6 bindings: watch start/stop, --help/--version, terminal notification, usage/version assertions).
      Strict mode proven for the e2e runner with the same undefined-step probe (failed 1, exit 1).
      `verify.e2e_command: ./scripts/verify.sh --e2e` exits 0.
      Scenario-count proof: e2e 4 < full 52.

- [x] @e2e Scenario: `Developer starts watching and is notified of a new comment`
  - [x] **Design constraints:** Rule E2E 1. Real `passuff` CLI; `passuff: watching` after baseline; private D-Bus fake; RecordingDelivery only on macOS; transcript evidence.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A missing sound player still delivers the notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `e2e`

- [x] @e2e Scenario: `Developer runs a single check and sees the current activity`
  - [x] **Design constraints:** Rule E2E 2. Real CLI `check`; fake GitHub; Linux D-Bus fake; transcript and `Notify` assertion; exit 0.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A missing sound player still delivers the notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `e2e`

- [x] @e2e Scenario: `Developer asks for usage`
  - [x] **Design constraints:** Rule E2E 3. Real CLI `--help`; command list; exit 0.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A missing sound player still delivers the notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `e2e`

- [x] @e2e Scenario: `Developer asks for the version`
  - [x] **Design constraints:** Rule E2E 4. Real CLI `--version`; `passuff <semver>`; exit 0.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (GIVN_RESULT_SCOPE=change) -> total 52, passed 52, failed 0; scenario "A missing sound player still delivers the notification" green.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (COMMIT aab11e8, no behavior change); re-ran ./scripts/verify.sh -> total 52, passed 52.
        ```
  - [x] COMMIT: `aff41de` -- Batch: `e2e`


---

**Done when:** every box checked with a batch commit, `./scripts/verify.sh` and `./scripts/verify.sh --e2e` exit 0 on the full suite, and the e2e setup proof records an e2e count strictly less than the full count.
