# Design: passuff-tui

## Scope and constraints

Add a second binary, `passuff-tui`, in the `passuff-cli` package. It renders the
existing `InboxService` view with ratatui and handles keys with crossterm. The
core contract and persisted state are unchanged. The TUI is a second adapter for
the same activity-inbox goal.

- `--snapshot` renders one frame to stdout and exits; the interactive mode uses
  the alternate screen and raw mode.
- Keys: `q`/Escape quit, `j`/Down and `k`/Up move, `r` read selected, `a` read all.
- No new crate. Packaging: the TUI ships in `passuff-cli` to share the Gherkin
  runner and `CARGO_BIN_EXE_passuff-tui`; ADR-0003 carries a refinement note for this package
  choice and the core boundary is unchanged.

## Technology decisions

| Decision | Choice | Rationale |
|---|---|---|
| TUI framework | ratatui 0.30.2 | Already resolved in the archived design; cached in the registry. |
| Terminal backend | crossterm 0.29.0 | ratatui's supported backend; cached. |
| Verification without a PTY | `--snapshot` uses `TestBackend` and prints the frame | Deterministic, no PTY dependency; key handling is tested through `TuiApp::handle_key`. |
| Data source | `InboxService` | Same read model and read state as the CLI; no new persistence. |
| Error mapping | Load config before the terminal; exit 2 on config error, 3 on state error | Same contract as the CLI read commands. |

Versions verified in the local registry: `ratatui-0.30.2`, `crossterm-0.29.0`.

## Architecture impact

### Contracts and Components

| Contract | Consumer use case | Consumers | Implementing component | Split trigger |
|---|---|---|---|---|
| `InboxService` (view, mark read) | review-activity-inbox | `TuiApp`, `InboxCli` | `Inbox` in `passuff-core` | A second read model with different consistency rules. |
| `TuiApp` (render/keys) | review-activity-inbox | `passuff-tui` binary, tests | `tui` module in `passuff-cli` | A second terminal toolkit or a separate TUI package. |

`TuiApp`:

```rust
pub struct TuiApp { view: InboxView, selected: usize }
pub enum TuiAction { Continue, Quit }
pub struct TuiLoadError { pub exit: i32, pub message: String }
pub async fn load_view(
    config: &Result<AppConfig, ConfigError>,
    inbox: &dyn InboxService,
) -> Result<InboxView, TuiLoadError>;

impl TuiApp {
    pub fn new(view: InboxView) -> Self;
    pub async fn reload(&mut self, inbox: &dyn InboxService, config: &CoreConfig) -> Result<(), CoreError>;
    pub async fn handle_key(&mut self, key: KeyEvent, inbox: &dyn InboxService, config: &CoreConfig) -> Result<TuiAction, CoreError>;
    pub fn render(&self, frame: &mut Frame);
    pub fn snapshot(&self) -> String; // TestBackend 100x30
}
```

Layout (100x30, bordered panels):

```
┌ passuff — Inbox ───────────────────────────────────────────┐
│personal @octo-personal   |   watch scope: all authored …  │
│1 unread of 1   |   Focus: activity                         │
│┌ Watched pull requests ──────────────────────────────────┐│
││Pull request  State  CI          Title                   ││
││acme/api#42   open   CI passing  Pull request acme/api#42││
│└─────────────────────────────────────────────────────────┘│
│┌ Activity ───────────────────────────────────────────────┐│
││  ID  Observed       Pull request  State   Event          ││
││▶ a1  2026-…         acme/api#42   unread  New comment …  ││
│└─────────────────────────────────────────────────────────┘│
│┌ Controls ───────────────────────────────────────────────┐│
││j/k move   r read   a read all   q quit                  ││
│└─────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────┘
```

The selected row is marked with `▶ ` and reversed video; the `▶` marker is
the assertion contract, so plain text remains comprehensible without color. `no activity yet` is shown when the list is empty.

## Runtime flow

1. Parse `--config`, `--state`, `--snapshot` with clap.
2. Load config (exit 2 on error) and build `Inbox::new(JsonStateStore)`.
3. Load the view (exit 3 on error).
4. Snapshot: print `TuiApp::new(view).snapshot()` and exit 0.
5. Interactive: enable raw mode/alternate screen, draw, read a key, dispatch:
   - quit -> restore terminal and exit 0
   - move -> adjust selection and redraw
   - read -> `mark_read`/`mark_all_read`, reload the view, clamp selection
