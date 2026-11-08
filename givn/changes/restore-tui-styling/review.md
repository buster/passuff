# Review: restore-tui-styling

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-2

Scope: the working tree at `657b44784eee34b568fdaf94e4c9995c7784f8ea`; `git status --short` was clean before this review, and the only file this review writes is this `review.md`. The change receipt is 264 scenarios at `7f0470683f9f186901f15c3094388cdae8cf74a2`, reused across the documentation-only revision `657b447`. Revisions were inspected with `git cat-file -t` and `git ls-tree`; no `git log`, `git show`, `git diff`, `git reflog`, or `git blame` was used.

## Implementation evidence check

### 0. `@e2e` tag integrity

The delta has seventeen scenario entries: four under `activity-inbox`, eleven under `inbox-navigation`, one under `stacked-inbox`, and one under `watch-scope`. Four delta scenarios carry `@e2e` (two under `activity-inbox`, one under `inbox-navigation`, one under `stacked-inbox`); no `@e2e` tag was removed or converted to a regular tag. The permanent corpus carries eighteen `@e2e` scenarios; the E2E scope is 22 executions (18 permanent + 4 delta modified copies), matching `tasks.md:16,199`. The delta E2E scenarios are modified copies of four inventory actions, not new actions.

### 1. Empty step body audit

`python3 scripts/check_steps.py --path givn/steps --path givn/e2e_steps` exits 0: `check_steps: no no-op step bodies` across the 17 tracked step files. The new steps call `assert_role`, `assert_line_role`, `assert_line_role_any`, `assert_role_member`, `assert_role_digits`, `assert_metadata_line_role`, `_assert`, or a persisted-value check; none is an empty body. The residual issue is assertion precision, not fabrication (F2, V6).

### 2. Checked scenarios and their commits

`tasks.md` has 40 `- [x]` entries: 20 task lines (setup plus 19 behavior tasks) and 20 `COMMIT:` lines, all with batch names. Tasks setup and 1-18 record `91422ab088b4233c718ca9f49776c7bea32f6c9a`; task 19 records `7efd2c8cf4e32d39a28be55dbb9ac8538af3d1d3`. Both are commit objects (`git cat-file -t`). `git ls-tree` shows `91422ab` contains `passuff/tui/frame.py`, `passuff/tui/app.py`, the step and E2E bindings, the four delta features, `tasks.md`, `design.md`, `docs/design/design-system.md`, `docs/glossary.md`, `README.md`, `scripts/verify.sh`, `tests/support/harness.py`, and the evidence; `7efd2c8` contains the settings-persistence feature and task change with the production modules. Every checked task has a revision touching production code or its regression test under a named batch.

### 3. Promised components

| Design promise | Tree | Consumer |
|---|---|---|
| `Span`, `StyledLine`, `render_styled` | `passuff/tui/frame.py:15-27,155-424` | `passuff/tui/app.py:21,374` |
| Palette owner `PALETTE` | `passuff/tui/frame.py:30-42` | `passuff/tui/app.py:21,357`; the design, glossary, and design system name `PALETTE` and role strings (D2 fixed) |
| `render`, `render_ansi`, `render_roles` | `passuff/tui/frame.py:427-454` | `passuff/tui/app.py:21,492-500`; the visual steps through `--snapshot`/`--roles` |
| Bordered panes and overlays | `passuff/tui/frame.py:126-152,248,286,298,320,336,381,402,410,412` | every surface; the outline widths are finding V5 |
| Save prompt | `passuff/tui/app.py:113-133`; `passuff/tui/frame.py:405-410` | the `inbox-navigation` save scenarios |
| Active-change runner and evidence | `scripts/verify.sh:48-54,73-93`; `tests/support/harness.py:46-70` | the change/E2E scopes; the multi-change fail clause is still absent (D7) |
| Step bindings | `givn/steps/common.py`, the four capability files, the four E2E files | `givn/steps/support.py`; the E2E files import the regular bindings |

`TerminalPresentation` is realized as the `passuff.tui.frame` module; the design's former `ROLE_STYLES`/`Role` symbols are removed and the durable documents agree on `PALETTE` and role-name strings.

### 4. Strict-mode proof

`./scripts/verify.sh --strictness` exits 0: the undefined-step probe errors with 1 undefined step, the skipped-scenario probe reports `skipped=1`, and the pass-only body is rejected by `check_steps.py`. `scripts/verify.sh:47` runs the no-op audit in every normal scope.

### 5. Every `@e2e` Then asserts the real interface

