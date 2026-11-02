# 11. Risks and Technical Debt

## Risks

| ID | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-001 | macOS attributes notifications to the terminal when the bare binary runs outside a bundle. | Medium | Medium | Ship the `Passuff.app` bundle with `dev.passuff.app`; run the documented macOS smoke test. |
| R-002 | Polling adds latency and consumes rate-limit budget compared with webhooks. | Low | Medium | Default 60-second interval; one query per account; report rate-limit reset times; targeted known-PR query only when needed. |
| R-003 | GitHub changes the GraphQL schema or response shape. | Low | Medium | The adapter validates the response and fails per account without corrupting state; update the query in a focused change. |
| R-004 | No desktop notification daemon is available, for example in an SSH or headless session. | Medium | Low | Report the delivery failure; keep the terminal transcript; state is already persisted. |
| R-005 | No supported sound player exists on a Linux system. | Medium | Low | Fall back to the XDG sound hint; the desktop notification and transcript still deliver. |
| R-006 | A crash after the core returns but before the CLI delivers loses a notification. | Low | Medium | At-most-once is the deliberate trade-off for the no-replay guarantee; the CLI prints the transcript before delivery; the limitation is documented. |
| R-007 | A burst larger than the 100-item observation window between two checks omits an event. | Low | Medium | Default 60-second cycle; the bound is documented; pagination or timestamp high-water marks can replace it if observed. |
| R-008 | A token leaks through logs, errors, or state. | Low | High | Tokens are resolved at runtime, never persisted, and redacted from every error path; the security test greps generated artifacts. |
| R-009 | The macOS desktop notification adapter is not automated in CI; Linux uses the private D-Bus fake. | Medium | Medium | Linux E2E asserts the real `notify-rust` path; unit-test notification construction; record the manual macOS smoke before archive. |
| R-010 | The core API is consumed only by the CLI in this change; the first TUI may expose missing query methods or error types. | Medium | Medium | Contract tests on `PassuffCore::refresh`; the inbox change adds the read/query API before the TUI; `cargo tree -p passuff-core` keeps UI dependencies out of the core. |
| R-011 | Linux E2E depends on `dbus-daemon` and the mock notification service. | Medium | Low | The setup task installs and verifies `dbus-daemon`; the E2E scope fails with a clear message when the private bus cannot start; the regular scope still runs. |

## Technical debt

| ID | Item | Impact | Plan to address |
|---|---|---|---|
| TD-001 | Real macOS notification delivery is not automated in CI. | A macOS-specific regression could go unnoticed until the manual smoke test. | Add a macOS CI runner with a scripted notification surface or a platform-specific fake; the Linux D-Bus fake is the model. |
| TD-002 | The state uses a bounded seen-ID window rather than an unbounded high-water mark. | A very large burst between two checks can omit events. | Replace the window with a timestamp high-water mark plus ID overlap if a real omission is observed. |
| TD-003 | Coverage merging uses a small project-owned Cobertura merger. | Extra code to maintain when the report format changes. | Replace with a maintained merger if the project grows or the format changes. |
| TD-004 | The core has no public PR-list or activity-history query API yet. | The first TUI cannot show the inbox until the follow-up change lands. | Add `pull_requests`, `activity`, and `mark_read` in `passuff-activity-inbox`, then build the TUI. |