6. Errors during read are printed after restoring the terminal and exit 3.

## CLI contract

```
passuff-tui [--config PATH] [--state PATH] [--snapshot]
```

- `--snapshot` needs no TTY and prints the frame to stdout.
- Interactive mode requires a terminal; a terminal setup failure exits 3 with a
  message naming the TUI.
- Exit codes: 0 success/quit, 2 configuration, 3 state/terminal.
- No token, body, command text, or enum name is rendered. Human labels reuse the
  status/inbox vocabulary.

## Testing strategy

- Runner: existing `./scripts/verify.sh`; strict `.fail_on_skipped()`; stub
  `unimplemented!()`; single scenario via
  `GIVN_SCENARIO="Developer opens the inbox TUI" GIVN_FEATURES=$PWD/givn/changes/passuff-tui/specs ./scripts/verify.sh`.
- Step file: `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`, one
  capability file; regular and e2e branches in the same steps.
- Regular: build the `Inbox` view from the temp state, render a snapshot in
  process, and drive `TuiApp::handle_key` for navigation/read/quit.
- E2E: spawn `CARGO_BIN_EXE_passuff-tui --snapshot --config --state`; assert the
  frame and exit 0; transcript evidence under the active change.
- Unit tests: empty snapshot, selected-row marker, configured watch scope,
  selection clamp after a shorter reload, failed reload after `r`, unbound key
  leaves state unchanged, `load_view` maps config/state errors to exits 2/3.
- `load_view` maps a configuration error to exit 2 and a state error to exit 3;
  the two regular error scenarios spawn the binary and assert its stderr and
  exit code.
- Coverage: the e2e delta includes the TUI snapshot; the regular scenarios cover
  the renderer and key paths.

## Interaction coverage

| Use case | Capability | Inventory entry | @e2e scenario | Interface |
|---|---|---|---|---|
| review-activity-inbox | activity-inbox | list unread activity | Developer lists unread activity | unchanged `passuff inbox` |
| review-activity-inbox | activity-inbox | mark activity read | Developer marks activity read | unchanged `passuff read` |
| review-activity-inbox | activity-inbox | open the inbox in the terminal UI | Developer opens the inbox TUI | real `passuff-tui --snapshot` |

The snapshot is the real renderer and binary; key behavior is regular.

## Visual Design Contract

- Interface: terminal (not a Web UI; no HTML/CSS/JavaScript)
- Viewports: 100x30 transcript
- Design system: none; text rules above
- Approval: the operator approved the 100x30 layout on 2026-11-02 as part of
  the instruction to implement and archive the TUI.

Persona: `multi-account-developer` (confirmed). Screens: inbox with activity,
empty inbox. Capture: one transcript for the @e2e scenario under
`givn/changes/passuff-tui/evidence/visual/<slug>/transcript.txt`; a desktop
screenshot of the same frame captured from Alacritty with
`alacritty -o window.dimensions.columns=100 -o window.dimensions.lines=30` and
`import -display :0 -window <id> alacritty.png`. Motion: none.

## Use-case traceability

| Rule | Decision | Evidence |
|---|---|---|
| T1 | Layout renders counts/accounts/scope/PRs/activity | snapshot scenarios |
| T2 | j/k clamp, q/Esc quit | navigation/quit scenarios, unit tests |
| T3 | r/a mark and reload | read scenarios |
| T4 | empty message | empty scenario |
| T5 | config 2, state 3 before terminal | load-path tests |
| T6 | snapshot uses TestBackend | e2e snapshot |
| T7 | human labels only | negative assertions |

## Risks

- A terminal is required for interactive mode; snapshot mode is the scripted
  surface.
- The snapshot does not exercise raw mode or resize events; those are crossterm
  behavior, covered by a manual smoke.
- The TUI ships in the CLI package; a future split is a packaging change, not a
  core boundary change.

## Arc42/ADR

Chapters 01, 03, 04, 05, 06, 07, 08, 10, 11, 12 are affected. No new ADR: ADR-0003 already fixes the core/UI boundary, and this change
does not alter it.