The four delta `@e2e` scenarios assert the printed snapshot, the role record, the exit status, or the browser fake's log. `Developer opens the inbox TUI`, `Developer opens an activity's details`, `Developer navigates the inbox panes`, and `Developer reviews a stacked inbox` use `--snapshot`/`--snapshot --roles` through `tests/support/world.py`. The delta E2E files import the regular bindings; the driver is the snapshot subprocess or the pty, not HTTP or fetch. The permanent CLI E2E scenarios assert the command transcript and exit status. No delta E2E Then asserts only the state file or the repository.

### 6. Exact `verify.e2e_command` and the files it invokes

`givn/commands.yaml` sets `verify.e2e_command: "./scripts/verify.sh --e2e"` and `verify.command: "./scripts/verify.sh"`. `scripts/verify.sh:74-86` runs `behave --stage e2e --tags="@e2e"` over `givn/specs/*/*.feature`, `givn/changes/*/specs/*/*.feature`, and any `GIVN_FEATURES`; `givn/e2e_environment.py` loads `givn/e2e_steps/`. The tree contains no second E2E step implementation; `grep` finds definitions only under `givn/steps`, `givn/e2e_steps`, and `tests/strictness`.

### 7. E2E scope selection

`scripts/verify.sh:11,16-24,73-93` selects `regular`, `e2e`, or `change` from `GIVN_RESULT_SCOPE`; `--e2e` sets `SCOPE=e2e`. The change scope runs every delta feature then the permanent non-E2E corpus; regular runs the permanent non-E2E corpus; E2E runs `--stage e2e --tags="@e2e"`. The receipt records change 264/264, 0 failed, 0 skipped. `tasks.md:16,199` records regular 247/247 and E2E 22/22 with zero failures and skips. Review does not re-run the suite.

### 8. E2E scope size and interaction coverage

The three touched use cases hold 11 normalized inventory entries: 7 for `review-activity-inbox`, 1 for `review-pull-request-stacks`, and 3 for `change-watch-scope`. Each has one design matrix row and one `@e2e` scenario; the four delta E2E scenarios are modified copies of four of those inventory actions, not new actions. The 22 E2E executions are 18 permanent plus the 4 delta copies; no capability has an excess E2E scenario for a variant or error case. The settings overlay, PR detail, help, save prompt, 80x24, ignored row, empty state, and picker are regular variants, correctly not `@e2e`.

### 9. Design conformance (commands, layout, framework, drivers)

The `verify` commands, step-file layout, `behave` stage split, snapshot flags, and `--snapshot --roles` assertion source match `design.md`. The durable palette symbol and role names now match `design.md:86,121-128` and the glossary (D2 fixed). The one remaining named deviation is the multi-change fail clause: `design.md:161-163` says the runner fails when more than one active change exists and the id is unset; `scripts/verify.sh:48-54` exports an empty `GIVN_CHANGE_ID` and `tests/support/harness.py:50-56` falls back to the single specs root. That is minor D7. The visual composition defect is V5, a rendering defect rather than a design-command change.

### 10. Interaction coverage cross-reference

| Use case | Inventory entry | Matrix row | `@e2e` scenario | Step file | Driving mechanism |
|---|---|---|---|---|---|
| review-activity-inbox | list unread activity | yes | Developer lists unread activity | `activity_inbox_steps.py` | subprocess `passuff inbox` |
| review-activity-inbox | mark activity read | yes | Developer marks activity read | `activity_inbox_steps.py` | subprocess `passuff read` |
| review-activity-inbox | open the inbox in the terminal UI | yes | Developer opens the inbox TUI | `activity_inbox_steps.py` | `--observe --snapshot`; role record |
| review-activity-inbox | open an activity's detail | yes | Developer opens an activity's details | `common.py` + `inbox_navigation_steps.py` | `--snapshot --keys enter` |
| review-activity-inbox | open the pull request from an activity | yes | Developer opens the pull request from an activity | `activity_inbox_e2e.py` | `--keys enter,enter` + fake opener |
| inbox-navigation | navigate the watched pull requests and activity | yes | Developer navigates the inbox panes | `common.py` + `inbox_navigation_steps.py` | `--snapshot --keys tab,j` |
| inbox-navigation | leave the inbox open while activity arrives | yes | The inbox shows activity observed while it is open | `inbox_navigation_e2e.py` | `--observe-for 4000 --snapshot` |
| stacked-inbox | review a stack and its activity | yes | Developer reviews a stacked inbox | `stacked_inbox_steps.py` | `--snapshot` with fake stack metadata |
| change-watch-scope | toggle a repository | yes | Developer toggles a repository in the watch scope | `watch_scope_steps.py` | `--picker --snapshot` + scope flags |
| change-watch-scope | switch the watch scope mode | yes | Developer switches the watch scope mode | `watch_scope_steps.py` | `--scope-mode --picker --snapshot` |
| change-watch-scope | narrow the focus | yes | Developer narrows the focus in the watch scope | `watch_scope_steps.py` | `--focus-set --picker --snapshot` |

Every inventory entry maps to a matrix row, an existing `@e2e` scenario, and step definitions using the promised mechanism. 11/11.

### 11. Local run command

`.venv/bin/python -m passuff --help` exits 0. The full local stack (fake GitHub GraphQL, private D-Bus, fake `gh`/sound/opener) is exercised by the committed 264-scenario receipt; review did not re-run the suite.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Visual contract refines the terminal goal | `docs/arc42/01-introduction-and-goals.md:42-44` | Affected | Visual Design Contract | task 17 | `docs/design/design-system.md` | yes |
| Styled frame with plain/ANSI/role projections | `docs/arc42/04-solution-strategy.md:77` | Affected | D1 | tasks 1-16 | `passuff/tui/frame.py:155-454` | yes |
| `TerminalPresentation` composes the frame, tables, controls, and overlays | `docs/arc42/05-building-block-view.md:72-73` | Affected | Modules | tasks 1-16 | `frame.py:126-152,248,286,298,320,336,381,402,410,412`; the composition exists but the outline widths are finding V5 | yes (composition present; V5 is visual) |
| Save prompt on quit | `docs/arc42/06-runtime-view.md:344-353` | Affected | save-prompt decision | tasks 11-14, 19 | `app.py:113-133`; `frame.py:405-410` | yes |
| Palette roles and accessibility | `docs/arc42/08-crosscutting-concepts.md:134-142` | Affected | palette model | tasks 1-16 | `frame.py:30-42` | yes (the help row role collapse is V6) |
| QS-034 visual fidelity follows the design system | `docs/arc42/10-quality-requirements.md:66` | Affected | Visual Design Contract | task 17 | the frame outlines are not closed (V5) and the help reference is clipped/collapsed (V6) | no |
| R-037 palette/composition drift | `docs/arc42/11-risks-and-technical-debt.md:43` | Affected | Risks | task 17 | the composition drift is present in the frame outlines (V5) and the help row (V6) | no |
| Chapter 12 links the glossary | `docs/arc42/12-glossary.md` | Affected | glossary task | task 17 | `docs/glossary.md:63-67` names `PALETTE`, role strings, `render_styled`, `render_roles`, and the snapshot flags, all present (D2 fixed) | yes |

`ARC42 CONFORMANCE: FINDINGS`; the visual fidelity rows (10, 11) do not match, so this block is not `CLEAN`.

## Ubiquitous language conformance

The glossary adds `Watn palette` (`PALETTE`), `Palette role` (role-name strings), `Styled frame` (`render_styled`), `Role record` (`render_roles`), and `Rendered snapshot` (the snapshot flags) at `docs/glossary.md:63-67`. `passuff/tui/frame.py:30-42` owns `PALETTE`; `render_styled`, `render_roles`, and the snapshot flags exist; the design system and `design.md` use the same role names and no longer name `ROLE_STYLES` or `Role`. `docs/domain-model.md:154` correctly says the visual layer adds no domain structure. D2 is resolved. No stale old-value sweep is outstanding.

## Use-case and Persona conformance

The three owning use cases keep their goals, triggers, guarantees, typed relationships, and capability ownership; the delta changes presentation and the T22 save prompt, not the domain structure. `review-pull-request-stacks` still extends `review-activity-inbox`; `change-watch-scope` remains separate. T25's palette matches the design system, and the composition elements exist; the frame outlines are not faithful (V5). T22 and T26 hold: the save prompt appears only for changed sort, direction, filters, ignored view, or notification settings; `y` persists, `n` discards, `Esc` cancels; the settings toggle updates the live `config.notifications` (`passuff/tui/app.py:299-305`) that `ObservationSession.deliver` reads (`passuff/tui/session.py:27-38`), so the immediate-application statement holds (U1 resolved). The save prompt defers persistence to `y` (`app.py:113-133`).

Persona disposition:

| Persona | Confirmed | Outcome in this change |
|---|---|---|
| `multi-account-developer` | yes, 2026-11-02 | The main inbox, activity detail, PR detail, settings, save prompt, help, picker, and stack surfaces are committed as real captures and role records. The Persona's functional goals are reachable; the visual contract has the frame-outline defect (V5) and the clipped help reference (V6). |

PERSONAS: multi-account-developer

## Coverage

Measured evidence, all regenerated after the implementation and distinct files with branch instrumentation: `coverage/non-e2e-cobertura.xml` 2048/2366 lines (86.56%), 712/896 branches (79.46%); `coverage/e2e-cobertura.xml` 1534/2366 (64.84%), 392/896 (43.75%); `coverage/cobertura-coverage.xml` 2081/2366 (87.95%), 729/896 (81.36%). In the merged report `tui/frame.py` is 339/359 lines and `tui/app.py` 286/422.

The measurement commands run only the regular and E2E scopes (`scripts/measure-coverage.sh:17,22`), so the delta non-E2E visual scenarios are outside the Cobertura reports. That is a measurement boundary, not missing tests: the settings overlay, PR detail, help, save prompt, 80x24, ignored-row, empty-state, and picker scenarios run in the change scope and are inside the 264/264 receipt. The uncovered lines `tui/app.py:127-132,439-443` and `tui/frame.py:344,373,376,404,419-420` are those delta-only and interactive paths.

Bucket 1 (dead code): none. `markdown_lines` and `render_from_paths` are gone from `passuff/tui/frame.py`; `grep` finds them only in the historical review/design-review prose (D1 resolved). No other changed symbol is unused.

Bucket 2 (missing test coverage): the residual role-precision gap (F2) and the visual-contract assertions (V5, V6). The unfocused-border and unfocused-title steps are identical line-level checks, the picker border and suffix steps are identical, and the metadata-values step can be satisfied by the dim border on the metadata line. The frame outlines and the help row are not asserted at all. These are missing assertions, not hard-to-test code; the fix is span-specific assertions plus the renderer corrections.

Bucket 3 (legitimately hard to test): the curses color-pair initialization and the 256-color fallback (`passuff/tui/app.py:346-370`) depend on the real terminal; the role record and ANSI snapshot are the terminal-independent assertions, and the design records the fallback as a hard-to-test risk. The interactive loop's start/quit is covered by the regular pty smoke. This row is justified.

No redundant unit tests: the `.feature` corpus is the only test surface.

## Visual review

The design declares `Interface: terminal` (`design.md:168-197`) with the 80x24 minimum, the 100x30 default snapshot, and the palette/composition in `docs/design/design-system.md`. The committed evidence is complete: 27 visual scenario directories, 13 role records, 18 ANSI transcripts, the eight real captures under `evidence/screens/real/`, the three comparisons, the three reference frames, and the durable main capture `docs/design/passuff-tui-alacritty.png` (byte-identical to `evidence/screens/real/main-inbox.png`, with the 81/15/214/114/252 palette in the pixels). V3 and V4 are resolved.

The states the review saw:

| Evidence | State seen |
|---|---|
| `evidence/screens/real/main-inbox.png` | main inbox; outer frame and title, account/watch-scope header, counts/focus, bordered watched table with stack column (`☰`, 1/2, 2/2), bordered activity table, bordered controls; accent headers, gold selected rows, green focused activity border, dim unfocused border |
| `evidence/screens/real/main-selected.png` | watched pane focused: green border and title, gold selected row and `▶`, dim unfocused activity border |
| `evidence/screens/real/activity-detail.png` | activity detail overlay: centered dim-bordered block over a cleared activity region, accent-bold title, accent labels, dim values, rule separator, bright-gray body, accent-bold hint keys |
| `evidence/screens/real/pr-detail.png` | PR detail overlay: centered dim-bordered block, accent-bold title, metadata labels accent and values dim, green-bold description heading, bright-gray body |
| `evidence/screens/real/settings-overlay.png` | settings overlay: centered dim-bordered block, accent-bold title, gold selected row, green enabled value, dim disabled value |
| `evidence/screens/real/save-prompt.png` | save prompt: centered dim-bordered block titled `Leave`, bright question, accent-bold keys |
| `evidence/screens/real/help-overlay.png` | help overlay: dim-bordered block, accent-bold title and headings; the second key row is clipped and all-cyan (V6) |
| `evidence/screens/real/scope-picker.png` | watch-scope picker: centered dim-bordered block, bright-bold brand, dim suffix, accent-bold mode, green focus marks, gold selected repository |
| `evidence/screens/comparisons/*.png` | side-by-side original/rebuilt main inbox, selected PR, and settings overlay; composition, palette, focus, selection, headers, and overlay placement compared |
| `evidence/reference/*.png` | the original frames; every original bordered block closes its outline at the top-right corner |
| `evidence/visual/the-80x24-frame-keeps-the-composition/`, `an-ignored-row-is-dim-and-marked/`, `the-empty-inbox-uses-the-primary-text/`, `a-missing-browser-opener-is-reported-and-keeps-the-detail/` | 80x24 composition, dim `⊘` ignored row, bright empty state, gold status line |

Findings against the contract:

- **V5 (blocking):** the rendered frame outlines do not close. The outer frame's top edge is four cells wider than its sides (`frame.py:414` builds `3 + 8 + 8 + (width - 16) + 1 = width + 4`; the transcript `evidence/visual/developer-opens-the-inbox-tui/transcript.txt:1` is 104 characters against 100 for the body). Every `_box` top edge is one cell narrower than its sides (`frame.py:127`; the settings transcript `.../the-settings-overlay-follows-the-palette/transcript.txt:7-8` has `┐` at column 81 and the body `│` at 82; the watched/activity/controls boxes show the same at 96 vs 97). The picker's top edge is six cells wider (`frame.py:337`; `.../the-scope-picker-renders-the-watn-palette/transcript.txt:5-6` has `┐` at 85 and the body `│` at 79). The real captures show the disconnected corners: `main-inbox.png`, `settings-overlay.png`, `save-prompt.png`, `pr-detail.png`, `help-overlay.png`, `activity-detail.png`, and `scope-picker.png` (the picker top edge visibly overhangs the body). The reference `reference/main-inbox.png` closes the outer frame at the top-right. This contradicts `design-system.md:40-41` (outer frame), `:48,56,60` (bordered blocks), and `:64` (modal overlays use a dim bordered block). A frame whose top edge does not meet its side is not the original composition; the Persona's visual-fidelity goal is not met on every required surface.
- **V6 (blocking):** the help key reference is clipped and its second row collapses to one accent-bold span. `frame.py:310` defines a row longer than the overlay; `_box` truncates it (`frame.py:131-133`) and rewrites the line with the first span's role, so the labels lose the dim role and the tail (`p watch scope`, `q close`) disappears. The committed role record `evidence/visual/the-developer-opens-the-key-reference/roles.json` shows the second key row as a single `accent_bold` span ending `, `; the real `help-overlay.png` shows the row all-cyan, ending `setting`, with the rest missing. The first row in the 100x30 snapshot is truncated mid-word (`... mark visible rea`). This contradicts `design-system.md:73` (help key/label rows) and the README's "full keymap" claim (`README.md:92`); the Persona opening the key reference cannot read the complete reference and cannot distinguish keys from labels on that row.

The palette and focus/selection roles are otherwise correct: the main, activity, PR, settings, save, and picker roles match `design-system.md` and the original (accent 81, bright 15, body 252, gold 214, green 114, rule 240, dim). The overlays are centered over a cleared rectangle inside the main frame, matching the original overlay placement (V2's defect is fixed); the watched, activity, and controls panes are bordered blocks (V1's missing-block defect is fixed), but their outlines are malformed (V5).

Persona judgment: the `multi-account-developer` can reach the inbox, activity, PR detail, settings, save, help, picker, and stack goals, and the palette is restored. The Persona's visual contract is not fully met because the frames do not close and the help reference is incomplete. `VISUAL-REVIEW: FAIL`.

## Scenario execution evidence

Receipt `givn/changes/restore-tui-styling/verification/tasks-verification.json`: status `passed`, head `7f0470683f9f186901f15c3094388cdae8cf74a2`, 264 total, 264 passed, 0 failed, 0 skipped; `tasks-result.json` records `{"failed":0,"passed":264,"skipped":0,"total":264}`. `tasks.md:16,199` records regular 247/247 and E2E 22/22. `./scripts/verify.sh --strictness` exits 0. The receipt is reused at `657b447`; the gate reports "documentation-only revision". Review does not re-run the suite.

## Overlap dispositions

| Scenario A | Scenario B | Decision |
|---|---|---|
| The developer opens the key reference | The developer quits without a save prompt when settings are unchanged | variant |
| The developer confirms saving settings on quit | The developer discards settings on quit | boundary |

## Split-or-keep

| Scenario | Decision | Keep reason |
|---|---|---|
| Developer opens the inbox TUI | keep | One frame and one palette contract; splitting would duplicate setup and fragment the composition. |
| Developer opens an activity's details | keep | One overlay contract; the role assertions describe one frame. |
| Developer navigates the inbox panes | keep | One focus/selection contract; splitting would duplicate the frame. |
| The pull request detail shows its metadata | keep | One overlay contract; the metadata and body roles describe one frame. |
| The developer mutes notifications from the settings dialog | keep | One settings-save regression path; the steps are sequential branches of one quit contract. |
| Developer reviews a stacked inbox | keep | One stack-column contract; splitting would duplicate the frame. |

## Substantive overlap dispositions

| Scenario A | Scenario B | Decision |
|---|---|---|
| The developer opens the key reference | Developer opens the pull request from an activity | variant |
| The developer quits without a save prompt when settings are unchanged | Developer opens the pull request from an activity | variant |
| The developer opens the key reference | A rejected stack query retries once and shows standalone rows | variant |
| The developer quits without a save prompt when settings are unchanged | A rejected stack query retries once and shows standalone rows | variant |
| The 80x24 frame keeps the composition | Developer opens the inbox TUI | variant |
| The developer opens the key reference | The developer confirms saving settings on quit | variant |
| The developer quits without a save prompt when settings are unchanged | The developer confirms saving settings on quit | variant |
| The developer opens the key reference | The developer discards settings on quit | variant |
| The developer quits without a save prompt when settings are unchanged | The developer discards settings on quit | variant |
| The developer opens the key reference | The developer cancels the quit from the save prompt | variant |
| The developer confirms saving settings on quit | The developer cancels the quit from the save prompt | variant |
| The developer discards settings on quit | The developer cancels the quit from the save prompt | variant |
| The developer quits without a save prompt when settings are unchanged | The developer cancels the quit from the save prompt | variant |
| Developer switches the watch scope mode | The scope picker renders the watn palette | variant |
| Switching to selected with no retained set observes nothing | Developer opens the inbox TUI | variant |
| Switching to all observes every repository | Developer opens the inbox TUI | variant |
| Reselecting catches up silently | Developer opens the inbox TUI | variant |
| Switching to selected with no retained set observes nothing | Developer opens an activity's details | variant |
| Switching to all observes every repository | Developer opens an activity's details | variant |
| Reselecting catches up silently | Developer opens an activity's details | variant |
| Switching to selected with no retained set observes nothing | An ignored row is dim and marked | variant |
| Reselecting catches up silently | An ignored row is dim and marked | variant |
| Switching to selected with no retained set observes nothing | The empty inbox uses the primary text | variant |
| Switching to all observes every repository | The empty inbox uses the primary text | variant |
| Reselecting catches up silently | The empty inbox uses the primary text | variant |
| Switching to selected with no retained set observes nothing | Developer navigates the inbox panes | variant |
| Switching to all observes every repository | Developer navigates the inbox panes | variant |
| Reselecting catches up silently | Developer navigates the inbox panes | variant |
| Switching to selected with no retained set observes nothing | The settings overlay follows the palette | variant |
| Reselecting catches up silently | The settings overlay follows the palette | variant |
| Switching to selected with no retained set observes nothing | The pull request detail shows its metadata | variant |
| Switching to all observes every repository | The pull request detail shows its metadata | variant |
| Reselecting catches up silently | The pull request detail shows its metadata | variant |
| A deselected repository stops observation | The developer opens the key reference | variant |
| Switching to selected with no retained set observes nothing | A missing browser opener is reported and keeps the detail | variant |
| Switching to all observes every repository | A missing browser opener is reported and keeps the detail | variant |
| Reselecting catches up silently | A missing browser opener is reported and keeps the detail | variant |
| Switching to selected with no retained set observes nothing | The 80x24 frame keeps the composition | variant |
| Reselecting catches up silently | The 80x24 frame keeps the composition | variant |
| A deselected repository stops observation | The developer confirms saving settings on quit | variant |
| A deselected repository stops observation | The developer discards settings on quit | variant |
| A deselected repository stops observation | The developer cancels the quit from the save prompt | variant |
| Switching to selected with no retained set observes nothing | The developer cancels the quit from the save prompt | variant |
| Reselecting catches up silently | The developer cancels the quit from the save prompt | variant |
| A deselected repository stops observation | The developer quits without a save prompt when settings are unchanged | variant |
| A deselected repository stops observation | The developer mutes notifications from the settings dialog | variant |
| Switching to selected with no retained set observes nothing | Developer reviews a stacked inbox | variant |
| Switching to all observes every repository | Developer reviews a stacked inbox | variant |
| Reselecting catches up silently | Developer reviews a stacked inbox | variant |
| The terminal UI shows the empty inbox | Developer opens the inbox TUI | variant |
| A failed browser launch keeps the activity detail open | Developer opens the inbox TUI | variant |
| The terminal UI shows the empty inbox | Developer opens an activity's details | variant |
| Developer opens the inbox TUI | Developer opens an activity's details | variant |
| A failed browser launch keeps the activity detail open | Developer opens an activity's details | variant |
| The terminal UI shows the empty inbox | The empty inbox uses the primary text | variant |
| Developer opens the inbox TUI | The empty inbox uses the primary text | variant |
| A failed browser launch keeps the activity detail open | The empty inbox uses the primary text | variant |
| The terminal UI shows the empty inbox | Developer navigates the inbox panes | variant |
| Developer opens the inbox TUI | Developer navigates the inbox panes | variant |
| A failed browser launch keeps the activity detail open | Developer navigates the inbox panes | variant |
| The terminal UI shows the empty inbox | The pull request detail shows its metadata | variant |
| Developer opens the inbox TUI | The pull request detail shows its metadata | variant |
| A failed browser launch keeps the activity detail open | The pull request detail shows its metadata | variant |
| The terminal UI shows the empty inbox | A missing browser opener is reported and keeps the detail | variant |
| Developer opens the inbox TUI | A missing browser opener is reported and keeps the detail | variant |
| A failed browser launch keeps the activity detail open | A missing browser opener is reported and keeps the detail | variant |
| Escape closes the activity detail | The developer cancels the quit from the save prompt | variant |
| The terminal UI shows the empty inbox | Developer reviews a stacked inbox | variant |
| Developer opens the inbox TUI | Developer reviews a stacked inbox | variant |
| A failed browser launch keeps the activity detail open | Developer reviews a stacked inbox | variant |
| Developer navigates the inbox panes | Developer opens the inbox TUI | variant |
| The focus cycles between watched pull requests and activity | Developer opens the inbox TUI | variant |
| The watched pull request selection scrolls | Developer opens the inbox TUI | variant |
| A long pull request key keeps its number visible | Developer opens the inbox TUI | variant |
| The watched table shows the creator and the assignee | Developer opens the inbox TUI | variant |
| A stack is marked with a symbol and its position | Developer opens the inbox TUI | variant |
| The developer sorts the watched list | Developer opens the inbox TUI | variant |
| The developer filters the activity to a stack | Developer opens the inbox TUI | variant |
| The developer ignores and restores a pull request | Developer opens the inbox TUI | variant |
| The inbox shows activity observed while it is open | Developer opens the inbox TUI | variant |
| The inbox reconciles open pull requests when the interval elapses | Developer opens the inbox TUI | variant |
| The activity table shows when the activity happened | Developer opens the inbox TUI | variant |
| The activity detail shows when the activity happened and when it was fetched | Developer opens the inbox TUI | variant |
| The activity detail marks an activity without a reported time as approximate | Developer opens the inbox TUI | variant |
| A missing browser opener is reported and keeps the detail | Developer opens the inbox TUI | variant |
| Developer navigates the inbox panes | Developer opens an activity's details | variant |
| The focus cycles between watched pull requests and activity | Developer opens an activity's details | variant |
| The watched pull request selection scrolls | Developer opens an activity's details | variant |
| A long pull request key keeps its number visible | Developer opens an activity's details | variant |
| The watched table shows the creator and the assignee | Developer opens an activity's details | variant |
| A stack is marked with a symbol and its position | Developer opens an activity's details | variant |
| The developer sorts the watched list | Developer opens an activity's details | variant |
| The developer filters the activity to a stack | Developer opens an activity's details | variant |
| The pull request detail shows its metadata | Developer opens an activity's details | variant |
| The developer ignores and restores a pull request | Developer opens an activity's details | variant |
| The inbox shows activity observed while it is open | Developer opens an activity's details | variant |
| The inbox reconciles open pull requests when the interval elapses | Developer opens an activity's details | variant |
| The activity table shows when the activity happened | Developer opens an activity's details | variant |
| The activity detail shows when the activity happened and when it was fetched | Developer opens an activity's details | variant |
| The activity detail marks an activity without a reported time as approximate | Developer opens an activity's details | variant |
| A missing browser opener is reported and keeps the detail | Developer opens an activity's details | variant |
| The focus cycles between watched pull requests and activity | An ignored row is dim and marked | variant |
| A long pull request key keeps its number visible | An ignored row is dim and marked | variant |
| The developer sorts the watched list | An ignored row is dim and marked | variant |
| The developer ignores and restores a pull request | An ignored row is dim and marked | variant |
| The inbox reconciles open pull requests when the interval elapses | An ignored row is dim and marked | variant |
| The activity table shows when the activity happened | An ignored row is dim and marked | variant |
| Developer navigates the inbox panes | The empty inbox uses the primary text | variant |
| The focus cycles between watched pull requests and activity | The empty inbox uses the primary text | variant |
| The watched pull request selection scrolls | The empty inbox uses the primary text | variant |
| A long pull request key keeps its number visible | The empty inbox uses the primary text | variant |
| The watched table shows the creator and the assignee | The empty inbox uses the primary text | variant |
| A stack is marked with a symbol and its position | The empty inbox uses the primary text | variant |
| The developer sorts the watched list | The empty inbox uses the primary text | variant |
| The developer filters the activity to a stack | The empty inbox uses the primary text | variant |
| The developer ignores and restores a pull request | The empty inbox uses the primary text | variant |
| The inbox shows activity observed while it is open | The empty inbox uses the primary text | variant |
| The inbox reconciles open pull requests when the interval elapses | The empty inbox uses the primary text | variant |
| The activity table shows when the activity happened | The empty inbox uses the primary text | variant |
| The activity detail shows when the activity happened and when it was fetched | The empty inbox uses the primary text | variant |
| The activity detail marks an activity without a reported time as approximate | The empty inbox uses the primary text | variant |
| A missing browser opener is reported and keeps the detail | The empty inbox uses the primary text | variant |
| The focus cycles between watched pull requests and activity | Developer navigates the inbox panes | variant |
| The watched pull request selection scrolls | Developer navigates the inbox panes | variant |
| A long pull request key keeps its number visible | Developer navigates the inbox panes | variant |
| The watched table shows the creator and the assignee | Developer navigates the inbox panes | variant |
| A stack is marked with a symbol and its position | Developer navigates the inbox panes | variant |
| The developer sorts the watched list | Developer navigates the inbox panes | variant |
| The developer filters the activity to a stack | Developer navigates the inbox panes | variant |
| The developer ignores and restores a pull request | Developer navigates the inbox panes | variant |
| The inbox shows activity observed while it is open | Developer navigates the inbox panes | variant |
| The inbox reconciles open pull requests when the interval elapses | Developer navigates the inbox panes | variant |
| The activity table shows when the activity happened | Developer navigates the inbox panes | variant |
| The activity detail shows when the activity happened and when it was fetched | Developer navigates the inbox panes | variant |
| The activity detail marks an activity without a reported time as approximate | Developer navigates the inbox panes | variant |
| A missing browser opener is reported and keeps the detail | Developer navigates the inbox panes | variant |
| The focus cycles between watched pull requests and activity | The settings overlay follows the palette | variant |
| A long pull request key keeps its number visible | The settings overlay follows the palette | variant |
| The developer sorts the watched list | The settings overlay follows the palette | variant |
| The developer ignores and restores a pull request | The settings overlay follows the palette | variant |
| The inbox reconciles open pull requests when the interval elapses | The settings overlay follows the palette | variant |
| The activity table shows when the activity happened | The settings overlay follows the palette | variant |
| Developer navigates the inbox panes | The pull request detail shows its metadata | variant |
| The focus cycles between watched pull requests and activity | The pull request detail shows its metadata | variant |
| The watched pull request selection scrolls | The pull request detail shows its metadata | variant |
| A long pull request key keeps its number visible | The pull request detail shows its metadata | variant |
| The watched table shows the creator and the assignee | The pull request detail shows its metadata | variant |
| A stack is marked with a symbol and its position | The pull request detail shows its metadata | variant |
| The developer sorts the watched list | The pull request detail shows its metadata | variant |
| The developer filters the activity to a stack | The pull request detail shows its metadata | variant |
| The developer ignores and restores a pull request | The pull request detail shows its metadata | variant |
| The inbox shows activity observed while it is open | The pull request detail shows its metadata | variant |
| The inbox reconciles open pull requests when the interval elapses | The pull request detail shows its metadata | variant |
| The activity table shows when the activity happened | The pull request detail shows its metadata | variant |
| The activity detail shows when the activity happened and when it was fetched | The pull request detail shows its metadata | variant |
| The activity detail marks an activity without a reported time as approximate | The pull request detail shows its metadata | variant |
| A missing browser opener is reported and keeps the detail | The pull request detail shows its metadata | variant |
| A standalone pull request shows no stack symbol | The developer opens the key reference | variant |
| The developer closes dialogs with q | The developer opens the key reference | variant |
| The stored activity body is bounded to 4 KiB | The developer opens the key reference | variant |
| Developer navigates the inbox panes | A missing browser opener is reported and keeps the detail | variant |
| The focus cycles between watched pull requests and activity | A missing browser opener is reported and keeps the detail | variant |
| The watched pull request selection scrolls | A missing browser opener is reported and keeps the detail | variant |
| A long pull request key keeps its number visible | A missing browser opener is reported and keeps the detail | variant |
| The watched table shows the creator and the assignee | A missing browser opener is reported and keeps the detail | variant |
| A stack is marked with a symbol and its position | A missing browser opener is reported and keeps the detail | variant |
| The developer sorts the watched list | A missing browser opener is reported and keeps the detail | variant |
| The developer filters the activity to a stack | A missing browser opener is reported and keeps the detail | variant |
| The developer ignores and restores a pull request | A missing browser opener is reported and keeps the detail | variant |
| The inbox shows activity observed while it is open | A missing browser opener is reported and keeps the detail | variant |
| The inbox reconciles open pull requests when the interval elapses | A missing browser opener is reported and keeps the detail | variant |
| The activity table shows when the activity happened | A missing browser opener is reported and keeps the detail | variant |
| The activity detail shows when the activity happened and when it was fetched | A missing browser opener is reported and keeps the detail | variant |
| The activity detail marks an activity without a reported time as approximate | A missing browser opener is reported and keeps the detail | variant |
| The focus cycles between watched pull requests and activity | The 80x24 frame keeps the composition | variant |
| A long pull request key keeps its number visible | The 80x24 frame keeps the composition | variant |
| The developer sorts the watched list | The 80x24 frame keeps the composition | variant |
| The developer ignores and restores a pull request | The 80x24 frame keeps the composition | variant |
| The inbox reconciles open pull requests when the interval elapses | The 80x24 frame keeps the composition | variant |
| The activity table shows when the activity happened | The 80x24 frame keeps the composition | variant |
| A standalone pull request shows no stack symbol | The developer confirms saving settings on quit | variant |
| The developer closes dialogs with q | The developer confirms saving settings on quit | variant |
| The developer opens the key reference | The developer confirms saving settings on quit | variant |
| The stored activity body is bounded to 4 KiB | The developer confirms saving settings on quit | variant |
| A standalone pull request shows no stack symbol | The developer discards settings on quit | variant |
| The developer closes dialogs with q | The developer discards settings on quit | variant |
| The developer opens the key reference | The developer discards settings on quit | variant |
| The stored activity body is bounded to 4 KiB | The developer discards settings on quit | variant |
| The focus cycles between watched pull requests and activity | The developer cancels the quit from the save prompt | variant |
| A long pull request key keeps its number visible | The developer cancels the quit from the save prompt | variant |
| A standalone pull request shows no stack symbol | The developer cancels the quit from the save prompt | variant |
| The developer sorts the watched list | The developer cancels the quit from the save prompt | variant |
| The developer closes dialogs with q | The developer cancels the quit from the save prompt | variant |
| The developer opens the key reference | The developer cancels the quit from the save prompt | variant |
| The developer ignores and restores a pull request | The developer cancels the quit from the save prompt | variant |
| The inbox reconciles open pull requests when the interval elapses | The developer cancels the quit from the save prompt | variant |
| The activity table shows when the activity happened | The developer cancels the quit from the save prompt | variant |
| Changing the focus in the picker filters the inbox immediately | The developer cancels the quit from the save prompt | variant |
| The stored activity body is bounded to 4 KiB | The developer cancels the quit from the save prompt | variant |
| A standalone pull request shows no stack symbol | The developer quits without a save prompt when settings are unchanged | variant |
| The developer closes dialogs with q | The developer quits without a save prompt when settings are unchanged | variant |
| The developer opens the key reference | The developer quits without a save prompt when settings are unchanged | variant |
| The stored activity body is bounded to 4 KiB | The developer quits without a save prompt when settings are unchanged | variant |
| A standalone pull request shows no stack symbol | The developer mutes notifications from the settings dialog | variant |
| The developer closes dialogs with q | The developer mutes notifications from the settings dialog | variant |
| The developer opens the key reference | The developer mutes notifications from the settings dialog | variant |
| The stored activity body is bounded to 4 KiB | The developer mutes notifications from the settings dialog | variant |
| Developer navigates the inbox panes | Developer reviews a stacked inbox | variant |
| The focus cycles between watched pull requests and activity | Developer reviews a stacked inbox | variant |
| The watched pull request selection scrolls | Developer reviews a stacked inbox | variant |
| A long pull request key keeps its number visible | Developer reviews a stacked inbox | variant |
| The watched table shows the creator and the assignee | Developer reviews a stacked inbox | variant |
| A stack is marked with a symbol and its position | Developer reviews a stacked inbox | variant |
| The developer sorts the watched list | Developer reviews a stacked inbox | variant |
| The developer filters the activity to a stack | Developer reviews a stacked inbox | variant |
| The developer ignores and restores a pull request | Developer reviews a stacked inbox | variant |
| The inbox shows activity observed while it is open | Developer reviews a stacked inbox | variant |
| The inbox reconciles open pull requests when the interval elapses | Developer reviews a stacked inbox | variant |
| The activity table shows when the activity happened | Developer reviews a stacked inbox | variant |
| The activity detail shows when the activity happened and when it was fetched | Developer reviews a stacked inbox | variant |
| The activity detail marks an activity without a reported time as approximate | Developer reviews a stacked inbox | variant |
| A missing browser opener is reported and keeps the detail | Developer reviews a stacked inbox | variant |
| Developer reviews a stacked inbox | Developer opens the inbox TUI | variant |
| A merged stack member remains in the group | Developer opens the inbox TUI | variant |
| The activity keeps the stack label | Developer opens the inbox TUI | variant |
| Developer reviews a stacked inbox | Developer opens an activity's details | variant |
| A merged stack member remains in the group | Developer opens an activity's details | variant |
| The activity keeps the stack label | Developer opens an activity's details | variant |
| The activity keeps the stack label | The empty inbox uses the primary text | variant |
| The activity keeps the stack label | Developer navigates the inbox panes | variant |
| Developer reviews a stacked inbox | The pull request detail shows its metadata | variant |
| A merged stack member remains in the group | The pull request detail shows its metadata | variant |
| The activity keeps the stack label | The pull request detail shows its metadata | variant |
| The activity keeps the stack label | A missing browser opener is reported and keeps the detail | variant |
| A merged stack member remains in the group | Developer reviews a stacked inbox | variant |
| The activity keeps the stack label | Developer reviews a stacked inbox | variant |
| Developer sets up passuff on first run | Developer opens the inbox TUI | variant |
| Developer sets up passuff on first run | Developer opens an activity's details | variant |
| Developer sets up passuff on first run | An ignored row is dim and marked | variant |
| Developer sets up passuff on first run | The empty inbox uses the primary text | variant |
| Developer sets up passuff on first run | Developer navigates the inbox panes | variant |
| Developer sets up passuff on first run | The settings overlay follows the palette | variant |
| Developer sets up passuff on first run | The pull request detail shows its metadata | variant |
| Developer sets up passuff on first run | A missing browser opener is reported and keeps the detail | variant |
| Developer sets up passuff on first run | The 80x24 frame keeps the composition | variant |
| Developer sets up passuff on first run | The developer cancels the quit from the save prompt | variant |
| Developer sets up passuff on first run | Developer reviews a stacked inbox | variant |

## README impact

README-IMPACT: updated - Terminal inbox, Coverage

The Terminal inbox key table now lists the sort (`s`/`S`), filter (`f`/`F`), unread (`u`), clear (`c`/`C`), settings (`,`), scope (`p`), and save-prompt (`y`/`n`/`Esc`) keys (`README.md:77-101`). The coverage badges and summary report 88%/81% (2081/2366, 729/896), matching `coverage/cobertura-coverage.xml` (`README.md:3-5,166-167`). R1 is resolved.

## Findings

| ID | Severity | Finding | Evidence | Status |
|---|---|---|---|---|
| V5 | blocking | The frame outlines do not close: the outer frame top is four cells wider than its sides, every `_box` top is one cell narrower, and the picker top is six cells wider. The reference frames close every box. | `passuff/tui/frame.py:127,337,414`; `evidence/visual/developer-opens-the-inbox-tui/transcript.txt:1` (104 vs 100); `evidence/visual/the-scope-picker-renders-the-watn-palette/transcript.txt:5-6` (85 vs 79); `evidence/visual/the-settings-overlay-follows-the-palette/transcript.txt:7-8` (81 vs 82); `evidence/screens/real/{main-inbox,settings-overlay,save-prompt,pr-detail,help-overlay,activity-detail,scope-picker}.png`; `evidence/reference/main-inbox.png`; `docs/design/design-system.md:40-41,48,56,60,64` | open (blocking) |
| V6 | blocking | The help key reference is clipped mid-word and its second row collapses to one accent-bold span: the labels lose the dim role and `p watch scope`/`q close` are absent; the README promises the full keymap. | `passuff/tui/frame.py:310,131-133`; `evidence/visual/the-developer-opens-the-key-reference/roles.json` (second key row is one `accent_bold` span ending `, `); `evidence/screens/real/help-overlay.png` (all-cyan row ending `setting`); `docs/design/design-system.md:73`; `README.md:92` | open (blocking) |
| F2 | major | The recorded F1 steps are line-specific now, but the residual role assertions are not span-specific: the unfocused-border and unfocused-title steps are identical, the picker border and suffix steps are identical, and the metadata-values step is satisfied by the dim border on the metadata line. | `givn/steps/inbox_navigation_steps.py:272-279`; `givn/steps/watch_scope_steps.py:137-149`; `givn/steps/common.py:257-264`; `givn/steps/support.py:109-116`; the metadata role records under `evidence/visual/` | open (bucket 2) |
| D7 | minor | `scripts/verify.sh:48-54` exports an empty `GIVN_CHANGE_ID` when more than one active change directory exists instead of failing as `design.md:161-163` promises; the harness's single-specs-root fallback masks it. | `scripts/verify.sh:48-54`; `tests/support/harness.py:50-56`; `design.md:161-163`; `givn/changes/` contains `python-rebuild/` and `restore-tui-styling/` | open (minor; fix with V5/V6) |
| T1 | minor | The tasks acceptance line still says 263/263 while the receipt and the rest of tasks.md record 264/264. | `givn/changes/restore-tui-styling/tasks.md:3` vs `:16,199` and `verification/tasks-verification.json` | open (minor; fix with V5/V6) |
| V1 | resolved | The watched, activity, and controls panes are bordered `_box` blocks with the focused block green, the unfocused block dim, and the selected rows gold. | `passuff/tui/frame.py:126-152,248,286,298`; `evidence/screens/real/main-inbox.png`, `main-selected.png` | resolved (the outline widths are V5) |
| V2 | resolved | Overlays are centered by `_composite` over a cleared rectangle inside the main frame; the main behind is replaced within the overlay rectangle. | `passuff/tui/frame.py:140-152,411-412`; `evidence/screens/real/settings-overlay.png`, `save-prompt.png`, `pr-detail.png`, `help-overlay.png`, `scope-picker.png`, `activity-detail.png`; `evidence/reference/settings-overlay.png` | resolved (the outline widths are V5) |
| V3 | resolved | Real Alacritty captures exist for the main inbox, selected PR, activity detail, PR detail, settings, save prompt, help, and scope picker; the comparisons and index are committed; transcripts/ANSI/roles are committed for the regular visual scenarios. | `evidence/screens/real/` (8 PNGs); `evidence/screens/comparisons/` (3 PNGs); `evidence/screens/README.md`; `evidence/visual/` (27 dirs, 13 roles, 18 ANSI) | resolved |
| V4 | resolved | The curses color pairs are initialized inside the loop after `initscr`; the real captures show the gold/green/cyan palette; the durable capture is styled. | `passuff/tui/app.py:346-370`; `docs/design/passuff-tui-alacritty.png` (byte-identical to `evidence/screens/real/main-inbox.png`, colored pixels in the 81/214/114 palette) | resolved |
| F1 | resolved | The recorded non-discriminating steps are line-specific and target the overlay, metadata, heading, and picker lines. | `givn/steps/common.py:257-264`; `givn/steps/activity_inbox_steps.py:230-237`; `givn/steps/inbox_navigation_steps.py:282-354`; `givn/steps/watch_scope_steps.py:137-154` | resolved (the residual span precision is F2) |
| D1 | resolved | `markdown_lines` and `render_from_paths` are removed; no consumer remains. | `grep` finds them only in the historical review/design-review prose; `passuff/tui/frame.py` ends at `render_roles` (`:449-454`) | resolved |
| D2 | resolved | The docs name `PALETTE` and role-name strings; no `ROLE_STYLES` or `Role` symbol remains. | `docs/glossary.md:63-67`; `givn/changes/restore-tui-styling/design.md:86,121-128`; `docs/design/design-system.md:13-25`; `passuff/tui/frame.py:30-42` | resolved |
| U1 | resolved | The settings toggle updates the live `config.notifications`, which `ObservationSession.deliver` reads; the save prompt defers persistence to `y`. | `passuff/tui/app.py:113-133,299-305`; `passuff/tui/session.py:27-38`; T22/T26 in `givn/specs/review-activity-inbox/usecase.md:69,74` | resolved |
| R1 | resolved | The README key table and coverage evidence are refreshed. | `README.md:77-101,3-5,166-167`; `coverage/cobertura-coverage.xml` | resolved |
| P1 | resolved | The proposal has the filled four-column capability routing table with `Route's recommendation` and the four EXTEND decisions. | `givn/changes/restore-tui-styling/proposal.md:79-89` | resolved |
| R4-17 | resolved | The design-review round 4 recorded 16 delta entries; the 17th scenario is the T22 settings-persistence regression (`The developer mutes notifications from the settings dialog`), added after that review. It is a variant/regression of the existing save-prompt rule, not a new behavior; `design.md` records 17. | `design.md:307-312`; `tasks.md:203-212`; `inbox-navigation.feature` (the settings-mute scenario); commit `7efd2c8` | resolved |

## Sign-off

- [x] Fabrication audit: clean - `check_steps.py` reports 0 no-op bodies across the 17 step files.
- [x] Every checked task has a verified commit touching production code - 40/40 checked; `91422ab` and `7efd2c8` contain the production, step, feature, task, and evidence changes with batch names.
- [x] Every promised component exists - the frame model, `PALETTE`, projections, save prompt, runner, harness, and bindings exist; `TerminalPresentation` is `passuff.tui.frame`.
- [x] Every consumer uses only its declared contract - `app.py` imports the frame's public render functions and palette; the visual steps use `--snapshot`/`--roles`, not the component.
- [x] Strict-mode proof present and passing - `./scripts/verify.sh --strictness` exits 0 with all three probes.
- [x] Scenario execution evidence is recorded per scope - receipt 264/264, regular 247/247, E2E 22/22, zero failed or skipped.
- [x] Coverage classification uses the most recent measured evidence - three distinct branch-instrumented reports; the delta-only paths are classified from the change receipt.
- [ ] Every coverage gap classified and resolved - F2 is bucket 2 and unresolved; V5/V6 add the missing visual assertions.
- [ ] Dead code deleted. Missing tests added. Hard-to-test gaps justified - D1 is resolved and the curses fallback is justified as bucket 3; the F2 assertions and the V5/V6 visual assertions are not added.
- [x] Redundant unit tests removed - there is no separate unit-test tree.
- [x] No `@wip` tags remain. No implementation-layer detail in the spec - `givn lint` reports clean; the delta features contain no `@wip`.
- [x] Canonical E2E policy was read from the resolved specs instruction; every normalized inventory action has one `@e2e` scenario and variants are regular scenarios - 11/11.
- [x] Every E2E scenario has a real-interface primary assertion and uses the reviewed driver; no browser UI scenario is replaced by HTTP/fetch - the four delta E2E scenarios assert the snapshot/roles and the permanent CLI scenarios assert transcripts.
- [x] The local run command starts the full stack cleanly - `.venv/bin/python -m passuff --help` exits 0; the committed receipt covers the fake GitHub, D-Bus, and binary twins.
- [x] The exact files `verify.e2e_command` invokes were identified and read; the tree (including untracked files) has no second E2E implementation.
- [x] The E2E scope is selected explicitly (`GIVN_RESULT_SCOPE` and `--e2e`); the receipt reports zero failures and skips.
- [ ] Implementation (commands, file layout, framework/driver) matches what `design.md` explicitly named - the multi-change fail clause is still missing (D7); the visual composition is V5.
- [x] Interaction coverage verified: every inventory entry maps to a matrix row, an existing `@e2e` scenario, and step definitions using the promised driving mechanism - 11/11.
- [x] No finding was excused with a classification outside the three buckets - D1 is resolved; F2 and the V5/V6 assertions are bucket 2; the curses fallback is bucket 3; D7/T1 are minor non-coverage findings for the same repair batch.
- [ ] Visual review completed with every Persona finding dispositioned - V5 and V6 remain open.

## Result

Execution evidence is present and mechanically sound: 264/264 change, 247/247 regular, 22/22 E2E, strictness exit 0, clean `check_steps.py`, and verified commits. The recorded corrections are fixed: P1 (routing table), V1 (bordered panes), V2 (centered/cleared overlays), V3 (real captures and evidence), V4 (color pairs and durable palette capture), the recorded F1 items, D1 (dead code), D2 (`PALETTE`/role strings), U1 (live notification settings), and R1 (README). The design-review 16-vs-17 count is resolved: the 17th scenario is the T22 settings-persistence regression, and `design.md` records 17.

The review fails on V5: the outer frame, every inner pane/overlay, and the picker draw a top edge that does not meet the side borders, so the "bordered block" composition is visibly broken in the committed captures and the reference comparison; and on V6: the help key reference is clipped and its second row collapses to all accent-bold, losing the dim labels and the tail keys. F2 (residual span-level role precision) is a bucket-2 gap; D7 and T1 are minor fixes for the same repair batch.

The exact `givn check review --change restore-tui-styling` result after writing this file is:

```text
EXIT=1
givn lint: 4 file(s) checked — clean
net delta: 9 added, 8 modified, 0 removed
  ✓ overlap dispositions passed
  ✓ change verification receipt reused across documentation-only revision 657b44784eee34b568fdaf94e4c9995c7784f8ea (tested at 7f0470683f9f186901f15c3094388cdae8cf74a2, 264 scenario(s))
error: review checks failed:
  integrity checks failed:
  review.md actual verdict is FAIL
```

VISUAL-REVIEW: FAIL

REVIEW: FAIL
