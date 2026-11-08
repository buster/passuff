# Tasks: restore-tui-styling

Acceptance: the change gate is 263/263, the regular corpus 247/247, and the E2E scope 22/22. The visual contract is terminal-interface rule 25 (palette and composition) and the save prompt is terminal-interface rule 22 (settings persistence). Work in behavior batches; each batch records the shared implementation revision with its batch name.

## Setup

- [x] Setup: active-change runner, styled frame, palette, and save prompt

- Batch: `styled-frame`
- Design constraints:
  - Runner: `./scripts/verify.sh`; the change scope resolves the active change's delta from `givn/changes/*/specs/*/*.feature`, and `GIVN_CHANGE_ID` selects the evidence directory. The archived `python-rebuild` hardcode is removed.
  - Frame: `passuff/tui/frame.py` owns `Span`, `StyledLine`, `PALETTE`, `render_styled`, `render` (plain), `render_ansi`, and `render_roles`; `app.py` imports the palette and uses curses color pairs, `A_BOLD`, and `A_DIM`.
  - Snapshot: `--snapshot`, `--snapshot --ansi`, and `--snapshot --roles`; the latter is the role-record assertion source.
  - Save prompt: `q` opens `Save sort and filter settings?` when the sort, direction, filters, ignored view, or notification settings changed; `y` persists, `n` discards, `Esc` cancels; no prompt without a change.
  - Strictness: `check_steps.py` runs in every normal scope; new steps start with `raise NotImplementedError`.
- Evidence: change 264/264, regular 247/247, E2E 22/22; `./scripts/verify.sh --strictness` exits 0; `check_steps.py` reports no no-op bodies.
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: styled-frame

## Batch: activity-inbox

- [x] 1. Developer opens the inbox TUI

- Batch: `activity-inbox`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 1 (one-screen composition), rule 7 (excerpt and body bound), and rule 25 (palette and composition); outer frame, header, unread and focus line, watched and activity tables, controls bar; roles for the title, login, headers, selected row, rules, body, and key hints.
- Components: `passuff/tui/frame.py`, `passuff/tui/app.py`.
- Step files: `givn/steps/activity_inbox_steps.py`, `givn/steps/common.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "Developer opens the inbox TUI"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: activity-inbox

- [x] 2. Developer opens an activity's details

- Batch: `activity-inbox`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 14 (activity detail) and rule 25 (palette and composition); bordered detail overlay, accent-bold title, accent metadata labels, dim metadata values, rule separator, bright-gray body, accent-bold hint keys.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/activity_inbox_steps.py`, `givn/steps/common.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "Developer opens an activity's details"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: activity-inbox

- [x] 3. An ignored row is dim and marked

- Batch: `activity-inbox`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 27 (ignored view) and rule 25 (palette); ignored row dim with the `⊘` marker.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "An ignored row is dim and marked"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: activity-inbox

- [x] 4. The empty inbox uses the primary text

- Batch: `activity-inbox`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 25 (palette and composition); the empty watched and activity text is bright primary text.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The empty inbox uses the primary text"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: activity-inbox

## Batch: inbox-navigation

- [x] 5. Developer navigates the inbox panes

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 2 (selection movement), rule 8 (focus cycling), and rule 25 (palette); focused pane border and title green, unfocused pane border and title dim, selected row gold.
- Components: `passuff/tui/frame.py`, `passuff/tui/app.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`, `givn/steps/common.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "Developer navigates the inbox panes"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 6. The settings overlay follows the palette

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 26 (settings) and rule 25 (palette); dim overlay border, accent-bold title, gold-bold selected row, green enabled values, dim disabled values.
- Components: `passuff/tui/frame.py`, `passuff/tui/app.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The settings overlay follows the palette"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 7. The pull request detail shows its metadata

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 24 (pull-request detail) and rule 25 (palette); `PR - <key>` title, dim border, accent-bold title, green-bold Markdown heading, accent labels, dim values.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`, `givn/steps/common.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The pull request detail shows its metadata"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 8. The developer opens the key reference

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 21 (help) and rule 25 (palette); accent-bold heading and key hints, dim labels.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The developer opens the key reference"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 9. A missing browser opener is reported and keeps the detail

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: the screen keeps the detail overlay and the status line is gold attention text; no browser behavior change.
- Components: `passuff/tui/frame.py`, `passuff/tui/app.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "A missing browser opener is reported and keeps the detail"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 10. The 80x24 frame keeps the composition

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 25 (viewports); COLUMNS=80 and LINES=24 keep the title, table headers, and focused title visible.
- Components: `passuff/tui/frame.py`, `passuff/tui/app.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The 80x24 frame keeps the composition"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 11. The developer confirms saving settings on quit

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 22 (settings persistence); `y` persists the sort, filters, ignored view, and notification settings, then exits; the prompt is a dim-bordered accent-titled overlay.
- Components: `passuff/tui/app.py`, `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The developer confirms saving settings on quit"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 12. The developer discards settings on quit

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 22 (settings persistence); `n` discards the unsaved settings and exits; the saved sort stays `updated`.
- Components: `passuff/tui/app.py`, `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The developer discards settings on quit"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 13. The developer cancels the quit from the save prompt

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 22 (settings persistence); `Esc` closes the prompt, the inbox stays open, the changed sort remains visible, and nothing is persisted.
- Components: `passuff/tui/app.py`, `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The developer cancels the quit from the save prompt"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

