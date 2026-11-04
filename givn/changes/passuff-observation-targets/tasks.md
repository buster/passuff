# Tasks: passuff-observation-targets

Release note for every batch commit: `Observe only changed open pull requests and reconcile the open set hourly to cut check-cycle allowance use.`

### Batch: `terminal-retirement`

- [x] Setup: confirm the existing Gherkin runner and strict mode.
  - `givn/commands.yaml` keeps `verify.command: ./scripts/verify.sh` and
    `verify.e2e_command: ./scripts/verify.sh --e2e`. The runner executes
    `.feature` files under `givn/specs/` and the change's `specs/`.
  - Strict-mode proof before implementation:
    `GIVN_FEATURES=$PWD/givn/changes/passuff-observation-targets/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    exited 1 with `givn test-result: total 7, passed 0, failed 7, skipped 0`;
    the `@wip` scenarios failed on undefined steps and the terminal scenarios
    failed because the comment and review still notified. The runner uses
    `.fail_on_skipped()`.
  - Step files: `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
    and `crates/passuff-cli/tests/steps/inbox_navigation.rs`; support in
    `crates/passuff-cli/tests/support/{mod.rs,e2e.rs}`.
  - E2E: the change adds no `@e2e` scenario; all 7 delta scenarios are regular.
    The permanent `@e2e` scenarios keep running through
    `./scripts/verify.sh --e2e` (16 scenarios before this change).

- [x] Runner: skip `@givn.removed` scenarios in
      `crates/passuff-cli/tests/features_runner.rs`; they are archive
      directives and carry a placeholder step, not executable behavior.

- [x] Core: retirement of known terminal pull requests.
  - [x] Add `state: PrState` and `created_at: String` to `ProbePullRequest`;
        add `state` and `createdAt` to `PROBE_QUERY` and `GqlProbePr`; map the
        GraphQL state with the existing state helper.
  - [x] In `crates/passuff-core/src/cycle.rs`, skip classification when the
        previous observation is terminal and the current observation is
        terminal; do not record an activity event or a notification. A
        transition into the terminal state still classifies.
  - [x] Unit test: a comment and a review on a terminal-to-terminal observation
        produce no events; a terminal transition still produces `StateChange`.

- [x] Scenario: `Activity on a merged or closed pull request does not notify`
  - [x] **Design constraints:** a known watched pull request already in the
        merged or closed state receives a comment; run one check; the
        transcript reports no notifications. The terminal transition itself
        remains covered by the permanent scenarios. D1: terminal records stay
        in state and remain visible.
  - [x] COMMIT: `3ecd00d` -- Batch: `terminal-retirement`

- [x] Scenario: `A review on a merged pull request does not notify`
  - [x] **Design constraints:** the same retirement rule applies to the
        reviews event family; a submitted review on a terminal pull request
        produces no notification.
  - [x] COMMIT: `3ecd00d` -- Batch: `terminal-retirement`

- [x] Scenario: `A comment on a merged pull request reaches the inbox` (removed)
  - [x] **Design constraints:** the permanent `inbox-navigation` scenario is
        removed by the delta; the runner skips the `@givn.removed`
        placeholder. No notification or inbox activity for post-terminal
        comments.
  - [x] COMMIT: `3ecd00d` -- Batch: `terminal-retirement`

### Batch: `observation-targets`

- [x] Probe: account-isolated failures and relevant-change signal.
  - [x] `ChangeSignal` carries `failures: Vec<AccountFailure>`; `probe`
        continues past a failed account and keeps the minimum allowance across
        successful accounts.
  - [x] `PassuffCore` caches the last `WatchSnapshot` (per-account
        `last_check_at` and watched open node ids) from every refresh; `probe`
        reports `changed` only for a changed or new open id, a state
        transition, or a short-lived id. A known terminal pull request whose
        only change is `updatedAt` reports no change.
  - [x] Unit tests: terminal `updatedAt`-only change is not relevant; open
        change, transition, reopen, and short-lived are relevant; a failed
        account keeps the other accounts' signal.

- [x] Gateway: observe selected ids and discovery.
  - [x] Replace `GitHubGateway::observe(known_node_ids)` with
        `observe(target_ids, discovery)`; `Discovery::{None, New, All}` fetches
        discovered open assigned/review-requested ids per mode. Remove
        `MAIN_QUERY`; keep the stack-field retry on `NODES_QUERY`.
  - [x] The two search queries gain `is:open`.
  - [x] `AccountObservation` exposes the ids actually observed so the core can
        tell a missing node from an observed one.

- [x] Core: targeted pass and per-account baseline commitment.
  - [x] `refresh_targeted` selects changed open, new open, transitions, and
        short-lived ids, applies the watch scope and ignore list, observes with
        `Discovery::New`, merges, and classifies.
  - [x] `refresh_reconciliation` selects all watched open ids plus the probe
        targets, observes with `Discovery::All`, and classifies.
  - [x] Commit the probe baseline per account only after its observation
        succeeded; a failed account keeps its previous entry; a requested id
        missing from the response keeps its previous entry or stays absent.
  - [x] `ObservationSession::tick` calls `refresh_targeted` when the signal is
        relevant; the headless `run_watch_cycle` and the watch loop call
        `refresh_reconciliation`.
  - [x] Unit tests: target selection matrix, missing-id retention, failed
        account retention, scope filtering.

- [x] Harness: `FakeGateway` records observation id sets, supports
      `fail_next_observe`, and returns probe fingerprints with `state` and
      `createdAt`; config builders carry the new fields; the World exposes the
      recorded observation sets.

- [x] Scenario: `The inbox observes only the open pull request that changed`
  - [x] **Design constraints:** two watched open pull requests; a comment on
        `acme/api#42`; the startup reconciliation observes both once; the
        probe-targeted pass observes only `acme/api#42`. The assertion inspects
        the observation request after the startup pass.
  - [x] COMMIT: `099d451` -- Batch: `observation-targets`

- [x] Scenario: `A failed observation keeps the change pending`
  - [x] **Design constraints:** a comment arrives; the next observation fails;
        the transcript reports `account personal failed`; the following check
        reports `New comment`. The probe baseline for the failed account must
        not advance (per-account commitment).
  - [x] COMMIT: `099d451` -- Batch: `observation-targets`

### Batch: `reconciliation`

- [x] Config: `reconciliation_interval_secs`.
  - [x] Top-level `u64`, default `3600`, validated to be at least `1`;
        `FileConfig`, `AppConfig`, and `CoreConfig` plumbing; configuration
        unit tests for the default, an explicit value, and zero.
- [x] Live session: startup reconciliation, interval scheduling, below-100
      deferral. `ObservationUpdate` carries the interval conflict handling when
      the configuration changes.
- [x] Scenario: `The inbox reconciles open pull requests when the interval
      elapses`
  - [x] **Design constraints:** an assigned (non-authored) open pull request
        with a comment that the probe cannot see; `reconciliation_interval_secs
        = 2`; the comment is served after the startup pass; the TUI runs for 6
        seconds and shows the comment. The startup pass must not be the pass
        under test.
  - [x] COMMIT: `099d451` -- Batch: `reconciliation`

### Closeout

- [x] Merge every delta step phrase into the permanent
      `crates/passuff-cli/tests/steps/{pr_activity_notification,inbox_navigation}.rs`.
- [x] Run `GIVN_FEATURES=$PWD/givn/specs ./scripts/verify.sh`,
      `./scripts/verify.sh --e2e`, and the delta scope; record the totals.
