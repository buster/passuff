# Review: python-rebuild

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-1

Scope: the current working tree at `a3a717ef0a15ab538ea4c888a15e184eef03b673`.
The operator forbids `git log`, `git show`, `git diff`, reflog, and blame, so
commit evidence was checked by object inspection only (`git cat-file -t`,
`git ls-tree`), which does not expose the change history.

## Implementation evidence check

### 0. `@e2e` tag integrity

`givn/changes/python-rebuild/specs/**/*.feature` carries one `@e2e` scenario:
`Developer opens an activity's details` (`@givn.modified @e2e`). The permanent
corpus carries 18 `@e2e` entries, exactly one per row of every use case's
`## Interactions` inventory. The change-scope count 229, the regular count 197,
and the E2E count 19 recompute from the feature files:

| Scope | Entries | Outline rows | Executions |
|---|---|---|---|
| Permanent non-E2E | 167 | 30 | 197 |
| Permanent `@e2e` | 18 | 0 | 18 |
| Delta (all) | 18 | 14 | 32 |
| Change (`delta` + permanent non-E2E) | — | — | 229 |
| E2E (permanent + delta `@e2e`) | 19 | 0 | 19 |

No `@e2e` tag was removed: 18 permanent E2E entries map one-to-one onto the 18
inventory rows, and each inventory row resolves to an existing scenario title.
`givn lint` reports no tag-removal warning and no inventory mismatch.

### 1. Empty step body audit

`python3 scripts/check_steps.py` **fails**:

```text
givn/steps/common.py:1476: no-op step reopens
exit 1
```

`the developer reopens passuff` is bound to a bare `return None`
(`givn/steps/common.py:1475-1476`) and is used by the permanent scenario
`Read state survives a restart` (`givn/specs/review-activity-inbox/activity-inbox.feature:91`).
That is a fabricated step by the review rule and by the project's own audit.
`scripts/verify.sh` never runs `check_steps.py` over the real step trees (only
`--strictness` runs it against `tests/strictness/pass_only.py`), which is why
the audit failure did not stop the runner.

Result: **1 no-op step body across 9 step files** (`givn/steps/common.py`,
`givn/e2e_steps/common.py`, and the seven one-line `givn/e2e_steps/*_e2e.py`
placeholders).

### 2. Checked scenarios and their commits

- 205 tasks are checked (`### [x]`), each with a `COMMIT:` line.
- 205 distinct `COMMIT:` lines all name
  `14e0b99b19b4851e203c78ac375f3ce69a863da9` and every line names its
  `Batch: <name>`; `givn changes` allows a shared batch revision only with the
  batch name, and the names are present.
- The object `14e0b99b19b4851e203c78ac375f3ce69a863da9` exists and is a commit.
  `git ls-tree -r` on it lists 31 files under `passuff/`, `givn/steps/`,
  `givn/e2e_steps/`; the production modules, the step bindings, the runner, and
  the test twins are all in that tree. It is not a comment-only, task-only, or
  feature-only commit.
- The task-evidence commit `dea85c84efab68cbcd8022794922fcdf8d5b23d1` exists and
  carries the change artifacts.
- The verification receipts `1c84c428e423afded5dc60cfae9a55253cd24f22` and
  `a3a717ef0a15ab538ea4c888a15e184eef03b673` exist; `givn check review` reports
  `change verification receipt passed (229 scenario(s) at a3a717ef...)`, and
  `verification/tasks-result.json` records `total=229 passed=229 failed=0
  skipped=0`.
- Not a characterization change (`CHARACTERIZATION` is absent); the rebuild is a
  normal change.

### 3. Promised components

| Promised in `design.md` | Present | Evidence |
|---|---|---|
| `passuff/cli.py`, `composition.py`, `config.py`, `credentials.py`, `github.py`, `classify.py`, `core.py`, `state.py`, `inbox.py`, `stacking.py`, `scope.py`, `setup.py`, `render.py`, `notify.py`, `sound.py`, `browser.py` | yes | module files exist |
| `passuff/tui/app.py`, `passuff/tui/frame.py` | yes | files exist |
| `passuff/status.py` (`WatchStatusReader`) | **no** | class lives in `passuff/inbox.py:54`; `passuff/status.py` does not exist |
| `passuff/tui/session.py` (`ObservationSession`) | **no** | class absent; live loop is `_observe`/`_interactive` in `passuff/tui/app.py:338,354` |
| `tests/support/recording_delivery.py` (`RecordingDelivery`) | **no** | absent; `DesktopDelivery(recording_path=...)` (`passuff/notify.py:17,27-30`) is a test hook inside production code and has no caller |
| `tests/support/bin/osascript`, `tests/support/bin/afplay` | **no** | only `gh`, `paplay`, `xdg-open` exist |
| `givn/steps/pr_activity_notification_steps.py`, `watch_status_steps.py`, `activity_inbox_steps.py`, `inbox_navigation_steps.py`, `stacked_inbox_steps.py`, `watch_scope_steps.py`, `initial_setup_steps.py` | **no** | `givn/steps/` contains only `common.py` (1788 lines, 237 bindings) |
| `givn/e2e_steps/<capability>_e2e.py` (“E2E bindings, one file per capability”) | placeholders | each of the seven files is a single docstring line with no binding; all bindings load through `givn/e2e_steps/common.py:5` importing `givn/steps/common.py` |
| `TuiApp` frame renderer, `PassuffCore`, `ActivityClassifier`, `ActivityInboxService`, `WatchStatusReader`, `ScopeEditor`, `SetupSession`, `GitHubApiAdapter`, `JsonStateStore`, `TomlConfigStore`, `SystemCredentialSource`, `DesktopDelivery`, `SystemSoundPlayer`, `SystemBrowserOpener` | yes | classes exist and are exercised |

A promised component that is absent is incomplete implementation, not a
“hard to test” gap.

### 4. Strict-mode proof

`./scripts/verify.sh --strictness` was run during this review and exits 0:
the undefined fixture errors, the skipped fixture counts `1/1 skipped`, and the
pass-only fixture is rejected by `check_steps.py`. The proof is present in
`tasks.md` (“Setup: runner, strictness, and coverage”) and is reproducible.

The proof covers the fixtures only. The durable tree it is meant to protect
fails the same audit (finding F1), and `verify.sh` does not run `check_steps.py`
in the `change`, `regular`, or `e2e` scopes.

### 5. Every `@e2e` Then asserts the real interface

All 19 `@e2e` scenarios assert on what the consumer sees or receives:

- TUI scenarios assert the rendered frame text through
  `the TUI shows "..."` on the subprocess output of
  `passuff-tui --snapshot` (`givn/steps/common.py:866-885,960-970`);
- CLI scenarios assert the subprocess transcript and exit status
  (`givn/steps/common.py:818-863`);
- notification scenarios assert the fake D-Bus service and the sound log, and
  the browser scenario asserts the opener URL log;
- counters (`the inbox has asked GitHub for the stack fields "2" times`) are
  secondary evidence beside a frame assertion.

No `@e2e` scenario asserts only repository or state-file content.

The terminal interface classification holds: the product renders a curses TUI
and a CLI transcript. The opposite classification — a rendered page is a Web UI
— does not apply, because nothing renders HTML/CSS/JS in a browser; the TUI is
driven through real subprocesses and a pty. A browser driver would be wrong
here.

### 6. Exact `verify.e2e_command` and the files it invokes

`givn/commands.yaml` declares `verify.e2e_command: "./scripts/verify.sh --e2e"`,
matching `design.md` exactly. `scripts/verify.sh:71-75` runs
`behave --stage e2e --tags="@e2e"` over `givn/specs/*/*.feature` and the delta
features; `--stage e2e` loads `givn/e2e_environment.py` and `givn/e2e_steps/`.
The only bindings reachable from that tree are
`givn/e2e_steps/common.py` → `givn/steps/common.py`.

A tree-wide search (tracked and untracked, excluding `.venv`) found exactly one
E2E implementation tree. There is no weaker parallel implementation to compare
against, and no second step tree was hidden by the command.

### 7. E2E scope selection

`scripts/verify.sh:11` reads `GIVN_RESULT_SCOPE` (default `regular`); `--e2e`
sets `SCOPE=e2e`; `change` runs the delta plus the permanent non-`@e2e` corpus;
`regular` runs the permanent non-`@e2e` corpus; `e2e` runs only `@e2e`. The
scope is selected explicitly, not inferred from the command string.

### 8. E2E scope size

The inventory is 18 normalized actions (3 watch-scope, 1 watch-status, 5
activity-inbox, 2 inbox-navigation, 1 stacked-inbox, 1 initial-setup, 5
pr-activity-notification). Each has exactly one `@e2e` scenario. The delta adds
one `@givn.modified @e2e` scenario that replaces the permanent
`Developer opens an activity's details`. Outlines in the delta are regular
scenarios; no action is double-covered.

### 9. Design conformance (commands, layout, framework)

| `design.md` says | As built | Match |
|---|---|---|
| `verify.command: ./scripts/verify.sh`, `verify.e2e_command: ./scripts/verify.sh --e2e` | identical in `givn/commands.yaml` | yes |
| behave via `behave.ini`, `givn/environment.py`, `givn/e2e_environment.py` | present | yes |
| stdlib `curses` + pure frame renderer, `--snapshot` | `passuff/tui/app.py`, `passuff/tui/frame.py` | yes |
| `httpx`, `tomllib`, `tomlkit`, `jeepney` | present | yes |
| `givn/steps/` regular bindings, one file per capability, named in the Step Definitions table | one 1788-line `common.py` | **no** (F2) |
| `givn/e2e_steps/`, one file per capability, bindings | seven docstring-only files | **no** (F2) |
| `passuff/status.py`, `passuff/tui/session.py` | absent | **no** (F3, F4) |
| `tests/support/recording_delivery.py`, `bin/osascript`, `bin/afplay` | absent | **no** (F5) |
| `tests/strictness/` fixtures removed after the proof | fixtures retained and tracked | **no** (F12) |

`design.md` was not updated for any of these deviations, and `design-review.md`
Q4 explicitly resolved the opposite (a staged per-capability E2E tree). The
Step Definitions table was not edited to match what was built.

### 10. Interaction coverage cross-reference

