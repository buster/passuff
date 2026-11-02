# Review: passuff-tui

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)
PERSONAS: multi-account-developer

This pass ran in its own fresh context with no author history. It wrote only
this file. It re-read the committed tree, the change artifacts, the merged
coverage report, and the verification receipts; it did not re-run the suite,
because the receipt is the execution evidence
(`verification/tasks-verification.json`, `status: passed`, `runner_exit: 0`,
`total 11, passed 11, failed 0, skipped 0`, tested head `14b7283`, reused across
the documentation-only revision `81fe11c`).

Result: **PASS**. Both blockers from the previous review are closed in the
merged coverage report and in the committed scenarios, no other
implementation-evidence finding was found, and every residual coverage region
is classified. `givn check review --change passuff-tui` exits 0 after this
verdict.

## Prior blocker closure

| ID | Prior blocker | Current evidence | Closed |
|---|---|---|---|
| B1 | The non-terminal guard (`tui_main.rs:65-68`) was untested | `activity-inbox.feature:107-112` adds `The terminal UI requires a terminal` (`starts the inbox TUI without a snapshot`, asserts `requires a terminal`, exit 3). `c5a559b` adds the step (`steps/activity_inbox_tui.rs:13-16`) and the harness branch (`support/mod.rs:817-819`, `run_tui_binary_with(&[])`). The merged report records `tui_main.rs:65 hits=1, :66 hits=1, :67 hits=1`, branch `50% (1/2)` on `:65`. Line `:68` reports `0`: it is the closing brace whose region records the *not-taken* else path, consistent with the process exiting at `:67`. | yes |
| B2 | `Down`/`Up` dispatch in `TuiApp::handle_key` and the `r`-on-empty-list path were untested | `activity-inbox.feature:33-51` now presses `down`, `up`, `j`, `j`, `k` with a selection assertion after each press; `support/mod.rs:795-796` maps them to `KeyCode::Down`/`KeyCode::Up`. The hit counts match the press sequence exactly: `tui.rs:80 hits=3` (down + 2×j), `:81 hits=2` (two moves, one clamp), `:86 hits=2` (up + k). The `r`-on-empty path is exercised by `activity-inbox.feature:24-31` (`The terminal UI shows the empty inbox` presses `r` with no records): `tui.rs:90 hits=3` (`if let Some(record)`), `:91 hits=2` (mark_read), `:92 hits=2` (reload) — one of the three evaluations skips the body. | yes |

## Implementation evidence check

### `@e2e` tag integrity

| Feature file | `@e2e` scenarios | Change |
|---|---|---|
| `givn/changes/passuff-tui/specs/review-activity-inbox/activity-inbox.feature` | 1 (`:114-115`, `Developer opens the inbox TUI`) | present in `45262ed` and unchanged in `a556688`/`c5a559b`; never removed |
| `givn/specs/review-activity-inbox/activity-inbox.feature` | 2 (`:170`, `:182`) | unchanged permanent inventory rows |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | unchanged |
| `givn/specs/inspect-watch-status/watch-status.feature` | 1 | unchanged |

No `@e2e` tag was removed or moved. No `@wip` remains in the delta.

### Step bodies

0 empty step bodies found across 5 files (113 annotated definitions:
`activity_inbox.rs` 12, `activity_inbox_tui.rs` 11, `pr_activity_notification_e2e.rs`
6, `pr_activity_notification.rs` 71, `watch_status.rs` 13). The only `{}` body in
the test tree is `support/e2e.rs:76` `FakeNotifications::close_notification`, a
D-Bus interface stub in the digital twin, not a step definition. Each TUI step
delegates to a harness action that exercises the real renderer
(`support/mod.rs:758-844`) and asserts on the rendered frame, the selected
record, or the on-disk read flag.

### Scenario commit evidence

10 checked scenario tasks, 10 `COMMIT:` entries, every one naming a batch
(`tasks.md:32,44,55,65,77,88,99,133,142,151`). Spot-checked with `git show --stat`:

