# Tasks: passuff-tui-live

### Batch: `live-observation`

- [x] Setup: confirm the existing Gherkin runner and strict mode.
  - `givn/commands.yaml` keeps `verify.command: ./scripts/verify.sh` and
    `verify.e2e_command: ./scripts/verify.sh --e2e`.
  - Strict-mode proof before implementation:
    `GIVN_FEATURES=$PWD/givn/changes/passuff-tui-live/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    exited 1 with `givn test-result: total 5, passed 0, failed 5, skipped 0`
    because the `@wip` scenarios had no bound steps (`Step doesn't match any
    function`). The runner uses `.fail_on_skipped()`.
  - Step files: `crates/passuff-cli/tests/steps/inbox_navigation.rs` and
    `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`; support in
    `crates/passuff-cli/tests/support/{mod.rs,e2e.rs}`.

- [x] Core: change probe over the GitHub boundary.
  - [x] `ApiAllowance`, `ProbePullRequest`, `ChangeProbe`, `ChangeSignal`.
  - [x] `GitHubGateway::probe` and `PROBE_QUERY` with `rateLimit`; the stack
        fallback is not needed because the probe requests no stack fields.
  - [x] `PassuffCore::probe` compares the sorted fingerprint, keeps `pending`
        until a successful cycle, and returns the minimum allowance.
  - [x] Unit tests: first probe changes, unchanged probe does not, a modified
        pull request does, an out-of-scope change does not, a failed cycle
        keeps `pending`.

- [x] CLI: observation session and pacing.
  - [x] `crates/passuff-cli/src/live.rs`: `ObservationSession::tick`,
        `next_delay`, check cycles only on a change and while
        `remaining >= 100`, delivery through `CliRuntime::deliver_report`,
        exponential probe backoff.
  - [x] `ObservationUpdate { config, changed, report, failure, next_delay }`;
        configuration reloaded from the path each tick.
  - [x] Pacing tests: 5,000/3,600 → 1.44 s, 100/3,600 → 72 s, 10/3,600 →
        720 s, missing allowance → 5 s, a past reset → 1 s.
  - [x] `delivery::build_delivery` shared by both binaries.

- [x] Terminal loop.
  - [x] Blocking input thread; `tokio::select!` over input and observation.
  - [x] Observation task ticks immediately, then sleeps `next_delay`.
  - [x] Footer failure line; cleared by the next successful update.
  - [x] `--snapshot --observe` and `--snapshot --observe-for <MILLIS>`;
        `--snapshot` without them stays view-only.

- [x] Scenario: `Developer opens the inbox TUI` (e2e, modified)
  - [x] **Design constraints:** `--snapshot --observe`; no external check; the
        inbox's own observation records `New comment from @alice` and
        `1 unread of 1`; exit 0.
  - [x] COMMIT: `PENDING` -- Batch: `live-observation`

- [x] Scenario: `The inbox shows activity observed while it is open` (e2e)
  - [x] **Design constraints:** real binary with `--snapshot --observe-for
        6000`; the comment is added to the mock 1.5 s into the run; the final
        frame shows `New comment from @alice` and `1 unread of 1`; exit 0.
  - [x] COMMIT: `PENDING` -- Batch: `live-observation`

- [x] Scenario: `The inbox keeps observation cheap while nothing changes`
  - [x] **Design constraints:** three unchanged probes; the fake gateway
        counts 1 full observation and 3 probes.
  - [x] COMMIT: `PENDING` -- Batch: `live-observation`

- [x] Scenario: `The inbox paces observation against the account allowance`
  - [x] **Design constraints:** 100 points with 3,600 seconds to reset; the
        planned delay is at least 30 seconds (design value 72 seconds).
  - [x] COMMIT: `PENDING` -- Batch: `live-observation`

- [x] Scenario: `A failed observation keeps the last inbox view`
  - [x] **Design constraints:** the first observation records the comment; the
        next probe fails; the frame still shows `New comment from @alice`; the
        loop continues and exits 0.
  - [x] COMMIT: `PENDING` -- Batch: `live-observation`

- [x] Unit test: a failed probe sets `failure` and keeps the previous view; the
      next successful update clears it (covered by the failure scenario and
      `ObservationSession::failed`).

**Done when:** every box checked with the commit; delta 5/5, delta e2e 2/2;
permanent 158/158 and permanent e2e 13/13; core lib 34/34; CLI lib 37/37.

### Closeout

- [x] Every phrase bound in the delta step files is bound in the permanent step
      files (`inbox_navigation.rs`, `activity_inbox_tui.rs`) — the delta step
      files do not exist; the steps were added directly to the permanent files.
- [x] README documents that the open inbox observes by itself, paces against
      the allowance, and that running the watch command at the same time is
      redundant.
