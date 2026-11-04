# 11. Risks and Technical Debt

## Risks

| ID | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-001 | macOS attributes notifications to the terminal when the bare binary runs outside a bundle. | Medium | Medium | Ship the `Passuff.app` bundle with `dev.passuff.app`; run the documented macOS smoke test. |
| R-002 | Polling adds latency and consumes rate-limit budget compared with webhooks. | Medium | Medium | The open inbox probes cheaply (1 point measured) and observes only the selected target ids on a change; the delay follows the reported allowance and defers below 100 points. The watch command keeps the configured interval. |
| R-003 | GitHub changes the GraphQL schema or response shape. | Low | Medium | The adapter validates the response and fails per account without corrupting state; update the query in a focused change. |
| R-004 | No desktop notification daemon is available, for example in an SSH or headless session. | Medium | Low | Report the delivery failure; keep the terminal transcript; state is already persisted. |
| R-005 | No supported sound player exists on a Linux system. | Medium | Low | Fall back to the XDG sound hint; the desktop notification and transcript still deliver. |
| R-006 | A crash after the core returns but before the CLI delivers loses a notification. | Low | Medium | At-most-once is the deliberate trade-off for the no-replay guarantee; the CLI prints the transcript before delivery; the limitation is documented. |
| R-007 | A burst larger than the 100-item observation window between two checks omits an event. | Low | Medium | Default 60-second cycle; the bound is documented; pagination or timestamp high-water marks can replace it if observed. |
| R-008 | A token leaks through logs, errors, or state. | Low | High | Tokens are resolved at runtime, never persisted, and redacted from every error path; the security test greps generated artifacts. |
| R-009 | The macOS desktop notification adapter is not automated in CI; Linux uses the private D-Bus fake. | Medium | Medium | Linux E2E asserts the real `notify-rust` path; unit-test notification construction; record the manual macOS smoke before archive. |
| R-010 | The core read API is consumed only by the CLI and the TUI; a future adapter may expose missing query methods or error types. | Medium | Medium | Contract tests on `PassuffCore::refresh` and `InboxService`; `cargo tree -p passuff-core` keeps UI dependencies out of the core. |
| R-011 | Linux E2E depends on `dbus-daemon` and the mock notification service. | Medium | Low | The setup task installs and verifies `dbus-daemon`; the E2E scope fails with a clear message when the private bus cannot start; the regular scope still runs. |
| R-012 | The gh OAuth mode depends on a gh version with `gh auth token --user` and `gh auth login --web`. | Medium | Medium | A missing or old gh is a credential failure naming gh; the README states the minimum; PAT accounts are unaffected. |
| R-013 | gh login cannot select the account; the browser session decides who authorizes. | Medium | High | Every gh OAuth account declares `github_user`; the resolved `viewer.login` is checked before observation, and a mismatch is an account failure naming both logins. |
| R-014 | gh login can change the host's git protocol preference. | Low | Low | passuff passes no `--git-protocol`; gh's prompt and the README explain the side effect, and the operator keeps control. |
| R-015 | A token file's permissions can change between configuration load and credential use. | Low | Medium | The resolver repeats the `0o077` check; a changed file becomes an account failure, not a silent read. |
| R-016 | The version-1 to version-2 rewrite fails on a read-only state filesystem. | Low | Medium | The load reports a state error (exit 3) instead of serving a partial view; the operator fixes the filesystem. |
| R-017 | Activity ids collide after a partial write or restored backup. | Low | Medium | Load recomputes `next_activity_id` from the maximum record number before appending. |
| R-018 | The global watch scope cannot express per-account exceptions. | Medium | Low | Deferred; the allowlist is one operator-level list and status states it explicitly. |
| R-019 | The interactive TUI is not driven through a PTY in CI. | Medium | Low | `--snapshot` asserts the real renderer; key paths are unit/Gherkin tested; a manual terminal smoke covers raw mode and resize. |
| R-020 | The TUI ships in the CLI package rather than a separate crate. | Low | Low | The core boundary is unchanged; a future split is a packaging change with the same `InboxService` contract. |
| R-021 | The stacked-PRs preview is unavailable for a repository, or GitHub rejects the stack fields. | Medium | Low | Observation retries once without the stack fields; rows render standalone; the account's other observations and state remain. |
| R-022 | Concurrent config writes or a failed atomic write leave a stale or partial scope. | Low | Medium | `ScopeEditor` writes a temporary file in the same directory and renames; a failure keeps the previous file and reports an error. |
| R-023 | A large known-repository list clips in the 100x30 picker. | Medium | Low | The picker clips rather than wrapping; a future change may add scrolling or filtering. |
| R-024 | The TUI's mark-read actions and the watcher both save the whole state file, so one can overwrite the other's recent change. | Low | Medium | Pre-existing width; the scope edit adds no state writer. A future change may add file locking or a merge-on-save store. |
| R-025 | A configuration writer drops the `mode` field and silently changes the scope semantics. | Low | Medium | `ScopeEditor` writes both `mode` and `repositories`; ADR-0006 records the compatibility contract; config unit tests cover the legacy shapes. |
| R-026 | Setup writes a configuration with an unverified credential, so the first check fails. | Medium | Medium | Setup validates the configuration shape and reports the first check's account failure; token-file permissions are checked during setup. |
| R-027 | A redraw during a large table scroll loses the selection or a pending key. | Low | Low | Input runs on its own thread; the selected PR is restored by key and a 25-row snapshot test asserts the scrolled row is visible. |
| R-028 | The inbox and the watch command both observe and write the whole state file. | Medium | Medium | Running both is documented as redundant; whole-file writes are atomic, but the last writer wins. A lock or a serialized store is deferred (TD-004). |
| R-030 | The platform browser opener is missing or fails (headless session, no `xdg-open`, non-zero exit). | Medium | Low | The popup stays open and the footer names `could not open browser: ...`; the URL remains visible in the popup for manual use. |
| R-029 | A change burst exhausts the account allowance and stalls observation. | Low | Medium | The delay spends at most half the remaining points across the window; targeted and reconciliation passes are deferred below 100 points; a failed probe backs off. |
| R-031 | A change the probe fingerprint cannot see (no update time, creation time, state, head commit, or CI change) is observed only by the reconciliation pass. | Medium | Low | The live inbox reconciles at startup and every `reconciliation_interval_secs` (default one hour); the headless watcher reconciles on its poll interval. |
| R-032 | Terminal retirement means activity after a merge or close is never reported. | High | Low | Accepted by the operator; the transition itself, a short-lived pull request, and a reopen are still observed and notified. |
| R-033 | An involved (assigned or review-requested) pull request is only as fresh as the reconciliation pass. | Medium | Low | Discovery runs on every pass; the reconciliation interval is configurable and defaults to one hour. |

## Technical debt

| ID | Item | Impact | Plan to address |
|---|---|---|---|
| TD-001 | Real macOS notification delivery is not automated in CI. | A macOS-specific regression could go unnoticed until the manual smoke test. | Add a macOS CI runner with a scripted notification surface or a platform-specific fake; the Linux D-Bus fake is the model. |
| TD-002 | The state uses a bounded seen-ID window rather than an unbounded high-water mark. | A very large burst between two checks can omit events. | Replace the window with a timestamp high-water mark plus ID overlap if a real omission is observed. |
| TD-004 | No cross-process lock serializes the inbox and the watch command's state writes. | A duplicate notification is possible in the overlap, and one writer can drop the other's baseline. | Add a store lock or a serialized writer if the overlap becomes common. |
| TD-005 | Coverage merging uses a small project-owned Cobertura merger. | Extra code to maintain when the report format changes. | Replace with a maintained merger if the project grows or the format changes. |
| TD-006 | The core has no public PR-list or activity-history query API yet. | Resolved by `InboxService` (`view`, `mark_read`, `mark_all_read`); the TUI is the remaining consumer. | Done in `passuff-activity-inbox`; the TUI change consumes the contract. |