| Revision | Batch | Touches |
|---|---|---|
| `45262ed` | `tui-render`, `tui-keys` | `src/tui.rs`, `src/tui_main.rs`, `Cargo.toml`/`Cargo.lock`, step file, `support/mod.rs`, `support/e2e.rs`, feature, design, arc42, glossary, domain model |
| `a556688` | `tui-review-fixes` | `src/tui.rs` (353 lines), `src/tui_main.rs` (73 lines), step file, `support/mod.rs`, feature, design, arc42 chapters, evidence |
| `c5a559b` | `tui-review-closure` | feature (3 scenarios), step file, `support/mod.rs`, design, README badges, transcripts |

No scenario commit is comment-, tag-, task-, feature-, or transcript-only: the
three closure scenarios are carried by `c5a559b`'s regression tests (the
`down`/`up` key mapping and the `run_tui_binary_without_snapshot` branch), and
the other seven by `45262ed`/`a556688` production and test code. The gate agrees
(`✓ scenario commits passed (10 scenario(s))`). The change is a normal change;
no `CHARACTERIZATION` marker is used.

### Promised components and contract use

Every component `design.md:28-49` promises exists:

| Promised | As built |
|---|---|
| `TuiApp`, `TuiAction`, `TuiLoadError`, `load_view` | `crates/passuff-cli/src/tui.rs:13-104` |
| `snapshot` (TestBackend 100×30), `render`, `reload`, `handle_key` | `tui.rs:59-103,109-274` |
| `passuff-tui` binary (`--config`, `--state`, `--snapshot`) | `crates/passuff-cli/src/tui_main.rs:17-74`, `Cargo.toml:14-16` |
| Step file `tests/steps/activity_inbox_tui.rs` | present, registered at `tests/features_runner.rs:5-6` |
| `tui` module exported by the CLI crate | `crates/passuff-cli/src/lib.rs:6` |

Contract use: `tui.rs:3` consumes `passuff_core::{CoreError, InboxService,
InboxView}` only; it does not import `Inbox` or `JsonStateStore`. The two places
that construct the implementation are the composition root
(`tui_main.rs:13,52`) and the tests (`support/mod.rs`, `tui.rs` test module),
which `givn instructions architecture-principles` names as the exceptions.
`tui_main.rs:11` uses `passuff_cli::tui::{load_view, TuiAction, TuiApp}`, the
declared surface, not module internals.

### Strict-mode proof

Present with output at `tasks.md:9-20`: an undefined TUI step in
`/tmp/tui-strict.feature` reports `total 1, passed 0, failed 1, skipped 0` and
cargo exits 1. The runner sets `.fail_on_skipped()`
(`tests/features_runner.rs:52`) and exits nonzero on zero total, any failure, or
any skip (`features_runner.rs:98-100`).

### E2e integrity (not downgraded)

Every delta `@e2e` Then step asserts the spawned binary's output and status:

- `activity-inbox.feature:115-124`: `the TUI shows "1 unread of 1"`,
  `the TUI shows "New comment from @alice"`, `the exit status is 0`.
- The harness (`support/mod.rs:758-761` → `support/e2e.rs:379-398`) spawns
  `CARGO_BIN_EXE_passuff-tui --snapshot --config <path> --state <path>` and
  reads **stdout**; nothing asserts only on the state file.

No step drives the interface through a lower-level boundary, and no in-process
fake substitutes for the binary in the e2e branch. This is a terminal UI, not a
Web UI: the opposite classification was considered, but the product renders no
HTML/CSS/JavaScript, so the browser-driver rule does not apply and a real
subprocess is the correct driver.

### Exact e2e command, parallel implementation, and scope

