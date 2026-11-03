# Tasks: passuff-activity-detail

### Batch: `activity-detail`

- [ ] Setup: confirm the runner and strict mode.
  - `verify.command`/`verify.e2e_command` unchanged.
  - `GIVN_FEATURES=$PWD/givn/changes/passuff-activity-detail/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    must exit 1 with the `@wip` scenarios failing on undefined steps.
  - Step file: `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`.

- [ ] Core: expose the pull request URL.
  - [ ] `PullRequestView.url` in `crates/passuff-core/src/inbox.rs`, copied from
        `WatchedPullRequest.url`.
  - [ ] Update the `tui.rs` test-view constructors.

- [ ] TUI: detail state, modal keys, rendering.
  - [ ] `ActivityDetail { record, pull_request, url }`; `TuiApp.popup`.
  - [ ] `TuiAction::OpenUrl(String)`; modal branch before the existing match:
        Escape closes, Enter returns `OpenUrl`, other keys are ignored.
  - [ ] Enter with `Pane::Activity` and a selected record opens the detail;
        Enter in `Pane::Watched` or with no activity changes nothing.
  - [ ] `TuiApp::close_popup`; observation reload leaves the popup untouched.
  - [ ] Render the centered `Clear` + bordered overlay (70% x 9) with the six
        content lines from the design.

- [ ] CLI: browser adapter.
  - [ ] `crates/passuff-cli/src/browser.rs`: `BrowserOpener`,
        `SystemBrowserOpener` (`xdg-open` on Linux, `open` on macOS); spawn and
        non-zero exit are errors.
  - [ ] `tui_main.rs` handles `OpenUrl` in the interactive loop and the
        `--keys` snapshot path: success closes the popup, failure sets the
        footer status and keeps it.
  - [ ] `--keys` accepts `enter` and `esc`.

- [ ] Tests and support.
  - [ ] World: recording/failing `BrowserOpener`, `run_tui_press` applies
        `OpenUrl`, new assertions (`the browser opened "<url>"`, `the browser
        will fail to open`).
  - [ ] E2E: fake `xdg-open` on `PATH` writing the URL to a log;
        `configure_child` prepends the bin dir; the browser assertion reads it.

- [ ] Scenario: `Developer opens an activity's details` (e2e)
  - [ ] **Design constraints:** `--keys "enter"`; popup shows `Activity a1`,
        `New comment from @alice`, `acme/api#42`, `Pull request acme/api#42`,
        `unread`, `Enter open in browser`; exit 0.
  - [ ] COMMIT: `PENDING` -- Batch: `activity-detail`

- [ ] Scenario: `Developer opens the pull request from an activity` (e2e)
  - [ ] **Design constraints:** `--keys "enter,enter"`; fake opener records the
        URL; popup closes; exit 0.
  - [ ] COMMIT: `PENDING` -- Batch: `activity-detail`

- [ ] Scenario: `Escape closes the activity detail`
  - [ ] **Design constraints:** Enter opens, Escape closes, focus stays activity.
  - [ ] COMMIT: `PENDING` -- Batch: `activity-detail`

- [ ] Scenario: `Enter with no activity shows no detail`
  - [ ] **Design constraints:** empty inbox; no detail; `no activity yet` remains.
  - [ ] COMMIT: `PENDING` -- Batch: `activity-detail`

- [ ] Scenario: `A failed browser launch keeps the activity detail open`
  - [ ] **Design constraints:** failing opener; popup stays; `could not open
        browser` in the frame.
  - [ ] COMMIT: `PENDING` -- Batch: `activity-detail`

**Done when:** every box checked with the commit; delta 5/5, delta e2e 2/2;
permanent 167/167 and permanent e2e 16/16.

### Closeout

- [ ] All new phrases are in the permanent step file.
- [ ] README documents Enter/Escape in the inbox and the browser launch.
