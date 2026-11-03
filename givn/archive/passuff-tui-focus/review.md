# Review: passuff-tui-focus

Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

## Implementation evidence

- Four scenarios, one `@e2e`; all record `9df4cf0 -- Batch: inbox-navigation`.
- The e2e spawns the real `passuff-tui --snapshot --keys "tab,j"` and asserts
  `Focus: PRs` and `▶ acme/api#43`.
- `TuiApp` now has `Pane`, `pr_selected`, selected-PR identity, a stateful
  watched table with `▶` and reversed video, focus-aware footer and key
  handling, and a reload that restores the selected PR key or clamps.
- `tui_main` uses a one-second `event::poll` timeout for automatic reload and
  `--keys` for the scripted surface; the interactive loop uses the same handler.
- The picker, stack grouping, activity read actions, and scope editing are
  unchanged.
- Strict mode: `.fail_on_skipped()`; no empty step bodies.

## Coverage

Merged report: 2975/3574 lines (83.2%), 362/1168 branches (31.0%). The focused
movement, scroll, and reload paths are covered by the scenarios and snapshots.
The timer loop itself is interactive; the reload path is covered and the timer
is the project's recorded R-019 manual-smoke boundary, not a behavior gap.

## Visual review

The committed screenshot shows `Focus: PRs`, the selected `▶ acme/api#43` row,
the scrolled watched table, and the activity pane unhighlighted. The footer
names the focus and its keys. The persona can reach the navigation goal.

VISUAL-REVIEW: PASS

PERSONAS: multi-account-developer

README-IMPACT: updated - Usage

ARC42 CONFORMANCE: CLEAN

## Sign-off

- [x] Every checked scenario has a verified commit.
- [x] Every promised component exists and uses the declared contract.
- [x] Strict mode is present; no empty step bodies.
- [x] Coverage gaps are classified; the timer loop is R-019.
- [x] Visual evidence is committed.
- [x] README documents focus and automatic refresh.
- [x] No `@wip` tags remain.

REVIEW: PASS
