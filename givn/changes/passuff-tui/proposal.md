# Proposal: passuff-tui

## Release note

Browse accounts, watched pull requests, and recorded activity in a full-screen terminal UI, and mark activity read from it.

## Use-Case Context

- Use-case ID: `review-activity-inbox` (existing; the TUI is a second adapter for the same goal)
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — confirmed by the operator's brief on 2026-11-02.

## Review Context

- Review kind: independent
- Reason: a new user-visible terminal interface and a second UI adapter over the core contract.

## Problem / Opportunity

The CLI read commands require a separate invocation for status, listing, and reading. The operator cannot keep the inbox open, move through activity, and acknowledge it while watching. The archived Plan calls for a TUI over the same core and read model.

## Proposed Solution

- A `passuff-tui` terminal interface shows the unread/total activity counts, configured accounts, watch scope, watched pull requests, and the activity list in one screen.
- `j`/`k` (or arrows) move the selection; `r` marks the selected record read; `a` marks all read; `q`/Escape quits.
- The list refreshes after a read, and the selection stays within bounds.
- An empty inbox shows "no activity yet"; configuration and state failures are reported before the interface starts, with exit 2 or 3.
- `--snapshot` renders one frame to stdout without a terminal and exits 0. It uses the same renderer and data as the interactive interface, so scripts and E2E can assert the screen.
- No token, comment body, command text, or enum name appears on screen.

## Capability Routing

| Proposed capability | Decision | Rationale |
|---|---|---|
| activity-inbox | `EXTEND activity-inbox` | The TUI is a second adapter for the same review/read goal; no new capability or use case. Route reported no signal (tie), so the decision is recorded from the use-case placement test. |

## Out of Scope

- Editing configuration or state from the TUI.
- Mouse input, filters, search, and history export.
- A separate `passuff-tui` workspace crate; this change ships a second binary in `passuff-cli` to share the Gherkin runner and test infrastructure while keeping the core boundary unchanged.
- Remote synchronization.

## Open Questions

### D1 — Where does the TUI binary live?

- Question: A new workspace crate or a second binary in `passuff-cli`?
- Options:
  1. **Second binary in `passuff-cli`** — gain: one test package, `CARGO_BIN_EXE_passuff-tui` available to the existing runner, no duplicated harness; cost: the CLI package depends on ratatui/crossterm. In practice: `crates/passuff-cli/src/bin/passuff-tui.rs`.
  2. **New `passuff-tui` crate** — gain: package isolation; cost: a second Gherkin runner or a cross-package binary path, duplicated test infrastructure. In practice: a new member with its own tests.
- Disposition: answered 2026-11-02 — option 1; ADR-0003 is refined with a note: the TUI binary lives in `passuff-cli`, and the core boundary is unchanged.

### D2 — How is the TUI verified without a terminal?

- Question: PTY-driven key injection or a snapshot render?
- Options:
  1. **Snapshot mode** — gain: deterministic frame text, no PTY dependency; cost: key handling is covered in-process rather than through a real terminal. In practice: `passuff-tui --snapshot` prints the rendered frame and exits.
  2. **PTY driver** — gain: real terminal input; cost: a new PTY dependency and flaky terminal sizing. In practice: a test harness spawns the binary under a pseudo-terminal.
- Disposition: answered 2026-11-02 — option 1; key behavior is tested through `TuiApp::handle_key`, the snapshot uses the real renderer and real binary.
