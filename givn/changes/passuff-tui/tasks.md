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
      Scenario-count proof: delta e2e 1 < full delta suite 11.

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

### Batch: `tui-keys`

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

### Batch: `tui-review-fixes`

- [x] Scenario: `The terminal UI reports a configuration error`
  - [x] **Design constraints:** T5. The spawned binary exits 2 before the terminal.
        Files: `tui_main.rs`, `load_view`, step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT a556688.
        ```
  - [x] COMMIT: `a556688` -- Batch: `tui-review-fixes`

- [x] Scenario: `The terminal UI reports a state error`
  - [x] **Design constraints:** T5. State path is a directory; the spawned binary exits 3.
        Files: `tui_main.rs`, `support/mod.rs`, step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; scenario green. COMMIT a556688.
        ```
  - [x] COMMIT: `a556688` -- Batch: `tui-review-fixes`

- [x] B1: quote the capability decision so the review gate reads `## Capability Routing`.
      Files: `proposal.md`. COMMIT `a556688`.
- [x] B2: align durable arc42 chapters with the built `InboxService` adapter
      (`05` edge and table, `06` runtime note, `10` QS-011, `11` R-010).
      COMMIT `a556688`.
- [x] B3: terminal setup/draw/event failures exit 3 with a `passuff-tui:` message;
      the selected row uses `▶` and reversed video.
      Files: `tui_main.rs`, `tui.rs`. COMMIT `a556688`.
- [x] B4: remove the dead `TuiApp::view` and `TuiApp::selected` accessors.
      COMMIT `a556688`.
- [x] B5: regression tests for the reload clamp, a failed reload after `r`, an
      unbound key, a configured watch scope, `load_view` exits 2/3, and the
      spawned binary's config/state error exits.
      Files: `tui.rs` unit tests; `activity-inbox.feature`, step file, support.
      ```
      cargo test -p passuff-cli --lib -> 14 passed
      ./scripts/verify.sh (delta) -> 10/10; permanent -> 112/112; e2e -> 7/7
      ```
      COMMIT `a556688`.

### Batch: `tui-review-closure`

- [x] Review finding: the non-terminal guard (`tui_main.rs:65-68`) is untested.
- [x] Review finding: `Down`/`Up` key dispatch and `r` on an empty activity list
      are untested.

- [x] Scenario: `The terminal UI requires a terminal`
  - [x] **Design constraints:** T5. Without `--snapshot` and without a TTY, exit 3.
        Files: `tui_main.rs`, step file, support.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 11/11; permanent -> 112/112; e2e -> 7/7.
        ```
  - [x] COMMIT: `c5a559b` -- Batch: `tui-review-closure`

- [x] Scenario: `The terminal UI shows the empty inbox`
  - [x] **Design constraints:** T4/T1. `r` with no activity keeps `0 unread of 0`.
        Files: `tui.rs`, step file.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 11/11; permanent -> 112/112; e2e -> 7/7.
        ```
  - [x] COMMIT: `c5a559b` -- Batch: `tui-review-closure`

- [x] Scenario: `The selection moves and clamps`
  - [x] **Design constraints:** T2. `j`/`k` and `Down`/`Up` move and clamp.
        Files: `tui.rs`, step file, support.
  - [x] Implement and run; REFACTOR and re-run.
        ```
        ./scripts/verify.sh (delta) -> 11/11; permanent -> 112/112; e2e -> 7/7.
        ```
  - [x] COMMIT: `c5a559b` -- Batch: `tui-review-closure`

- [x] Coverage after the closure: 1955/2320 lines (84.3%), 203/632 branches (32.1%);
      the non-terminal guard and both arrow arms are hit in the merged report.
      COMMIT `c5a559b`.

---

**Done when:** every box checked with the commit, the regular and end-to-end suites exit 0, and the setup proof records an end-to-end count (1) less than the full delta count (11).
