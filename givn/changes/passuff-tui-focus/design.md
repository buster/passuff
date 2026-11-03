# Design: passuff-tui-focus

## Scope and constraints

Add pane focus, watched-PR selection and scrolling, focus-aware key handling
and footer, and automatic read-model refresh to the terminal inbox. No
persisted format, configuration, or notification change.

## Technology decisions

| Decision | Choice | Rationale |
|---|---|---|
| Focus model | `Pane { Watched, Activity }`; default Activity | Keeps existing activity-key behavior; `Tab`/`BackTab` cycles. |
| Watched selection | Index over the ordered row list, with the PR key remembered | The table order is groups then standalone; the key survives refresh. |
| Scrolling | `TableState` with selected on the watched table | Ratatui keeps the selected row visible; no custom offset. |
| Refresh (D1) | Fixed 1-second tick using `event::poll(timeout)` | No configuration change; a key wakes the poll immediately. |
| Selection across refresh (D2) | Preserve the selected PR key; clamp if it vanished; activity index clamps | Stable focus; no stale row. |
| E2E keys | `--keys tab,j,...` applies the real `handle_key` before `--snapshot` | Real process/renderer without a PTY; the interactive loop uses the same handler. |
| Stack headers (D3) | Not selectable | Only PR rows are selectable. |

## Architecture impact

### Contracts and Components

| Contract | Consumer use case | Consumers | Implementing component today | Split trigger |
|---|---|---|---|---|
| `InboxService` | review-activity-inbox | `TuiApp` | `Inbox` in `passuff-core::inbox` | A second read model. |
| `TuiApp` keys/focus | review-activity-inbox | `passuff-tui` loop, `--keys` | `TuiApp` in `passuff-cli::tui` | A second terminal toolkit or a detail screen. |

### Data model and rendering

- `TuiApp` gains `focus`, `pr_selected`, `selected_pr: Option<PullRequestKey>`.
- `ordered_pull_requests()` returns group members in stack order followed by
  standalone rows; the watched table and selection use the same list.
- The watched table uses `row_highlight_style(REVERSED|BOLD)` and
  `highlight_symbol("▶ ")` only while Watched is focused.
- The activity table uses its selection only while Activity is focused.
- Footer:
  - Watched: `Focus: PRs  Tab activity  j/k move  p scope  q quit`
  - Activity: `Focus: activity  Tab PRs  j/k move  r read  a all  p scope  q quit`
- `reload` reloads the view, restores the selected PR by key (otherwise clamps),
  and clamps the activity index.
- The main loop uses `event::poll(1s)`: a key is handled immediately; a timeout
  reloads the view and redraws. The picker loop keeps its existing behavior.
- `--keys` parses `tab`, `j`, `k`, `down`, `up`, `r`, `a`; it applies them to
  the app before a snapshot.

## Decisions

- **Decides D1:** fixed 1-second refresh; realize in `tui_main` and the reload
  unit test.
- **Decides D2:** preserve the selected PR key; realize in `TuiApp::reload`.
- **Decides D3:** PR rows only; summary lines are not selectable.

## Use-case traceability

| Rule | Decision | Evidence |
|---|---|---|
| T8 focus/footer | `Tab`/`BackTab`, focus-aware footer | focus scenario; e2e |
| T9 selection/scroll | focused j/k/arrows; watched TableState | scroll scenario |
| T10 auto refresh/preserve | poll tick + key-preserving reload | refresh scenario; reload unit test |
| T11 no focus/replay change | refresh only reloads the read model | refresh scenario |
| Persona | multi-account-developer | focus/scroll transcript |

## Testing strategy

- Runner: existing `./scripts/verify.sh`; strict `.fail_on_skipped()`.
- Delta: `GIVN_FEATURES=$PWD/givn/changes/passuff-tui-focus/specs ./scripts/verify.sh`.
- E2E: same with `--e2e`; the e2e step spawns `passuff-tui --snapshot --keys`.
- Step file: `crates/passuff-cli/tests/steps/inbox_navigation.rs`; register it.
- Unit tests: focus cycle, focused movement, scroll snapshot with 25 rows,
  reload preserves the selected key and clamps, footer strings.
- Auto-refresh timer: the loop is interactive; the reload path is unit-tested,
  and the timer is the project's recorded R-019 manual-smoke boundary.

## Visual Design Contract

- Interface: terminal
- Viewports: 100x30
- Screens covered: inbox with watched focus and scroll; activity focus
- Approval: the operator approved the 100x30 terminal layout in the archived
  `passuff-tui` design; this change adds focus/selection styling in the same
  table style.
- Capture: `@e2e` transcript under
  `givn/changes/passuff-tui-focus/evidence/visual/developer-navigates-the-inbox-panes/`;
  an Alacritty screenshot of the focused/selected table.
- Motion: none.
- Accessibility: `▶` marker plus reversed video, footer names the focused
  pane, no color-only meaning.

## E2E smoke test infrastructure

- E2E runner: delta scope with `GIVN_RESULT_SCOPE=e2e`.
- Step file: `crates/passuff-cli/tests/steps/inbox_navigation.rs`; the spawn
  helper is in `support/e2e.rs` and passes `--keys`.
- Local infrastructure: wiremock GitHub twin; real `passuff-tui` child.
- Strict proof: existing `.fail_on_skipped()`.

## Interaction Coverage Matrix

| Inventory entry | @e2e scenario title | Real interface | Driving mechanism |
|---|---|---|---|
| navigate the watched pull requests and activity, with automatic refresh | Developer navigates the inbox panes | CLI / terminal | Spawn `CARGO_BIN_EXE_passuff-tui --snapshot --keys "tab,j"`; assert the focused footer and `▶` row. |

## Coverage process boundaries

| Process | Started by | Instrumented artifact | Profile output | Merge step | Non-zero production probe |
|---|---|---|---|---|---|
| `passuff-tui --keys` | `./measure-coverage.sh` (e2e) | `target/llvm-cov-target/debug/passuff-tui` | `e2e-child-%p-%m.profraw` | `cargo llvm-cov report` | `tui.rs` focus/render/scroll |

## External boundary cases

- Empty activity and watched panes: movement is a no-op; footer still names focus.
- Selected PR disappears on refresh: clamp to the nearest row.
- Refresh during a read/reload error: state error path unchanged.
- Keys while the picker is open: picker handles them; inbox focus unchanged.
- Long stack: table scrolls; summary lines remain headers.
- A refresh does not change focus or replay notifications.

## Local runnability

- `cargo run -p passuff-cli --bin passuff-tui -- --config <path> --state <path>`
- Scripted: `--snapshot --keys "tab,j"` after seeding state.

## Risks

- Ratatui's table scroll behavior is relied on; a snapshot test with 25 rows
  proves the selected row is visible.
- Automatic refresh can redraw while a key is pending; `poll` handles the key
  first and only reloads on timeout.
- Refresh loads the whole view every second; the state is small and local.
