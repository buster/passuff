# Tasks: passuff-pr-watch

Behavior batches. Non-@e2e first, then @e2e. One commit may serve several scenarios in a batch.

Legend: `[ ]` pending · `[x]` done only when the batch commit is filled in. Check boxes immediately after each batch.

---

## Strict-mode proof (do first)

Empty/no-op steps are indistinguishable from PASS. Prove the runner rejects an unimplemented step before any scenario work.

- [ ] **Proof-of-strictness**: run `cargo test -p passuff-cli --features test-support --test features_runner` against an undefined/`unimplemented!()` step. Confirm non-zero exit and `failed > 0`.

      ```
      <!-- paste command + output -->
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

- [ ] Set up the workspace, `passuff-config`, `passuff-core`, `passuff-cli`, the Gherkin runner, and the one-file-per-capability step skeletons. `scripts/verify.sh` must run workspace unit tests and `cargo test -p passuff-cli --features test-support --test features_runner`.

- [ ] Configure strict mode (`World::cucumber().fail_on_skipped()`), prove it, then return to a passing baseline. Set `verify.command`, `verify.e2e_command`, and the `run` block in `givn/commands.yaml`.


---

## Non-E2E Tasks


### Batch: `classification-core`

- [ ] Scenario: `A review comment from another person notifies`
  - [ ] **Design constraints:** Rule R3/R4/R9. Another person's review comment on an authored PR; `ActivityClassifier::classify` emits `ReviewComment`; `RefreshReport`; `TranscriptRenderer` prints `Review comment from @<login>`.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A submitted review notifies`
  - [ ] **Design constraints:** Rule R5/R9. Review states approved/changes requested/commented; `ActivityClassifier::classify`; exact `Review <state> by @<login>` string.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A review from a configured account stays silent`
  - [ ] **Design constraints:** Rule R9. Configured login set filter; own review is not an `ActivityEvent`; no notification.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A commit push stays silent`
  - [ ] **Design constraints:** Rule R10. A new head OID alone is silent; `ci_failed` resets for the new commit; no comment/review/state event.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `Activity on someone else's pull request stays silent`
  - [ ] **Design constraints:** Rule R2/R11. Authored-only filter; a non-authored PR is ignored by `PassuffCore::refresh`.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A failing CI result notifies`
  - [ ] **Design constraints:** Rule R6. `CiResult` FAILURE/ERROR sets `ci_failed` and emits `CiFailure`; `CI failing` transcript string.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A CI recovery notifies`
  - [ ] **Design constraints:** Rule R7. `ci_failed` clears on SUCCESS and emits `CiRecovery`; `CI recovered` string.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A CI recovery across pending notifies`
  - [ ] **Design constraints:** Rule R7. PENDING preserves `ci_failed`; a later SUCCESS clears it and emits recovery; no notification on the pending check.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A bot comment is handled according to the bot setting`
  - [ ] **Design constraints:** Rule R12/R23. `bot_comments` filter; configured login set precedence; exact `@dependabot[bot]` login.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `Multiple events on one pull request produce one notification`
  - [ ] **Design constraints:** Rule R13. Group by PR per cycle; one `Notification`; one sound; `RenderedNotification` body lists events.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A disabled event family suppresses its notification`
  - [ ] **Design constraints:** Rule R23. `EventToggles`; `comments` covers conversation and review comments; prior CI state set by the scenario.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`

- [ ] Scenario: `A non-notified change stays silent`
  - [ ] **Design constraints:** Rule R25. Draft/assignees/labels/milestone/base branch are fetched but ignored by `ActivityClassifier::classify`.
        Files: `crates/passuff-core/src/{domain,classifier,github,cycle}.rs`, `crates/passuff-cli/src/{transcript,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `classification-core`


### Batch: `lifecycle-state`

- [ ] Scenario: `A pull request merged or closed while the watcher was stopped is reported`
  - [ ] **Design constraints:** Rule R8/R16. Known `WatchedPullRequest`; persisted state; state transition emits once; no replay of seen activity.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`

- [ ] Scenario: `A short-lived pull request merged or closed between checks notifies`
  - [ ] **Design constraints:** Rule R27. No prior observation; account has a prior cycle; `createdAt > last_check_at`; record `current.state_reported = true`.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`

- [ ] Scenario: `A merge performed by the configured account still notifies`
  - [ ] **Design constraints:** Rule R8. State change notifies regardless of actor; own action is limited to comments/reviews.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`

- [ ] Scenario: `A reopened pull request stays silent`
  - [ ] **Design constraints:** Rule R28. Reopen sets state OPEN and resets `state_reported`; no event.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`

- [ ] Scenario: `A reopened pull request notifies again when closed`
  - [ ] **Design constraints:** Rule R28. Reopen reset; later close emits; known closed/merged records retained 30 days.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`

