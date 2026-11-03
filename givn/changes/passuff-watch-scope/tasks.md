# Tasks: passuff-watch-scope

Behavior batch: config mode, picker, hide/restore, catch-up, reload, and the
end-to-end proof.

---

## Strict-mode proof

- [x] **Proof-of-strictness**: the delta runner rejects an undefined step with `failed > 0`.

      ```
      GIVN_FEATURES=/tmp/watch-scope-strict.feature ./scripts/verify.sh
      FAILED: Strict probe :: Undefined watch-scope step fails
        undefined Given  -> an undefined watch-scope step
          Step doesn't match any function
      givn test-result: total 1, passed 0, failed 1, skipped 0
      cargo exit status: 1
      ```

---

### Batch: `watch-scope`

- [x] Setup: `WatchMode`/`FileWatchConfig`/`WatchConfig`, `ScopeSnapshot` on
      `WatcherState`, `InboxView.scope_label`, `ScopeEditor`, `ScopePicker`,
      scripted flags, runtime reload, and the step file.
      Files: `passuff-config/src/lib.rs`, `passuff-core/src/{domain,cycle,inbox,state}.rs`,
      `passuff-cli/src/{scope,tui,tui_main,lib,main,inbox_cli,transcript}.rs`,
      `tests/support/{mod,e2e}.rs`, `tests/steps/watch_scope.rs`,
      `tests/features_runner.rs`.
      COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Developer toggles a repository in the watch scope` (e2e)
  - [x] **Design constraints:** mode+list persistence; toggle hides/restores;
        e2e spawns `--snapshot --picker --scope-select/-deselect/-add`; config
        and state assertions read the e2e files; exit 0.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Developer switches the watch scope mode` (e2e)
  - [x] **Design constraints:** all/selected mode; selected set retained; picker
        shows the mode; `--scope-mode`; exit 0.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `A deselected repository stops observation`
  - [x] **Design constraints:** out-of-scope observation stops; retained records hidden; no notification.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Adding a repository by owner/name`
  - [x] **Design constraints:** manual entry validates and selects; mode selected.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `The picker shows the known repositories`
  - [x] **Design constraints:** picker lists known repos, selected state, mode.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `The last deselection observes nothing`
  - [x] **Design constraints:** selected empty; status label `watching nothing`.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Switching to all observes every repository`
  - [x] **Design constraints:** all mode observes both repos; selected set retained.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `An invalid repository entry is rejected`
  - [x] **Design constraints:** exact `must be owner/repo`; previous scope unchanged.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Reselecting catches up silently`
  - [x] **Design constraints:** check while deselected sets `last_scope`; reselect
        records activity unread without a notification; TUI shows it.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `A scope write failure keeps the previous scope`
  - [x] **Design constraints:** `cannot write configuration`; previous scope.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Selecting a repository with no records records a baseline`
  - [x] **Design constraints:** no catch-up; fresh baseline.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Deselecting hides status and activity`
  - [x] **Design constraints:** status and TUI hide the repo/activity; state retains.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Two accounts share the global scope`
  - [x] **Design constraints:** one global scope filters both accounts' records.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `A reload failure keeps the previous scope`
  - [x] **Design constraints:** corrupt config reports the error; no crash.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `Switching to selected with no retained set observes nothing`
  - [x] **Design constraints:** all→selected restores the retained empty set;
        a known repository is not auto-selected; `watching nothing`.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Scenario: `The watcher reloads the scope without a restart`
  - [x] **Design constraints:** `run_watch_cycle(path)` reloads and rebuilds;
        a comment on the newly selected repository between cycles is recorded.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

- [x] Unit tests: config legacy/duplicate/labels; `ScopeEditor` select/deselect/
      mode/add/write failure; `InboxService` filtering/unread; picker snapshots.
  - [x] COMMIT: `35d05eb` -- Batch: `watch-scope`

---

**Done when:** every box checked with the commit, both suites exit 0, and the
e2e count (2) is less than the full delta count (16).

Evidence:
- Delta: 16/16 passed.
- Delta e2e: 2/2 passed.
- Permanent regular: 130/130 passed.
- Permanent e2e: 9/9 passed.
- Unit: `passuff-config` 16, `passuff-core` 29, `passuff-cli` 23.
