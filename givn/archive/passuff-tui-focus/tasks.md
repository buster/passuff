# Tasks: passuff-tui-focus

### Batch: `inbox-navigation`

- [x] Setup: pane focus, watched-PR selection/scrolling, focus-aware footer,
      one-second refresh, `--keys`, step file, and tests.
      COMMIT: `9df4cf0` -- Batch: `inbox-navigation`

- [x] Scenario: `Developer navigates the inbox panes` (e2e)
  - [x] **Design constraints:** real `--snapshot --keys "tab,j"`; focus PRs;
        `▶ acme/api#43`; exit 0.
  - [x] COMMIT: `9df4cf0` -- Batch: `inbox-navigation`

- [x] Scenario: `The focus cycles between watched pull requests and activity`
  - [x] **Design constraints:** default activity; Tab to PRs; Tab back.
  - [x] COMMIT: `9df4cf0` -- Batch: `inbox-navigation`

- [x] Scenario: `The watched pull request selection scrolls`
  - [x] **Design constraints:** 25 PRs; 24 moves; last row visible with `▶`.
  - [x] COMMIT: `9df4cf0` -- Batch: `inbox-navigation`

- [x] Scenario: `A refresh preserves the selected pull request`
  - [x] **Design constraints:** selected key survives reload; new PR appears.
  - [x] COMMIT: `9df4cf0` -- Batch: `inbox-navigation`

- [x] Unit tests: focus/selection in `TuiApp`; reload key preservation.
  - [x] COMMIT: `9df4cf0` -- Batch: `inbox-navigation`

**Done when:** every box checked with the commit; delta 4/4, delta e2e 1/1,
permanent 154/154, permanent e2e 12/12; e2e count 1 < delta count 4.