- [x] 14. The developer quits without a save prompt when settings are unchanged

- Batch: `inbox-navigation`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 22 (settings persistence); `q` exits without the prompt when nothing changed.
- Components: `passuff/tui/app.py`, `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The developer quits without a save prompt when settings are unchanged"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: inbox-navigation

## Batch: stacked-inbox

- [x] 15. Developer reviews a stacked inbox

- Batch: `stacked-inbox`
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints: rules R2 (stack grouping) and R3 (position/size and base); the `☰` header is accent bold and the stack positions are bright.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/stacked_inbox_steps.py`, `givn/steps/common.py`; E2E `givn/e2e_steps/stacked_inbox_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "Developer reviews a stacked inbox"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: stacked-inbox

## Batch: watch-scope

- [x] 16. The scope picker renders the watn palette

- Batch: `watch-scope`
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints: the picker overlay has a dim border, bright-bold brand, dim title suffix, accent-bold mode, and gold selected repository.
- Components: `passuff/tui/frame.py`.
- Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
- Runner: `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "The scope picker renders the watn palette"`
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: watch-scope

## Batch: docs-and-verification

- [x] 17. Update the durable visual contract, glossary, and arc42 chapters

- Batch: `docs-and-verification`
- Design constraints: `docs/design/design-system.md` records the exact palette and composition; `docs/glossary.md` adds `Watn palette`, `Palette role`, `Styled frame`, `Role record`, and `Rendered snapshot`; arc42 chapters 1/4/5/6/8/10/11/12 and `docs/domain-model.md` reflect the visual contract and the save prompt.
- Evidence: `docs/design/design-system.md`, `docs/glossary.md`, `docs/arc42/*`, `givn/changes/restore-tui-styling/arc42.md`.
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: docs-and-verification

- [x] 18. Record the change, regular, and E2E evidence

- Batch: `docs-and-verification`
- Evidence: change 264/264, regular 247/247, E2E 22/22; `./scripts/verify.sh --strictness` exits 0; `check_steps.py` clean; the verification receipt is generated by `givn check tasks`.
- [x] COMMIT: 91422ab088b4233c718ca9f49776c7bea32f6c9a -- Batch: docs-and-verification

## Batch: settings-persistence

- [x] 19. The developer mutes notifications from the settings dialog

- Batch: `settings-persistence`
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints: terminal-interface rule 22 (settings persistence); the settings toggle is in memory until `q` opens `Save sort and filter settings?`, `y` persists it, `n` discards, and `Esc` cancels. The old immediate save is removed.
- Components: `passuff/tui/app.py`, `passuff/tui/frame.py`.
- Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
- Runner: `./scripts/verify.sh --name "The developer mutes notifications from the settings dialog"`
- [x] COMMIT: 7efd2c8cf4e32d39a28be55dbb9ac8538af3d1d3 -- Batch: settings-persistence
