# Tasks: passuff-tui

Behavior batches. Regular scenarios first, then the end-to-end proof.

Legend: an unchecked box means pending; a checked box is done only when the commit is filled in.

---

## Strict-mode proof

- [x] **Proof-of-strictness**: the TUI delta against an undefined step exits nonzero with `failed > 0`.

      ```
      ./scripts/verify.sh with GIVN_FEATURES=/tmp/tui-strict.feature
      FAILED: Strict probe :: Undefined TUI step fails
        undefined Given  -> no TUI step binds this phrase
      givn test-result: total 1, passed 0, failed 1, skipped 0
      cargo exit status: 1
      ```

---

## Setup

- [x] Add `tui.rs` (`TuiApp`, `load_view`, snapshot), the `passuff-tui` binary, ratatui/crossterm, the step file, and the active-change evidence path.

      COMMIT 45262ed. Delta 10/10; permanent 112/112; delta e2e 1/1; permanent e2e 7/7; unit 47 passed.
      Scenario-count proof: delta e2e 1 < full delta suite 10.

---

### Batch: `tui-render`

- [x] Scenario: `The terminal UI shows the inbox`
  - [x] **Design constraints:** T1/T7. Counts, account, scope, PR title, activity; no enum/body/token.
        Files: `tui.rs`, `tui_main.rs`, step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-render`

- [x] Scenario: `The terminal UI shows the empty inbox`
  - [x] **Design constraints:** T4/T1. `no activity yet`, `0 unread of 0`.
        Files: `tui.rs`.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-render`

- [x] Scenario: `The terminal UI reports a configuration error`
  - [x] **Design constraints:** T5. Exit 2 before the terminal.
        Files: `tui.rs` (`load_view`), step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-render`

- [x] Scenario: `The terminal UI reports a state error`
  - [x] **Design constraints:** T5. v1 migration save failure, exit 3.
        Files: `tui.rs`, support guard.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-render`

### Batch: `tui-keys`

- [x] Scenario: `The selection moves and clamps`
  - [x] **Design constraints:** T2. j moves to the comment, j clamps, k returns to CI.
        Files: `tui.rs`, step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-keys`

- [x] Scenario: `The selected activity is marked read`
  - [x] **Design constraints:** T3. r marks only the selected record; 1 unread of 2.
        Files: `tui.rs`, support.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-keys`

- [x] Scenario: `Every activity is marked read`
  - [x] **Design constraints:** T3. a marks all; 0 unread of 2.
        Files: `tui.rs`, support.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-keys`

- [x] Scenario: `The terminal UI quits`
  - [x] **Design constraints:** T2. q and Escape return Quit.
        Files: `tui.rs`, step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-keys`

- [x] Scenario: `Developer opens the inbox TUI` (end-to-end)
  - [x] **Design constraints:** T6. Real binary `--snapshot`, seeded state, transcript evidence.
        Files: `tui_main.rs`, e2e support.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; e2e -> 7/7; scenario green. COMMIT 45262ed.
        ```
  - [x] COMMIT: `45262ed` -- Batch: `tui-keys`

---

**Done when:** every box checked with the commit, the regular and end-to-end suites exit 0, and the setup proof records an end-to-end count (1) less than the full delta count (10).
