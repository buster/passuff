# Review: restore-tui-styling

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-2

Scope: the working tree at `e520e0dbb086de01717108edb2ab965119309dd2`; `git status --short` was clean before this review, and the only file this review writes is this `review.md`. The change receipt is 264 scenarios at `4fd7ae98d41c45b46470a4825e5174fca5f29172`, reused across the documentation-only receipt commit `e520e0d`. Revisions were inspected with `git cat-file -t` and `git ls-tree`; no `git log`, `git show`, `git diff`, `git reflog`, or `git blame` was used.

## Implementation evidence check

### 0. `@e2e` tag integrity

The delta has seventeen scenario entries: four under `activity-inbox`, eleven under `inbox-navigation`, one under `stacked-inbox`, and one under `watch-scope`. Four delta scenarios carry `@e2e`: two under `activity-inbox`, one under `inbox-navigation`, and one under `stacked-inbox`. No `@e2e` tag was removed or converted to a regular tag. The permanent corpus carries eighteen `@e2e` scenarios; the E2E scope is 22 executions (18 permanent + 4 delta modified copies), matching `tasks.md:16,199`. The change scope is 264 (247 regular + 17 delta), matching the receipt.

### 1. Empty step body audit

`python3 scripts/check_steps.py --path givn/steps --path givn/e2e_steps` exits 0: `check_steps: no no-op step bodies`, across the 17 tracked step files. The new steps call `assert_role`, `assert_role_member`, `assert_role_digits`, `assert_any_role`, `assert_metadata_lines`, `_assert`, or a persisted-value check; none is an empty body. The role-step audit is not clean, however: several new role steps are non-discriminating and can pass without the claimed element (finding F1).

### 2. Checked scenarios and their commits

`tasks.md` has 40 `- [x]` entries: 20 task lines (setup plus 19 behavior tasks) and 20 `COMMIT:` lines, all with batch names. Tasks setup and 1-18 record `d13d99ef65bd289dbe55627358589a38a1bd0159`; task 19 records `7efd2c8cf4e32d39a28be55dbb9ac8538af3d1d3`. Both are commit objects (`git cat-file -t`). `git ls-tree` shows `d13d99` contains `passuff/tui/frame.py`, `passuff/tui/app.py`, `scripts/verify.sh`, `tests/support/harness.py`, `tests/support/world.py`, `docs/design/design-system.md`, `docs/glossary.md`, the four capability step files, the four E2E files, and the delta features. `7efd2c8` contains `passuff/tui/app.py`, `givn/steps/inbox_navigation_steps.py`, the modified `inbox-navigation.feature`, and `tasks.md`. No checked task points to a comment-only, tag-only, task-only, feature-only, or transcript-only commit.

### 3. Promised components

| Design promise | Tree | Consumer |
|---|---|---|
| `Span`, `StyledLine`, `render_styled` | `passuff/tui/frame.py:16-28,145-451` | `passuff/tui/app.py:21,358` |
| Palette owner | `passuff/tui/frame.py:31-43` (`PALETTE`) | `passuff/tui/app.py:21,364,392`; the design and glossary name `ROLE_STYLES`, which does not exist (finding D2) |
| `render`, `render_ansi`, `render_roles` | `passuff/tui/frame.py:454-481` | `passuff/tui/app.py:492-500`; the visual steps through `--snapshot` |
| Save prompt | `passuff/tui/app.py:113-133`; `passuff/tui/frame.py:434-439` | the `inbox-navigation` save scenarios |
| Active-change runner and evidence | `scripts/verify.sh:51-66`; `tests/support/harness.py:46-70`; `tests/support/world.py:525-585` | the change/E2E scopes; the runner's literal "fail on more than one active change" clause is not implemented (finding D7) |
| Step bindings | `givn/steps/common.py`, the four capability files, the four E2E files | `givn/steps/support.py`; the E2E files import the regular bindings |

The design also names a `Role` symbol (`design.md:86`); the implementation uses role-name strings and `PALETTE` keys, not a `Role` type. This is the same durable-symbol finding D2, not a missing rendering capability.

### 4. Strict-mode proof

`./scripts/verify.sh --strictness` exits 0: the undefined-step probe errors with 1 undefined step, the skipped-scenario probe reports `skipped=1`, and the pass-only body is rejected by `check_steps.py`. `scripts/verify.sh:47` runs the no-op audit in every normal scope.

