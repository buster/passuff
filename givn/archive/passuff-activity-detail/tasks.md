# Tasks: passuff-activity-detail

### Batch: `activity-detail`

- [x] Setup: confirm the runner and strict mode.
  - `verify.command`/`verify.e2e_command` unchanged.
  - `GIVN_FEATURES=$PWD/givn/changes/passuff-activity-detail/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    exited 1 with `givn test-result: total 5, passed 0, failed 5, skipped 0`
    before the steps existed.
  - Step file: `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`.

- [x] Core: expose the pull request URL.
  - [x] `PullRequestView.url` in `crates/passuff-core/src/inbox.rs`, copied from
        `WatchedPullRequest.url`.
  - [x] Update the `tui.rs` test-view constructors.

- [x] TUI: detail state, modal keys, rendering.
  - [x] `ActivityDetail { record, pull_request, url }`; `TuiApp.popup`.
  - [x] `TuiAction::OpenUrl(String)`; modal branch before the existing match:
        Escape closes, Enter returns `OpenUrl`, other keys are ignored.
  - [x] Enter with `Pane::Activity` and a selected record opens the detail;
        Enter in `Pane::Watched` or with no activity changes nothing.
  - [x] `TuiApp::close_popup`; observation reload leaves the popup untouched.
  - [x] Render the centered `Clear` + bordered overlay (70% x 9) with the six
        content lines from the design.

- [x] CLI: browser adapter.
  - [x] `crates/passuff-cli/src/browser.rs`: `BrowserOpener`,
        `SystemBrowserOpener` (`xdg-open` on Linux, `open` on macOS); spawn and
        non-zero exit are errors.
  - [x] `tui_main.rs` handles `OpenUrl` in the interactive loop and the
        `--keys` snapshot path: success closes the popup, failure sets the
        footer status and keeps it.
  - [x] `--keys` accepts `enter` and `esc`.

- [x] Tests and support.
  - [x] World: recording/failing `BrowserOpener`, `run_tui_press` applies
        `OpenUrl`, new assertions (`the browser opened "<url>"`, `the browser
        will fail to open`).
  - [x] E2E: fake `xdg-open` on `PATH` writing the URL to a log;
        `configure_child` prepends the bin dir; the browser assertion reads it.

- [x] Scenario: `Developer opens an activity's details` (e2e)
  - [x] **Design constraints:** `--keys "enter"`; popup shows `Activity a1`,
        `New comment from @alice`, `acme/api#42`, `Pull request acme/api#42`,
        `unread`, `Enter open in browser`; exit 0.
  - [x] COMMIT: `21aa389` -- Batch: `activity-detail`

- [x] Scenario: `Developer opens the pull request from an activity` (e2e)
  - [x] **Design constraints:** `--keys "enter,enter"`; fake opener records the
        URL; popup closes; exit 0.
  - [x] COMMIT: `21aa389` -- Batch: `activity-detail`

- [x] Scenario: `Escape closes the activity detail`
  - [x] **Design constraints:** Enter opens, Escape closes, focus stays activity.
  - [x] COMMIT: `21aa389` -- Batch: `activity-detail`

- [x] Scenario: `Enter with no activity shows no detail`
  - [x] **Design constraints:** empty inbox; no detail; `no activity yet` remains.
  - [x] COMMIT: `21aa389` -- Batch: `activity-detail`

- [x] Scenario: `A failed browser launch keeps the activity detail open`
  - [x] **Design constraints:** failing opener; popup stays; `could not open
        browser` in the frame.
  - [x] COMMIT: `21aa389` -- Batch: `activity-detail`

**Done when:** every box checked with the commit; delta 5/5, delta e2e 2/2;
permanent 167/167 and permanent e2e 16/16.

### Closeout

- [x] All new phrases are in the permanent step file.
- [x] README documents Enter/Escape in the inbox and the browser launch.