| Inventory entry | Design matrix row | `@e2e` scenario | Driving mechanism | Match |
|---|---|---|---|---|
| watch-scope: toggle a repository | yes | Developer toggles a repository in the watch scope | `passuff-tui --picker --snapshot --scope-select/--scope-deselect` | yes |
| watch-scope: switch the mode | yes | Developer switches the watch scope mode | `--scope-mode selected --picker --snapshot` | yes |
| watch-scope: narrow the focus | yes | Developer narrows the focus in the watch scope | `--focus-set authored --picker --snapshot` | yes |
| watch-status: show the watch state | yes | Developer sees the watch state | subprocess `passuff status` | yes |
| activity-inbox: list unread | yes | Developer lists unread activity | subprocess `passuff inbox` | yes |
| activity-inbox: mark read | yes | Developer marks activity read | subprocess `passuff read <id>` | yes |
| activity-inbox: open the TUI | yes | Developer opens the inbox TUI | `passuff-tui --observe --snapshot`; a pty smoke is promised for this row but is not in this scenario (it exists in the regular scenario `The developer starts the inbox TUI binary`) | **partial** (F13) |
| activity-inbox: open a detail | yes | Developer opens an activity's details (delta) | `--snapshot --keys "enter"`; asserts the comment body | yes |
| activity-inbox: open the pull request | yes | Developer opens the pull request from an activity | `--snapshot --keys "enter,enter"` + fake `xdg-open` log | yes |
| inbox-navigation: navigate panes | yes | Developer navigates the inbox panes | `--snapshot --keys "tab,j"` | yes |
| inbox-navigation: live refresh | yes | The inbox shows activity observed while it is open | `--observe-for` + injected comment | yes |
| stacked-inbox: review a stack | yes | Developer reviews a stacked inbox | `--snapshot` with fake stack metadata | yes |
| initial-setup: first run | yes | Developer sets up passuff on first run | subprocess `passuff setup` + TUI `--snapshot` | yes |
| pr-activity: start watching | yes | Developer starts watching and is notified of a new comment | background `passuff` + fake D-Bus/sound | yes |
| pr-activity: one check | yes | Developer runs a single check and sees the current activity | subprocess `passuff check` | yes |
| pr-activity: usage | yes | Developer asks for usage | subprocess `passuff --help` | yes |
| pr-activity: version | yes | Developer asks for the version | subprocess `passuff --version` | yes |
| pr-activity: notification test | yes | The developer tests notifications on the command line | subprocess `passuff notify-test` | yes |

Every inventory row has a matrix row and one `@e2e` scenario. The one driving
mechanism that is promised but absent from its scenario is recorded as F13.

### 11. Local run command

`python -m passuff --help` (the declared `run.command`) runs and exits 0 in this
tree. The full stack with the digital twins is started by `./scripts/verify.sh`
(one twin set per scenario: fake GitHub on loopback, private D-Bus, fake `gh`,
`paplay`, `xdg-open`, per-scenario XDG dirs). The suite was not re-run: the
verification receipt is the execution evidence.

### 12. Additional evidence findings from this check

- **F6 — the 80-character list-row excerpt is not implemented and its scenario
  does not assert it.** T7
  (`givn/specs/review-activity-inbox/usecase.md:54`) requires the activity list
  row to show at most the recorded 80-character event excerpt. The TUI renders
  the fixed summary only (`passuff/tui/frame.py:154-160`), no truncation exists
  anywhere in `passuff/`, and the delta step
  `the activity list row excerpt is at most 80 characters`
  (`givn/steps/common.py:1758-1767`) checks `len(line) <= 100` on any line
  containing “comment by” — a line that is not the list row and a bound that is
  not 80. The delta scenario
  `The activity list row is bounded to the recorded excerpt` therefore passes
  without exercising the rule it names.
- **F7 — the 4 KiB body bound is not implemented.** `design.md` (“Watch state”)
  and T7 bound the stored comment/review body at 4 KiB. `passuff/classify.py:53`
  stores the body unbounded and no scenario covers the bound.
- **F8 — macOS sound is not implemented.** `README.md:55`,
  `docs/reference.md:288`, `docs/arc42/03-context-and-scope.md:42`, and
  `docs/arc42/07-deployment-view.md:30` state macOS sound uses `afplay`;
  `passuff/sound.py:9` lists only `paplay`, `pw-play`, `aplay`,
  `canberra-gtk-play`, so macOS sound fails with “sound unavailable”.

## Arc42 implementation conformance

Affected chapters were re-derived from the implementation, `design.md`, and
`tasks.md` before accepting `arc42.md`. Rows 2, 3, 9, and 10 were re-derived as
unaffected and their files carry current content.

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Six use cases, including `set-up-passuff` | `01-introduction-and-goals.md:34-41` | Affected: corrected to six | six use cases in Use-Case Context | tasks route all seven capabilities | six `givn/specs/*/usecase.md` files | yes |
| Gherkin runner `behave`; the runner contract | `04-solution-strategy.md:60`, `08-crosscutting-concepts.md:227` | Affected: replaced the custom runner | V2 `behave` | setup task | `behave.ini`, `scripts/verify.sh`, `givn/environment.py` | yes |
| Terminal UI: `curses` + pure frame renderer | `04-solution-strategy.md:77` | Affected: replaced the TUI library | V3 | TUI tasks | `passuff/tui/app.py`, `frame.py` | yes |
| Separate E2E step tree with bindings per capability | `arc42.md` ch4/ch8 claim; `05-building-block-view.md:74,164,180` | Affected: separate E2E steps, component map unchanged | Step Definitions table names seven regular files and per-capability E2E files | tasks name the same files | one `common.py`; E2E per-capability files are docstring placeholders | **no** (F2) |
| `ObservationSession` owns probe/reconcile scheduling and drives notifications | `05-building-block-view.md:51,74`, `06-runtime-view.md:299`, `08-crosscutting-concepts.md:129`, `glossary.md` “Terminal UI” | Affected: corrected; component map unchanged | contract `ObservationSchedule` → `passuff.tui.session.ObservationSession` | TUI tasks name `passuff/tui/session.py` | no such module or class; `_observe`/`_interactive` in `app.py:338,354` | **no** (F3) |
| `StackGrouper` groups stacks for the read models | `05-building-block-view.md:70,164` | Affected: component map unchanged | contract `StackedInbox` → `StackGrouper` | stacked-inbox tasks name `passuff/stacking.py` | `StackGrouper` exists but is never imported; the TUI groups inline (`frame.py:141-147`) | **no** (F5) |
| `RecordingDelivery` under `tests/support/recording_delivery.py`, macOS adapter tests | `05-building-block-view.md:77,180`, `04-solution-strategy.md:76`, `08-crosscutting-concepts.md:235`, `10-quality-requirements.md:44` | Affected: test double moved out of production | module layout keeps it under `tests/support/`; Q10 | not scheduled as a file in any task | file absent; a `recording_path` hook sits in production (`notify.py:17,27-30`) with no caller | **no** (F5) |
| List row shows at most the 80-character excerpt; detail body bounded at 4 KiB | `05`/`08` claim; `glossary.md` “Activity detail” | Affected: activity-detail body corrected | “the list row keeps the 80-character excerpt (T7)” | delta task 151 | no truncation in `passuff/`; row shows the fixed summary | **no** (F6, F7) |
| macOS audio player `afplay` | `03-context-and-scope.md:42`, `07-deployment-view.md:30` | ch3/ch7 unaffected | macOS `afplay` in the local stack | setup/e2e tasks name fake `afplay` | `sound.py:9` omits `afplay`; no fake `afplay` | **no** (F8) |
| `Passuff.app` deferred 2026-11-08; TD-007 | `07-deployment-view.md:58-61`, `11-risks-and-technical-debt.md:53` | Affected: dated deferral and TD-007 | deferred to a packaging change | — | no bundle expected; the rest of the Python package ships | yes |
| TOML writes with `tomlkit`, atomic replace | `06-runtime-view.md:255`, `08-crosscutting-concepts.md:144` | Affected | `TomlConfigStore` | scope tasks | `passuff/config.py`, `passuff/scope.py` | yes |
| Typed configuration rejects unknown keys with exit 2 | `08-crosscutting-concepts.md` | Affected | Data model: unknown keys rejected | config tasks | `passuff/config.py`; delta scenario `An invalid configuration is reported with exit 2` | yes |
| TD-005 resolved by this change | `11-risks-and-technical-debt.md:51` | Affected: “recorded TD-005 as planned” | coverage scripts land in this change | setup task | `scripts/merge-coverages.sh` uses `coverage combine` + `coverage xml`; no custom merger remains | **no** — TD-005 is still written as an unresolved plan (F10) |

ARC42 CONFORMANCE: FINDINGS

The durable chapters 4, 5, 8, and 11 describe components, a step layout, an
excerpt rule, a macOS audio path, and a debt item that the implementation does
not have or no longer matches. Either the implementation is completed to the
reviewed documents or the documents are corrected in the same change; a silent
divergence is not acceptable.

## Ubiquitous language conformance

The glossary is the single vocabulary authority and the specs/design reuse its
terms. Two entries name code symbols that do not exist:

- `Inbox` — `docs/glossary.md:59` names `passuff.status.WatchStatusReader`;
  there is no `passuff.status` module (the class is `passuff.inbox.WatchStatusReader`).
- `Terminal UI` and `Watcher` — `docs/glossary.md:62,67` name
  `ObservationSession`; no such class exists.