- `givn/commands.yaml` `verify.e2e_command` is `./scripts/verify.sh --e2e`.
  Reading the script (`scripts/verify.sh:16-40`): it exports
  `GIVN_RESULT_SCOPE=e2e`, defaults the feature path to `$root/givn/specs`, and
  runs `cargo test -p passuff-cli --features test-support --test features_runner`.
  The invoked file is therefore `crates/passuff-cli/tests/features_runner.rs`,
  which binds `crates/passuff-cli/tests/steps/activity_inbox_tui.rs` plus the
  shared support modules — not a file that merely contains "e2e" in its name.
- The parallel-implementation search was run over the whole tree including
  untracked files (`git status --porcelain` is empty). `CARGO_BIN_EXE_passuff-tui`
  appears in exactly two places: `support/mod.rs:834` (regular scenario harness)
  and `support/e2e.rs:387` (e2e harness). Both spawn the real binary; the e2e one
  is the stronger of the two (it asserts stdout and exit status), and it is the
  one the `@e2e` scenario uses. No weaker duplicate was reviewed.
- Scope isolation is explicit: `GIVN_RESULT_SCOPE` is set by the script
  (`verify.sh:18-22`) and applied by the runner filter
  (`features_runner.rs:58-64`, `match scope { "e2e" => is_e2e, "regular" => !is_e2e, _ => true }`).
  The change receipt is the `change` scope (11/11, which includes the delta
  `@e2e` scenario); the e2e command is the `e2e` scope over the permanent specs
  (recorded in `tasks.md:29-31` as delta e2e 1/1 and permanent e2e 7/7). Both
  executions report zero failures and zero skips.

### Implementation vs `design.md`

| `design.md` names | As built | Match |
|---|---|---|
| ratatui 0.30.2, crossterm 0.29.0 (`:22-23`) | `Cargo.toml:24-25` (`ratatui = "0.30"`, `crossterm = "0.29"`), `Cargo.lock` resolves `0.30.2` / `0.29.0` | yes |
| `--snapshot` uses `TestBackend` and prints the frame (`:21`) | `tui.rs:257-274` | yes |
| Step file `crates/passuff-cli/tests/steps/activity_inbox_tui.rs`, regular and e2e in one file (`:104-105`) | present; one file, both branches | yes |
| Runner `./scripts/verify.sh`, single-scenario command via `GIVN_SCENARIO` (`:101-103`) | `scripts/verify.sh`; the runner reads `GIVN_SCENARIO` (`features_runner.rs:35,68-70`) | yes |
| Exit codes 0/2/3, terminal setup failure exits 3 naming the TUI (`:88-95`) | `tui_main.rs:53-73,81-92` | yes |
| Layout 100×30 with the `▶` marker and reversed video (`:33-62`) | `tui.rs:242-244`, snapshot `100×30` at `:258`; visible in the committed transcript | yes |

One non-material difference: `proposal.md:51` lists the illustrative path
`crates/passuff-cli/src/bin/passuff-tui.rs`; the binary is
`crates/passuff-cli/src/tui_main.rs` declared as `[[bin]]`. `design.md` names no
binary path and the layout choice (a second binary in `passuff-cli`) is
unchanged, so no reviewed design decision was silently overridden.

### Interaction coverage

Inventory from `givn/specs/review-activity-inbox/usecase.md:78-84`, design
matrix from `design.md:127-131`, scenarios from the feature files:

| Use case | Capability | Inventory entry | Design matrix row | `@e2e` scenario | Driving mechanism in steps | Match |
|---|---|---|---|---|---|---|
| review-activity-inbox | activity-inbox | list unread activity | yes (`design.md:129`) | `Developer lists unread activity` (`givn/specs/review-activity-inbox/activity-inbox.feature:170-171`) | `support/e2e.rs:404-420` spawns `CARGO_BIN_EXE_passuff`, reads stdout | yes |
| review-activity-inbox | activity-inbox | mark activity read | yes (`:130`) | `Developer marks activity read` (`:182-183`) | `e2e_run_read_all` → `e2e_run_subcommand(["read","--all"])`, stdout + exit | yes |
| review-activity-inbox | activity-inbox | open the inbox in the terminal UI | yes (`:131`, Interface `real passuff-tui --snapshot`) | `Developer opens the inbox TUI` (`activity-inbox.feature:114-115`) | `support/e2e.rs:379-398` spawns `CARGO_BIN_EXE_passuff-tui --snapshot`, reads stdout | yes |