- [ ] Scenario: `Historical activity is not replayed on first observation`
  - [ ] **Design constraints:** Rule R14. First observation of an open PR records `Baseline`; no events; `baselines_recorded`.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`

- [ ] Scenario: `A pull request first seen after the first cycle records a baseline`
  - [ ] **Design constraints:** Rule R26. First observation of every PR records a baseline, even later; `RefreshReport.baselines_recorded`; no events.
        Files: `crates/passuff-core/src/{state,classifier,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `lifecycle-state`


### Batch: `accounts-state`

- [ ] Scenario: `Each configured account is observed`
  - [ ] **Design constraints:** Rule R1/R17. Union configured login set; per-account observation; each account's notification.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `A comment from another configured account stays silent`
  - [ ] **Design constraints:** Rule R9. Configured login set includes the other account; own action filter.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `A failed account does not stop the other account`
  - [ ] **Design constraints:** Rule R17/R29/R31. Account failure; healthy account still observed; exit 1; no empty step.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `A restart does not replay notified activity`
  - [ ] **Design constraints:** Rule R15. Persisted seen IDs; state loaded on restart; no duplicate.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `A corrupt state file is replaced with a baseline`
  - [ ] **Design constraints:** Rule R18. Quarantine corrupt/unknown version; fresh `Baseline`; `Fresh baseline recorded for <key>`.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `An invalid configuration is rejected`
  - [ ] **Design constraints:** Rule R19/R31. `ConfigError`; `CliRuntime::run_check(Err(...))`; exit 2.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `A state write failure is reported with a nonzero status`
  - [ ] **Design constraints:** Rule R30/R31. `with_ports` failing `SnapshotStore`; `CoreError::State`; exit 3; no notification.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `No authored pull requests produce no notifications`
  - [ ] **Design constraints:** Rule R2/R17/R31. Empty authored set; `RefreshReport` empty; exit 0.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`

- [ ] Scenario: `A single check with no changes reports no notifications`
  - [ ] **Design constraints:** Rule R13/R21/R31. Empty cycle; no notifications; no desktop/sound; exit 0.
        Files: `crates/passuff-core/src/{credential,state,github,cycle}.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `accounts-state`


### Batch: `delivery-sound`

- [ ] Scenario: `A configured sound is used`
  - [ ] **Design constraints:** Rule R24. `NotificationsConfig.sound_file`; `DesktopDelivery` with `RecordingSoundPlayer`; path recorded.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `delivery-sound`

- [ ] Scenario: `A disabled sound produces no sound`
  - [ ] **Design constraints:** Rule R20/R24. `NotificationsConfig.sound = false`; no `SoundPlayer` call; desktop notification still recorded.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `delivery-sound`

- [ ] Scenario: `A disabled desktop notification keeps the transcript`
  - [ ] **Design constraints:** Rule R20. `NotificationsConfig.desktop = false`; transcript printed; sound still played.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `delivery-sound`

- [ ] Scenario: `An unavailable desktop surface keeps the transcript`
  - [ ] **Design constraints:** Rule R20/7a. `DesktopDelivery` error; transcript still printed; exit 0; `desktop notification failed`.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `delivery-sound`

- [ ] Scenario: `A missing sound player still delivers the notification`
  - [ ] **Design constraints:** Rule R24/7b. `SoundOutcome::Unavailable`; desktop notification delivered; transcript `sound unavailable`; exit 0.
        Files: `crates/passuff-cli/src/{transcript,delivery,sound,lib}.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `delivery-sound`


---

## E2E Smoke Tests

- [ ] Bring up the local environment: fake GitHub, private `dbus-daemon` with `FakeNotificationService`, fake sound player, temp config/state. Confirm clean startup.

- [ ] Set up e2e step definitions in `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, prove strict mode for the e2e runner, set `verify.e2e_command`, and confirm the runner reaches the fake environment.

- [ ] @e2e Scenario: `Developer starts watching and is notified of a new comment`
  - [ ] **Design constraints:** Rule E2E 1. Real `passuff` CLI; `passuff: watching` after baseline; private D-Bus fake; RecordingDelivery only on macOS; transcript evidence.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `e2e`

- [ ] @e2e Scenario: `Developer runs a single check and sees the current activity`
  - [ ] **Design constraints:** Rule E2E 2. Real CLI `check`; fake GitHub; Linux D-Bus fake; transcript and `Notify` assertion; exit 0.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `e2e`

- [ ] @e2e Scenario: `Developer asks for usage`
  - [ ] **Design constraints:** Rule E2E 3. Real CLI `--help`; command list; exit 0.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `e2e`

- [ ] @e2e Scenario: `Developer asks for the version`
  - [ ] **Design constraints:** Rule E2E 4. Real CLI `--version`; `passuff <semver>`; exit 0.
        Files: `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`, `crates/passuff-cli/tests/support/`, `crates/passuff-cli/src/lib.rs`
  - [ ] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
        <!-- paste test/implementation evidence -->
        ```
  - [ ] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
        <!-- paste REFACTOR evidence -->
        ```
  - [ ] COMMIT: `pending` -- Batch: `e2e`


---

**Done when:** every box checked with a batch commit, `./scripts/verify.sh` and `./scripts/verify.sh --e2e` exit 0 on the full suite, and the e2e setup proof records an e2e count strictly less than the full count.
