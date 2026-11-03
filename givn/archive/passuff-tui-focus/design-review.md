# Design Review: passuff-tui-focus

Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

## Findings and closure

| Finding | Closure |
|---|---|
| D1 refresh interval open | Fixed 1-second poll timeout; no config change. |
| D2 selection preservation open | Selected PR key is restored on reload; missing keys clamp. |
| D3 stack headers | Not selectable; only PR rows. |
| Default focus vs existing behavior | Default Activity keeps existing `j`/`k` activity movement; `Tab` reaches the watched table. |
| Watched table scroll | `TableState` with selected and `▶` highlight; the 25-row scenario asserts the last row is visible. |
| Auto-refresh without losing keys | `event::poll(1s)` handles a key first; reload only on timeout. |
| E2E driving | `--keys` applies the real `TuiApp::handle_key` before `--snapshot`; the interactive loop uses the same handler. |

## Verification

- Delta 4/4; delta e2e 1/1; permanent 154/154; permanent e2e 12/12; CLI unit 32.
- The timer loop itself is interactive; the reload path is covered by the refresh
  scenario and the selected-key preservation is in `TuiApp::reload`. This is the
  project's R-019 manual-smoke boundary, not a missing behavior.
- Contracts: `TuiApp` remains the only TUI component; `InboxService` unchanged.
- Visual: `▶` plus reversed video and focus-aware footer; no color-only meaning.

DESIGN-REVIEW: PASS