3 inventory rows, 3 matrix rows, 3 matching `@e2e` scenarios, no unmapped row
and no extra row. Variants (empty inbox, selection, read/mark-all, quit,
configuration/state/terminal errors) stay regular, so no action is
over-covered.

## Arc42 implementation conformance

ARC42 CONFORMANCE: CLEAN

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| A full-screen terminal UI over the same inbox is an essential requirement | `01-introduction-and-goals.md:9,25,46` | row 1 Yes | Scope (`:3-14`) | setup (`:24-29`) | `crates/passuff-cli/src/tui_main.rs:17-74` | yes |
| The TUI is a terminal surface over local config/state with no GitHub call | `03-context-and-scope.md:48` | row 3 Yes | `:5-9`, `:88-95` | setup, B2 (`:103-105`) | `tui.rs:19-34` uses `InboxService::view`; no gateway import | yes |
| Second adapter over `InboxService`, shipped in the CLI package | `04-solution-strategy.md:20,41,63` | row 4 Yes | `:5-14`, `:28-29` | setup, B2 | `Cargo.toml:14-16`, `lib.rs:6` | yes |
| `TuiApp`/`passuff-tui` building block and the `TUI -.->|InboxService| Core` edge | `05-building-block-view.md:33,52,67` | row 5 Yes (`TuiApp`, `TuiAction`, binary) | `:24-49` | B3 (`:106-108`) | `tui.rs:42-104`, binary `tui_main.rs` | yes |
| Terminal inbox runtime flow: load → errors 2/3 → snapshot or interactive loop | `06-runtime-view.md:93,203-216` | row 6 Yes | `:69-86` | B2, B3 | `tui_main.rs:51-122` matches the five numbered steps | yes |
| A second binary ships in the same workstation install | `07-deployment-view.md:58-60` | row 7 Yes | `:7-12` | setup | `Cargo.toml:10-16` (two bins in one package) | yes |
| Terminal rendering, key handling, snapshot mode, error exits as crosscutting concepts | `08-crosscutting-concepts.md:84-91` | row 8 Yes | `:16-24`, `:88-95` | B3 | `tui.rs:109-274`, `tui_main.rs:61-67` | yes |
| No new ADR; ADR-0003 governs the adapter boundary | `09-architecture-decisions.md:13`, `adr/ADR-0003-library-first-core.md:101-107` | row 9 No — "No new ADR; ADR-0003 covers the adapter boundary" | `:160-163` "No new ADR … does not alter it" | — | ADR-0003 keeps the same decision outcome and adds a dated refinement note for the package choice | yes — the impact row is about a *new* decision, and the decision outcome is unchanged; the refinement is recorded in the ADR body as the ADR itself states |
| QS-020 for the terminal inbox and QS-011 for `InboxService` reuse | `10-quality-requirements.md:43,52` | row 10 Yes (QS-020) | `:143-151` (T1–T7) | B2 (`:103-105`) | `activity-inbox.feature:6-51` and the TUI e2e assert both rows | yes |
| R-019 (no PTY), R-020 (package choice), R-010 (`InboxService` consumers) | `11-risks-and-technical-debt.md:16,25,26` | row 11 Yes | `:8-12`, `:153-157` | B2 | `tui_main.rs:65-68` guard, `Cargo.toml`, doc-only surfaces | yes |
| "Terminal UI" term | `12-glossary.md:1-12` → `docs/glossary.md:42` | row 12 Yes | — | — | code symbol `TuiApp` exists (`tui.rs:42`) | yes |