### 5. Every `@e2e` Then asserts the real interface

The four delta `@e2e` scenarios assert the printed snapshot, the role record, the exit status, or the browser fake's log. `Developer opens the inbox TUI`, `Developer opens an activity's details`, `Developer navigates the inbox panes`, and `Developer reviews a stacked inbox` all use `--snapshot`/`--snapshot --roles` through `tests/support/world.py`. No delta `@e2e` Then asserts only the state file or the repository. The permanent E2E CLI scenarios assert the command transcript and exit status; the browser scenario asserts the fake `xdg-open` log (`givn/e2e_steps/activity_inbox_e2e.py:24-27`).

### 6. Exact `verify.e2e_command` and the files it invokes

`givn/commands.yaml` sets `verify.e2e_command: "./scripts/verify.sh --e2e"` and `verify.command: "./scripts/verify.sh"`. `scripts/verify.sh:74-86` runs `behave --stage e2e --tags="@e2e"` over `givn/specs/*/*.feature`, `givn/changes/*/specs/*/*.feature`, and any `GIVN_FEATURES`; `givn/e2e_environment.py` loads `givn/e2e_steps/`. The tree contains no second E2E step implementation; `grep` for `@step` finds definitions only under `givn/steps`, `givn/e2e_steps`, and `tests/strictness`. The delta glob is all active changes, not the single selected change; with the empty `python-rebuild` change this selects the same features, but the design's literal fail clause is not implemented (D7).

### 7. E2E scope selection

`scripts/verify.sh:11,16-24,73-93` selects `regular`, `e2e`, or `change` from `GIVN_RESULT_SCOPE`; `--e2e` sets `SCOPE=e2e`. The change scope runs every delta feature then the permanent non-E2E corpus; regular runs the permanent non-E2E corpus; E2E runs `--stage e2e --tags="@e2e"`. The receipt records change 264/264, 0 failed, 0 skipped. `tasks.md:16,199` records regular 247/247 and E2E 22/22 with zero failures and skips. Review does not re-run the suite.

### 8. E2E scope size and interaction coverage

The three touched use cases hold 11 normalized inventory entries: 7 for `review-activity-inbox`, 1 for `review-pull-request-stacks`, and 3 for `change-watch-scope`. Each has one design matrix row and one `@e2e` scenario; the four delta E2E scenarios are modified copies of four of those inventory actions, not new actions. The 22 E2E executions are 18 permanent plus the 4 delta copies; no capability has an excess E2E scenario for a variant or error case. The settings overlay, PR detail, help, save prompt, ignored row, empty state, 80x24 frame, and picker are regular/delta visual variants of those inventory actions.

### 9. Design conformance (commands, layout, framework, drivers)

The `verify` commands, step-file layout, `behave` stage split, snapshot flags, and `--snapshot --roles` assertion source match `design.md`. The implementation deviates from the reviewed design in the durable palette symbol (`PALETTE` vs `ROLE_STYLES`), the absent `Role` symbol, the "fail on more than one active change" clause, and the table/controls composition (D2, D7, V1); `design.md` was not updated for those deviations.

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

Every inventory entry maps to a matrix row, an existing `@e2e` scenario, and step definitions using the promised mechanism.

### 11. Local run command

`.venv/bin/python -m passuff --help` exits 0. The full local stack (fake GitHub GraphQL, private D-Bus, fake `gh`/sound/opener) is exercised by the committed 264-scenario receipt; review did not re-run the suite.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Visual contract refines the terminal goal | `docs/arc42/01-introduction-and-goals.md:42-44` | Affected | Visual Design Contract | task 17 | `docs/design/design-system.md` | yes |
| Styled frame with plain/ANSI/role projections | `docs/arc42/04-solution-strategy.md:77` | Affected | D1 | tasks 1-16 | `passuff/tui/frame.py:145-481` | yes |
| `TerminalPresentation` composes tables and controls | `docs/arc42/05-building-block-view.md:72-73` | Affected | Modules | tasks 1-16 | `frame.py:218-223,265-270,303-322` renders rules, not the design-system bordered blocks | no (V1) |
| Save prompt on quit | `docs/arc42/06-runtime-view.md:344-353` | Affected | save-prompt decision | tasks 11-14, 19 | `app.py:113-133`; `frame.py:434-439` | yes |
| Palette roles and accessibility | `docs/arc42/08-crosscutting-concepts.md:134-142` | Affected | palette model | tasks 1-16 | `frame.py:31-43` | yes |
| QS-034 visual fidelity follows the design system | `docs/arc42/10-quality-requirements.md:66` | Affected | Visual Design Contract | task 17 | composition mismatch (V1) | no |
| R-037 palette/composition drift | `docs/arc42/11-risks-and-technical-debt.md:43` | Affected | Risks | task 17 | the composition drift is present (V1) | no |
| Chapter 12 links the glossary | `docs/arc42/12-glossary.md` | Affected | glossary task | task 17 | `docs/glossary.md:63-64` names `ROLE_STYLES`/`Role`, absent from the code (D2) | no |

`ARC42 CONFORMANCE: FINDINGS`; the visual composition and the stale glossary symbols are unresolved, so this block is not `CLEAN`.

## Ubiquitous language conformance

The glossary adds `Watn palette`, `Palette role`, `Styled frame`, `Role record`, and `Rendered snapshot` (`docs/glossary.md:63-67`). `render_styled`, `render_roles`, and the snapshot flags exist; `docs/domain-model.md:154` correctly says the visual layer adds no domain structure. The glossary's symbols `ROLE_STYLES` and `Role` do not exist in the code; the actual symbol is `PALETTE` (`passuff/tui/frame.py:31`) and the roles are strings. This is finding D2. The save prompt's durable wording is T22, and T26 still says settings persist immediately in `ui.json` (finding U1). No single authoritative source remains for those terms.

## Use-case and Persona conformance

The three owning use cases keep their goals, triggers, guarantees, typed relationships, and capability ownership; the delta changes presentation and the T22 save prompt, not the domain structure. `review-pull-request-stacks` still extends `review-activity-inbox`; `change-watch-scope` remains separate. T25's palette agrees with `docs/design/design-system.md` and the original `theme.rs`, but its composition claim and the design system's `bordered block` rows are not met by the main tables/controls (V1). T22 agrees with the save prompt and the receipt scenarios; T26 is stale (U1). The theme/picker/details save prompt copy in `docs/reference.md:113,118` matches.

Persona disposition:

| Persona | Confirmed | Outcome in this change |
|---|---|---|
| `multi-account-developer` | yes, 2026-11-02 | The main inbox, activity detail, stack, and picker goals are visible in the ANSI/role evidence; the settings and save-prompt goals have no committed visual evidence, and the main-frame original composition is not restored (V1, V3). |

PERSONAS: multi-account-developer

## Coverage

Measured evidence, all regenerated after the implementation and distinct files with branch instrumentation: `coverage/non-e2e-cobertura.xml` 2029/2349 lines (86.38%), 702/886 branches (79.23%); `coverage/e2e-cobertura.xml` 1520/2349 (64.71%), 387/886 (43.68%); `coverage/cobertura-coverage.xml` 2062/2349 (87.78%), 719/886 (81.15%). In the merged report `tui/frame.py` is 322/344 lines (93.6%) and `tui/app.py` 284/420 (67.6%).

The measurement commands run only the regular and E2E scopes (`scripts/measure-coverage.sh:17,22`), so the delta non-E2E visual scenarios are outside the Cobertura reports. That is a measurement boundary, not missing tests: the settings overlay, PR detail, help, save prompt, 80x24, ignored-row, empty-state, and picker scenarios run in the change scope and are inside the 264/264 receipt. The uncovered lines `tui/app.py:127-132,439-443` and `tui/frame.py:377,404,407,432,446-447` are exactly those delta-only paths.

Bucket 1 (dead code): `markdown_lines` (`passuff/tui/frame.py:90-91`) and `render_from_paths` (`passuff/tui/frame.py:484-485`) have no consumer; `grep` finds them only at their definitions, and the merged report leaves `render_from_paths` at `hits=0`. These are bucket 1 and must be deleted (D1).

Bucket 2 (missing test coverage): the non-discriminating role steps (F1) are missing assertions, not hard-to-test code. The `metadata values are dim` step is identical to the labels step; the overlay-border steps assert the outer frame's `┌`; the PR-detail heading and picker-brand steps accept the main frame's spans. These need discriminating assertions.

Bucket 3 (legitimately hard to test): the curses color-pair initialization and the 256-color fallback (`passuff/tui/app.py:388-402`) depend on the real terminal; the role record and ANSI snapshot are the terminal-independent assertions, and the design records the fallback as a hard-to-test risk. The interactive loop's start/quit is covered by the regular pty smoke; this row is justified.

No redundant unit tests: the `.feature` corpus is the only test surface.

## Visual review

The design declares `Interface: terminal` (`design.md:170-190`) with the 80x24 minimum, the 100x30 default snapshot, and the palette/composition in `docs/design/design-system.md`. The 22 E2E executions produce 18 committed evidence directories because the four delta E2E scenarios reuse four permanent slugs; five directories carry `roles.json`, fifteen carry `transcript.ansi`, and the CLI-only scenarios carry `transcript.txt`. The states the review saw:

| Evidence | State seen |
|---|---|
| `developer-opens-the-inbox-tui` | main inbox, header, counts/focus, watched/activity rows, controls, accent/gold/green/body roles |
| `developer-opens-an-activity-s-details` | activity detail overlay, accent-bold title, accent labels, dim values, rule separator, body, hint roles |
| `developer-navigates-the-inbox-panes` | focused/unfocused titles and borders, selected row, header roles |
| `developer-reviews-a-stacked-inbox` | `☰` header, `1/2`/`2/2` bright positions, selected row |
| `developer-toggles-a-repository-in-the-watch-scope` | picker, mode, selected repository, focus marks |
| `developer-switches-the-watch-scope-mode` | picker, `Mode: selected`, focus marks |
| `developer-narrows-the-focus-in-the-watch-scope` | picker, focus marks, watched row |
| the remaining permanent/CLI transcripts | startup, status, inbox/read, check, notifications, usage/version |

The ANSI/role evidence shows the palette exactly as the design system records it (81, 15, 214, 114, 252, 240, dim/bold); the palette part of the Persona goal is visible. The composition is not the original:

- The original reference `evidence/reference/main-inbox.png` and the original `Block::bordered()` calls (`/home/buster/projects/gh-notify/crates/passuff-cli/src/tui.rs:1744,1824,1926,1978`) show bordered blocks for the watched table, activity table, and controls bar. The rebuilt frame draws only `── title ──` rules (`passuff/tui/frame.py:218-223,265-270,303-322`), and the committed screenshot shows no inner table or controls borders. This contradicts `docs/design/design-system.md:48,56,60` (V1).
- The original settings overlay uses `Block::bordered()`, `Clear`, and a centered rect (`tui.rs:2045-2049`); the rebuilt overlay is appended at the bottom at full width and never clears the background (`frame.py:336,352,370-372,411,430,439`). This contradicts `docs/design/design-system.md:64` (V2).
- `docs/design/passuff-tui-alacritty.png` is a grayscale/plain frame: its 195-colour histogram contains no accent/gold/green text pixels; it does not show the palette. The committed ANSI transcripts are the styled artifact, but the design's "rendered reference" capture is not styled (V4).
- There is no committed visual evidence for the settings overlay, save prompt, PR detail, or help. `tests/support/harness.py:46-70` writes evidence only for `e2e`-tagged scenarios; the settings and save-prompt delta scenarios are not `@e2e` (`inbox-navigation.feature:24-37,100-149`). The settings and save-prompt Persona goals therefore cannot be judged from committed evidence (V3).

Persona judgment: the `multi-account-developer` main inbox, activity detail, stack, and picker goals are visible; the settings/save-prompt goals are not evidenced, and the main inbox is not the original composition. `VISUAL-REVIEW: FAIL`.

## Scenario execution evidence

Receipt `givn/changes/restore-tui-styling/verification/tasks-verification.json`: status `passed`, head `4fd7ae98d41c45b46470a4825e5174fca5f29172`, 264 total, 264 passed, 0 failed, 0 skipped; `tasks-result.json` records `{"failed":0,"passed":264,"skipped":0,"total":264}`. `tasks.md:16,199` records regular 247/247 and E2E 22/22. `./scripts/verify.sh --strictness` exits 0. The receipt is reused at `e520e0d`; the gate reports "documentation-only revision". Review does not re-run the suite.

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

The change is user-visible (the TUI palette and the new `y`/`n` save-prompt keys), so the README's Terminal inbox key table and coverage evidence are in scope. Neither section was edited: `README.md:77-101` still says `q`/`Esc` quits and lists neither `y` nor `n`, and `README.md:3-5,166-167` still reports 89%/83% (1931/2178, 685/828) while the regenerated merged report is 87.78%/81.15% (2062/2349, 719/886). This is finding R1; the declared update is not an edit.

## Findings

| ID | Severity | Finding | Evidence | Status |
|---|---|---|---|---|
| V1 | blocking | The main watched table, activity table, and controls bar are not the design system's bordered blocks and not the original composition. | `docs/design/design-system.md:48,56,60`; `passuff/tui/frame.py:218-223,265-270,303-322`; `evidence/reference/main-inbox.png`; `/home/buster/projects/gh-notify/crates/passuff-cli/src/tui.rs:1824,1926,1978` | open |
| V2 | blocking | Modal overlays are appended at full width with no clear rectangle; the original uses centered `Clear`/`Block` overlays. | `docs/design/design-system.md:64`; `passuff/tui/frame.py:336,352,370-372,411,430,439`; `tui.rs:2045-2049` | open |
| V3 | blocking | No committed visual evidence for the settings overlay, save prompt, PR detail, or help; the settings/save-prompt Persona goals cannot be judged. | `tests/support/harness.py:46-70`; `givn/changes/restore-tui-styling/specs/review-activity-inbox/inbox-navigation.feature:24-37,100-149`; `givn/changes/restore-tui-styling/evidence/visual/` | open |
| V4 | major | The committed "styled" screenshot is grayscale/plain and does not show the palette. | `docs/design/passuff-tui-alacritty.png` (195-colour histogram, no colored text pixels); `docs/design/design-system.md:100-105`; `Isetest-design.md:318` | open |
| F1 | blocking | Several new role steps are non-discriminating: metadata values repeats the labels check; overlay-border steps match the outer frame's `┌`; PR-detail heading and picker brand match main-frame spans; the settings selected-row `▶` matches the main-frame row. | `givn/steps/common.py:262-264`; `givn/steps/support.py:46-54`; `givn/steps/activity_inbox_steps.py:230-232`; `givn/steps/inbox_navigation_steps.py:282-284,307-309,317-319,337-339`; `givn/steps/watch_scope_steps.py:137-144`; `givn/steps/inbox_navigation_steps.py:293-295` | open |
| D1 | blocking | Dead code: `markdown_lines` and `render_from_paths` have no consumer; `render_from_paths` is uncovered. | `passuff/tui/frame.py:90-91,484-485`; `grep` finds no consumer; merged report `hits=0` at line 485 | open (bucket 1) |
| D2 | blocking | The durable design and glossary name `ROLE_STYLES` and `Role`; the code owns `PALETTE` and string roles, and no `Role` symbol exists. | `docs/glossary.md:63-64`; `givn/changes/restore-tui-styling/design.md:86`; `passuff/tui/frame.py:31-43` | open |
| U1 | blocking | T26 still says settings persist in `ui.json` and apply immediately to live delivery; the save prompt makes persistence conditional and the live `ObservationSession` reads `config.notifications`, not the toggled `view`. | `givn/specs/review-activity-inbox/usecase.md:74`; `passuff/tui/app.py:113-133,299-305`; `passuff/tui/session.py:19-37`; `design.md:43-48` | open |
| R1 | blocking | The user-visible change did not update the README Terminal inbox key table or the coverage evidence. | `README.md:3-5,77-101,166-167`; `coverage/cobertura-coverage.xml` | open |
| P1 | blocking (gate) | The proposal's `## Capability Routing` table has three columns; `givn check review` requires the `Route's recommendation` column and reports the four delta capabilities as undeclared. | `givn/changes/restore-tui-styling/proposal.md:79-84`; `givn check review` error; archived four-column table at `givn/archive/python-rebuild/proposal.md:81-89` | open |
| T1 | minor | The tasks acceptance line still says 263/263 while the receipt and the rest of tasks.md record 264/264. | `givn/changes/restore-tui-styling/tasks.md:3` vs `:16,199` and `verification/tasks-verification.json` | open |
| D7 | minor | `scripts/verify.sh:51-57` exports an empty `GIVN_CHANGE_ID` when more than one active change directory exists instead of failing as `design.md:161-163` promises; the harness's single-specs-root fallback masks it. | `scripts/verify.sh:51-57`; `tests/support/harness.py:52-56`; `design.md:161-163` | open |
| R4-17 | resolved | The design-review round-4 PASS recorded 16 delta entries; the 17th scenario is the T22 settings-mute regression test, added after that review, and `design.md` now records 17. It adds a regression test for the existing save-prompt rule, not a new behavior; it does not change the reviewed plan. | `design.md:315-320`; `tasks.md:203-212`; `inbox-navigation.feature:150-162`; commit `7efd2c8` | resolved |

## Sign-off

- [ ] Fabrication audit: clean - `check_steps.py` reports 0 no-op bodies, but the non-discriminating role steps remain (F1).
- [x] Every checked task has a verified commit touching production code - 40/40 checked; `d13d99` and `7efd2c8` contain the production, step, feature, and task changes with batch names.
- [ ] Every promised component exists - the frame, palette, snapshots, save prompt, runner, and bindings exist; the promised `Role`/`ROLE_STYLES` symbols do not (D2).
- [x] Every consumer uses only its declared contract - `app.py` imports the frame's public render functions and palette; the visual steps use `--snapshot`/`--snapshot --roles`, not the component.
- [x] Strict-mode proof present and passing - `./scripts/verify.sh --strictness` exits 0 with all three probes.
- [x] Scenario execution evidence is recorded per scope - receipt 264/264, regular 247/247, E2E 22/22, zero failed or skipped.
- [x] Coverage classification uses the most recent measured evidence - three distinct branch-instrumented reports; the delta-only paths are classified from the change receipt.
- [ ] Every coverage gap classified and resolved - the dead code (D1) and the missing discriminating assertions (F1) remain.
- [ ] Dead code deleted. Missing tests added. Hard-to-test gaps justified - D1 and F1 are open; the curses terminal fallback is justified as bucket 3.
- [x] Redundant unit tests removed - there is no separate unit-test tree.
- [x] No `@wip` tags remain. No implementation-layer detail in the spec - `givn lint` reports clean; the delta features contain no `@wip`.
- [x] Canonical E2E policy was read from the resolved specs instruction; every normalized inventory action has one `@e2e` scenario and variants are regular scenarios - 11/11.
- [x] Every E2E scenario has a real-interface primary assertion and uses the reviewed driver; no browser UI scenario is replaced by HTTP/fetch - the four delta E2E scenarios assert the snapshot/roles and the permanent CLI scenarios assert transcripts.
- [x] The local run command starts the full stack cleanly - `.venv/bin/python -m passuff --help` exits 0; the committed receipt covers the fake GitHub, D-Bus, and binary twins.
- [x] The exact files `verify.e2e_command` invokes were identified and read; the tree (including untracked files) has no second E2E implementation.
- [x] The E2E scope is selected explicitly (`GIVN_RESULT_SCOPE` and `--e2e`); the receipt reports zero failures and skips.
- [ ] Implementation (commands, file layout, framework/driver) matches what `design.md` explicitly named - the palette symbol, `Role` symbol, runner fail clause, and composition diverge (D2, D7, V1).
- [x] Interaction coverage verified: every inventory entry maps to a matrix row, an existing `@e2e` scenario, and step definitions using the promised driving mechanism - 11/11.
- [x] No finding was excused with a classification outside the three buckets - D1 is bucket 1, F1 is bucket 2, and the curses fallback is bucket 3; no deferral or invented exception is used.

## Result

Execution evidence is present and mechanically sound: 264/264 change, 247/247 regular, 22/22 E2E, strictness exit 0, clean `check_steps.py`, and verified commits. The round-4 design-review PASS versus the 17th scenario is resolved: the scenario is a T22 regression test, not a new behavior, and `design.md` records 17 entries.

The review fails on the visual contract and evidence (V1-V4), the non-discriminating role steps (F1), the dead code (D1), the durable-doc drift (D2, U1), the README (R1), and the proposal routing gate (P1). The exact `givn check review --change restore-tui-styling` result after writing this file is:

```text
EXIT=1
  ✓ overlap dispositions passed
  ✓ change verification receipt reused across documentation-only revision e520e0dbb086de01717108edb2ab965119309dd2 (tested at 4fd7ae98d41c45b46470a4825e5174fca5f29172, 264 scenario(s))
error: review checks failed:
  integrity checks failed:
  review.md actual verdict is FAIL
  capability routing decisions do not match the built delta:
  proposal.md has no filled '## Capability Routing' table — the delta capabilities [activity-inbox, inbox-navigation, stacked-inbox, watch-scope] are undeclared; record decisions (EXTEND <capability> / NEW in <usecase-id>)
```

REVIEW: FAIL
VISUAL-REVIEW: FAIL