Both are finding **F4**/**F3** residues of the same divergence. The structural
model (`docs/domain-model.md`) records no structural change for this rebuild
and its boundaries/relationships match the implemented aggregates; no edit is
required there, and none was made.

## Use-case and Persona conformance

Six use cases are the owning contracts and each was checked against the
rejection criteria: `watch-pull-requests` (goal: be notified of others' activity
on authored pull requests; primary actor developer; trigger names the
watcher/check), `inspect-watch-status`, `review-activity-inbox`,
`review-pull-request-stacks`, `change-watch-scope`, `set-up-passuff`. No `Goal`
joins two verb phrases, every `Trigger` names the primary actor, extensions
carry their main-flow step number, preconditions are not re-established by the
flow, and no goal/trigger/guarantee names a mechanism. Capability names are
goal-shaped. No leaf's main flow names three or more actor goals.

Persona disposition:

| Persona | Confirmed | Outcome in this change |
|---|---|---|
| `multi-account-developer` (`givn/personas/multi-account-developer.md`) | yes, 2026-11-02 | Every user-visible capability the Persona uses is rebuilt unchanged: the watcher transcript and notification, status, inbox/read, the terminal inbox with details and stacks, scope editing, and first-run setup. The rebuild restores the outcomes; it adds none. The terminal transcripts under `evidence/visual/` show the Persona reaching the inbox, activity detail, stack, scope picker, and status goals. |

No other Persona is invented or promoted. Actors in the specs remain
interaction roles (`developer`), not Personas.

Use-case conformance is preserved. The functional gaps F6 (list-row excerpt),
F7 (body bound), and F8 (macOS sound) are implementation failures against the
owning rules, not use-case defects.

## Coverage

Measured evidence: `coverage/cobertura-coverage.xml` (merged, lines 1845/2191,
branches 649/842), `coverage/non-e2e-cobertura.xml` (1838/2191, 644/842), and
`coverage/e2e-cobertura.xml` (1376/2191, 360/842). The three reports are
distinct files with distinct measurement timestamps and counts; the E2E report
is not a copy of the non-E2E report. Branch instrumentation is present.

Every gap is classified in the three allowed buckets.

### Bucket 1 — dead code (delete)

| Region | Evidence | Fix |
|---|---|---|
| `passuff/stacking.py` (all 21 lines, 0%) | `StackGrouper` has no importer anywhere in `passuff/`, `tests/`, `givn/`; the TUI groups stacks inline | delete the module, or make `StackedInbox` → `StackGrouper` the TUI's only grouping path and test it |
| `passuff/composition.py` (all 6 lines, 0%) | `openers()` has no caller; `cli.py` is the composition root | delete, or route the CLI/TUI adapter construction through it |
| `passuff/render.py:99-121` `render_activity_detail` | no caller; `passuff/tui/frame.py` renders the detail | delete, or use it from the TUI |
| `passuff/notify.py:17,27-30` `recording_path` | never passed by any caller; a test hook in production code | delete, and add the promised `tests/support/recording_delivery.py` |

### Bucket 2 — missing test coverage (add tests, then implement/refactor)

| Region | Missed lines / branches | Gap |
|---|---|---|
| `passuff/config.py` | 43 lines, 28 branches | the validation branches (unknown key, bad intervals, watch table/array, duplicate repository, ignored entry, read errors) — the delta outline covers four of them; the rest are unexercised |
| `passuff/setup.py` | 14 lines, 14 branches | prompt validation (empty label, unknown auth mode, empty env var, token command/file rules, duplicate label, watch mode) |
| `passuff/credentials.py` | 8 lines, 4 branches | gh run failure, credential-command failure/empty output, non-regular token file, non-private token file, “exactly one source” |
| `passuff/github.py` | 19 lines, 13 branches | pending/build CI mapping, review-requested relation, transport/malformed-JSON/no-data error mapping, stack-rejection message |
| `passuff/state.py` | 23 lines, 10 branches | quarantine failure, duplicate-id repair, unknown-version/failed-load paths, save failure paths |
| `passuff/classify.py` | 8 lines, 7 branches | review-comment events, CI recovery without notification, own-action early exits |
| `passuff/inbox.py` | 6 lines, 2 branches | unparsable time fallbacks, unknown key |
| `passuff/scope.py` | 3 lines, 4 branches | all-mode deselect, invalid mode, invalid focus relation |
| `passuff/core.py` | 14 lines, 12 branches | target-selection guards, discovery failure fallback, per-account failure reporting |
| `passuff/render.py:59` | 1 line | digest truncation when more than five pull requests are grouped |
| `passuff/tui/app.py` | 122 lines, 40 branches | scroll/home/end/picker-cursor handlers, filter clearing, detail scroll keys |
| `passuff/tui/frame.py` | 18 lines, 17 branches | review/review-comment labels, CI-recovered label, assignee-only rows, empty-state branches |

### Bucket 3 — legitimately hard to test (concrete technical justification)

| Region | Justification |
|---|---|
| `passuff/notify.py:62-69` `_send_macos` — the real `osascript` process call | executing the real macOS notification surface requires a macOS host and a logged-in Notification Center session; the suite runs on Linux. The surface itself is covered by the twin on Linux; the real macOS call is TD-001's open item. |
| `passuff/browser.py:15` — the `darwin` `open` branch | the same macOS-only process call; on Linux the `xdg-open` branch runs and the fake opener records it. |

Not bucket 3, and recorded as missing test coverage instead: the macOS
notification **double** is a promised boundary that would make the recording
path testable, so its absence stays bucket 2 (F5). External-input validation
and every error path above are bucket 2, not “defensive code”.

### E2E coverage

The canonical E2E policy was read from `givn instructions specs`. The interface
is a terminal (CLI subprocess + curses TUI). Every normalized inventory action
has exactly one `@e2e` scenario, each primary assertion is on the transcript or
the rendered frame, and the real drivers are subprocesses, `--snapshot`, a pty
smoke, the fake D-Bus service, the fake sound player, and the fake opener. No
action is over-covered. No `@e2e` scenario was downgraded. The E2E count (19)
is strictly less than the change count (229), and the scope is explicit.

### Redundant unit tests

There is no separate unit-test tree: the `.feature` corpus is the only test
surface, so no redundant unit test exists to remove.

## Visual review

The design declares `Interface: terminal` (`design.md` “Visual Design
Contract”) with viewports 80x24 / 120x40 and a transcript per `@e2e` scenario.
The committed transcripts under
`givn/changes/python-rebuild/evidence/visual/<scenario-slug>/transcript.txt`
were read during this review:

| Scenario slug | State seen |
|---|---|
| `developer-toggles-a-repository-in-the-watch-scope` | scope picker, “watching nothing” empty state |
| `developer-switches-the-watch-scope-mode` | picker, `Mode: selected`, focus marks |
| `developer-narrows-the-focus-in-the-watch-scope` | picker with focus marks and the watched row |
| `developer-sees-the-watch-state` | status transcript |
| `developer-lists-unread-activity` | inbox list row with id, summary, key, time, read state |
| `developer-marks-activity-read` | empty inbox after the mark |
| `developer-opens-the-inbox-tui` | live-observe frame |
| `developer-opens-an-activity-s-details` | activity detail overlay with the comment body and `Enter open in browser` |
| `developer-opens-the-pull-request-from-an-activity` | frame before the open intent |
| `developer-navigates-the-inbox-panes` | two-pane frame with focus on PRs |
| `the-inbox-shows-activity-observed-while-it-is-open` | frame with the injected activity |
| `developer-reviews-a-stacked-inbox` | stack positions `1/2`, `2/2`, base |
| `developer-sets-up-passuff-on-first-run` | empty inbox after setup |
| `developer-runs-a-single-check-and-sees-the-current-activity` | check transcript |
| `developer-asks-for-usage` / `developer-asks-for-the-version` | CLI output |
| `the-developer-tests-notifications-on-the-command-line` | both surfaces reported |

Persona judgment (`multi-account-developer`): the transcripts show the label,
resolved login, watch scope, unread count, per-row key/summary/time/read state,
stack position and base, the activity detail body, and the browser affordance.
The Persona can reach the inbox, detail, stack, scope, and status goals with
what is shown. No finding names a failed Persona goal in the transcripts that
exist.

The evidence is **incomplete**: the `@e2e` scenario
`Developer starts watching and is notified of a new comment` has no committed
transcript. `tests/support/harness.py:52` writes a transcript only when
`world.last` is set, and `World.start_watch` (`tests/support/world.py:524`)
leaves it `None`, so the scenario that proves the Persona's primary outcome
(start the watcher, receive the notification) has no visual evidence. The
design's contract says every `@e2e` scenario commits one.

VISUAL-REVIEW: UNVERIFIED

A human must check: (1) start the watcher with the fake GitHub, D-Bus, and
sound twins and confirm the terminal transcript shows “New comment from
@alice”, the notification, and the single sound for the Persona's primary
outcome; (2) confirm the missing transcript is committed for
`developer-starts-watching-and-is-notified-of-a-new-comment` so the evidence
set covers 19 of 19 `@e2e` runs.

## Scenario execution evidence

Receipt status: proven. `givn check review` reports
`change verification receipt passed (229 scenario(s) at a3a717ef0a15ab538ea4c888a15e184eef03b673)`;
the receipt `1c84c428e423afded5dc60cfae9a55253cd24f22` records change 229/229,
regular 197/197, E2E 19/19, with zero failed and zero skipped. Review does not
re-run the suite.

## Overlap dispositions

One row per `[SUBST]` and `[SHAPE]` finding reported by
`givn lint --change python-rebuild` involving this change's delta (614 pairs).
Decision rule: `duplicate` only when the same rule and the same Given/When
path produce the same assertion and observable state; `boundary` when the two
scenarios are the minimum/maximum or inside/outside cases of one rule;
`variant` otherwise — a distinct rule or distinct invariant that shares step
vocabulary. No pair meets the duplicate test: every shared Then is the generic
`the exit status is <n>` or a shared transcript phrase whose Given/When path
differs. The finding kind ([SUBST] or [SHAPE]) is recorded in the lint output;
the disposition is per scenario pair.

| Scenario A | Scenario B | Decision |
|---|---|---|
| A CI recovery across pending notifies | A CLI usage error exits 2 | `variant` |
| A CI recovery across pending notifies | A GitHub response failure is an account failure | `variant` |
| A CI recovery across pending notifies | A delivery surface failure keeps the transcript | `variant` |
| A CI recovery across pending notifies | A missing configuration is a configuration error | `variant` |
| A CI recovery across pending notifies | A missing or failing gh is an account failure | `variant` |
| A CI recovery across pending notifies | A quarantine failure is a state error | `variant` |
| A CI recovery across pending notifies | An invalid configuration is reported with exit 2 | `variant` |
| A CI recovery across pending notifies | An unknown state version is quarantined and replaced | `variant` |
| A CI recovery across pending notifies | Deselecting a repository in all mode is refused | `variant` |
| A CI recovery across pending notifies | Duplicate watch scope entries are rejected | `variant` |
| A CI recovery notifies | A CLI usage error exits 2 | `variant` |
| A CI recovery notifies | A GitHub response failure is an account failure | `variant` |
| A CI recovery notifies | A delivery surface failure keeps the transcript | `variant` |
| A CI recovery notifies | A missing configuration is a configuration error | `variant` |
| A CI recovery notifies | A missing or failing gh is an account failure | `variant` |
| A CI recovery notifies | A quarantine failure is a state error | `variant` |
| A CI recovery notifies | An invalid configuration is reported with exit 2 | `variant` |
| A CI recovery notifies | An unknown state version is quarantined and replaced | `variant` |
| A CI recovery notifies | Deselecting a repository in all mode is refused | `variant` |
| A CI recovery notifies | Duplicate watch scope entries are rejected | `variant` |
| A CLI usage error exits 2 | A GitHub response failure is an account failure | `variant` |
| A CLI usage error exits 2 | A delivery surface failure keeps the transcript | `variant` |
| A CLI usage error exits 2 | A disabled desktop notification keeps the transcript | `variant` |
| A CLI usage error exits 2 | A failed account does not stop the other account | `variant` |
| A CLI usage error exits 2 | A missing or failing gh is an account failure | `variant` |
| A CLI usage error exits 2 | A missing sound player still delivers the notification | `variant` |
| A CLI usage error exits 2 | A missing token file is an account failure | `variant` |
| A CLI usage error exits 2 | A one-shot check never starts a browser login | `variant` |
| A CLI usage error exits 2 | A personal access token for a different login is an account failure | `variant` |
| A CLI usage error exits 2 | An invalid configuration is reported with exit 2 | `variant` |
| A CLI usage error exits 2 | An unavailable desktop surface keeps the transcript | `variant` |
| A CLI usage error exits 2 | An unknown state version is quarantined and replaced | `variant` |
| A CLI usage error exits 2 | Developer lists unread activity | `variant` |
| A CLI usage error exits 2 | Developer runs a single check and sees the current activity | `variant` |
| A CLI usage error exits 2 | Developer sees the watch state | `variant` |
| A CLI usage error exits 2 | Duplicate watch scope entries are rejected | `variant` |
| A CLI usage error exits 2 | Inbox with no state shows no activity | `variant` |
| A CLI usage error exits 2 | Status with no state shows empty sections | `variant` |
| A CLI usage error exits 2 | The developer tests notifications | `variant` |
| A CLI usage error exits 2 | The developer tests notifications on the command line | `variant` |
| A CLI usage error exits 2 | The notification test can mute the desktop notification | `variant` |
| A CLI usage error exits 2 | The notification test can mute the sound | `variant` |
| A CLI usage error exits 2 | The notification test reports an unavailable surface | `variant` |
| A CLI usage error exits 2 | The notification test reports disabled surfaces | `variant` |
| A GitHub response failure is an account failure | A personal access token for a different login is an account failure | `variant` |
| A delivery surface failure keeps the transcript | A missing sound player still delivers the notification | `variant` |
| A delivery surface failure keeps the transcript | The developer tests notifications | `variant` |
| A deselected repository stops observation | A rejected stack query retries once and shows standalone rows | `variant` |
| A deselected repository stops observation | The activity list row is bounded to the recorded excerpt | `variant` |
| A failed account does not stop the other account | A GitHub response failure is an account failure | `variant` |
| A failed browser launch keeps the activity detail open | A missing browser opener is reported and keeps the detail | `variant` |
| A failed browser launch keeps the activity detail open | Developer opens an activity | `variant` |
| A failed browser login does not stop the other account | A GitHub response failure is an account failure | `variant` |
| A failed observation keeps the change pending | A CLI usage error exits 2 | `variant` |
| A failed observation keeps the change pending | A GitHub response failure is an account failure | `variant` |
| A failed observation keeps the change pending | A delivery surface failure keeps the transcript | `variant` |
| A failed observation keeps the change pending | A missing configuration is a configuration error | `variant` |
| A failed observation keeps the change pending | A missing or failing gh is an account failure | `variant` |
| A failed observation keeps the change pending | A quarantine failure is a state error | `variant` |
| A failed observation keeps the change pending | An invalid configuration is reported with exit 2 | `variant` |
| A failed observation keeps the change pending | An unknown state version is quarantined and replaced | `variant` |
| A failed observation keeps the change pending | Deselecting a repository in all mode is refused | `variant` |
| A failed observation keeps the change pending | Duplicate watch scope entries are rejected | `variant` |
| A failing CI result notifies | A CLI usage error exits 2 | `variant` |
| A failing CI result notifies | A GitHub response failure is an account failure | `variant` |
| A failing CI result notifies | A delivery surface failure keeps the transcript | `variant` |
| A failing CI result notifies | A missing configuration is a configuration error | `variant` |
| A failing CI result notifies | A missing or failing gh is an account failure | `variant` |
| A failing CI result notifies | A quarantine failure is a state error | `variant` |
| A failing CI result notifies | An invalid configuration is reported with exit 2 | `variant` |
| A failing CI result notifies | An unknown state version is quarantined and replaced | `variant` |
| A failing CI result notifies | Deselecting a repository in all mode is refused | `variant` |
| A failing CI result notifies | Duplicate watch scope entries are rejected | `variant` |
| A gh OAuth account uses the existing gh session | A GitHub response failure is an account failure | `variant` |
| A gh OAuth account uses the existing gh session | A reopen within the retention window is observed | `variant` |
| A gh OAuth account uses the existing gh session | A replayed pull request node does not duplicate activity | `variant` |
| A legacy command credential remains a personal access token account | A GitHub response failure is an account failure | `variant` |
| A legacy command credential remains a personal access token account | A reopen within the retention window is observed | `variant` |
| A legacy command credential remains a personal access token account | A replayed pull request node does not duplicate activity | `variant` |
| A legacy environment credential remains a personal access token account | A GitHub response failure is an account failure | `variant` |
| A legacy environment credential remains a personal access token account | A reopen within the retention window is observed | `variant` |
| A legacy environment credential remains a personal access token account | A replayed pull request node does not duplicate activity | `variant` |
| A long pull request key keeps its number visible | A missing browser opener is reported and keeps the detail | `variant` |
| A long pull request key keeps its number visible | A rejected stack query retries once and shows standalone rows | `variant` |
| A long pull request key keeps its number visible | Developer opens an activity | `variant` |
| A long pull request key keeps its number visible | The activity list row is bounded to the recorded excerpt | `variant` |
| A merge performed by the configured account still notifies | A CLI usage error exits 2 | `variant` |
| A merge performed by the configured account still notifies | A GitHub response failure is an account failure | `variant` |
| A merge performed by the configured account still notifies | A delivery surface failure keeps the transcript | `variant` |
| A merge performed by the configured account still notifies | A missing configuration is a configuration error | `variant` |
| A merge performed by the configured account still notifies | A missing or failing gh is an account failure | `variant` |
| A merge performed by the configured account still notifies | A quarantine failure is a state error | `variant` |
| A merge performed by the configured account still notifies | An invalid configuration is reported with exit 2 | `variant` |
| A merge performed by the configured account still notifies | An unknown state version is quarantined and replaced | `variant` |
| A merge performed by the configured account still notifies | Deselecting a repository in all mode is refused | `variant` |
| A merge performed by the configured account still notifies | Duplicate watch scope entries are rejected | `variant` |
| A merged stack member remains in the group | Developer opens an activity | `variant` |
| A missing browser opener is reported and keeps the detail | Developer opens an activity | `variant` |
| A missing browser opener is reported and keeps the detail | Developer reviews a stacked inbox | `variant` |
| A missing configuration is a configuration error | A CLI usage error exits 2 | `variant` |
| A missing configuration is a configuration error | A GitHub response failure is an account failure | `variant` |
| A missing configuration is a configuration error | A delivery surface failure keeps the transcript | `variant` |
| A missing configuration is a configuration error | A disabled desktop notification keeps the transcript | `variant` |
| A missing configuration is a configuration error | A failed account does not stop the other account | `variant` |
| A missing configuration is a configuration error | A missing or failing gh is an account failure | `boundary` |
| A missing configuration is a configuration error | A missing sound player still delivers the notification | `variant` |
| A missing configuration is a configuration error | A missing token file is an account failure | `boundary` |
| A missing configuration is a configuration error | A one-shot check never starts a browser login | `boundary` |
| A missing configuration is a configuration error | A personal access token for a different login is an account failure | `variant` |
| A missing configuration is a configuration error | A quarantine failure is a state error | `boundary` |
| A missing configuration is a configuration error | An invalid configuration is reported with exit 2 | `boundary` |
| A missing configuration is a configuration error | An unavailable desktop surface keeps the transcript | `variant` |
| A missing configuration is a configuration error | An unknown state version is quarantined and replaced | `boundary` |
| A missing configuration is a configuration error | Developer lists unread activity | `variant` |
| A missing configuration is a configuration error | Developer runs a single check and sees the current activity | `variant` |
| A missing configuration is a configuration error | Developer sees the watch state | `variant` |
| A missing configuration is a configuration error | Duplicate watch scope entries are rejected | `boundary` |
| A missing configuration is a configuration error | Inbox with no state shows no activity | `variant` |
| A missing configuration is a configuration error | Status with no state shows empty sections | `variant` |
| A missing configuration is a configuration error | The developer tests notifications | `variant` |
| A missing configuration is a configuration error | The developer tests notifications on the command line | `variant` |
| A missing configuration is a configuration error | The notification test can mute the desktop notification | `variant` |
| A missing configuration is a configuration error | The notification test can mute the sound | `variant` |
| A missing configuration is a configuration error | The notification test reports an unavailable surface | `variant` |
| A missing configuration is a configuration error | The notification test reports disabled surfaces | `variant` |
| A missing or failing gh is an account failure | A GitHub response failure is an account failure | `variant` |
| A missing or failing gh is an account failure | A delivery surface failure keeps the transcript | `variant` |
| A missing or failing gh is an account failure | A disabled desktop notification keeps the transcript | `variant` |
| A missing or failing gh is an account failure | A failed account does not stop the other account | `variant` |
| A missing or failing gh is an account failure | A missing sound player still delivers the notification | `variant` |
| A missing or failing gh is an account failure | A missing token file is an account failure | `boundary` |
| A missing or failing gh is an account failure | A one-shot check never starts a browser login | `boundary` |
| A missing or failing gh is an account failure | A personal access token for a different login is an account failure | `variant` |
| A missing or failing gh is an account failure | An invalid configuration is reported with exit 2 | `boundary` |
| A missing or failing gh is an account failure | An unavailable desktop surface keeps the transcript | `variant` |
| A missing or failing gh is an account failure | An unknown state version is quarantined and replaced | `boundary` |
| A missing or failing gh is an account failure | Developer lists unread activity | `variant` |
| A missing or failing gh is an account failure | Developer runs a single check and sees the current activity | `variant` |
| A missing or failing gh is an account failure | Developer sees the watch state | `variant` |
| A missing or failing gh is an account failure | Duplicate watch scope entries are rejected | `boundary` |
| A missing or failing gh is an account failure | Inbox with no state shows no activity | `variant` |
| A missing or failing gh is an account failure | Status with no state shows empty sections | `variant` |
| A missing or failing gh is an account failure | The developer tests notifications | `variant` |
| A missing or failing gh is an account failure | The developer tests notifications on the command line | `variant` |
| A missing or failing gh is an account failure | The notification test can mute the desktop notification | `variant` |
| A missing or failing gh is an account failure | The notification test can mute the sound | `variant` |
| A missing or failing gh is an account failure | The notification test reports an unavailable surface | `variant` |
| A missing or failing gh is an account failure | The notification test reports disabled surfaces | `variant` |
| A missing token file is an account failure | A delivery surface failure keeps the transcript | `variant` |
| A missing token file is an account failure | An invalid configuration is reported with exit 2 | `boundary` |
| A missing token file is an account failure | Duplicate watch scope entries are rejected | `boundary` |
| A personal access token whose login differs only in case is accepted | A GitHub response failure is an account failure | `variant` |
| A personal access token whose login differs only in case is accepted | A reopen within the retention window is observed | `variant` |
| A personal access token whose login differs only in case is accepted | A replayed pull request node does not duplicate activity | `variant` |
| A pull request I did not author is watched | A GitHub response failure is an account failure | `variant` |
| A pull request I did not author is watched | A reopen within the retention window is observed | `variant` |
| A pull request I did not author is watched | A replayed pull request node does not duplicate activity | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A CLI usage error exits 2 | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A GitHub response failure is an account failure | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A delivery surface failure keeps the transcript | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A missing configuration is a configuration error | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A missing or failing gh is an account failure | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A quarantine failure is a state error | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | An invalid configuration is reported with exit 2 | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | An unknown state version is quarantined and replaced | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | Deselecting a repository in all mode is refused | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | Duplicate watch scope entries are rejected | `variant` |
| A pull request without stack metadata stays standalone | A rejected stack query retries once and shows standalone rows | `boundary` |
| A pull request without stack metadata stays standalone | The activity list row is bounded to the recorded excerpt | `variant` |
| A quarantine failure is a state error | A CLI usage error exits 2 | `variant` |
| A quarantine failure is a state error | A GitHub response failure is an account failure | `variant` |
| A quarantine failure is a state error | A delivery surface failure keeps the transcript | `variant` |
| A quarantine failure is a state error | A disabled desktop notification keeps the transcript | `variant` |
| A quarantine failure is a state error | A failed account does not stop the other account | `variant` |
| A quarantine failure is a state error | A missing or failing gh is an account failure | `boundary` |
| A quarantine failure is a state error | A missing sound player still delivers the notification | `variant` |
| A quarantine failure is a state error | A missing token file is an account failure | `boundary` |
| A quarantine failure is a state error | A one-shot check never starts a browser login | `boundary` |
| A quarantine failure is a state error | A personal access token for a different login is an account failure | `variant` |
| A quarantine failure is a state error | An invalid configuration is reported with exit 2 | `boundary` |
| A quarantine failure is a state error | An unavailable desktop surface keeps the transcript | `variant` |
| A quarantine failure is a state error | An unknown state version is quarantined and replaced | `boundary` |
| A quarantine failure is a state error | Developer lists unread activity | `variant` |
| A quarantine failure is a state error | Developer runs a single check and sees the current activity | `variant` |
| A quarantine failure is a state error | Developer sees the watch state | `variant` |
| A quarantine failure is a state error | Duplicate watch scope entries are rejected | `boundary` |
| A quarantine failure is a state error | Inbox with no state shows no activity | `variant` |
| A quarantine failure is a state error | Status with no state shows empty sections | `variant` |
| A quarantine failure is a state error | The developer tests notifications | `variant` |
| A quarantine failure is a state error | The developer tests notifications on the command line | `variant` |
| A quarantine failure is a state error | The notification test can mute the desktop notification | `variant` |
| A quarantine failure is a state error | The notification test can mute the sound | `variant` |
| A quarantine failure is a state error | The notification test reports an unavailable surface | `variant` |
| A quarantine failure is a state error | The notification test reports disabled surfaces | `variant` |
| A read command rejects invalid configuration | A CLI usage error exits 2 | `variant` |
| A read command rejects invalid configuration | A GitHub response failure is an account failure | `variant` |
| A read command rejects invalid configuration | A delivery surface failure keeps the transcript | `variant` |
| A read command rejects invalid configuration | A missing configuration is a configuration error | `variant` |
| A read command rejects invalid configuration | A missing or failing gh is an account failure | `variant` |
| A read command rejects invalid configuration | A quarantine failure is a state error | `variant` |
| A read command rejects invalid configuration | An invalid configuration is reported with exit 2 | `variant` |
| A read command rejects invalid configuration | An unknown state version is quarantined and replaced | `variant` |
| A read command rejects invalid configuration | Deselecting a repository in all mode is refused | `variant` |
| A read command rejects invalid configuration | Duplicate watch scope entries are rejected | `variant` |
| A reload failure keeps the previous scope | A CLI usage error exits 2 | `variant` |
| A reload failure keeps the previous scope | A GitHub response failure is an account failure | `variant` |
| A reload failure keeps the previous scope | A delivery surface failure keeps the transcript | `variant` |
| A reload failure keeps the previous scope | A missing configuration is a configuration error | `variant` |
| A reload failure keeps the previous scope | A missing or failing gh is an account failure | `variant` |
| A reload failure keeps the previous scope | A quarantine failure is a state error | `variant` |
| A reload failure keeps the previous scope | An invalid configuration is reported with exit 2 | `variant` |
| A reload failure keeps the previous scope | An unknown state version is quarantined and replaced | `variant` |
| A reload failure keeps the previous scope | Deselecting a repository in all mode is refused | `variant` |
| A reload failure keeps the previous scope | Duplicate watch scope entries are rejected | `variant` |
| A reopen after the retention window is not observed | A personal access token for a different login is an account failure | `variant` |
| A reopen within the retention window is observed | A GitHub response failure is an account failure | `variant` |
| A reopen within the retention window is observed | A failed account does not stop the other account | `variant` |
| A reopen within the retention window is observed | A failed browser login does not stop the other account | `variant` |
| A reopen within the retention window is observed | A personal access token for a different login is an account failure | `variant` |
| A reopen within the retention window is observed | A pull request opened after the last check is an activity | `variant` |
| A reopen within the retention window is observed | A replayed pull request node does not duplicate activity | `variant` |
| A reopen within the retention window is observed | A short-lived pull request merged or closed between checks notifies | `variant` |
| A reopen within the retention window is observed | Each configured account is observed | `variant` |
| A review comment from another person notifies | A CLI usage error exits 2 | `variant` |
| A review comment from another person notifies | A GitHub response failure is an account failure | `variant` |
| A review comment from another person notifies | A delivery surface failure keeps the transcript | `variant` |
| A review comment from another person notifies | A missing configuration is a configuration error | `variant` |
| A review comment from another person notifies | A missing or failing gh is an account failure | `variant` |
| A review comment from another person notifies | A quarantine failure is a state error | `variant` |
| A review comment from another person notifies | An invalid configuration is reported with exit 2 | `variant` |
| A review comment from another person notifies | An unknown state version is quarantined and replaced | `variant` |
| A review comment from another person notifies | Deselecting a repository in all mode is refused | `variant` |
| A review comment from another person notifies | Duplicate watch scope entries are rejected | `variant` |
| A stack is marked with a symbol and its position | A missing browser opener is reported and keeps the detail | `variant` |
| A stack is marked with a symbol and its position | Developer opens an activity | `variant` |
| A standalone pull request shows no stack symbol | A rejected stack query retries once and shows standalone rows | `variant` |
| A standalone pull request shows no stack symbol | The activity list row is bounded to the recorded excerpt | `variant` |
| A state write failure is reported | A CLI usage error exits 2 | `variant` |
| A state write failure is reported | A GitHub response failure is an account failure | `variant` |
| A state write failure is reported | A delivery surface failure keeps the transcript | `variant` |
| A state write failure is reported | A missing configuration is a configuration error | `variant` |
| A state write failure is reported | A missing or failing gh is an account failure | `variant` |
| A state write failure is reported | A quarantine failure is a state error | `variant` |
| A state write failure is reported | An invalid configuration is reported with exit 2 | `variant` |
| A state write failure is reported | An unknown state version is quarantined and replaced | `variant` |
| A state write failure is reported | Deselecting a repository in all mode is refused | `variant` |
| A state write failure is reported | Duplicate watch scope entries are rejected | `variant` |
| A state write failure is reported with a nonzero status | A CLI usage error exits 2 | `variant` |
| A state write failure is reported with a nonzero status | A GitHub response failure is an account failure | `variant` |
| A state write failure is reported with a nonzero status | A delivery surface failure keeps the transcript | `variant` |
| A state write failure is reported with a nonzero status | A missing configuration is a configuration error | `boundary` |
| A state write failure is reported with a nonzero status | A missing or failing gh is an account failure | `boundary` |
| A state write failure is reported with a nonzero status | A quarantine failure is a state error | `variant` |
| A state write failure is reported with a nonzero status | An invalid configuration is reported with exit 2 | `boundary` |
| A state write failure is reported with a nonzero status | An unknown state version is quarantined and replaced | `boundary` |
| A state write failure is reported with a nonzero status | Deselecting a repository in all mode is refused | `variant` |
| A state write failure is reported with a nonzero status | Duplicate watch scope entries are rejected | `boundary` |
| A submitted review notifies | A CLI usage error exits 2 | `variant` |
| A submitted review notifies | A GitHub response failure is an account failure | `variant` |
| A submitted review notifies | A delivery surface failure keeps the transcript | `variant` |
| A submitted review notifies | A missing configuration is a configuration error | `variant` |
| A submitted review notifies | A missing or failing gh is an account failure | `variant` |
| A submitted review notifies | A quarantine failure is a state error | `variant` |
| A submitted review notifies | An invalid configuration is reported with exit 2 | `variant` |
| A submitted review notifies | An unknown state version is quarantined and replaced | `variant` |
| A submitted review notifies | Deselecting a repository in all mode is refused | `variant` |
| A submitted review notifies | Duplicate watch scope entries are rejected | `variant` |
| A token file path starting with ~ is expanded | A GitHub response failure is an account failure | `variant` |
| A token file path starting with ~ is expanded | A reopen within the retention window is observed | `variant` |
| A token file path starting with ~ is expanded | A replayed pull request node does not duplicate activity | `variant` |
| A token file readable by other local users is refused | A CLI usage error exits 2 | `variant` |
| A token file readable by other local users is refused | A GitHub response failure is an account failure | `variant` |
| A token file readable by other local users is refused | A delivery surface failure keeps the transcript | `variant` |
| A token file readable by other local users is refused | A missing configuration is a configuration error | `boundary` |
| A token file readable by other local users is refused | A missing or failing gh is an account failure | `boundary` |
| A token file readable by other local users is refused | A quarantine failure is a state error | `boundary` |
| A token file readable by other local users is refused | An invalid configuration is reported with exit 2 | `boundary` |
| A token file readable by other local users is refused | An unknown state version is quarantined and replaced | `boundary` |
| A token file readable by other local users is refused | Deselecting a repository in all mode is refused | `variant` |
| A token file readable by other local users is refused | Duplicate watch scope entries are rejected | `boundary` |
| A token file readable only by the operator is used | A GitHub response failure is an account failure | `variant` |
| A token file readable only by the operator is used | A reopen within the retention window is observed | `variant` |
| A token file readable only by the operator is used | A replayed pull request node does not duplicate activity | `variant` |
| A version-1 state loads with empty history | A CLI usage error exits 2 | `variant` |
| A version-1 state loads with empty history | A GitHub response failure is an account failure | `variant` |
| A version-1 state loads with empty history | A delivery surface failure keeps the transcript | `variant` |
| A version-1 state loads with empty history | A missing configuration is a configuration error | `variant` |
| A version-1 state loads with empty history | A missing or failing gh is an account failure | `variant` |
| A version-1 state loads with empty history | A quarantine failure is a state error | `variant` |
| A version-1 state loads with empty history | An invalid configuration is reported with exit 2 | `variant` |
| A version-1 state loads with empty history | An unknown state version is quarantined and replaced | `variant` |
| A version-1 state loads with empty history | Deselecting a repository in all mode is refused | `variant` |
| A version-1 state loads with empty history | Duplicate watch scope entries are rejected | `variant` |
| An ignored pull request produces no activity | A CLI usage error exits 2 | `variant` |
| An ignored pull request produces no activity | A GitHub response failure is an account failure | `variant` |
| An ignored pull request produces no activity | A delivery surface failure keeps the transcript | `variant` |
| An ignored pull request produces no activity | A missing configuration is a configuration error | `variant` |
| An ignored pull request produces no activity | A missing or failing gh is an account failure | `variant` |
| An ignored pull request produces no activity | A quarantine failure is a state error | `variant` |
| An ignored pull request produces no activity | An invalid configuration is reported with exit 2 | `variant` |
| An ignored pull request produces no activity | An unknown state version is quarantined and replaced | `variant` |
| An ignored pull request produces no activity | Deselecting a repository in all mode is refused | `variant` |
| An ignored pull request produces no activity | Duplicate watch scope entries are rejected | `variant` |
| An invalid configuration is rejected | A CLI usage error exits 2 | `variant` |
| An invalid configuration is rejected | A GitHub response failure is an account failure | `variant` |
| An invalid configuration is rejected | A delivery surface failure keeps the transcript | `variant` |
| An invalid configuration is rejected | A missing configuration is a configuration error | `variant` |
| An invalid configuration is rejected | A missing or failing gh is an account failure | `boundary` |
| An invalid configuration is rejected | A quarantine failure is a state error | `boundary` |
| An invalid configuration is rejected | An invalid configuration is reported with exit 2 | `boundary` |
| An invalid configuration is rejected | An unknown state version is quarantined and replaced | `boundary` |
| An invalid configuration is rejected | Deselecting a repository in all mode is refused | `variant` |
| An invalid configuration is rejected | Duplicate watch scope entries are rejected | `boundary` |
| An invalid configuration is reported with exit 2 | A delivery surface failure keeps the transcript | `variant` |
| An invalid configuration is reported with exit 2 | A missing sound player still delivers the notification | `variant` |
| An invalid configuration is reported with exit 2 | The developer tests notifications | `variant` |
| An unavailable desktop surface keeps the transcript | A delivery surface failure keeps the transcript | `variant` |
| An unavailable desktop surface keeps the transcript | An invalid configuration is reported with exit 2 | `variant` |
| An unavailable desktop surface keeps the transcript | Duplicate watch scope entries are rejected | `variant` |
| Deselecting a repository in all mode is refused | A CLI usage error exits 2 | `variant` |
| Deselecting a repository in all mode is refused | A GitHub response failure is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A delivery surface failure keeps the transcript | `variant` |
| Deselecting a repository in all mode is refused | A disabled desktop notification keeps the transcript | `variant` |
| Deselecting a repository in all mode is refused | A failed account does not stop the other account | `variant` |
| Deselecting a repository in all mode is refused | A missing configuration is a configuration error | `variant` |
| Deselecting a repository in all mode is refused | A missing or failing gh is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A missing sound player still delivers the notification | `variant` |
| Deselecting a repository in all mode is refused | A missing token file is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A one-shot check never starts a browser login | `variant` |
| Deselecting a repository in all mode is refused | A personal access token for a different login is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A quarantine failure is a state error | `variant` |
| Deselecting a repository in all mode is refused | An invalid configuration is reported with exit 2 | `variant` |
| Deselecting a repository in all mode is refused | An unavailable desktop surface keeps the transcript | `variant` |
| Deselecting a repository in all mode is refused | An unknown state version is quarantined and replaced | `variant` |
| Deselecting a repository in all mode is refused | Developer lists unread activity | `variant` |
| Deselecting a repository in all mode is refused | Developer runs a single check and sees the current activity | `variant` |
| Deselecting a repository in all mode is refused | Developer sees the watch state | `variant` |
| Deselecting a repository in all mode is refused | Duplicate watch scope entries are rejected | `variant` |
| Deselecting a repository in all mode is refused | Inbox with no state shows no activity | `variant` |
| Deselecting a repository in all mode is refused | Status with no state shows empty sections | `variant` |
| Deselecting a repository in all mode is refused | The developer tests notifications | `variant` |
| Deselecting a repository in all mode is refused | The developer tests notifications on the command line | `variant` |
| Deselecting a repository in all mode is refused | The notification test can mute the desktop notification | `variant` |
| Deselecting a repository in all mode is refused | The notification test can mute the sound | `variant` |
| Deselecting a repository in all mode is refused | The notification test reports an unavailable surface | `variant` |
| Deselecting a repository in all mode is refused | The notification test reports disabled surfaces | `variant` |
| Developer lists unread activity | A delivery surface failure keeps the transcript | `variant` |
| Developer lists unread activity | An invalid configuration is reported with exit 2 | `variant` |
| Developer lists unread activity | Duplicate watch scope entries are rejected | `variant` |
| Developer marks activity read | A CLI usage error exits 2 | `variant` |
| Developer marks activity read | A GitHub response failure is an account failure | `variant` |
| Developer marks activity read | A delivery surface failure keeps the transcript | `variant` |
| Developer marks activity read | A missing configuration is a configuration error | `variant` |
| Developer marks activity read | A missing or failing gh is an account failure | `variant` |
| Developer marks activity read | A quarantine failure is a state error | `variant` |
| Developer marks activity read | An invalid configuration is reported with exit 2 | `variant` |
| Developer marks activity read | An unknown state version is quarantined and replaced | `variant` |
| Developer marks activity read | Deselecting a repository in all mode is refused | `variant` |
| Developer marks activity read | Duplicate watch scope entries are rejected | `variant` |
| Developer navigates the inbox panes | A missing browser opener is reported and keeps the detail | `variant` |
| Developer navigates the inbox panes | Developer opens an activity | `variant` |
| Developer opens the inbox TUI | A missing browser opener is reported and keeps the detail | `variant` |
| Developer opens the inbox TUI | Developer opens an activity | `variant` |
| Developer reviews a stacked inbox | Developer opens an activity | `variant` |
| Developer sees the watch state | A delivery surface failure keeps the transcript | `variant` |
| Developer sees the watch state | An invalid configuration is reported with exit 2 | `variant` |
| Developer sees the watch state | Duplicate watch scope entries are rejected | `variant` |
| Developer sets up passuff on first run | A missing browser opener is reported and keeps the detail | `variant` |
| Developer sets up passuff on first run | A rejected stack query retries once and shows standalone rows | `variant` |
| Developer sets up passuff on first run | Developer opens an activity | `variant` |
| Duplicate watch scope entries are rejected | A delivery surface failure keeps the transcript | `variant` |
| Duplicate watch scope entries are rejected | A missing sound player still delivers the notification | `variant` |
| Duplicate watch scope entries are rejected | An invalid configuration is reported with exit 2 | `boundary` |
| Duplicate watch scope entries are rejected | The developer tests notifications | `variant` |
| Enter with no activity shows no detail | A rejected stack query retries once and shows standalone rows | `variant` |
| Enter with no activity shows no detail | The activity list row is bounded to the recorded excerpt | `variant` |
| Inbox hides activity outside the focus | A CLI usage error exits 2 | `variant` |
| Inbox hides activity outside the focus | A GitHub response failure is an account failure | `variant` |
| Inbox hides activity outside the focus | A delivery surface failure keeps the transcript | `variant` |
| Inbox hides activity outside the focus | A missing configuration is a configuration error | `variant` |
| Inbox hides activity outside the focus | A missing or failing gh is an account failure | `variant` |
| Inbox hides activity outside the focus | A quarantine failure is a state error | `variant` |
| Inbox hides activity outside the focus | An invalid configuration is reported with exit 2 | `variant` |
| Inbox hides activity outside the focus | An unknown state version is quarantined and replaced | `variant` |
| Inbox hides activity outside the focus | Deselecting a repository in all mode is refused | `variant` |
| Inbox hides activity outside the focus | Duplicate watch scope entries are rejected | `variant` |
| Inbox reports a state error | A CLI usage error exits 2 | `variant` |
| Inbox reports a state error | A GitHub response failure is an account failure | `variant` |
| Inbox reports a state error | A delivery surface failure keeps the transcript | `variant` |
| Inbox reports a state error | A missing configuration is a configuration error | `variant` |
| Inbox reports a state error | A missing or failing gh is an account failure | `variant` |
| Inbox reports a state error | A quarantine failure is a state error | `variant` |
| Inbox reports a state error | An invalid configuration is reported with exit 2 | `variant` |
| Inbox reports a state error | An unknown state version is quarantined and replaced | `variant` |
| Inbox reports a state error | Deselecting a repository in all mode is refused | `variant` |
| Inbox reports a state error | Duplicate watch scope entries are rejected | `variant` |
| Read all marks every activity read | A CLI usage error exits 2 | `variant` |
| Read all marks every activity read | A GitHub response failure is an account failure | `variant` |
| Read all marks every activity read | A delivery surface failure keeps the transcript | `variant` |
| Read all marks every activity read | A missing configuration is a configuration error | `variant` |
| Read all marks every activity read | A missing or failing gh is an account failure | `variant` |
| Read all marks every activity read | A quarantine failure is a state error | `variant` |
| Read all marks every activity read | An invalid configuration is reported with exit 2 | `variant` |
| Read all marks every activity read | An unknown state version is quarantined and replaced | `variant` |
| Read all marks every activity read | Deselecting a repository in all mode is refused | `variant` |
| Read all marks every activity read | Duplicate watch scope entries are rejected | `variant` |
| Read marks one activity read | A CLI usage error exits 2 | `variant` |
| Read marks one activity read | A GitHub response failure is an account failure | `variant` |
| Read marks one activity read | A delivery surface failure keeps the transcript | `variant` |
| Read marks one activity read | A missing configuration is a configuration error | `variant` |
| Read marks one activity read | A missing or failing gh is an account failure | `variant` |
| Read marks one activity read | A quarantine failure is a state error | `variant` |
| Read marks one activity read | An invalid configuration is reported with exit 2 | `variant` |
| Read marks one activity read | An unknown state version is quarantined and replaced | `variant` |
| Read marks one activity read | Deselecting a repository in all mode is refused | `variant` |
| Read marks one activity read | Duplicate watch scope entries are rejected | `variant` |
| Read refuses an unknown activity id | A CLI usage error exits 2 | `variant` |
| Read refuses an unknown activity id | A GitHub response failure is an account failure | `variant` |
| Read refuses an unknown activity id | A delivery surface failure keeps the transcript | `variant` |
| Read refuses an unknown activity id | A missing configuration is a configuration error | `variant` |
| Read refuses an unknown activity id | A missing or failing gh is an account failure | `variant` |
| Read refuses an unknown activity id | A quarantine failure is a state error | `variant` |
| Read refuses an unknown activity id | An invalid configuration is reported with exit 2 | `variant` |
| Read refuses an unknown activity id | An unknown state version is quarantined and replaced | `variant` |
| Read refuses an unknown activity id | Deselecting a repository in all mode is refused | `variant` |
| Read refuses an unknown activity id | Duplicate watch scope entries are rejected | `variant` |
| Read reports a state error | A CLI usage error exits 2 | `variant` |
| Read reports a state error | A GitHub response failure is an account failure | `variant` |
| Read reports a state error | A delivery surface failure keeps the transcript | `variant` |
| Read reports a state error | A missing configuration is a configuration error | `variant` |
| Read reports a state error | A missing or failing gh is an account failure | `variant` |
| Read reports a state error | A quarantine failure is a state error | `variant` |
| Read reports a state error | An invalid configuration is reported with exit 2 | `variant` |
| Read reports a state error | An unknown state version is quarantined and replaced | `variant` |
| Read reports a state error | Deselecting a repository in all mode is refused | `variant` |
| Read reports a state error | Duplicate watch scope entries are rejected | `variant` |
| Read requires an activity id or all | A CLI usage error exits 2 | `variant` |
| Read requires an activity id or all | A GitHub response failure is an account failure | `variant` |
| Read requires an activity id or all | A delivery surface failure keeps the transcript | `variant` |
| Read requires an activity id or all | A missing configuration is a configuration error | `variant` |
| Read requires an activity id or all | A missing or failing gh is an account failure | `variant` |
| Read requires an activity id or all | A quarantine failure is a state error | `variant` |
| Read requires an activity id or all | An invalid configuration is reported with exit 2 | `variant` |
| Read requires an activity id or all | An unknown state version is quarantined and replaced | `variant` |
| Read requires an activity id or all | Deselecting a repository in all mode is refused | `variant` |
| Read requires an activity id or all | Duplicate watch scope entries are rejected | `variant` |
| Read state survives a restart | A CLI usage error exits 2 | `variant` |
| Read state survives a restart | A GitHub response failure is an account failure | `variant` |
| Read state survives a restart | A delivery surface failure keeps the transcript | `variant` |
| Read state survives a restart | A missing configuration is a configuration error | `variant` |
| Read state survives a restart | A missing or failing gh is an account failure | `variant` |
| Read state survives a restart | A quarantine failure is a state error | `variant` |
| Read state survives a restart | An invalid configuration is reported with exit 2 | `variant` |
| Read state survives a restart | An unknown state version is quarantined and replaced | `variant` |
| Read state survives a restart | Deselecting a repository in all mode is refused | `variant` |
| Read state survives a restart | Duplicate watch scope entries are rejected | `variant` |
| Recording activity survives a delivery failure | A CLI usage error exits 2 | `variant` |
| Recording activity survives a delivery failure | A GitHub response failure is an account failure | `variant` |
| Recording activity survives a delivery failure | A delivery surface failure keeps the transcript | `variant` |
| Recording activity survives a delivery failure | A missing configuration is a configuration error | `variant` |
| Recording activity survives a delivery failure | A missing or failing gh is an account failure | `variant` |
| Recording activity survives a delivery failure | A quarantine failure is a state error | `variant` |
| Recording activity survives a delivery failure | An invalid configuration is reported with exit 2 | `variant` |
| Recording activity survives a delivery failure | An unknown state version is quarantined and replaced | `variant` |
| Recording activity survives a delivery failure | Deselecting a repository in all mode is refused | `variant` |
| Recording activity survives a delivery failure | Duplicate watch scope entries are rejected | `variant` |
| Reselecting catches up silently | A missing browser opener is reported and keeps the detail | `variant` |
| Reselecting catches up silently | A rejected stack query retries once and shows standalone rows | `variant` |
| Reselecting catches up silently | Developer opens an activity | `variant` |
| Reselecting catches up silently | The activity list row is bounded to the recorded excerpt | `variant` |
| Selecting a repository with no records records a baseline | A CLI usage error exits 2 | `variant` |
| Selecting a repository with no records records a baseline | A GitHub response failure is an account failure | `variant` |
| Selecting a repository with no records records a baseline | A delivery surface failure keeps the transcript | `variant` |
| Selecting a repository with no records records a baseline | A missing configuration is a configuration error | `variant` |
| Selecting a repository with no records records a baseline | A missing or failing gh is an account failure | `variant` |
| Selecting a repository with no records records a baseline | A quarantine failure is a state error | `variant` |
| Selecting a repository with no records records a baseline | An invalid configuration is reported with exit 2 | `variant` |
| Selecting a repository with no records records a baseline | An unknown state version is quarantined and replaced | `variant` |
| Selecting a repository with no records records a baseline | Deselecting a repository in all mode is refused | `variant` |
| Selecting a repository with no records records a baseline | Duplicate watch scope entries are rejected | `variant` |
| Stack membership updates after a change | A rejected stack query retries once and shows standalone rows | `variant` |
| Stack membership updates after a change | The activity list row is bounded to the recorded excerpt | `variant` |
| Status lists configured accounts before the first check | A delivery surface failure keeps the transcript | `variant` |
| Status lists configured accounts before the first check | An invalid configuration is reported with exit 2 | `variant` |
| Status lists configured accounts before the first check | Duplicate watch scope entries are rejected | `variant` |
| Status lists multiple accounts | A delivery surface failure keeps the transcript | `variant` |
| Status lists multiple accounts | An invalid configuration is reported with exit 2 | `variant` |
| Status lists multiple accounts | Duplicate watch scope entries are rejected | `variant` |
| Status reports unread and total activity | A CLI usage error exits 2 | `variant` |
| Status reports unread and total activity | A GitHub response failure is an account failure | `variant` |
| Status reports unread and total activity | A delivery surface failure keeps the transcript | `variant` |
| Status reports unread and total activity | A missing configuration is a configuration error | `variant` |
| Status reports unread and total activity | A missing or failing gh is an account failure | `variant` |
| Status reports unread and total activity | A quarantine failure is a state error | `variant` |
| Status reports unread and total activity | An invalid configuration is reported with exit 2 | `variant` |
| Status reports unread and total activity | An unknown state version is quarantined and replaced | `variant` |
| Status reports unread and total activity | Deselecting a repository in all mode is refused | `variant` |
| Status reports unread and total activity | Duplicate watch scope entries are rejected | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A CLI usage error exits 2 | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A GitHub response failure is an account failure | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A delivery surface failure keeps the transcript | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A missing configuration is a configuration error | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A missing or failing gh is an account failure | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A quarantine failure is a state error | `variant` |
| Status shows the watch scope when a repository allowlist is configured | An invalid configuration is reported with exit 2 | `variant` |
| Status shows the watch scope when a repository allowlist is configured | An unknown state version is quarantined and replaced | `variant` |
| Status shows the watch scope when a repository allowlist is configured | Deselecting a repository in all mode is refused | `variant` |
| Status shows the watch scope when a repository allowlist is configured | Duplicate watch scope entries are rejected | `variant` |
| Status shows the watch scope when all repositories are watched | A CLI usage error exits 2 | `variant` |
| Status shows the watch scope when all repositories are watched | A GitHub response failure is an account failure | `variant` |
| Status shows the watch scope when all repositories are watched | A delivery surface failure keeps the transcript | `variant` |
| Status shows the watch scope when all repositories are watched | A missing configuration is a configuration error | `variant` |
| Status shows the watch scope when all repositories are watched | A missing or failing gh is an account failure | `variant` |
| Status shows the watch scope when all repositories are watched | A quarantine failure is a state error | `variant` |
| Status shows the watch scope when all repositories are watched | An invalid configuration is reported with exit 2 | `variant` |
| Status shows the watch scope when all repositories are watched | An unknown state version is quarantined and replaced | `variant` |
| Status shows the watch scope when all repositories are watched | Deselecting a repository in all mode is refused | `variant` |
| Status shows the watch scope when all repositories are watched | Duplicate watch scope entries are rejected | `variant` |
| Status uses human labels for closed and pending | A CLI usage error exits 2 | `variant` |
| Status uses human labels for closed and pending | A GitHub response failure is an account failure | `variant` |
| Status uses human labels for closed and pending | A delivery surface failure keeps the transcript | `variant` |
| Status uses human labels for closed and pending | A missing configuration is a configuration error | `variant` |
| Status uses human labels for closed and pending | A missing or failing gh is an account failure | `variant` |
| Status uses human labels for closed and pending | A quarantine failure is a state error | `variant` |
| Status uses human labels for closed and pending | An invalid configuration is reported with exit 2 | `variant` |
| Status uses human labels for closed and pending | An unknown state version is quarantined and replaced | `variant` |
| Status uses human labels for closed and pending | Deselecting a repository in all mode is refused | `variant` |
| Status uses human labels for closed and pending | Duplicate watch scope entries are rejected | `variant` |
| Status with no state shows empty sections | A delivery surface failure keeps the transcript | `variant` |
| Status with no state shows empty sections | An invalid configuration is reported with exit 2 | `variant` |
| Status with no state shows empty sections | Duplicate watch scope entries are rejected | `variant` |
| Switching to all observes every repository | A missing browser opener is reported and keeps the detail | `variant` |
| Switching to all observes every repository | Developer opens an activity | `variant` |
| Switching to selected with no retained set observes nothing | A missing browser opener is reported and keeps the detail | `variant` |
| Switching to selected with no retained set observes nothing | A rejected stack query retries once and shows standalone rows | `variant` |
| Switching to selected with no retained set observes nothing | Developer opens an activity | `variant` |
| Switching to selected with no retained set observes nothing | The activity list row is bounded to the recorded excerpt | `variant` |
| The activity detail marks an activity without a reported time as approximate | A missing browser opener is reported and keeps the detail | `variant` |
| The activity detail marks an activity without a reported time as approximate | Developer opens an activity | `variant` |
| The activity detail shows and scrolls a long comment | A rejected stack query retries once and shows standalone rows | `variant` |
| The activity detail shows and scrolls a long comment | The activity list row is bounded to the recorded excerpt | `variant` |
| The activity detail shows when the activity happened and when it was fetched | A missing browser opener is reported and keeps the detail | `variant` |
| The activity detail shows when the activity happened and when it was fetched | Developer opens an activity | `variant` |
| The activity keeps the stack label | A missing browser opener is reported and keeps the detail | `variant` |
| The activity keeps the stack label | Developer opens an activity | `variant` |
| The activity table shows when the activity happened | A missing browser opener is reported and keeps the detail | `variant` |
| The activity table shows when the activity happened | A rejected stack query retries once and shows standalone rows | `variant` |
| The activity table shows when the activity happened | Developer opens an activity | `variant` |
| The activity table shows when the activity happened | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer closes dialogs with q | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer closes dialogs with q | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer filters the activity to a pull request | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer filters the activity to a pull request | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer filters the activity to a stack | A missing browser opener is reported and keeps the detail | `variant` |
| The developer filters the activity to a stack | Developer opens an activity | `variant` |
| The developer filters the activity to unread records | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer filters the activity to unread records | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer ignores and restores a pull request | A missing browser opener is reported and keeps the detail | `variant` |
| The developer ignores and restores a pull request | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer ignores and restores a pull request | Developer opens an activity | `variant` |
| The developer ignores and restores a pull request | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer mutes notifications from the settings dialog | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer mutes notifications from the settings dialog | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer opens the key reference | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer opens the key reference | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer scrolls the pull request detail | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer scrolls the pull request detail | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer sorts the watched list | A missing browser opener is reported and keeps the detail | `variant` |
| The developer sorts the watched list | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer sorts the watched list | Developer opens an activity | `variant` |
| The developer sorts the watched list | The activity list row is bounded to the recorded excerpt | `variant` |
| The focus cycles between watched pull requests and activity | A missing browser opener is reported and keeps the detail | `variant` |
| The focus cycles between watched pull requests and activity | A rejected stack query retries once and shows standalone rows | `variant` |
| The focus cycles between watched pull requests and activity | Developer opens an activity | `variant` |
| The focus cycles between watched pull requests and activity | The activity list row is bounded to the recorded excerpt | `variant` |
| The inbox reconciles open pull requests when the interval elapses | A missing browser opener is reported and keeps the detail | `variant` |
| The inbox reconciles open pull requests when the interval elapses | A rejected stack query retries once and shows standalone rows | `variant` |
| The inbox reconciles open pull requests when the interval elapses | Developer opens an activity | `variant` |
| The inbox reconciles open pull requests when the interval elapses | The activity list row is bounded to the recorded excerpt | `variant` |
| The inbox shows activity observed while it is open | A missing browser opener is reported and keeps the detail | `variant` |
| The inbox shows activity observed while it is open | Developer opens an activity | `variant` |
| The last deselection observes nothing | A CLI usage error exits 2 | `variant` |
| The last deselection observes nothing | A GitHub response failure is an account failure | `variant` |
| The last deselection observes nothing | A delivery surface failure keeps the transcript | `variant` |
| The last deselection observes nothing | A missing configuration is a configuration error | `variant` |
| The last deselection observes nothing | A missing or failing gh is an account failure | `variant` |
| The last deselection observes nothing | A quarantine failure is a state error | `variant` |
| The last deselection observes nothing | An invalid configuration is reported with exit 2 | `variant` |
| The last deselection observes nothing | An unknown state version is quarantined and replaced | `variant` |
| The last deselection observes nothing | Deselecting a repository in all mode is refused | `variant` |
| The last deselection observes nothing | Duplicate watch scope entries are rejected | `variant` |
| The notification test can mute the desktop notification | A delivery surface failure keeps the transcript | `variant` |
| The notification test can mute the desktop notification | An invalid configuration is reported with exit 2 | `variant` |
| The notification test can mute the desktop notification | Duplicate watch scope entries are rejected | `variant` |
| The notification test can mute the sound | A delivery surface failure keeps the transcript | `variant` |
| The notification test can mute the sound | An invalid configuration is reported with exit 2 | `variant` |
| The notification test can mute the sound | Duplicate watch scope entries are rejected | `variant` |
| The notification test reports an unavailable surface | A delivery surface failure keeps the transcript | `variant` |
| The notification test reports an unavailable surface | An invalid configuration is reported with exit 2 | `variant` |
| The notification test reports an unavailable surface | Duplicate watch scope entries are rejected | `variant` |
| The notification test reports disabled surfaces | A delivery surface failure keeps the transcript | `variant` |
| The notification test reports disabled surfaces | An invalid configuration is reported with exit 2 | `variant` |
| The notification test reports disabled surfaces | Duplicate watch scope entries are rejected | `variant` |
| The pull request detail shows its metadata | Developer opens an activity | `variant` |
| The terminal UI shows the empty inbox | A missing browser opener is reported and keeps the detail | `variant` |
| The terminal UI shows the empty inbox | Developer opens an activity | `variant` |
| The watched pull request selection scrolls | A missing browser opener is reported and keeps the detail | `variant` |
| The watched pull request selection scrolls | Developer opens an activity | `variant` |
| The watched table hides a pull request outside the focus | A rejected stack query retries once and shows standalone rows | `variant` |
| The watched table hides a pull request outside the focus | The activity list row is bounded to the recorded excerpt | `variant` |
| The watched table shows the creator and the assignee | A missing browser opener is reported and keeps the detail | `variant` |
| The watched table shows the creator and the assignee | Developer opens an activity | `variant` |
| Two accounts share the global scope | A rejected stack query retries once and shows standalone rows | `boundary` |
| Two accounts share the global scope | The activity list row is bounded to the recorded excerpt | `variant` |

## Split-or-keep

`givn lint --change python-rebuild` reports zero `[LONG]` findings. No scenario
in the change or the permanent corpus exceeds the long-scenario threshold, so
no split-or-keep row is required. No scenario was removed and re-added by this
change: the single `@givn.modified` scenario supersedes the permanent
`Developer opens an activity's details` by title at archive time, and the
permanent copy remains until then.

## README impact

README-IMPACT: updated - Install, Prerequisites, Development, Coverage

The README was edited for this change: Install states the Python 3.11 package
and the `pip install .` path with the `passuff` and `passuff-tui` entry points;
Prerequisites lists Python 3.11+, `gh` 2.40+, the Linux players, and
Notification Center/`afplay`; Development names the `behave` runner scopes and
the strictness proof; Coverage names the two Cobertura reports. Commands,
Terminal inbox, Notifications, and Configuration were also rewritten for the
rebuilt product.

## Findings

| ID | Severity | Finding | Fix |
|---|---|---|---|
| F1 | critical | Fabricated step body: `givn/steps/common.py:1476` `the developer reopens passuff` is `return None`; `python scripts/check_steps.py` exits 1 on the current tree. | Give the step an observable effect or delete it and let the following command perform the restart; run `check_steps.py` over `givn/steps` and `givn/e2e_steps` in `scripts/verify.sh`'s normal scopes so the audit cannot be bypassed. |
| F2 | critical | Promised step layout missing: the seven per-capability files named in `design.md`'s Step Definitions table and in every `tasks.md` task do not exist; all 237 bindings live in `givn/steps/common.py`. The seven `givn/e2e_steps/<capability>_e2e.py` files are one-line docstrings with no bindings, contrary to “E2E bindings, one file per capability” and `design-review.md` Q4's operator resolution. | Split the bindings into the named per-capability files in both trees (shared phrases stay in `givn/steps/common.py`), or update `design.md` with the single-module decision and re-run design-review; either way the tasks' step-file references must match the tree. |
| F3 | critical | Contract without implementation: `ObservationSchedule` → `passuff.tui.session.ObservationSession` does not exist; live observation is `_observe`/`_interactive` in `passuff/tui/app.py:338,354`. `arc42` ch5/ch6/ch8 and the glossary all name `ObservationSession`. | Add `passuff/tui/session.py` with `ObservationSession` and have the TUI use it, or update `design.md`, the arc42 chapters, and the glossary to the as-built responsibility. |
| F4 | major | `passuff/status.py` does not exist; `WatchStatusReader` lives in `passuff/inbox.py:54`, so one module owns the `ActivityInbox` and `WatchStatus` contracts. `design.md`'s module layout and `docs/glossary.md:59` name `passuff.status.WatchStatusReader`. | Split `WatchStatusReader` into `passuff/status.py`, or correct the design and glossary symbols. |
| F5 | major | Dead code and a missing promised double: `passuff/stacking.py` `StackGrouper` has no consumer (0% coverage) while `arc42` ch5 says read models “group stacks through `StackGrouper`”; `passuff/composition.py` `openers()` has no caller; `passuff/render.py:99-121` `render_activity_detail` has no caller; `passuff/notify.py:27-30` `recording_path` is a test hook in production with no caller and `tests/support/recording_delivery.py` is absent. | Delete the dead modules/blocks or wire them to their declared consumers; move the recording double to `tests/support/recording_delivery.py` and use it from the macOS adapter test. |
| F6 | critical | The 80-character list-row excerpt (T7) is unimplemented and its delta scenario does not assert it: `passuff/tui/frame.py:154-160` renders the fixed summary, and `givn/steps/common.py:1758-1767` checks `<= 100` on any line containing “comment by”, not the list row and not 80. | Implement the bounded excerpt in the list row and make the step assert the row's excerpt length `<= 80` from the frame the consumer sees. |
| F7 | major | The 4 KiB stored body bound (`design.md` data model, T7) is unimplemented: `passuff/classify.py:53` stores the body unbounded and no scenario covers it. | Bound the stored body at 4 KiB and add a scenario asserting the stored and rendered bound. |
| F8 | major | macOS sound is unimplemented although `README.md:55`, `docs/reference.md:288`, and arc42 ch3/ch7 require `afplay`: `passuff/sound.py:9` lists only the Linux players. | Add `afplay` to the player selection for `sys.platform == "darwin"` and cover it with the fake `afplay` twin, or correct the documents. |
| F9 | major | Visual evidence incomplete: the `@e2e` scenario `Developer starts watching and is notified of a new comment` has no committed transcript (17 of 19 `@e2e` runs have one) because `World.start_watch` (`tests/support/world.py:524`) never sets `world.last` and `tests/support/harness.py:52` only writes a transcript when it is set. | Capture and commit `evidence/visual/developer-starts-watching-and-is-notified-of-a-new-comment/transcript.txt` from the watcher output; a human must then confirm the notification and sound for the Persona's primary goal. |
| F10 | major | Arc42 conformance is not clean (see `ARC42 CONFORMANCE: FINDINGS`): ch4/ch5/ch8 document `ObservationSession`, `RecordingDelivery`, the per-capability E2E step tree, and the 80-character excerpt that the implementation lacks; ch11 TD-005 is still written as an unresolved plan although the coverage scripts landed in this change. | Complete the implementation to the documents or correct chapters 4, 5, 8, and 11 and the change-level `arc42.md`; a design change re-runs design-review. |
| F11 | major | The glossary names nonexistent code symbols: `passuff.status.WatchStatusReader` (`docs/glossary.md:59`) and `ObservationSession` (`docs/glossary.md:62,67`). | Resolve together with F3/F4 so one authoritative symbol exists per term. |
| F12 | minor | `tests/strictness/` fixtures are retained and tracked although `design.md` and the setup task say they are removed after the proof; `scripts/verify.sh --strictness` still depends on them. | Either keep them and record the decision in `design.md`, or remove them and run `check_steps.py` over the durable trees instead. |
| F13 | minor | Driving-mechanism mismatch: `design.md`'s Interaction Coverage Matrix promises a pty smoke for `Developer opens the inbox TUI`; that `@e2e` scenario runs only `--snapshot --observe`. The pty smoke exists in a regular scenario (`The developer starts the inbox TUI binary`). | Add the pty step to the `@e2e` scenario or correct the matrix row to the as-built driving mechanism. |
| F14 | major | `givn check review` cannot read the task evidence: tasks are `### [x] N. <title>` headings with a separate `- COMMIT: <hash> -- Batch: <name>` line, and the gate reports `[integrity] no \`- [x] COMMIT:\` lines found in tasks.md — revision identifiers must be recorded` and `integrity check failed: hash presence`. The evidence exists (205/205) but not in the shape the gate reads. | Record each checked task's revision in the gate's expected shape (a checked task line carrying `COMMIT: <hash>`), or fix the task authoring format for the whole file; a line in `review.md` does not satisfy it, so this is a `tasks.md` fix. |
| F15 | major | The verification receipt the review gate reads is untracked: `givn/changes/python-rebuild/verification/tasks-verification.json` is not in `git ls-files` (only `tasks-result.json` is), and removing it makes `givn check review` fail with `change verification receipt missing`. The receipt that proves 229/229 is therefore not part of the change's committed evidence. | Commit `verification/tasks-verification.json` with the task-evidence batch so a clean checkout can be reviewed and archived. |

## Sign-off

- [ ] Fabrication audit: clean — **fails**: `scripts/check_steps.py` exits 1 on `givn/steps/common.py:1476` (F1).
- [ ] Every checked task has a verified commit touching production code — the evidence exists (205/205 name `14e0b99...` with a batch name, and that commit tree contains `passuff/` and the step bindings), but `givn check review` cannot read the format and fails on hash presence (F14).
- [ ] Every promised component exists — **fails**: `passuff/status.py`, `passuff/tui/session.py`, the seven per-capability step files, `tests/support/recording_delivery.py`, `bin/osascript`, `bin/afplay` (F2-F5).
- [ ] Every consumer uses only its declared contract — **fails**: `StackGrouper` and `openers()` have no consumer, and `ObservationSession` has no implementation (F3, F5).
- [x] Strict-mode proof present and passing — `./scripts/verify.sh --strictness` exits 0; the durable-tree audit it cannot reach fails (F1).
- [x] Scenario execution evidence is recorded per scope — change 229/229, regular 197/197, E2E 19/19; review does not re-run the suite. The receipt itself is untracked (F15).
- [x] Coverage classification uses the most recent measured evidence — merged, non-E2E, and E2E Cobertura reports are distinct.
- [ ] Every coverage gap classified and resolved — classified in the three buckets; the bucket 1 and bucket 2 items are unresolved findings (F5-F7).
- [ ] Dead code deleted. Missing tests added. Hard-to-test gaps justified. — **fails**: bucket 1 dead code remains; bucket 2 gaps have no tests.
- [x] Redundant unit tests removed — there is no separate unit-test tree.
- [x] No `@wip` tags remain. No implementation-layer detail in the spec.
- [x] Canonical E2E policy read from the resolved specs instruction; every normalized inventory action has one `@e2e` scenario and variants are regular scenarios.
- [x] Every E2E scenario has a real-interface primary assertion and uses the reviewed driver; no browser UI scenario is replaced by HTTP/fetch.
- [x] The local run command starts the full stack cleanly — `python -m passuff --help` exits 0; the twins start per scenario under `./scripts/verify.sh`.
- [x] The exact files `verify.e2e_command` invokes were identified and read; the tree was searched for a second E2E implementation and none exists.
- [x] The E2E scope is selected explicitly (`GIVN_RESULT_SCOPE` plus `--e2e`); the receipt reports zero failures and skips.
- [ ] Implementation matches what `design.md` names — **fails**: step layout, `status.py`, `session.py`, `recording_delivery.py`, and the pty driver deviate without a design update (F2, F3, F4, F5, F13).
- [ ] Interaction coverage verified — every inventory entry maps to a matrix row and an `@e2e` scenario, but one driving mechanism is missing (F13).
- [x] No finding is excused with a classification outside the three buckets.

## Result

The change is not ready to archive. The implementation passes its own runner and
the receipts are internally consistent, but four evidence-level defects block
sign-off: a fabricated step that the project's own audit rejects (F1), the
reviewed step layout and three promised components missing from the tree
(F2-F5), a specified boundary whose scenario asserts something else (F6), and
an arc42/glossary set that describes the plan rather than the code (F10, F11).
The visual evidence is incomplete for the Persona's primary outcome (F9), and
the task-evidence format defeats the review gate's hash-presence check (F14)
while the receipt it depends on is untracked (F15).
`givn check review --change python-rebuild` therefore exits 1 with
`integrity check failed: hash presence`, `review.md actual verdict is FAIL`, and
`visual review is VISUAL-REVIEW: UNVERIFIED`.

REVIEW: FAIL