Chapter 02 is correctly marked not affected (no new platform constraint), and no
affected chapter is a placeholder or a stale contradiction. The `tasks.md` B2
mapping resolves to the chapters actually edited in `45262ed`/`a556688`
(`05`, `06`, `10` QS-011/QS-020, `11` R-010/R-019/R-020).

## Ubiquitous language and structural model

The glossary term **Terminal UI** (`docs/glossary.md:42`: "The full-screen inbox
adapter over `InboxService`; a second consumer of the read model", code symbol
`TuiApp`, anti-terms dashboard/GUI) is the term used by the use case
(`usecase.md:44-51` T1–T7), the design, the feature titles, and the code
(`TuiApp` in `tui.rs`). Existing terms are reused unchanged: **Inbox**,
**Activity record**, **Read state**, **Watch scope**, **Watched pull request**.
No competing synonym appears in README, glossary, ADRs, docs, tests, or code:
the scenarios say "activity", "watched pull requests", "watch scope", and
"read", never "list", "item", "feed", or "dashboard".

Structural impact: the change adds a UI adapter over the existing read model, not
a new aggregate. `docs/domain-model.md:129` adds the relationship
`Inbox → Terminal UI` ("A UI adapter renders the read model and marks records
read", governed by `review-activity-inbox` T1–T7). The structural overview
deliberately draws aggregates, entities, and external surfaces only; the two
pre-existing read-model rows (`Activity Record → Inbox`, `Watched Pull Request →
Watch status`) follow the same convention, so the table and diagram remain
consistent and no diagram edit was required. No stale term remains.

## Use-case and Persona conformance

Owning use case `review-activity-inbox` (`givn/specs/review-activity-inbox/usecase.md`):

- Goal/trigger: "The developer reviews the activity passuff recorded and marks it
  read"; the trigger names the same developer (`:7-13`). One triage goal; marking
  read is how the review concludes, not a second goal.
- Primary actor: `developer` (`:5`); trigger names it.
- Capabilities: `activity-inbox` only (`:75-77`); placement holds — the TUI is a
  channel for the same review goal, not a capability of its own
  (`proposal.md:44`, `givn check review`: "capability 'activity-inbox' already
  owned by givn/specs/review-activity-inbox").
- Rules T1–T7 (`:44-51`) were added by this change and each maps to an
  implemented, asserted behaviour: T1 `activity-inbox.feature:6-22`, T2 `:33-51`,
  T3 `:53-77`, T4 `:24-31`, T5 `:92-112`, T6 `:114-124`, T7 `:18-21` negatives.
- Extensions 1a/1b (config exit 2, state exit 3) are preserved by T5 and by
  `load_view` (`tui.rs:19-34`).
- Mixin/extension relationships: none (`:86-91`), unchanged.
- No mutation under an observing goal: the read commands and the TUI mark read
  state, which the success guarantee names.
- Inconsistent preconditions: none; the state file may be missing (`:18-19`),
  and the empty-inbox scenario covers it.

Persona `multi-account-developer` (confirmed in `proposal.md:6`, listed at
`usecase.md:73`) is a review lens, not an invented identity. Its outcomes hold:
the screen shows the configured account and resolved login
(`personal @octo-personal`), the watch scope, the watched pull request, the
unread count, and the activity list; it can mark read and quit; the state
survives restarts (unchanged core). No Persona was invented or silently
promoted. Actors remain interaction roles.

## Coverage

Measured evidence (`coverage/cobertura-coverage.xml`, root attributes, branch
instrumentation present):

| Report | Lines | Branches |
|---|---|---|
| merged | 1955/2320 (84.3%) | 203/632 (32.1%) |
| non-e2e | 1768/2320 (76.2%) | 220/804 (27.4%) |
| e2e | 826/2320 (35.6%) | 104/804 (12.9%) |

The README badges (`README.md:4-5`) match the merged report. Both scopes carry
branch data; nothing was downgraded to line-only.

Bucket classification of the residual gaps, using only the three buckets:

1. **Dead code (delete): empty.** Nothing in the changed tree is unused. The
   former accessors `TuiApp::view`/`TuiApp::selected` are already removed
   (`tasks.md:109`); `selected_summary` and `footer_keys` are both live
   (`support/mod.rs:846-853`, `tui.rs:105-107,252`).
2. **Missing test coverage: empty for change-introduced logic.** `tui.rs` has
   234/236 lines hit; the two uncovered lines (`tui.rs:341,348`) are bodies of
   `FakeInbox::mark_all_read` in the `#[cfg(test)]` module — the `a` key is
   covered through the real `Inbox` by `activity-inbox.feature:67-77`. Every
   change-introduced production line in the renderer, key dispatch, load and
   error mapping is hit, including the two prior blocker regions (above).
3. **Legitimately hard to test (concrete justification):**
   - `tui_main.rs:76-97` (`run_terminal`), `:99-122` (`drive`), `:124-127`
     (`restore`), and the continuation at `:69-73`: interactive mode requires a
     TTY — crossterm raw mode, the alternate screen, and a blocking key read.
     The operator recorded the verification decision (proposal D2,
     `proposal.md:56-60`: snapshot over a PTY driver) and the residual risk is
     registered as R-019 (`11-risks-and-technical-debt.md:25`). The rendered
     screen itself is asserted through the same `render` code by the snapshot
     and every scenario.
   - `tui_main.rs:37-49`: the `--config`/`--state` fallback closures resolve
     `directories::ProjectDirs` host paths and run only when neither flag is
     passed. Every Gherkin and unit path supplies explicit paths; a fixture that
     omitted them would resolve the operator's real platform configuration
     directory, so covering it asserts platform-directory resolution rather than
     a product rule. No use-case rule (T1–T7) or CLI-contract clause names a
     default path. The identical pre-existing block (`main.rs:24-41`, also
     uncovered) carries the same recorded disposition
     (`givn/archive/passuff-pr-watch/review.md:63`, bucket 3, R19).

No fourth bucket is used. No redundant unit test was removed or added: the TUI
scenarios and the 8 `tui.rs` unit tests assert different things (screen text vs
selection clamp after reload, failed reload, unbound key, `load_view` exit
mapping).

## Visual review

Interface: terminal (declared at `design.md:133-146`; not a Web UI — no HTML,
CSS, or JavaScript is rendered). Evidence under
`givn/changes/passuff-tui/evidence/visual/developer-opens-the-inbox-tui/`:

| Declared UI state (`design.md:143`) | Evidence seen |
|---|---|
| inbox with activity | `transcript.txt` (17-line frame: `passuff — Inbox`, `personal @octo-personal`, `watch scope: all authored pull requests`, `1 unread of 1`, watched-PR table `acme/api#42 open CI passing Pull request acme/api#42`, activity row `▶ a1 2026-11-02T23:03:37… acme/api#42 unread New comment from @alice`, controls `j/k move r read a read all q quit`, `exit status: 0`) and `alacritty.png` (live Alacritty frame, same layout, `2 unread of 2`, `▶` on `a1`, activity rows for `New comment from @alice` and `CI failing: build`) |
| empty inbox | no separate artifact: the state is not traversed by the `@e2e` scenario, whose capture set the contract fixes at one transcript plus one screenshot of the same frame (`design.md:143-145`). The state renders through the same `render` path (`tui.rs:207-211`) and is asserted by `activity-inbox.feature:24-31` (`no activity yet`, `0 unread of 0`) |

Baseline: every control carries a visible label (`j/k move r read a read all q
quit`); no internal identifier or enum name is visible (human labels `open`, `CI
passing`, `unread`, `New comment from @alice`); the panel titles are real
headings; no duplicated content; no token, comment body, or helper command text
appears; the `▶` marker plus reversed video makes the selection readable without
colour. Persona `multi-account-developer` can see which account and watch scope
it is looking at, which pull request is watched, what arrived, and how to
acknowledge or leave — its goal is reachable from what is shown.

Observations, dispositioned:

- The PNG is a live interactive frame (`2 unread of 2`, title `Split the
  migration`) rather than a literal re-render of the transcript frame (`1 unread
  of 1`). It shows the same declared state and the same layout, and it is the
  stronger artifact for the interactive path; the interface rules above hold in
  both. No contract rule about what the consumer must read is broken.
- The empty-inbox state is evidenced by its scenario assertions rather than a
  separate capture; the contract's capture set does not promise one.

No Persona finding remains. The evidence is complete for the single `@e2e`
scenario and the declared viewport (100×30).

VISUAL-REVIEW: PASS

## Overlap dispositions

The three shape-match findings `givn lint`/`givn check review` report for this
change are dispositioned below; the ten subset advisories follow in the next
section.

| Scenario A | Scenario B | Decision | Reason |
|---|---|---|---|
| The terminal UI reports a configuration error | The terminal UI reports a state error | variant | Same binary-start skeleton across two failure classes; A asserts exit 2 before the view load, B asserts exit 3 from the state read; merging hides which exit contract regressed |
| The terminal UI reports a configuration error | The terminal UI requires a terminal | variant | Different failure: configuration load (exit 2) vs the terminal capability guard at `tui_main.rs:65-68` (exit 3); the second is the regression test for prior blocker B1 |
| The terminal UI reports a state error | The terminal UI requires a terminal | variant | Different failure class and different code path (`load_view` vs the `is_terminal` guard); merging would make the B1 regression test invisible |

### Subset advisories

| Scenario A | Scenario B | Decision | Reason |
|---|---|---|---|
| The terminal UI shows the empty inbox | The terminal UI shows the inbox | boundary | T4 empty state vs T1 populated state; the empty scenario alone asserts `no activity yet` and `r` on an empty list |
| The terminal UI shows the empty inbox | Developer opens the inbox TUI | boundary | Same precondition family, different subject: the regular scenario asserts the empty rendering in process, the e2e asserts the real binary's stdout and exit 0 |
| Every activity is marked read | The selected activity is marked read | variant | T3 has two key commands with different postconditions (`a` → both read, `r` → one read one unread); merging loses the selected-only rule |
| Every activity is marked read | The terminal UI reports a configuration error | boundary | Shape-only match; T3 read state vs T5 exit codes and no shared outcome |
| Every activity is marked read | The terminal UI reports a state error | boundary | Shape-only match; distinct rule (T3 vs T5) |
| Every activity is marked read | The terminal UI requires a terminal | boundary | Shape-only match; distinct rule (T3 vs the terminal guard) |
| Developer starts watching and is notified of a new comment | The terminal UI reports a configuration error | boundary | Permanent notification capability vs delta TUI configuration failure; different capability and driver |
| Developer starts watching and is notified of a new comment | The terminal UI reports a state error | boundary | Distinct capability and rule (notify vs exit 3) |
| Developer starts watching and is notified of a new comment | The terminal UI requires a terminal | boundary | Distinct capability; the shared `Given a configured account` hides unrelated outcomes |
| Developer starts watching and is notified of a new comment | Developer opens the inbox TUI | boundary | Different capabilities (notification delivery vs inbox browsing) and different binaries |

## Split-or-keep

| Scenario | Decision | Reason |
|---|---|---|
| The terminal UI shows the inbox | keep | One screen is the subject: T1's positive content and T7's four negatives assert the *same* rendered frame (15 steps). Splitting duplicates the five-step fixture and separates a negative from the frame that must not contain it. |
| The selection moves and clamps | keep | One key sequence is the subject (17 steps): each `Then` establishes the selection the next `When` acts on, so the state is not expressible as independent cases without repeating the fixture and weakening the clamp proof. |

No split is chosen, so no split work is outstanding.

## Residual advisories (non-blocking)

| ID | Observation | Disposition |
|---|---|---|
| A1 | T2's upper clamp (`k`/`up` at selection 0) is not asserted directly: the scenario presses `up` from index 1 and `k` from index 1, so the `saturating_sub` guard is executed (`tui.rs:86`, 2 hits) but never at index 0. The rule is implemented; the boundary value is unasserted. | Non-critical; recorded for the next change (a `k` press at index 0 in `activity-inbox.feature:33-51` would close it). It is not an uncovered region, so it does not alter the coverage classification above. |
| A2 | `proposal.md:51` names the illustrative binary path `crates/passuff-cli/src/bin/passuff-tui.rs`; the binary is `src/tui_main.rs` under `[[bin]]`. | Non-material: `design.md` names no path, and the reviewed packaging decision (a second binary in `passuff-cli`) is unchanged. |
| A3 | The archived `TuiApp` snippet in `design.md:28-49` does not list `selected_summary`/`footer_keys`. | Additional public helpers used by the step harness; no declared contract is broken. |

## README impact decision

README-IMPACT: updated - Usage

- Usage (`README.md:94-114`) lists `passuff-tui` ("open the inbox in a
  full-screen terminal UI") and `passuff-tui --snapshot` ("print one TUI frame
  and exit"), added by `79a6daa`; the exit-code line still matches the TUI
  contract (0 success, 2 configuration/usage, 3 state/terminal).
- The coverage badges (`README.md:4-5`) are refreshed to 84.3% / 32.1% and match
  the merged report. No placeholder is open.

## Sign-off checklist

- [x] Fabrication audit clean: 0 empty step bodies; every checked task has a
      commit touching production code or its regression test (10 scenarios, 10
      `COMMIT:` entries, batches named).
- [x] Every promised component exists; every consumer uses only its declared
      contract (`InboxService`; composition root and tests excepted).
- [x] Strict-mode proof present and passing (`tasks.md:9-20`).
- [x] Scenario execution evidence recorded per scope in the verification receipt;
      the suite was not re-run in this review.
- [x] Coverage classified from the most recent measured evidence (merged
      report); branch instrumentation intact; no line-only downgrade.
- [x] Every coverage gap classified and resolved: bucket 1 empty, bucket 2
      empty, bucket 3 justified.
- [x] Dead code deleted; no redundant unit test kept.
- [x] No `@wip`; no `@e2e` removed; no implementation-layer detail in the spec.
- [x] Canonical E2E policy read from `givn instructions specs`: every normalized
      inventory action has exactly one `@e2e` scenario; variants are regular.
- [x] Every E2E scenario asserts the real interface (spawned binary stdout and
      exit status); no HTTP/fetch substitute; not a Web UI.
- [x] The exact files `verify.e2e_command` invokes were identified
      (`scripts/verify.sh --e2e` → `tests/features_runner.rs` with
      `steps/activity_inbox_tui.rs`); the tree (including untracked files) was
      searched for a parallel implementation — one e2e harness exists.
- [x] E2E scope selected explicitly through `GIVN_RESULT_SCOPE`; both required
      executions report zero failures and skips.
- [x] Implementation (commands, file layout, framework/driver) matches
      `design.md`; the one path difference is unclaimed by the design.
- [x] Interaction coverage verified: 3 inventory rows → 3 matrix rows → 3
      matching `@e2e` scenarios → real drivers.
- [x] The run declaration passes the gate (`product_type: cli`; a one-shot
      command with no long-running stack or digital twin needed to start it).
      Not re-executed in this pass, which reuses the execution receipt.
- [x] No finding was excused with a classification outside the three buckets.

## Verdict

Both prior blockers are closed, the implementation evidence is complete, the
Arc42 and domain artifacts match the built change, and every residual region is
classified. No blocker remains.

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

REVIEW: PASS
