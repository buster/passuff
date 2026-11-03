Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)
PERSONAS: multi-account-developer
README-IMPACT: updated - terminal inbox keys

# Review: passuff-activity-detail

## Implementation evidence check

- **Empty step bodies:** 0 across 10 step files; the two new steps assert a
  recorded URL and set a failure flag.
- **@e2e integrity:** both planned E2E scenarios are present with `@e2e`; no
  tag was removed.
- **Strict-mode proof:** `tasks.md` records the pre-implementation run exiting 1
  at `total 5, passed 0, failed 5, skipped 0`.
- **Scenario commits:** all five scenarios record `21aa389 -- Batch:
  activity-detail`, which contains the popup, the browser adapter, the binary
  wiring, and the support/steps. The later `f04a675`, `c64b4c9`, `9175ab7`, and
  `0fbdc12` commits add opener failure tests, fix construction, use the default
  constructor, and shorten the displayed time; they contain no scenario
  evidence and change no scenario assertion.
- **Components promised by `design.md`:** `ActivityDetail`, `TuiApp::popup`,
  `TuiAction::OpenUrl`, the modal key branch, the overlay renderer,
  `SystemBrowserOpener`, the `--keys` mapping, and the World opener all exist.
- **E2E interface:** the two E2E scenarios invoke the real `passuff-tui` binary;
  the detail scenario asserts the rendered frame, and the open scenario asserts
  the URL recorded by the fake `xdg-open` on `PATH` plus the closed popup. Both
  are primary assertions on the real interface, not repository state.
- **Design deviations:** none. The `with_program` constructor is a test seam on
  the same adapter; the design's step file and command are unchanged.

### Interaction inventory cross-reference

| Inventory entry | Matrix row | @e2e scenario | Driving mechanism |
|---|---|---|---|
| activity-inbox / list unread activity | present | Developer lists unread activity (permanent) | real `passuff inbox` subprocess |
| activity-inbox / mark activity read | present | Developer marks activity read (permanent) | real `passuff read --all` subprocess |
| activity-inbox / open the inbox in the terminal UI | present | Developer opens the inbox TUI (permanent, modified earlier) | real `passuff-tui --snapshot --observe` |
| activity-inbox / open an activity's detail | present | Developer opens an activity's details | real `passuff-tui --snapshot --keys "enter"` |
| activity-inbox / open the pull request from an activity | present | Developer opens the pull request from an activity | real `passuff-tui --snapshot --keys "enter,enter"` with a fake `xdg-open` |
| inbox-navigation / navigate the watched pull requests and activity | present | Developer navigates the inbox panes (permanent) | real `passuff-tui --snapshot --keys` |
| inbox-navigation / leave the inbox open while activity arrives | present | The inbox shows activity observed while it is open | real `passuff-tui --snapshot --observe-for` with a live wiremock change |

No inventory entry is unmapped and no extra E2E scenario was added.

## Architecture conformance from the first implementation

- `TuiApp` interprets keys and returns `OpenUrl`; it never spawns a process.
- `SystemBrowserOpener` is the only process launcher, created at the binary
  composition root. The in-process test World substitutes a recording opener.
- The popup reads only `InboxView` data; the core gains one read-model field
  (`PullRequestView.url`) and no new dependency.
- The modal branch precedes the inbox keys, so Escape cannot quit while the
  popup is open and movement cannot change the selection underneath.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Browser launch is an outbound context | 03 context table | Yes | Browser adapter | CLI batch | `browser.rs`, `tui_main` | Yes |
| Detail overlay + opener strategy | 04 strategy | Yes | Key behaviour | TUI + CLI | `handle_key`, `apply_action` | Yes |
| `SystemBrowserOpener` building block | 05 block table | Yes | Contracts | CLI batch | `crates/passuff-cli/src/browser.rs` | Yes |
| Open-detail flow | 06 runtime view | Yes | Key behaviour | Scenarios | modal branch + overlay | Yes |
| Modal precedence and failure footer | 08 crosscutting | Yes | Key behaviour | Scenarios | popup branch, `set_status` | Yes |
| QS-027 | 10 quality | Yes | Traceability | E2E scenarios | E2E evidence | Yes |
| R-030 | 11 risks | Yes | Decisions | Failure scenario | `could not open browser: ...` | Yes |
| Activity detail term | glossary | Yes | Data model | TUI batch | `ActivityDetail` | Yes |

Chapters 02 and 07 remain unaffected as claimed. No durable chapter is stale.

## Coverage classification

Merged coverage after the change: 3,362/4,054 lines (83%) and 422/1,374
branches (31%). The last two commits (default constructor, shorter timestamp)
are call-site/style changes measured after this run.

| Region | Lines | Classification |
|---|---|---|
| `browser.rs` macOS program branch | 1 | Platform-only; the Linux build never takes `open`. Covered by the macOS manual smoke path described in R-019's scope. |
| `tui.rs` pre-existing render/test helpers | 36 | Pre-existing from earlier changes; the new popup paths are covered by the E2E snapshots. |
| `tui_main.rs` raw-mode interactive loop and restore | ~175 | Pre-existing R-019; the snapshot/observe surfaces exercise the same handlers and opener path. |

## Findings

| ID | Finding | Disposition |
|---|---|---|
| F1 | `SystemBrowserOpener` failure paths (missing binary, non-zero exit) had no automated coverage. | Added `with_program` and two unit tests in `f04a675`; both pass. |
| F2 | The `SystemBrowserOpener` unit-struct call in `tui_main` broke after the test seam; caught by the receipt verification. | Fixed in `c64b4c9`; the default constructor is used in `9175ab7`. |
| F3 | The popup displayed the full nanosecond timestamp, making the detail line noisy. | Shortened to seconds in `0fbdc12`; the E2E frame shows `2026-11-03T13:18:00`. |
| F4 | The macOS `open` branch is unreachable on Linux CI. | Classified; platform-only. |

No critical finding remains.

VISUAL-REVIEW: PASS

REVIEW: PASS
