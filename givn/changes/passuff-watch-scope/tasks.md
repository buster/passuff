# Tasks: passuff-watch-scope

Behavior batch: config mode, picker, hide/restore, catch-up, reload, and the
end-to-end proof.

---

## Strict-mode proof

- [ ] **Proof-of-strictness**: the delta runner rejects an undefined step with `failed > 0`.

      ```
      GIVN_FEATURES=/tmp/watch-scope-strict.feature ./scripts/verify.sh
      ```

---

### Batch: `watch-scope`

- [ ] Setup: `WatchMode`/`FileWatchConfig`/`WatchConfig`, `ScopeSnapshot` on
      `WatcherState`, `InboxView.scope_label`, `ScopeEditor`, `ScopePicker`,
      scripted flags, reload, and the step file.
      Files: `passuff-config/src/lib.rs`, `passuff-core/src/{domain,cycle,inbox}.rs`,
      `passuff-cli/src/{scope,tui,tui_main,lib,inbox_cli,transcript}.rs`,
      `tests/support/{mod,e2e}.rs`, `tests/steps/watch_scope.rs`,
      `tests/features_runner.rs`.
      COMMIT: pending

- [ ] Scenario: `Developer toggles a repository in the watch scope` (e2e)
  - [ ] **Design constraints:** mode+list persistence; toggle hides/restores;
        catch-up derived from `last_scope`; e2e spawns `--snapshot --picker
        --scope-select/-deselect/-add`; config/state assertions read the e2e
        files; exit 0.
  - [ ] COMMIT: pending

- [ ] Scenario: `Developer switches the watch scope mode` (e2e)
  - [ ] **Design constraints:** all/selected mode; selected set retained; picker
        shows the mode; `--scope-mode`; exit 0.
  - [ ] COMMIT: pending

- [ ] Scenarios: `The picker shows the known repositories`,
      `The last deselection observes nothing`,
      `Switching to all observes every repository`,
      `An invalid repository entry is rejected`,
      `Reselecting catches up silently`,
      `A scope write failure keeps the previous scope`,
      `Selecting a repository with no records records a baseline`,
      `Deselecting hides status and activity`,
      `Two accounts share the global scope`,
      `A reload failure keeps the previous scope`,
      `Switching to selected with no retained set observes nothing`.
  - [ ] **Design constraints:** owner/name validation; exact error copy;
        hide/retain; status uses `scope_label`; catch-up silent; `last_scope`
        advances only without account failures; writer atomicity; two-account
        filtering.
  - [ ] COMMIT: pending

- [ ] Scenario: `The watcher reloads the scope without a restart`
  - [ ] **Design constraints:** held `CliRuntime`; both cycles use
        `run_watch_cycle(path)` (reload + core rebuild); a comment on the newly
        selected repository between cycles is recorded as unread.
  - [ ] COMMIT: pending

- [ ] Unit tests: config legacy/duplicate/labels; `ScopeEditor` actions and
      write failure; `InboxService` filtering/unread; catch-up comparison and
      `catch_up_survives_account_failure`; runtime reload; picker snapshots
      (all/selected).
  - [ ] COMMIT: pending

---

**Done when:** every box checked with the commit, both suites exit 0, and the
e2e count (2) is less than the full delta count.
