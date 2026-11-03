# Tasks: passuff-tui-live

### Batch: `live-observation`

- [ ] Setup: confirm the existing Gherkin runner and strict mode.
  - `givn/commands.yaml` keeps `verify.command: ./scripts/verify.sh` and
    `verify.e2e_command: ./scripts/verify.sh --e2e`.
  - Strict-mode proof before implementation:
    `GIVN_FEATURES=$PWD/givn/changes/passuff-tui-live/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    exited 1 with `givn test-result: total 5, passed 0, failed 5, skipped 0`
    because the `@wip` scenarios have no bound steps (`Step doesn't match any
    function`). The runner uses `.fail_on_skipped()`; undefined and pending
    steps fail.
  - Step skeletons live in `crates/passuff-cli/tests/steps/inbox_navigation.rs`
    and `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`; support in
    `crates/passuff-cli/tests/support/{mod.rs,e2e.rs}`.

- [ ] Core: change probe over the GitHub boundary.
  - [ ] `ApiAllowance`, `ProbePullRequest`, `ChangeProbe`, `ChangeSignal` in
        `crates/passuff-core/src/domain.rs`; glossary terms already updated.
  - [ ] `GitHubGateway::probe` and the `PROBE_QUERY` (open authored pull
        requests: identity, `updatedAt`, head OID, CI rollup state, repository,
        plus `rateLimit { cost remaining resetAt }`) in
        `crates/passuff-core/src/github.rs`; reuse the fallback when the stack
        fields are rejected.
  - [ ] `PassuffCore::probe` compares each account's sorted fingerprint with the
        last committed one, keeps `pending` until a check cycle succeeds, and
        returns `changed` plus the minimum allowance.
  - [ ] Unit tests: first probe changes, unchanged probe does not, a changed
        fingerprint does, a failed cycle keeps `pending`, watch scope filters
        the fingerprint.

- [ ] CLI: observation session and pacing.
  - [ ] `crates/passuff-cli/src/live.rs` with `ObservationSession::tick` and
        `ObservationSession::next_delay`; check cycles run only on a change and
        only while `remaining >= 100`; notifications go through
        `CliRuntime::deliver_report`; probe failures back off exponentially.
  - [ ] `ObservationUpdate { config, changed, report, failure, next_delay }`;
        the configuration is reloaded from the existing path each tick.
  - [ ] `ObservationSession::next_delay` unit tests: 5,000/3,600 → 1.44 s,
        100/3,600 → 72 s, 10/3,600 → 720 s, floor 1 s, never past the reset,
        missing allowance → 5 s.
  - [ ] `delivery::build_delivery` shared by `main.rs` and `tui_main.rs`.

- [ ] Terminal loop.
  - [ ] Replace the 1-second `event::poll` with a blocking input thread feeding
        an `mpsc` channel.
  - [ ] Spawn the observation task; `tokio::select!` over input and observation;
        redraw on an update and reload the read model; keep the picker in the
        same select loop.
  - [ ] Footer status line for `failure`; cleared by the next successful update.
  - [ ] `--snapshot --observe` (one probe and, on a change, one cycle) and
        `--snapshot --observe-for <MILLIS>` (real loop against `TestBackend`).
        `--snapshot` alone stays view-only.

- [ ] Steps and E2E support.
  - [ ] Bind the new phrases in `inbox_navigation.rs`; bind the observing-open
        phrase in `activity_inbox_tui.rs`.
  - [ ] `e2e_run_tui_observe` and `e2e_run_tui_observe_for` in `e2e.rs`; a
        staged `Respond` that returns the baseline first and the comment on
        later observations; probe controls on the fake gateway in `mod.rs`.

- [ ] Scenario: `Developer opens the inbox TUI` (e2e, modified)
  - [ ] **Design constraints:** `--snapshot --observe`; no external check; the
        inbox's own observation records `New comment from @alice` and
        `1 unread of 1`; exit 0.
  - [ ] COMMIT: `<filled at commit>` -- Batch: `live-observation`

- [ ] Scenario: `The inbox shows activity observed while it is open` (e2e)
  - [ ] **Design constraints:** real binary with `--snapshot --observe-for 4000`;
        the first observation is the baseline, the comment arrives afterwards;
        the final frame shows `New comment from @alice` and `1 unread of 1`;
        exit 0.
  - [ ] COMMIT: `<filled at commit>` -- Batch: `live-observation`

- [ ] Scenario: `The inbox keeps observation cheap while nothing changes`
  - [ ] **Design constraints:** opening forces one full observation; three
        unchanged probes; the fake gateway counts 1 full observation and 4
        probes; no full cycle when the probe reports no change.
  - [ ] COMMIT: `<filled at commit>` -- Batch: `live-observation`

- [ ] Scenario: `The inbox paces observation against the account allowance`
  - [ ] **Design constraints:** 100 points with 3,600 seconds to reset; the
        planned delay is at least 30 seconds (design value 72 seconds).
  - [ ] COMMIT: `<filled at commit>` -- Batch: `live-observation`

**Done when:** every box checked with the commit; delta 4/4, delta e2e 2/2;
permanent 158/158 and permanent e2e 13/13.

### Batch: `failure-fallback`

- [ ] Scenario: `A failed observation keeps the last inbox view`
  - [ ] **Design constraints:** the first observation succeeds and records the
        comment; the next probe fails; the frame still shows
        `New comment from @alice`; the loop continues and exits 0.
  - [ ] COMMIT: `<filled at commit>` -- Batch: `failure-fallback`

- [ ] Unit test: a failed probe sets `failure` and keeps the previous view; the
      next successful update clears it.

**Done when:** scenario green with the commit; delta 5/5, delta e2e 2/2;
permanent 158/158 and permanent e2e 13/13; no `@wip` left in the delta.

### Closeout

- [ ] Every phrase bound in the delta step files is bound in the permanent step
      files (`inbox_navigation.rs`, `activity_inbox_tui.rs`).
- [ ] README documents that the open inbox observes by itself and paces against
      the allowance, and that running the watch command at the same time is
      redundant.
