Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-2

# Design Review: restore-tui-styling

Reviewed artifacts: `proposal.md`, the four delta `.feature` files, `design.md`,
`arc42.md`, the owning use cases (`review-activity-inbox`,
`review-pull-request-stacks`, `change-watch-scope`), the permanent specs,
`docs/design/design-system.md`, `docs/glossary.md`, `docs/domain-model.md`,
`docs/arc42/` (twelve chapters, `README.md`, the ADR register and files),
`givn/commands.yaml`, `scripts/verify.sh`, `scripts/check_steps.py`,
`tests/support/harness.py`, the current TUI modules, and the original reference
(`/home/buster/projects/gh-notify`: `passuff.gif`, `theme.rs`, `tui.rs`, and the
three committed frames). No git history was used. Only this file was written.

Independent verdict: the plan has the right shape and the palette values match
the original (`theme.rs` accent 81, bright white, body 252, gold 214, green 114,
rule 240, dim). It is not ready for tasks. The test runner and visual-evidence
path still point at the archived `python-rebuild` change; the new visual steps
are umbrella phrases and the role model cannot express the design system's
bright-bold title; the plan does not cover the pull-request detail, help, and
save-prompt screens it promises; the save prompt changes quit keys and
contradicts the plan's "does not change keys" statement while T22 is unhandled;
T25 contradicts the design system; the palette has two owners; the glossary is
missing the new vocabulary; the contract table is incomplete; and arc42
chapter 6 is marked "No" while the session-to-view flow changes. Two operator
questions remain: the save prompt's scope and the visual-direction approval.

## Ranked questions

### Q1 — The delta runner and the visual-evidence path still point at `python-rebuild`

- Status: BLOCKING (testability and evidence)
- Evidence:
  - `scripts/verify.sh:51` builds the delta list from
    `givn/changes/python-rebuild/specs/*/*.feature`; `givn/changes/` now
    contains only `restore-tui-styling`, so the delta list is empty.
  - `tests/support/harness.py:54` writes every `@e2e` transcript under
    `givn/changes/python-rebuild/evidence/visual/<slug>/`; the new change's
    evidence would be created in a resurrected directory.
  - `design.md:239-243` says the runner is "unchanged" and gives
    `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name
    "Developer opens the inbox TUI"` as the single-scenario command. With the
    delta empty, the `change` scope runs only the permanent non-E2E corpus, and
    the `--e2e` variant runs the permanent `@e2e` scenario with the same title
    (the pre-change version), not the delta scenario.
  - `design.md:93-95,156-159` requires each `@e2e` scenario to commit
    `transcript.txt` and `transcript.ansi` under
    `givn/changes/<id>/evidence/visual/<scenario-slug>/`; the harness cannot
    produce that path for this change.
- Question: Where does the delta root and the evidence change id come from?
- Recommendation: Add an explicit test-harness task to `design.md`/`tasks.md`:
  derive the delta root from the active change (`givn/changes/*/specs` or a
  `GIVN_CHANGE`/`GIVN_CHANGE_ID` value), derive the evidence path from the same
  value, and assert after a run that
  `givn/changes/restore-tui-styling/evidence/visual/.../transcript.ansi` exists.
  For the single-scenario RED/GREEN command, either point `behave -n` directly
  at the delta file or fix `verify.sh` first. The current "runner unchanged"
  claim is false and must not survive into tasks.
- Resolution: unresolved — the design and the harness/runner task list must be
  updated before tasks are written.

### Q2 — The new visual steps have no concrete assertion contract, and the role model cannot express the design system

- Status: BLOCKING (testability)
- Evidence:
  - `design.md:179-188` names six phrases. Five are umbrella phrases:
    `the TUI renders the watn inbox layout and palette`,
    `the detail overlay renders the watn palette`,
    `the settings overlay renders the watn palette`,
    `the watched table renders the stack column in the watn palette`, and
    `the picker renders the watn palette`. Only the focus phrase names a
    concrete role (`design.md:183`).
  - `design.md:106-114` lists the roles as `accent`, `accent_bold`, `bright`,
    `body`, `gold`, `gold_bold`, `green`, `green_bold`, `rule`, `dim`. The
    design system requires a **bright bold** title span
    (`design-system.md:33-34`, `design-system.md:68-69`: `passuff` is "bright
    bold"), but no `bright_bold` role exists and `bright` is "normal"
    (`design-system.md:17`). The role record is a JSON array of
    `{\"text\": ..., \"role\": ...}` spans (`design.md:113-114`); it has no
    attribute field, so "bright bold" is not representable.
  - The focus phrase at `design.md:183` and
    `inbox-navigation.feature:17` asserts only the border colours. The
    proposal promises "the focused pane's border and title are green, the
    unfocused pane is dim" (`proposal.md:55-56`); the design system requires
    the focused title green and the unfocused title dim
    (`design-system.md:42-43`). The step does not assert the titles.
  - The new steps start as `raise NotImplementedError` stubs
    (`design.md:188`). RED can fail on the stub, but GREEN can satisfy the
    phrase by asserting one role anywhere; the plan gives no discriminating
    expectation. `design.md:251-253` then claims the styled renderer is covered
    by these steps.
- Question: Exactly which spans and roles must each phrase assert, and how is
  the bright-bold title represented in the role record?
- Recommendation: Add a per-phrase assertion table to `design.md` with concrete
  expected spans, for example:
  - main: `passuff` `bright_bold`, `— Inbox` `dim`; account `bright`, login
    `accent`, separators `dim`; unread count `gold_bold`; `Focus: ` `dim` and
    `PRs` `accent_bold`; table headers `accent_bold`; selected row `gold_bold`
    with `▶`; ignored row `dim` with `⊘`; controls title `accent`, keys
    `accent_bold`, labels `dim`;
  - activity detail: title `accent`, labels `accent`, values `dim`, rule
    separators `rule`, body `body`;
  - settings: selected label `gold_bold`, enabled values `green`, disabled
    values `dim`, hint keys `accent_bold`;
  - picker: `Mode: ` `dim`, mode `accent_bold`, `[x]` `green`, `[ ]` `dim`,
    selected repository `gold`, footer keys `accent_bold`;
  - stack column: `☰` header `accent_bold`, position cells with the expected
    roles.
  Add `bright_bold` to the `Role` list and to `docs/design/design-system.md`,
  or add an `attrs` field to the role-record spans and define the expected
  attributes. Extend the focus phrase to assert the focused title green and the
  unfocused title dim. Name the role-record capture mechanism (same flags,
  same keys, same state) in `givn/steps/support.py` and
  `tests/support/harness.py`. The D1 fallback ("if the role record is
  unavailable, the visual review reads the ANSI transcript",
  `design.md:33-34`) must name the trigger, the switch cost, and an owner, or
  be removed; the scenarios depend on the role record.
- Resolution: unresolved.

### Q3 — The plan promises screens and states with no scenario; the save prompt is not in the current TUI

- Status: BLOCKING (scope and missing scenarios)
- Evidence:
  - The proposed solution lists "modal overlays for activity detail,
    pull-request detail, help, settings, the save prompt, and the watch-scope
    picker" (`proposal.md:41-52`). The design's `Screens covered` lists the same
    (`design.md:151-152`, `design.md:129-130`).
  - The delta only covers the main frame, activity detail, focus, stack column,
    settings, and picker (`activity-inbox.feature:5-33`,
    `inbox-navigation.feature:5-27`, `stacked-inbox.feature:5-22`,
    `watch-scope.feature:5-15`). There is no scenario for the pull-request
    detail overlay, the help overlay, or the save prompt.
  - The design declares states `default`, `focused`, `selected`, `ignored`,
    `read`, `unread`, `enabled`, `disabled`, `empty`, `error`
    (`design.md:143-145`), but the palette of the ignored row, the empty
    inbox, and the error screens is not asserted; those permanent scenarios
    remain text-only.
  - The design declares the 80x24 minimum and 120x40 recommended viewports
    (`design.md:138`, `design-system.md:75-77`) and says the 80x24 minimum
    "must be checked" (`design.md:278-279`); no scenario renders at 80x24.
  - The traceability table claims T24 (pull-request detail), T21 (help), and
    T27 (ignored row) are evidenced by the modified navigation/detail
    scenarios (`design.md:228-231`), but those scenarios do not open or assert
    those overlays/states.
  - The save prompt is **not present** in the current TUI: `frame.py` renders
    `help`, `settings`, `picker`, `activity_detail`, and `pr_detail`
    (`passuff/tui/frame.py:189-214`); quit sets `view["quit"]` and the app
    calls `save_ui` unconditionally (`passuff/tui/app.py:195,439`). The
    original TUI has the save prompt and `QuitSave`/`QuitDiscard`
    (`/home/buster/projects/gh-notify/crates/passuff-cli/src/tui.rs:48-49,2008-2020`),
    and its keys are `y`/`n`/`Esc` (`tui.rs:1312-1330`).
  - The durable rule T22 already requires "Sort, direction, filters, ignored
    view, and notification settings persist across runs **when the developer
    confirms saving on exit**" (`givn/specs/review-activity-inbox/usecase.md:69`).
    The design calls the save prompt an "existing overlay"
    (`design.md:76-77,129`) but it is not in the rebuilt TUI, and the design
    also says the change "does not change keys" (`design.md:13-14`); the
    proposal's out-of-scope includes keyboard behaviour
    (`proposal.md:80-82`). These statements contradict each other.
- Question: Which screens and states are in scope, and is the T22 save prompt a
  behaviour restoration or out of scope?
- Recommendation: Add scenarios for the pull-request detail, help, and save
  prompt overlays, the ignored row, the empty/error/disabled states, and the
  80x24 viewport, or narrow the design's `Screens covered`/states to the
  scenarios the change actually proves. For the save prompt, make a recorded
  decision: if in scope (it is specified by T22 and listed in the proposed
  solution), add a save-prompt scenario and an `inbox-navigation` T22 row,
  restore the `y`/`n`/`Esc` prompt, and correct the "does not change keys" and
  out-of-scope statements; if out of scope, remove the save prompt from the
  proposed solution, the design, and the design system and record the operator
  decision. The design must not call a screen "existing" when the current tree
  does not render it.
- Resolution: unresolved — see the operator question on the save prompt.

### Q4 — The counts line has no reachable evidence path, and the arc42 runtime view is marked unchanged

- Status: BLOCKING (testability and arc42)
- Evidence:
  - `design.md:82-85` says `session.py` updates the view's `fetching`,
    `next_delay`, and `budget` fields "through the existing `on_report`
    callback"; the counts line renders the spinner, countdown, and
    `remaining/5000` "only when the session supplies them"
    (`design.md:84-85`).
  - The live loop calls `ObservationSession(...).start(stop)` with **no**
    callback (`passuff/tui/app.py:323`); `start` accepts `on_report`
    (`passuff/tui/session.py:60-71`), but `run_for` (used by
    `--observe-for`, `app.py:422`) never calls it
    (`passuff/tui/session.py:48-58`). The `--snapshot` path does not create a
    session. No path supplies the counts-line fields to the snapshot evidence.
  - The reference frames show `✳ fetching` and `4607/5000` on the counts line
    (`evidence/reference/main-inbox.png`); the design's main `@e2e` scenario
    has no assertion for the spinner, countdown, or budget.
  - `arc42.md:12` marks chapter 6 "No" because "the observation, session, and
    redraw flow is unchanged"; `docs/arc42/06-runtime-view.md:304` only says
    "the terminal loop reloads the inbox view, redraws, and preserves focus".
    The design adds a session-to-view update and a new counts-line state.
- Question: How do the fetching/countdown/budget fields reach the snapshot or
  the visual evidence, and does the runtime view change?
- Recommendation: Define the session-to-view contract (the report fields and
  the view keys), wire the callback in `_interactive` and in the snapshot/
  observe path (or define a last-report fixture), add a scenario that asserts
  the spinner/fetching/budget text and roles, and update `arc42.md` row 6 to
  "Yes" plus chapter 6's live-inbox step 5. If these fields are intentionally
  live-only and cannot be snapshot-tested, record that as a visual-contract
  limitation and remove them from the promised composition; do not leave the
  design claiming they are supplied while no scenario can observe them.
- Resolution: unresolved.

### Q5 — T25 contradicts the durable design system

- Status: BLOCKING (durable contract)
- Evidence:
  - `givn/specs/review-activity-inbox/usecase.md:72` (T25) says "**dim rules**,
    borders, and unfocused pane titles". `docs/design/design-system.md:23-24`
    defines `rule` as indexed 240 "horizontal rules and table dividers" and
    `dim` separately for borders, unfocused titles, metadata values, and
    secondary labels. The original `theme.rs` also has a distinct `rule()` (240)
    and `border()`/`dim()`.
  - T25's composition omits the counts/focus line; it does not say the table
    headers are bold (`design-system.md:44-51`); it omits the `▶`/`⊘` markers
    (`design-system.md:46-48`) and the green focused title
    (`design-system.md:42-43`).
  - The design's D2 makes the design system the durable source
    (`design.md:36-40`); the proposal's historical "dim rules" wording
    (`proposal.md:51`) has been copied into the durable rule instead of the
    design system.
- Question: Which durable wording is authoritative, and will T25 be corrected?
- Recommendation: Correct T25 to the design system: rule 240 for rules/table
  dividers; dim for borders, unfocused titles, metadata, and secondary labels;
  accent-bold table headers; the counts/focus line; the `▶`/`⊘` markers; the
  green focused border **and title**. Add the correction as a durable-doc task
  before implementation. Do not copy the proposal's "dim rules" again.
- Resolution: unresolved.

### Q6 — The palette has two owners

- Status: BLOCKING (component contract / single source)
- Evidence:
  - The `TerminalPresentation` contract is described as owning the "styled
    frame model, palette mapping, and picker overlay"
    (`design.md:67`), and `docs/arc42/05-building-block-view.md:73` says it owns
    "the role-tagged frame model and palette mapping".
  - `design.md:78-80` plans `passuff/tui/app.py` to "set up curses color pairs
    for the palette (accent 81, bright white/15, body 252, gold 214, green 114,
    rule 240)"; the indexed values would be duplicated in the app.
  - `docs/design/design-system.md:10-11` says "The `Role` names are the
    contract; the indexed value is the rendering"; the design system is the
    single durable source.
- Question: Which module owns the role-to-indexed/attribute table?
- Recommendation: One table (in `passuff/tui/frame.py`, or a small roles module)
  owns the role names, indexed values, and attributes. `render_ansi` and
  `render_roles` consume it, and `app.py` imports it to set up curses pairs.
  Remove the literal list from the `app.py` bullet and from any other renderer.
  The `bright_bold` role from Q2 belongs in the same table.
- Resolution: unresolved.

### Q7 — The glossary does not hold the new vocabulary, and "snapshot" is reused

- Status: BLOCKING (ubiquitous language)
- Evidence:
  - `docs/glossary.md` has no entry for `watn`, `palette`, `role`, `role
    record`, or `styled frame`. The design and the new step phrase use `watn`
    (`design.md:181`, `docs/design/design-system.md:10`), and T25 uses
    "watn's palette" (`usecase.md:72`).
  - `docs/glossary.md:33` lists "snapshot — a snapshot is any observation; the
    baseline is the first one", and `docs/glossary.md:46` says an Observation
    is "a snapshot of one pull request". The design uses "snapshot" for the
    rendered frame (`design.md:93-95`, `docs/design/design-system.md:90-92`);
    the two meanings are not distinguished.
  - `arc42.md:18` marks chapter 12 "No" and says the palette/role vocabulary
    belongs only to the design system. The design system is the right home for
    the values, but the glossary is the project's sole vocabulary authority and
    should at least delegate.
- Question: Where are `watn`, `palette role`, and `TUI snapshot` defined?
- Recommendation: Add a glossary entry or a delegation line for the terminal
  visual vocabulary: define `watn` as the original terminal styling the palette
  reproduces; define `TUI snapshot` as the `--snapshot` rendered-frame
  projection (distinct from an observation snapshot); state that the palette
  role names and indexed values are owned by
  `docs/design/design-system.md`; link that document from the glossary. Update
  the arc42 chapter 12 note to match. Define the term before the phrase
  `the watn inbox layout` is used in a scenario.
- Resolution: unresolved.

### Q8 — Contracts and components fail three checks

- Status: BLOCKING (component checks)
- Evidence:
  - The `TerminalPresentation` contract's consumer use cases are
    `review-activity-inbox` and `change-watch-scope` (`design.md:67`). The
    `stacked-inbox` capability belongs to `review-pull-request-stacks`, whose
    watched-table stack column is rendered by the same frame
    (`design.md:204,230`); that use case is missing from the contract row.
  - The same row lists "the visual step definitions" as consumers
    (`design.md:67`), but the design's Black-Box-First policy says no scenario
    imports `passuff.tui.frame` and the visual steps consume the public
    `--snapshot --roles` output (`design.md:218-222`). The step definitions are
    consumers of the CLI, not of the contract.
  - The picker screen has two owners: `ScopePicker` is "Terminal screen for the
    scope mode, known repositories, manual entry, and errors"
    (`docs/arc42/05-building-block-view.md:65`), while `TerminalPresentation`
    claims "picker overlay" (`design.md:67`) and "modal overlays"
    (`docs/arc42/05-building-block-view.md:73`). The design does not say which
    component owns the picker's state, interaction, and rendering.
  - The palette mapping is claimed by both `TerminalPresentation`
    (`design.md:67`) and `TuiApp` (`docs/arc42/05-building-block-view.md:72`,
    `design.md:78-80`), which is the same duplicate-ownership problem as Q6.
- Question: What are the contract's consumers and the component split?
- Recommendation: Add `review-pull-request-stacks` to the contract's consumer
  use cases. Replace "the visual step definitions" with the `--snapshot` CLI
  entry point. State explicitly that `TerminalPresentation` owns the frame
  model, role/palette mapping, and overlay rendering; `ScopePicker` owns the
  picker's state and interaction and delegates its rendering to
  `TerminalPresentation`; `TuiApp` owns key dispatch, view state, and the
  curses drawing loop and imports the role table rather than duplicating it.
  Update the chapter 5 rows to match.
- Resolution: unresolved.

### Q9 — The Visual Design Contract records the reference but not the approved direction

- Status: OPERATOR QUESTION
- Evidence: `design.md:135-161` contains the required `Interface: terminal`,
  viewports, design system, primitives, states, screens, reference frames,
  capture mechanism, and motion. It records the three reference frames from
  the original recording but does not record the operator's approval of that
  direction or its source/date. The change brief asks to restore the original
  styling, but the artifact does not record that decision.
- Question: Has the operator approved the original watn composition and
  indexed palette (the three committed frames) as the design direction? If
  not, two or three rendered options must be presented and one approved.
- Recommendation: Record the approval source and date in the `Rendered
  reference` line of the Visual Design Contract. If the operator has not
  approved the original as the direction, present rendered options and record
  the chosen one before this review signs off.
- Resolution: OPEN — operator answer required.

### Q10 — Minor defects with concrete fixes

- Status: MINOR (each needs a design/artifact edit)
- Evidence and recommendation:
  - **D2 has no proposal source.** `design.md:36-40` says "Decides D2"; the
    proposal's only open question is D1 (`proposal.md:87-108`). Relabel D2 as a
    design decision without a `D<n>` or add the question to the proposal; do
    not leave a ledger entry whose ID is absent from the proposal.
  - **The 256-colour fallback is undefined.** `design.md:275-277` says the
    palette "falls back to the named colors when `curses.COLORS` is
    insufficient"; `docs/design/design-system.md:13-24` gives no fallback
    mapping; R-037 repeats the fallback (`docs/arc42/11:43`). Define the exact
    named-colour mapping (accent, bright, body, gold, green, rule, dim, and
    bright_bold) or require 256 colours and delete the fallback claim. No
    scenario covers it.
  - **The added scenarios do not assert the exit status.**
    `inbox-navigation.feature:20-27` and `watch-scope.feature:5-15` end with the
    palette step and no `And the exit status is 0`. Add the assertion (or make
    the palette step assert it); otherwise a crash after printing the frame
    could pass.
  - **"Two header lines" contradicts the design system.**
    `design.md:121` says "Two header lines and one counts/focus line";
    `docs/design/design-system.md:29-40` defines one header line (account,
    logins, watch scope) and one counts/focus line. Align the design.
  - **The D1 fallback lacks the carried-decision fields.** `design.md:33-34`
    says the ANSI transcript is the default "if the role record is
    unavailable", but gives no trigger, switch cost, or durable owner, and the
    scenarios depend on the role record. Make the role record foundational or
    define the SGR-parsing fallback in the steps with the four fields.
  - **The traceability table labels regular scenarios as E2E evidence.**
    `design.md:229,231` lists the added settings and picker scenarios under
    `E2E evidence`; those scenarios are `@givn.added` without `@e2e`. Relabel
    the column (Evidence) or list the permanent `@e2e` titles, and include
    T22 in the `inbox-navigation` row (Q3).
  - **The role record is not committed.** `docs/design/design-system.md:96`
    says the visual review reads the role record, but `design.md:156-159` only
    commits `transcript.txt` and `transcript.ansi`. Commit the role record per
    scenario or amend the design system to say the review reads the styled
    transcript.
  - **The 80x24 viewport has no evidence.** `design.md:138` and
    `docs/design/design-system.md:75` declare 80x24 minimum; no scenario
    renders at 80x24. Add a viewport scenario or drop the minimum claim (Q3).
  - **The two long scenarios are advisory only.** The modified activity-detail
    scenario (`activity-inbox.feature:18-33`, 16 steps) and the modified
    stacked scenario (`stacked-inbox.feature:6-22`, 15 steps) exceed the lint
    threshold. Keep them for now and record the final review's split-or-keep
    decision; do not add another step to them without revisiting the split.
  - **The new CLI flags are public output.** `--snapshot --ansi` and
    `--snapshot --roles` are added to `passuff-tui`; the design should name
    them in the public interface/README impact discussion for the final
    review. The proposal's D1 covers the evidence mechanism, so this is not a
    scope change.
- Resolution: unresolved (all are design/artifact edits).

### Q11 — Arc42 and ADR assessment

- Status: MINOR (the chapter 6 omission is blocking; see Q4)
- Evidence and recommendation:
  - The independent 12-row verdict is in the branch below. My verdict differs
    from `arc42.md` only on chapter 6, which I mark "Yes" because the design
    changes the session-to-view update and the live-inbox counts line
    (`design.md:82-85`, `passuff/tui/app.py:323`,
    `docs/arc42/06-runtime-view.md:304`).
  - No new ADR is required. The cheaper home is `design.md` §Contracts and
    Components plus `docs/arc42/05-building-block-view.md:73` and
    `docs/design/design-system.md`; `TerminalPresentation` is a presentation
    component inside the existing terminal adapter, and the palette is a
    durable visual constraint owned by the design system. The assessment's
    wording "the change adds no lasting component" (`arc42.md:24-28`) is
    imprecise: `TerminalPresentation` is recorded as a component in chapter 5.
    State that the frame module is pre-existing and the contract is change-
    local; no new ADR.
  - Chapter 1's quality-goal table (`docs/arc42/01-introduction-and-goals.md:33-41`)
    has no visual-fidelity row; the new QS-034 lives in chapter 10. The "No"
    for chapter 1 is acceptable if visual fidelity is recorded as a refinement
    of Inspectability (priority 3); add that sentence to `arc42.md`.
  - Chapter 9's register has no new ADR; correct. Chapter 12's "No" is
    acceptable only with the glossary delegation from Q7.
  - The structural model: `docs/domain-model.md` has no new aggregate,
    entity, or relationship; the frame model is internal to the terminal
    adapter. The design should record that no-impact assessment (a no-op edit
    to the model is not evidence, so state the reason rather than editing it
    silently).
- Resolution: unresolved (chapter 6 and the wording).

## Grilling branches

### Scope

- The proposal's proposed solution (all overlays and the counts line,
  `proposal.md:41-52`) is not matched by the spec: the PR detail, help, and
  save prompt are missing, and the counts-line spinner/budget has no assertion
  (Q3, Q4).
- The modified scenarios preserve every permanent step and add only the visual
  contract: `activity-inbox.feature:5-15` vs the permanent "Developer opens the
  inbox TUI"; `:17-33` vs "Developer opens an activity's details";
  `inbox-navigation.feature:5-18` vs "Developer navigates the inbox panes";
  `stacked-inbox.feature:5-22` vs "Developer reviews a stacked inbox". Verified
  step-for-step; no permanent step is lost.
- The added settings and picker scenarios are regular variants of the existing
  `@e2e` actions; they assert a palette invariant distinct from the text
  outcome, so they are not duplicate E2E actions. The lint subset advisories
  are expected; the final review records `variant`.
- The save prompt is the scope contradiction (Q3): the proposal lists it, the
  proposal's out-of-scope excludes keyboard behaviour, the design calls it
  existing, the current TUI does not render it, and T22 requires the "confirms
  saving on exit" behaviour. This needs an operator decision.
- The added `--snapshot --ansi` and `--snapshot --roles` flags are inside the
  proposal's D1 evidence question (`proposal.md:89-108`); they are not a scope
  change, but the design must name them as public output.

### Tech choices

- "Both" in D1 is the right evidence mechanism: an ANSI transcript for the
  human artifact and a structured role record for assertions
  (`design.md:28-34`, `design-system.md:90-97`). The alternatives in the
  proposal are real; the choice is not over-engineered.
- The role model is incomplete (`bright_bold`, Q2); the palette has two owners
  (Q6); the 256-colour fallback is undefined (Q10). These are the technical
  defects, not the choice of ANSI/role record.
- `curses` remains the right terminal adapter; the design adds no dependency
  and keeps the plain `render()` projection (`design.md:70-77`). `Textual` or a
  screenshot-only artifact would be heavier or weaker; the design's choice is
  sound once the role table is single-sourced.
- The design's "the role record is a JSON array of lines, each an array of
  spans" (`design.md:113-114`) is a simple, testable format; add the
  `bright_bold`/attribute decision (Q2).

### Missing scenarios

- Missing screens: PR detail, help, save prompt (Q3).
- Missing states: ignored row (`⊘` dim), empty inbox, configuration/state
  error, settings disabled, selected-row gold (the umbrella phrases do not
  name them) (Q2/Q3).
- Missing viewports: 80x24 minimum, 120x40 recommended (Q3/Q10).
- Missing counts-line evidence: fetching spinner, countdown, API budget (Q4).
- Missing save-prompt behaviour: T22's "confirms saving on exit" has no
  scenario, and the current TUI auto-saves (Q3).
- The permanent empty/error/ignored scenarios exist as text scenarios; they
  are not modified to assert the palette. If the visual contract declares
  those states, either add the palette assertion to those scenarios or remove
  the states from the contract.

### Testability

- The design names the strict-mode mechanism (`behave` fails undefined steps
  plus `scripts/check_steps.py` for no-op bodies, `design.md:244-246`,
  `scripts/check_steps.py:42-70`), the not-implemented stub pattern
  (`raise NotImplementedError("<phrase>")`, `design.md:188`), and a single-
  scenario run command (`design.md:242-243`). The stub and strict-mode parts
  are correct; the run command does not select the delta (Q1).
- The six new steps: one is concrete (focus border), five are umbrella phrases
  with no per-step expectation (Q2). The role record cannot express the
  `bright_bold` title (Q2). The focus step omits the focused/unfocused titles
  (Q2).
- The added settings/picker scenarios do not assert the exit status (Q10).
- The `--roles`/`--ansi` outputs are testable in principle; the design must
  name the capture mechanism and the per-phrase expectations before the tasks
  are written.
- The 80x24 viewport has no test (Q3/Q10); the counts line has no test (Q4).
- `GIVN_RESULT_SCOPE=change` exists in `scripts/verify.sh:76-85`, but the delta
  list is empty (Q1); the runner contract's `GIVN_FEATURES` path is not
  documented in the design.

### E2E fidelity

- The interface is a terminal TUI; the real driver is the `passuff-tui`
  subprocess with `--snapshot`, `--keys`, `--picker`, and `--observe-for`. The
  modified `@e2e` scenarios keep their text primary assertions and add the
  visual contract; the added settings/picker scenarios are regular variants,
  not extra `@e2e` actions.
- The opposite classification (Web UI) does not hold: the surface is a curses/
  ANSI terminal frame, not HTML/CSS/JS rendered in a browser; no browser
  driver applies, and the evidence is a transcript, not a page. The design
  should state this rationale (minor).
- No `@e2e` tag is removed; the four delta files preserve the tags
  (`activity-inbox.feature:5,17`; `inbox-navigation.feature:5`;
  `stacked-inbox.feature:5`); the added settings/picker scenarios are regular
  (`inbox-navigation.feature:20`; `watch-scope.feature:5`).
- The visual assertions use the real CLI output (`--snapshot --roles`); the
  Black-Box-First policy is correct (`design.md:218-222`). The evidence path
  and runner selection are the fidelity blockers (Q1), not the driver.
- The save prompt, if restored, is driven through the same real TUI; it needs
  an `@e2e` or regular scenario that presses `y`/`n`/`Esc` (Q3).

### Interaction Coverage

- The use-case inventory is exact: `review-activity-inbox` has 7 entries
  (`usecase.md` §Interactions: activity-inbox 5, inbox-navigation 2),
  `review-pull-request-stacks` has 1, `change-watch-scope` has 3; total 11.
  The design's matrix has 11 rows (`design.md:195-207`). No missing or extra
  row.
- Each inventory entry has a matching `@e2e` title; the modified/new visual
  scenarios are variants of those actions. The matrix's driving mechanisms
  `--snapshot`, `--snapshot --keys`, `--observe-for 4000`, and the picker flags
  are the real CLI (subject to Q1/Q4).
- The matrix's `Real interface` and `Driving mechanism` columns are correct,
  but the `Change` text overstates the evidence for the PR detail/help/ignored
  states (Q3). The matrix is complete; the scenarios are not.

### Risk

- The single most likely failure is the hardcoded `python-rebuild` paths (Q1):
  the implementation would run no delta scenario, write evidence into the
  wrong change directory, and the final visual review would find no evidence.
  The mitigation is the explicit path task and an evidence-existence assertion.
- The second risk is coverage theatre (Q2/Q3): the umbrella steps pass while
  the overlay/states remain unstyled, and the missing screens are never
  verified.
- The third is the save-prompt scope conflict (Q3): the implementation either
  changes the quit behaviour against the proposal's out-of-scope clause or
  omits a T22-required screen.
- The design's own risk list (`design.md:269-280`) names palette drift,
  size-sensitivity, the 256-colour fallback, and the spinner wiring; it does
  not name the stale runner/evidence paths or the missing screen coverage. Add
  both.

### Use-case and Persona context

- The owning use cases are `review-activity-inbox` (activity-inbox,
  inbox-navigation), `review-pull-request-stacks` (stacked-inbox, Extends
  `review-activity-inbox` with the R9 condition, `usecase.md:54,91-93`), and
  `change-watch-scope` (watch-scope). The change preserves the goal, main
  flow, rules, and typed relationships; it adds the T25 visual rule.
- The use-case scope rejection criteria: the goals are single outcomes; the
  actors and triggers are concrete; the capability ownership matches the
  proposal's routing; the extensions reference their steps; no capability is
  placed under the wrong use case. The one visible rule defect is T25 (Q5).
- Persona `multi-account-developer` is confirmed
  (`givn/personas/multi-account-developer.md:3-5`); the design records the
  screens in scope (`design.md:233-235`). Gherkin actors remain "developer";
  no Persona biography appears as an actor. No invented Persona.
- The save prompt's T22 rule is in `inbox-navigation`; the design's
  traceability row omits T22 (Q3/Q10).

### Decision ledger

- D1 resolves in the design: "Both a styled transcript and a structured role
  record" (`design.md:28-34`). The proposal's D1 is the only proposal question
  (`proposal.md:89-108`); it is decided, not carried.
- The design's D2 (`design.md:36-40`) has no proposal question ID; relabel it
  (Q10). Its "trigger: a theme change; switch cost: one design-system update
  plus the palette assertion; owner: this change" is a future-change note; the
  owner is not durable. D2 is decided, so the carried-decision fields are not
  strictly required, but the wording should not imply a carried decision.
- The D1 fallback is the only carried-like element; it lacks trigger, switch
  cost, and owner and the scenarios depend on the role record (Q2).
- No other proposal `D<n>` is unresolved.

### Contracts and components

- `TerminalPresentation` is the only declared contract; its consumer use cases
  and consumers are incomplete (Q8), and its palette ownership conflicts with
  `TuiApp` (Q6). The stack capability's use case is missing from the row.
- `ScopePicker` and `TerminalPresentation` both claim the picker screen (Q8);
  `TuiApp` and `TerminalPresentation` both claim the palette (Q6).
- The visual step definitions are listed as contract consumers while the
  design also promises no scenario imports the frame (Q8); the CLI snapshot
  entry point is the real consumer.
- The internal helpers (`_sort_prs`, `markdown_lines`, `_activity_type`) are
  correctly internal and need no contract. `tests/support/harness.py` is test
  support, not a product component; no contract is required.
- The composition root (the CLI entry point) is the only place that should
  name implementations; the design should say so for `frame.py`/`app.py`
  (Q6/Q8).

### Decision timing

- D1 is foundational and is decided now; correct. D2 is decided now; correct.
- The D1 fallback is a carried hedge without the four fields and the scenarios
  depend on the role record; make it foundational or complete the fields (Q2).
- The 256-colour fallback is an unresolved behaviour with no default mapping;
  decide it now or remove it (Q10).
- The 80x24/120x40 viewports are declared now but not tested; that is an
  unenforced contract, not a carried decision (Q3/Q10).
- The save prompt is a behaviour decision, not a deferred design question;
  it needs the operator's scope decision now (Q3).

### Visual design contract

- The section exists with `Interface: terminal` and viewports
  (`design.md:137-139`); the design system, tokens, primitives, states,
  screens, rendered reference, capture mechanism, and motion are present
  (`design.md:140-161`).
- The approved direction is not recorded (Q9). The three reference frames are
  the rendered reference, but the operator's approval/source is absent.
- The declared screens and states exceed the scenarios (Q3); the capture path
  is wrong for this change (Q1); the role record is not committed (Q10).
- The `Interface: terminal` classification is correct; add the opposite Web UI
  rationale (E2E fidelity branch).
- The save prompt is in the screens list but not in the current TUI (Q3).
- The design system's `rule`/`dim` distinction is correct; T25 must follow it
  (Q5).

### Deferrals

- No mandatory check is explicitly deferred. What is effectively deferred
  without an operator decision: the counts line (Q4), the 80x24 viewport, the
  missing screens/states (Q3), the 256-colour fallback (Q10), and the save
  prompt scope (Q3). A deferral of a mandatory check requires the operator's
  recorded decision; none is present, so the review cannot sign off.

### Persona disposition

- `multi-account-developer` is confirmed 2026-11-02
  (`givn/personas/multi-account-developer.md:3-5`). Relevance: the developer
  keeps a terminal open and scans the inbox for comments, reviews, CI, merges,
  stacks, and the watch scope; the palette, focus, and selection restore the
  scan the Persona uses. Evidence: the three committed reference frames and the
  final visual review; the design records the screens
  (`design.md:233-235`). The design does not record the Persona-specific
  outcome (what the Persona must see to reach the goal) or the evidence plan;
  the final visual review must record it per Persona. No issue found for
  invented Personas or biography actors.

### Interface classification

- The design declares `Interface: terminal` (`design.md:137`). The opposite
  classification — Web UI — does not hold: the surface is a curses/ANSI
  terminal frame, no HTML/CSS/JS is rendered in a browser, no browser driver
  is used, and the evidence is a transcript. The terminal classification is
  correct; the design should state this rationale so the opposite test is
  recorded (minor).

### ADR qualification

- No new ADR. The frame model is a presentation component inside the existing
  terminal adapter; the durable visual constraint is owned by
  `docs/design/design-system.md`; the contract is recorded in `design.md` and
  chapter 5. The cheaper home is complete: `design.md` §Contracts and
  Components + `docs/arc42/05-building-block-view.md:73` +
  `docs/design/design-system.md`. `arc42.md:22-39` records the four
  qualification answers; the "adds no lasting component" wording should say
  the frame module is pre-existing and the contract is change-local (Q11).
  No NEW_ADR, AMEND_ADR, or SUPERSEDE_ADR is required.

### Architecture documentation

The twelve-row verdict was derived before comparing with `arc42.md`.

| # | Chapter | My verdict | `arc42.md` | Match? | Content check |
|---|---|---|---|---|---|
| 1 | Introduction and Goals | No | No | Yes | Goals/stakeholders unchanged; QS-034 is in chapter 10. Add a note that visual fidelity refines Inspectability (`docs/arc42/01:33-41`). |
| 2 | Architecture Constraints | No | No | Yes | Python, curses, Gherkin, and the palette are existing constraints; the 256-colour requirement/fallback is a risk (R-037), not a new constraint. |
| 3 | Context and Scope | No | No | Yes | The same terminal surface; the snapshot projections are the terminal transcript interface already recorded (`docs/arc42/03:38-51`). |
| 4 | Solution Strategy | Yes | Yes | Yes | The terminal UI row is updated to the styled frame/role projections (`docs/arc42/04:77`); content matches the design. |
| 5 | Building Block View | Yes | Yes | Yes | `TerminalPresentation` is added (`docs/arc42/05:73`); `TuiApp`/`ObservationSession` updated. The `ScopePicker`/`TerminalPresentation` overlap is Q8. |
| 6 | Runtime View | Yes | No | **No** | The live-inbox flow changes: the session report updates the view/counts line (`design.md:82-85`, `passuff/tui/app.py:323`); `docs/arc42/06:304` only says the loop reloads/redraws. `arc42.md:12` is an omission (Q4). |
| 7 | Deployment View | No | No | Yes | No dependency, service, port, or deployment change; the TUI stays a stdlib `curses` binary. |
| 8 | Cross-cutting Concepts | Yes | Yes | Yes | The palette roles, projections, accessibility, and evidence path are added (`docs/arc42/08:137`); content matches the design. |
| 9 | Architecture Decisions | No | No | Yes | No new ADR; the register is unchanged and correct. The qualification wording is Q11. |
| 10 | Quality Requirements | Yes | Yes | Yes | QS-034 is added (`docs/arc42/10:66`) and matches the visual contract. |
| 11 | Risks and Technical Debt | Yes | Yes | Yes | R-037 is added (`docs/arc42/11:43`) and reflects palette drift and the fallback. The missing runner/evidence risk is Q1. |
| 12 | Glossary | No | No | Yes | No domain term is added in the design's own vocabulary; the palette/role vocabulary is the design system's (Q7 requires a glossary delegation). |

**Chapter-file integrity scan (all 12 files).** All twelve files exist under
`docs/arc42/`; `README.md` and `adr/` exist. Every file has substantive content
or a deliberate register/link; the only `placeholder` matches are policy text
in `docs/arc42/README.md:62,65`, not scaffold placeholders. Zero box-drawing
characters (`┌┐└┘│─├┤┬┴┼╔╗╚╝║═╠╣╦╩╬╭╮╰╯`) were found. Zero non-table ASCII
pseudo-diagrams (`+---+`, `===`, multi-line `|` blocks) were found. Mermaid
diagrams are present in chapters 03, 05, 06, 07, and 10. The integrity finding
is content, not file absence or ASCII art: chapter 6 (Q4), chapter 5
(component overlap, Q8), and chapter 1 (quality-goal note, Q11).

### Ubiquitous language

- The glossary is the vocabulary authority; the design adds `watn`, `palette`,
  `role`, `role record`, `styled frame`, and the `snapshot` reuse (Q7).
- T25 uses `dim rules` against the design system's `rule` 240 (Q5).
- The design's contract names (`TerminalPresentation`, `Role`, `Span`,
  `StyledLine`) are design-level names; the role names are the durable
  contract and belong with the design system. The glossary should delegate,
  not duplicate, the palette values.

### Structural model

- No new aggregate, entity, or relationship: the Watched Pull Request,
  Activity Record, Notification, Account, Watch Scope, and read models are
  unchanged; the frame model is internal to the terminal adapter. The design
  does not state this assessment; record it in `design.md`/`arc42.md` rather
  than editing `docs/domain-model.md` silently. No domain-model.md edit is
  required for this presentation change.

## Findings and decisions

| ID | Severity | Finding | Evidence | Required action |
|---|---|---|---|---|
| F1 | blocking | Delta runner and evidence path hardcoded to `python-rebuild`; the single-scenario command cannot run the delta | `scripts/verify.sh:51`, `tests/support/harness.py:54`, `design.md:239-243,156-159` | Derive the delta root and change id; update the runner/harness; assert the evidence path; fix the single-scenario command (Q1) |
| F2 | blocking | Five umbrella visual steps; role model lacks `bright_bold`; focus step omits the titles | `design.md:179-188,106-114`, `design-system.md:33-34,42-43,68-69`, `inbox-navigation.feature:17` | Add the per-phrase span/role table and `bright_bold`/attrs; extend the focus step (Q2) |
| F3 | blocking | PR detail, help, save prompt, ignored/empty/error/disabled states and 80x24 have no palette scenario; the save prompt is not in the current TUI and changes quit keys; T22 is unhandled | `proposal.md:41-52`, `design.md:129-152,143-145,228-231`, `passuff/tui/frame.py:189-214`, `passuff/tui/app.py:195,439`, `usecase.md:69`, original `tui.rs:48-49,1312-1330,2008-2020`, `design.md:13-14`, `proposal.md:80-82` | Add the missing scenarios; decide the save prompt in scope/out of scope; correct the key/out-of-scope statements; add the T22 row (Q3) |
| F4 | blocking | Counts-line fields have no reachable snapshot evidence; arc42 chapter 6 marked No | `design.md:82-85`, `passuff/tui/app.py:323,422`, `passuff/tui/session.py:48-71`, `arc42.md:12`, `docs/arc42/06:304` | Define the session-to-view contract and a test path; update arc42 row 6 and chapter 6 (Q4) |
| F5 | blocking | T25 contradicts the design system (`dim rules`, missing counts/focus line, missing bold headers/title/markers) | `usecase.md:72`, `design-system.md:23-24,42-51` | Correct T25 to the design system (Q5) |
| F6 | blocking | The palette has two owners; app.py duplicates the indexed values | `design.md:67,78-80`, `design-system.md:10-24`, `docs/arc42/05:72-73` | Single role table owned by the frame/presentation component; app.py imports it (Q6) |
| F7 | blocking | Glossary lacks `watn`/palette/role terms; `snapshot` reused with two meanings | `docs/glossary.md:33,46`, `design.md:93-95,106-114,181`, `arc42.md:18` | Add the glossary entries/delegation; define the terms (Q7) |
| F8 | blocking | Contract table omits `review-pull-request-stacks`; lists step definitions as consumers; `ScopePicker`/`TerminalPresentation` and `TuiApp`/`TerminalPresentation` overlap | `design.md:67,218-222`, `docs/arc42/05:65,72-73` | Correct the consumer use cases/consumers and the component split (Q8) |
| F9 | operator | The visual direction (original reference) is not recorded as approved | `design.md:135-161` | Record the operator's approval/source or present options (Q9) |
| F10 | minor | D2 has no proposal question ID | `design.md:36-40`, `proposal.md:87-108` | Relabel D2 as a design decision (Q10) |
| F11 | minor | 256-colour fallback mapping undefined; no scenario | `design.md:275-277`, `design-system.md:13-24`, `docs/arc42/11:43` | Define the named-colour mapping or require 256 colours (Q10) |
| F12 | minor | Added settings/picker scenarios lack the exit-status assertion | `inbox-navigation.feature:20-27`, `watch-scope.feature:5-15` | Add `And the exit status is 0` (Q10) |
| F13 | minor | "Two header lines" contradicts the design system's one header line + counts/focus line | `design.md:121`, `design-system.md:29-40` | Align the design (Q10) |
| F14 | minor | D1 fallback lacks trigger/switch cost/owner; the scenarios depend on the role record | `design.md:33-34,188` | Make the role record foundational or complete the carried fields (Q10) |
| F15 | minor | Traceability table labels regular scenarios as E2E evidence and omits T22 | `design.md:229,231`, `inbox-navigation` traceability | Relabel/complete the evidence column; add T22 (Q10/Q3) |
| F16 | minor | The role record is not committed while the design system says the visual review reads it | `design-system.md:96`, `design.md:156-159` | Commit the role record or amend the design system (Q10) |
| F17 | minor | 80x24 viewport has no evidence | `design.md:138`, `design-system.md:75` | Add a viewport scenario or drop the claim (Q10) |
| F18 | minor | Arc42 chapter 1/10 note and the "adds no lasting component" wording | `arc42.md:24-28`, `docs/arc42/01:33-41` | Add the refinement note and the pre-existing-frame wording (Q11) |
| F19 | advisory | Two long scenarios and the lint subset advisories | `activity-inbox.feature:18-33`, `stacked-inbox.feature:6-22`, `givn lint` | Final review records split-or-keep/overlap dispositions; no design change required now |

`givn lint --change restore-tui-styling` exits 2: six `@wip` findings (expected
before implementation) and the long-scenario/subset advisories. No blocking
lint error.

## Operator questions — resolved 2026-11-08

1. **Save prompt scope — RESOLVED (Option 2: include the prompt).** T22
   requires saving on confirmation
   (`givn/specs/review-activity-inbox/usecase.md:69`), the proposed solution
   lists the save prompt (`proposal.md:49`), but the original proposal's
   out-of-scope excluded keyboard behaviour (`proposal.md:80-82`), the design
   said the change "does not change keys" (`design.md:13-14`), and the current
   TUI auto-saves with no prompt (`passuff/tui/app.py:195,439`). The operator
   answered on 2026-11-08: include the save-on-quit prompt. The proposal,
   design, out-of-scope, design system, T22, and R-038 now agree
   (`proposal.md:7,39-41,62-66,88-91`, `design.md:12-15,43-48`,
   `docs/design/design-system.md:72-74`, `docs/arc42/11:44`). The remaining gap
   is test coverage of the `n` and `Esc` branches (R2-4), not the scope.
2. **Visual-direction approval — RESOLVED (approved 2026-11-08).** The
   operator approved the original watn composition and indexed palette (the
   three committed reference frames) as the design direction on 2026-11-08
   (`proposal.md:39-41`, `design.md:12-13`, and the frames under
   `givn/changes/restore-tui-styling/evidence/reference/`). No rendered options
   are outstanding.

## Sign-off

The review context is recorded at the top. The plan cannot be signed off:

- Q1, Q2, Q3, Q4, Q5, Q6, Q7, and Q8 are blocking and unresolved.
- Q9 is an open operator question.
- D1 resolves; D2 is decided but mislabelled.
- The modified scenarios preserve the permanent steps; the new steps are not
  concrete enough; the missing screens/states are real.
- The arc42 independent verdict differs on chapter 6; the chapter-file
  integrity scan is clean.
- The interaction inventory is complete (11/11), the E2E tags are intact, and
  the terminal interface is correctly classified; those branches pass.
- No mandatory check is deferred with an operator decision; the effective
  deferrals in Q3/Q4/Q10 prevent a pass.

Earlier round verdict: FAIL (superseded)

---

## Re-review (2026-11-08) — after the operator decisions and hardening

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-2

This is the second independent pass. The original review above is unchanged;
the prior findings are re-checked, the operator decisions are recorded, the new
findings are listed, and the new verdict is at the end.

Artifacts read in this pass: `proposal.md`, all four delta `.feature` files,
`design.md`, `arc42.md`, the three owning use cases, the permanent specs,
`docs/design/design-system.md`, `docs/glossary.md`, `docs/domain-model.md`, the
twelve `docs/arc42/` chapters and the ADR register, `scripts/verify.sh`,
`tests/support/harness.py`, the current TUI modules, and the original reference
(`/home/buster/projects/gh-notify`: `tui.rs`, `theme.rs`, `scope.rs`, the three
committed frames). No git history was used. Only this file was written.

### Operator decisions (recorded, not invented)

**Save-prompt question — answered 2026-11-08.**

- **Option 1 — out of scope.** Meaning: keep the rebuilt quit behavior (`q`
  saves and exits, no prompt) and remove the prompt from the proposal, design,
  and design system. Gain: no quit-key change and one less screen; the change
  stays purely visual. Cost: T22's "persists when the developer confirms saving
  on exit" stays unimplemented and the restored UI differs from the approved
  original. Example: `q` writes `ui.json` and exits with no question.
- **Option 2 — include the prompt (chosen).** Meaning: `q` opens
  `Save sort and filter settings?` only when the sort, direction, filters,
  ignored view, or notification settings changed; `y` persists and exits, `n`
  discards and exits, `Esc` cancels; with no change `q` exits without a prompt.
  Gain: T22 is honored and the operator's approved original quit contract is
  restored. Cost: new quit keys `y`/`n`/`Esc`, a new prompt screen, and every
  quit test must know the settings state. Example: `s` then `q` shows the
  prompt; `y` writes `"sort": "name"` to `ui.json` and exits; `n` exits without
  writing; `Esc` returns to the inbox.
- **Operator decision: Option 2.** Recorded in
  `proposal.md:7,39-41,62-66,88-91`, `design.md:12-15,43-48`,
  `givn/specs/review-activity-inbox/usecase.md:69` (T22),
  `specs/review-activity-inbox/inbox-navigation.feature:95-109`,
  `docs/design/design-system.md:72-74`, and
  `docs/arc42/11-risks-and-technical-debt.md:44` (R-038).

**Visual-direction question — answered 2026-11-08.** The operator approved the
original main-branch composition and indexed palette (the three committed
reference frames) as the target. Recorded in `proposal.md:39-41`,
`design.md:12-13`, and the committed frames under
`givn/changes/restore-tui-styling/evidence/reference/`. No rendered options are
outstanding.

### Prior finding re-check

| ID | Prior finding | Status | Evidence |
|---|---|---|---|
| F1 | Delta runner and evidence path hardcoded to `python-rebuild` | Resolved in the plan | `design.md:101-107,151-162,290-308,323-324`; the current `scripts/verify.sh:56` and `tests/support/harness.py:54` are the code to change; the plan names the active-change delta and `GIVN_CHANGE_ID` |
| F2 | Umbrella visual steps; no `bright_bold`; focus titles missing | Resolved | `design.md:119-126,210-238`; `design-system.md:18`; `inbox-navigation.feature:17-20`; the phrase list is concrete |
| F3 | PR detail, help, save prompt, ignored, settings, picker, 80x24 missing | Resolved in scope | `inbox-navigation.feature:24-35,37-56,58-68,84-93,95-109`; `activity-inbox.feature:51-59`; `watch-scope.feature:5-18`; operator Option 2 |
| F4 | Counts-line fields unreachable; arc42 chapter 6 "No" | Resolved | Live fetching/countdown/budget removed (`proposal.md:92-93`, `design.md:190-192`, `design-system.md:40-41`); `arc42.md:12`; `docs/arc42/06-runtime-view.md:340-353` |
| F5 | T25 contradicted the design system | Resolved | `usecase.md:72` now records rule 240, dim borders/titles, accent-bold headers, gold selected rows/`▶`, green focused borders/titles, the counts line; matches `design-system.md:13-25` |
| F6 | Palette had two owners | Resolved | `design.md:83-93,119-122` (`frame.py` owns `ROLE_STYLES`; `app.py` imports it); `design-system.md:10-11`; `glossary.md:63` |
| F7 | Glossary missing `watn`/palette/role terms | Resolved | `glossary.md:63-66` adds `Watn palette`, `Palette role`, `Styled frame`, `Role record`; `arc42.md:18` |
| F8 | Contract consumers and component split incomplete | Resolved in the design | `design.md:71-79` (three consumer use cases, `TuiApp`/`--snapshot`, `ScopeEditor` split); `docs/arc42/05:72-73`; the chapter-5 `ScopePicker` wording is R2-9 |
| F9 | Visual direction not recorded as approved | Resolved | `design.md:12-13`; `proposal.md:39-41`; operator approval 2026-11-08 |
| F10 | `D2` has no proposal question ID | Still open (minor) | `design.md:37-41` "Decides D2" and `:43-48` "Decides D3"; `proposal.md:99-119` has only D1 |
| F11 | 256-colour fallback mapping undefined | Still open (minor) | `design.md:332-334`; `docs/arc42/11:43` (R-037); the design system has no fallback mapping |
| F12 | Added settings/picker scenarios lack exit status | Partial (minor) | `inbox-navigation.feature:24-35,84-93`; `activity-inbox.feature:51-59`; `watch-scope.feature:5-18`; the save-prompt scenario `:109` has it; the permanent corpus also varies (14/36) |
| F13 | "Two header lines" contradicted the design system | Resolved | `design.md:134-135`; `design-system.md:36-41` |
| F14 | D1 fallback lacks trigger, switch cost, owner | Still open (minor) | `design.md:34-35` |
| F15 | Traceability column labelled E2E evidence; T22 missing | Partial (minor) | T22 and the save-prompt scenario are now in `design.md:281`; the column still lists regular scenarios; T24/T27 sit under the other capability's row |
| F16 | Role record not committed | Partial (minor) | `design.md:186-190` commits it; `design-system.md:96-99` still lists only `transcript.txt` and `transcript.ansi` |
| F17 | 80x24 had no evidence | Resolved | `inbox-navigation.feature:84-93`; `design.md:167-168,231`; `design-system.md:76-80` |
| F18 | arc42 chapter 1 note; "adds no lasting component" wording | Partial (minor) | `arc42.md:24`; `docs/arc42/01:33-41`; `arc42.md:18` claims the snapshot distinction is clarified |
| F19 | Long scenarios and lint advisories | Advisory | `givn lint --change restore-tui-styling` exits 2 with the expected `@wip` and shape advisories; the final review records split/keep |

### New findings (round 2)

**R2-1 — BLOCKING: the overlay-title roles contradict the design system, the
design, and the delta.**
Evidence:
- `docs/design/design-system.md:58` says "Modal overlays use a dim bordered
  block with an accent title"; `:67` says the settings overlay has an "accent
  title"; `:72` says the save prompt has an "accent title".
- `design.md:143-146` says the save-prompt title `Leave` is `accent` and the
  overlays use "accent titles"; the design's own step phrases say the settings
  title is "cyan-blue and bold" (`design.md:223`) and the save-prompt title is
  "cyan-blue and bold" (`design.md:229-230`).
- The delta asserts "the settings title is cyan-blue and bold"
  (`inbox-navigation.feature:32`), "the detail overlay title is cyan-blue and
  bold" (`activity-inbox.feature:45`), and "the save prompt title is cyan-blue
  and bold" (`inbox-navigation.feature:103`).
- The operator-approved original: the activity-detail title is `accent_bold`
  (`tui.rs:754`), the PR-detail title and `Description:` label are
  `accent_bold` (`tui.rs:831,886`), the settings title is `accent_bold`
  (`tui.rs:2045`), the help heading is `accent_bold` (`tui.rs:2081`), and the
  save-prompt title is `accent` (`tui.rs:2023`).

Consequence: the save-prompt step cannot pass against the design (accent vs
accent_bold), and the settings/activity steps cannot pass against the design
system (accent vs accent_bold). The implementer must violate one artifact.
Recommendation: record the original roles in the design system and the design —
activity detail, PR detail, settings, and help titles `accent_bold`; save-prompt
title `accent` — and change `inbox-navigation.feature:103` and
`design.md:229-230` to `the save prompt title is cyan-blue`; keep the
activity/settings steps bold. If the author instead chooses normal accent for
all modal titles, the design system must say so and the activity/settings steps
must drop `and bold`; that deviates from the approved original and needs a
recorded decision.

**R2-2 — BLOCKING: the PR-detail heading assertion has no target in the scenario.**
Evidence: `inbox-navigation.feature:41` sets the description to "Adds the live
inbox" (plain text); `:53` asserts "the pull request detail heading is green and
bold"; `design-system.md:64` reserves green bold for Markdown headings;
`design.md:226` lists the same phrase. The original renders Markdown headings
`green_bold` (`tui.rs:491`) and the `Description:` label `accent_bold`
(`tui.rs:886`). The plain description produces no green-bold span, so the step
cannot be satisfied in GREEN. Recommendation: either assert the `Description:`
label role (`accent_bold`) for this step, or give the scenario's description a
Markdown heading (for example `# Adds the live inbox`; the renderer strips the
marker and still shows "Adds the live inbox") and name the phrase accordingly.
Do not leave an unassertable step.

**R2-3 — BLOCKING: "the rules use the rule color" has no rule span in the main frame.**
Evidence: `activity-inbox.feature:23` places the step in the main-inbox
scenario; `design-system.md:42-56` defines the main composition (outer frame,
header, counts/focus line, watched table, activity table, controls) with dim
borders and no horizontal rule or table divider; the rule color is specified
for the detail separators (`design-system.md:61-64`; original `separator()` at
`tui.rs:729-731`). With no rule span the step is either vacuous or fails
against the dim frame border; either way it does not prove the rule color.
Recommendation: move the step to the activity-detail scenario (and/or the
PR-detail scenario), where the `─` separators exist, or specify a rule in the
main composition and assert it.

**R2-4 — BLOCKING: the save prompt's `n` and `Esc` branches have no scenario.**
Evidence: `proposal.md:62-66` and `design.md:43-48` specify `y` persists and
exits, `n` discards and exits, `Esc` cancels, and no prompt without a change;
`inbox-navigation.feature:95-109` exercises only `y`. The permanent outline
"The terminal UI quits" (`activity-inbox.feature:280-290`) proves `q` and `Esc`
quit with no state change, i.e. the no-change branch, but not `n` discard or
`Esc` cancel with the prompt open. Recommendation: add a scenario outline (or
two scenarios) for `n` (the saved sort is unchanged; the TUI exits) and `Esc`
(the prompt closes; the TUI stays open; the changed sort is not persisted), and
update the design's twelve-scenario count. The operator decision names all three
keys; the spec must make them executable.

**R2-5 — Minor: the counts/focus line roles are unasserted.**
`design-system.md:38-40` assigns the unread count `gold_bold`, ` unread of `
`dim`, the total `bright`, `Focus: ` `dim`, and the pane name `accent_bold`;
`design.md:134-135,171-172` promises the line; the scenarios assert only its
text (`activity-inbox.feature:12`; `inbox-navigation.feature:15`). Add two role
steps (for example `the unread count is gold and bold` and `the focus label is
dim`) or remove the line from the primitive list. This is the one prior Q2
recommendation that did not land.

**R2-6 — Minor: the palette table's use column is imprecise.**
`design-system.md:15` assigns table headers and key hints to `accent` although
`:16` assigns them to `accent_bold`; `:16` names a "focused controls title" that
`design.md:141` renders as `accent`; `:22` assigns the focused title to `green`
although `:23` assigns it to `green_bold`; `:23` still names the out-of-scope
fetching spinner (`design.md:190-192`). Tighten the use column so each role is
single-valued.

**R2-7 — Minor: the glossary's `Palette role` list omits the bold roles.**
`glossary.md:64` lists `accent`, `bright`, `body`, `gold`, `green`, `rule`,
`dim`; the design's `Role` list has eleven roles including `accent_bold`,
`bright_bold`, `gold_bold`, `green_bold` (`design.md:119-120`), and the design
system defines them (`design-system.md:13-25`). `glossary.md:65-66` defines
`Styled frame` and `Role record` but does not state that a TUI snapshot is
distinct from an observation snapshot, and T25 still says "muted metadata
values" (`usecase.md:72`) where the role is `dim`. Align the vocabulary.

**R2-8 — Minor: the design system's evidence bullet omits the role record.**
`design.md:186-190` commits `transcript.txt`, `transcript.ansi`, and the role
record; `design-system.md:96-99` lists only the first two while `:99` says the
review reads the role record. Add the role record to the committed set.

**R2-9 — Minor (carried): the remaining documentation gaps.**
- `D2` and `D3` are design labels, not proposal questions (`design.md:37-41`,
  `:43-48`; `proposal.md:99-119`).
- The D1 fallback has no trigger, switch cost, or owner (`design.md:34-35`).
- The 256-colour fallback has no mapping (`design.md:332-334`;
  `docs/arc42/11:43`).
- The traceability column is still labelled "E2E evidence" while listing
  regular scenarios, and T24/T27 sit under the other capability's row
  (`design.md:278-283`).
- Chapter 5 still describes `ScopePicker` as the terminal screen while
  `TerminalPresentation` composes the picker overlay; record the state/
  interaction versus rendering split (`docs/arc42/05:65,73`;
  `design.md:77-79`).
- `arc42.md:24` still says the change "adds no lasting component" while
  `TerminalPresentation` is recorded in chapter 5; state that the frame module
  is pre-existing and the contract is change-local. Chapter 1 has no
  visual-fidelity refinement note (`docs/arc42/01:33-41`).
- The design does not record the structural-model no-impact assessment
  (`docs/domain-model.md` needs no edit, but the reason should be stated).
- The active-change resolution does not say which change id wins when more than
  one directory exists under `givn/changes/` (`design.md:101-107,151-162`).
- The `☺` people column is promised (`proposal.md:49-50`;
  `design-system.md:45-46`) but no scenario asserts its presence or role.

### Delta and scenario checks

- Twelve scenario entries: 3 `activity-inbox` (`activity-inbox.feature:5,29,51`),
  7 `inbox-navigation` (`inbox-navigation.feature:5,24,37,58,70,84,95`),
  1 `stacked-inbox` (`stacked-inbox.feature:5`), 1 `watch-scope`
  (`watch-scope.feature:5`). Total 12; matches `design.md:301-305`.
- The six modified scenarios preserve every permanent step and only add visual
  or prompt assertions: "Developer opens the inbox TUI" (permanent
  `activity-inbox.feature` steps vs delta `:5-27`), "Developer opens an
  activity's details" (delta `:29-49`), "Developer navigates the inbox panes"
  (delta `inbox-navigation.feature:5-22`), "The pull request detail shows its
  metadata" (delta `:37-56`), "The developer opens the key reference" (delta
  `:58-68`), "A missing browser opener is reported and keeps the detail" (delta
  `:70-82`), and "Developer reviews a stacked inbox" (delta
  `stacked-inbox.feature:5-23`). No permanent step is lost or reordered. The
  added PR-detail heading step is an assertion with no target (R2-2), not a
  lost permanent step.
- New step phrases: 51 distinct visual/prompt phrases; all appear in the
  design's step-definition list (`design.md:210-238`), and every listed phrase
  is used by a delta scenario. The phrases name concrete roles except the
  contradictions in R2-1, R2-2, and R2-3.

### Save-prompt consistency check

- Proposal: release note `:7`, solution `:62-66`, out-of-scope `:88-91`.
- Design: context `:12-15`, D3 `:43-48`, layout `:143-144`, steps `:229-231`,
  traceability `:281`, risk `:337-338`.
- T22 `usecase.md:69`; design system `design-system.md:72-74`; R-038
  `docs/arc42/11:44`; chapter 6 `docs/arc42/06:350-353`; chapter 8
  `docs/arc42/08:140-142`.
- All agree on the trigger set, the `y`/`n`/`Esc` meanings, and the no-change
  quit. The two defects are the save-prompt title role (R2-1) and the untested
  `n`/`Esc` branches (R2-4). The design system's "the current sort/filter
  changes persist only on `y`" narrows D3's list (direction, ignored view,
  notification settings); add the full list or point to D3.

### Test-infrastructure fix check

The plan is followable. `design.md:151-162` names the exact change to
`scripts/verify.sh` (delta from the active change, not the archived
`python-rebuild` path) and to `tests/support/harness.py` (active change id from
`GIVN_CHANGE_ID` or the active change directory). `design.md:290-308` gives the
verification commands and states the change scope and E2E scope both see the
active delta. The current hardcodes (`scripts/verify.sh:56`,
`tests/support/harness.py:54`) are the code to change, not a plan defect. The
plan does not state the selection rule when more than one change directory
exists (R2-9); with one active change the resolution is unambiguous. The design
system's evidence list must add the role record (R2-8).

### Grilling branches (round 2)

**Scope.** The proposal's screens are now all in the delta: main frame, activity
detail, PR detail, help, settings, save prompt, picker, stack column, ignored
row, status line, 80x24. The save prompt is in scope by the operator's Option 2.
Residual scope gaps are the untested `n`/`Esc` branches (R2-4), the unasserted
counts/focus roles (R2-5), and the unasserted `☺` people column (R2-9). Nothing
extra was added beyond the proposal and D1.

**Tech choices.** D1's "both" (styled transcript and role record) remains the
right evidence mechanism; the alternatives are real and the choice is not
over-engineered (`proposal.md:99-119`, `design.md:29-35`,
`design-system.md:91-100`). The single role table in `frame.py` with the app
importing it is the right single-owner design (`design.md:83-93,119-122`).
`curses` remains the right terminal adapter; no dependency is added. The
256-colour fallback remains undefined (F11).

**Missing scenarios.** R2-2 (no target for the PR-detail heading), R2-3 (no
target for the rule step), R2-4 (`n`/`Esc`), R2-5 (counts/focus roles), and the
`☺` column are the gaps. The design's states list still includes `empty` and
`error` (`design.md:173-174`) while the permanent empty/error scenarios remain
text-only; the design system gives them no distinct palette. Either add the
palette assertion to those scenarios or drop the two states from the contract
(minor). The no-change quit branch is covered by the permanent outline
(`activity-inbox.feature:280-290`).

**Testability.** The strict-mode mechanism (`behave` undefined-step failure plus
`scripts/check_steps.py`), the not-implemented stub pattern
(`raise NotImplementedError`, `design.md:238`), and the single-scenario command
(`design.md:296-300`) are present and correct; the runner fix makes the command
select the delta. Every scenario can fail in RED. The GREEN contract is broken
by R2-1 (contradictory expected roles), R2-2 (no target), and R2-3 (no target);
the role record format itself is testable.

**E2E fidelity.** The real driver remains the `passuff-tui` subprocess with
`--snapshot`, `--keys`, `--picker`, and `--observe-for`; the visual steps
consume the public `--snapshot` output, not the frame module
(`design.md:268-274`). The delta keeps the `@e2e` tags on the modified E2E
scenarios (`activity-inbox.feature:5,29`; `inbox-navigation.feature:5`;
`stacked-inbox.feature:5`); no `@e2e` tag was removed. The added settings,
80x24, save-prompt, ignored, and picker scenarios are regular variants; the
permanent `@e2e` scenarios still cover their inventory actions. No action is
duplicated in a second `@e2e` scenario. The terminal classification holds; the
opposite Web UI rationale is still not stated in the design (minor).

**Interaction Coverage.** The inventory is 11 entries: `review-activity-inbox`
7 (`usecase.md` §Interactions: activity-inbox 5, inbox-navigation 2),
`review-pull-request-stacks` 1, `change-watch-scope` 3. The design's matrix has
11 rows (`design.md:247-259`), one per entry. No missing or extra row. The save
prompt is not a new inventory entry; it modifies the quit action, which the
inventory has never listed, and its scenario is a regular variant. The matrix's
driving mechanisms are the real CLI.

**Risk.** The single most likely failure is now the title-role contradiction
(R2-1): the implementer follows the design system (accent) and the save-prompt
or settings scenario fails, or follows the feature (bold) and the design system
is violated. The second is the unassertable PR-detail heading (R2-2) and the
vacuous rule step (R2-3). The third is the missing `n`/`Esc` coverage (R2-4).
The runner/evidence path fix is planned and small. The design's risk list
(`design.md:326-341`) names palette drift, size, the save prompt, and the
runner, but not the title-role and unassertable-step risks.

**Use-case and Persona context.** The owning use cases keep their identity,
goals, rules, and typed relationships; T25 is corrected and T22 is now
implemented. `review-pull-request-stacks` extends `review-activity-inbox`;
`change-watch-scope` remains separate. `multi-account-developer` is confirmed
2026-11-02 and is used as a review lens, not as a Gherkin actor; no invented
Persona. The scope rejection criteria pass; T25's wording now matches the design
system.

**Decision ledger.** The proposal's D1 resolves in `design.md:29-35`. The
operator decisions are recorded. `D2` and `D3` are design-decision labels with
no proposal question (F10); they are decided, not carried. No proposal `D<n>`
is unresolved. The D1 fallback still lacks the four carried fields (F14).

**Contracts and components.** One contract, `TerminalPresentation`, with the
three consumer use cases, consumers `TuiApp` and `--snapshot`, and the
implementing component `passuff.tui.frame` (`design.md:71-79`). `ScopeEditor`
owns the scope edit; `TuiApp` composes it; `TerminalPresentation` renders the
picker. The visual step definitions are no longer listed as consumers. The
palette has one owner. The design does not name the CLI entry point as the
composition root (minor). Chapter 5's `ScopePicker` row still claims the
terminal screen without the rendering delegation (R2-9). The internal helpers
need no contract.

**Decision timing.** D1, D2, and D3 are foundational and decided now; no
scenario depends on a carried answer. The D1 fallback is a contingency, not a
decision usage supplies; it still lacks a trigger, switch cost, and owner
(F14). The 256-colour fallback is an unresolved behaviour (F11). Contracts are
current requirements and are not carried.

**Visual design contract.** The section exists with `Interface: terminal`,
viewports, design system, tokens, primitives, states, screens, rendered
reference, capture mechanism, and motion (`design.md:164-192`). The approved
direction is recorded in the Context (`design.md:12-13`) rather than in the
`Rendered reference` line; the approval is in `design.md`, so the requirement
is met. R2-1 breaks the contract's role consistency; the states list exceeds the
evidence for `empty` and `error`; the counts/focus primitive is unasserted.

**Deferrals.** No mandatory check is deferred with an operator decision. The
live fetching/countdown/budget is removed from scope, not deferred. No `@e2e`
tag is removed. No deferral blocks sign-off.

**Persona disposition.** `multi-account-developer` is confirmed 2026-11-02
(`givn/personas/multi-account-developer.md:3-5`). Relevance: the developer scans
the inbox in a terminal and needs the palette, focus, selection, and overlays to
be readable. Evidence: the three committed reference frames, the role record,
and the final visual review; the design records the screens
(`design.md:285-288`). The final review must record the per-Persona visual
outcome. No invented Persona or biography actor.

**Interface classification.** `Interface: terminal` is correct. The opposite
classification (Web UI) does not hold: the surface is a curses/ANSI terminal
frame, no HTML/CSS/JS is rendered in a browser, no browser driver applies, and
the evidence is a transcript and a role record. The design should record this
rationale (minor).

**ADR qualification.** No new ADR. `TerminalPresentation` is a presentation
component inside the existing terminal adapter; the durable visual constraint is
the design system; the contract is recorded in `design.md` and chapter 5. The
cheaper home is `design.md` §Contracts and Components +
`docs/arc42/05:73` + `docs/design/design-system.md`. `arc42.md:24`'s "adds no
lasting component" wording should say the frame module is pre-existing and the
contract is change-local (R2-9). No `NEW_ADR`, `AMEND_ADR`, or `SUPERSEDE_ADR`.

**Architecture documentation.** The independent 12-row verdict was derived
before comparing with `arc42.md`; it matches on every row:

| # | Chapter | My verdict | `arc42.md` | Match? | Content check |
|---|---|---|---|---|---|
| 1 | Introduction and Goals | No | No | Yes | Goals/stakeholders unchanged; QS-034 is in chapter 10; the visual-fidelity refinement note is still missing (R2-9) |
| 2 | Architecture Constraints | No | No | Yes | Python, curses, Gherkin, and the palette are existing constraints; the 256-colour fallback is a risk, not a new constraint |
| 3 | Context and Scope | No | No | Yes | The same terminal surface; the snapshot projections are the terminal interface already recorded |
| 4 | Solution Strategy | Yes | Yes | Yes | The terminal UI row is updated to the styled frame and projections (`docs/arc42/04:77`) |
| 5 | Building Block View | Yes | Yes | Yes | `TuiApp` and `TerminalPresentation` are updated (`docs/arc42/05:72-73`); the `ScopePicker` wording is R2-9 |
| 6 | Runtime View | Yes | Yes | Yes | The save-prompt quit flow is in step 5 (`docs/arc42/06:350-353`); the styled snapshot is in step 2 (`:345`) |
| 7 | Deployment View | No | No | Yes | No dependency, service, port, or deployment change |
| 8 | Cross-cutting Concepts | Yes | Yes | Yes | The palette, composition, accessibility, evidence path, and save prompt are recorded (`docs/arc42/08:130-145`) |
| 9 | Architecture Decisions | No | No | Yes | No new ADR; the register is unchanged |
| 10 | Quality Requirements | Yes | Yes | Yes | QS-034 is added (`docs/arc42/10:66`) and matches the visual contract |
| 11 | Risks and Technical Debt | Yes | Yes | Yes | R-037 and R-038 are added (`docs/arc42/11:43-44`) |
| 12 | Glossary | Yes | Yes | Yes | The glossary gained the four terms (`glossary.md:63-66`); chapter 12 links to it (`docs/arc42/12:1-10`); the "clarified the distinction" claim is slightly overstated (R2-9) |

Chapter-file integrity: all twelve files exist with substantive content; zero
box-drawing characters (`┌┐└┘│─├┤┬┴┼╔╗╚╝║═╠╣╦╩╬╭╮╰╯`) were found; the
`|`-with-`---` matches are Markdown table separator rows, not pseudo-diagrams.
Mermaid diagrams remain in the chapters that use them.

**Ubiquitous language.** `Watn palette`, `Palette role`, `Styled frame`, and
`Role record` are in `glossary.md:63-66`; the design and specs use them. The
`Palette role` list omits the bold roles (R2-7); `muted` in T25 and the
proposal's D1 example is not a role; the `snapshot` senses are not explicitly
distinguished. No term is invented in the code without a glossary entry.

**Structural model.** No new aggregate, entity, or relationship: the Watched
Pull Request, Activity Record, Notification, Account, Watch Scope, and read
models are unchanged; `Role`/`Span`/`StyledLine` are presentation types inside
the terminal adapter. `docs/domain-model.md` needs no edit; the design should
state that assessment (R2-9).

### Sign-off (round 2)

The plan resolves F1-F9, F13, F17, and the save-prompt and visual-direction
operator questions; the delta counts and the interaction matrix are correct,
the modified scenarios preserve the permanent steps, the E2E tags are intact,
and the test-infrastructure fix is a followable plan. It cannot be signed off:

- R2-1, R2-2, R2-3, and R2-4 are blocking.
- The operator decided the save prompt (Option 2) and approved the original
  visual direction (2026-11-08); no operator question is open. The remaining
  work is author correction of the design system, the design, and the delta,
  not a new operator decision. If the author chooses to depart from the
  approved original on the overlay-title roles, that departure needs the
  operator's confirmation.
- The prior minors F10, F11, F12, F14, F15, F16, and F18 remain; they do not
  block tasks on their own but must not be lost.

Earlier round verdict: FAIL (superseded)

## Round 3 re-review (independent)

Third round after the round-2 FAIL (R2-1 to R2-4 blocking; R2-5 to R2-9
minor) and the operator's Option 2 decision (save-on-quit prompt in scope).
Re-read: `proposal.md`, the four delta `.feature` files, `design.md`,
`arc42.md`, the three owning use cases, the permanent specs,
`docs/design/design-system.md`, `docs/glossary.md`, `docs/domain-model.md`,
all twelve `docs/arc42/` chapters, the step bindings under `givn/steps/` and
`givn/e2e_steps/`, `tests/support/world.py`, `scripts/verify.sh`, the current
`passuff/tui/app.py` and `frame.py`, and the operator-approved reference
(`/home/buster/projects/gh-notify/crates/passuff-cli/src/theme.rs` and
`tui.rs`). No git history was used. Only this file was written.

The round-3 outcome: the plan shape, delta count, scenario set, step binding
map, interaction matrix, `y`/`n`/`Esc` branches, test-infrastructure plan, and
arc42 content are sound. Two role assignments in the durable design system
still contradict the executable assertions, and the save prompt's no-change
branch has no observable assertion. The two role assignments are transcription
errors against the approved original, corrected by one line each in
`docs/design/design-system.md`; the third needs one delta scenario. No operator
decision is involved.

### Prior finding re-check (round 2)

| ID | Status | Evidence |
|---|---|---|
| R2-1 overlay titles | Resolved for titles | `design-system.md:65-66` (activity detail, PR detail, settings, help titles accent bold; save prompt title accent); `design.md:146-148`; delta `activity-inbox.feature:47`, `inbox-navigation.feature:32,54,67,107`. The original agrees (`tui.rs:2045` settings `accent_bold`, `tui.rs:2154` keys/help title `accent_bold`, `tui.rs:2176` activity overlay title `accent_bold`, `tui.rs:2023` save title `accent`). No title contradiction remains. |
| R2-2 PR-detail heading | Resolved | `inbox-navigation.feature:42` sets `# Adds the live inbox`; `:51` still sees "Adds the live inbox" because `markdown_lines` strips the marker (`passuff/tui/frame.py:27`); `:55` asserts the green-bold heading and now has a target; `:54` adds the title assertion. |
| R2-3 rule step | Resolved | The main-inbox scenario has no rule step; `activity-inbox.feature:50` puts "the detail separator uses the rule color" in the activity detail, where T14 defines the rule (`givn/specs/review-activity-inbox/usecase.md:61`); T25 now says "rule-colored (240) separators in the overlays" (`usecase.md:72`); the composition has no main-frame rule (`design-system.md:40-62`). |
| R2-4 save prompt `n`/`Esc` | Resolved in the delta; one runner gap | `inbox-navigation.feature:115-125` (`n`, saved sort stays `updated`) and `:127-138` (`Esc`, prompt closes, `sort: name ↑`, saved sort `updated`). The no-change branch rests on the permanent quit outline, whose only assertion is exit 0 (R3-16). The `--keys` whitelist does not accept `y`/`n` yet (R3-5). |
| R2-5 counts roles | Resolved (assertions added) | `activity-inbox.feature:13-15`; `design-system.md:44-46`. The unread-count role itself is R3-1. |
| R2-6 palette table | Partially resolved | Use column tightened (`design-system.md:15-25`); fetching spinner removed. The unread count is still under `gold` (R3-1) and the overlay key rows still say "accent" (R3-2). |
| R2-7 glossary | Resolved | `glossary.md:64` lists all eleven roles; `:65-67` adds `Styled frame`, `Role record`, `Rendered snapshot`; `usecase.md:72` says "dim metadata values". |
| R2-8 evidence | Resolved | `design-system.md:103-105` commits `transcript.txt`, `transcript.ansi`, and `roles.json`. |
| R2-9 carried minors | Partially resolved | D2/D3 relabeled (`design.md:38,44`); D1 fallback removed (`design.md:29-36`); 256-colour mapping (`design-system.md:30-34`, `design.md:337-342`); exit statuses added to settings, picker, ignored, 80x24, and save scenarios; Evidence column renamed and extended (`design.md:286`); chapter-1 note present (`docs/arc42/01:42-44`); ADR wording fixed (`arc42.md:24`). Still open: T24/T27 placement (R3-4), multi-change tie-break (R3-7), `☺` column (R3-8), empty/error states (R3-9), structural-model assessment (R3-10), chapter-5 `ScopePicker` (R3-11), Web-UI rationale (R3-12). |

### New findings (round 3)

**R3-1 — BLOCKING: the unread-count role contradicts the palette table, the
composition, the delta, and the approved original.**

The design system is the durable role contract and the source for
`ROLE_STYLES`; its table assigns both the element and the attribute. Evidence:
- `docs/design/design-system.md:20` — `gold` | 214 | **normal** | "attention
  text, **the unread count**, status".
- `docs/design/design-system.md:21` — `gold_bold` | 214 | bold | "selected row
  text and the `▶` marker" (the unread count is not listed).
- `docs/design/design-system.md:44` — "unread count **gold bold**".
- `design.md:215` — the step `the unread count is gold and bold`.
- `specs/review-activity-inbox/activity-inbox.feature:13` — "the unread count
  is gold and bold".
- Approved original: `tui.rs:1785` renders the count with
  `theme::gold_bold()`.

Consequence: an implementer who builds `ROLE_STYLES` from the table renders the
unread count with `gold` (normal) and the scenario fails; an implementer who
follows the composition passes but the table is wrong. The durable contract has
one correct answer (`gold_bold`) and one wrong statement.
Fix: move "the unread count" from the `gold` row to the `gold_bold` row
(`design-system.md:21`); no other artifact changes.

**R3-2 — BLOCKING: the save-prompt key role contradicts the design, the delta,
the key-hint row, and the approved original.**

- `docs/design/design-system.md:79` — "**Save prompt** — accent (normal) title
  `Leave`, bright question, **accent keys**, dim labels".
- `docs/design/design-system.md:16` — `accent_bold` = "**key hints**, table
  headers".
- `design.md:145` — "the keys `accent_bold`"; `design.md:236` — "the save
  prompt keys are cyan-blue and bold".
- `specs/review-activity-inbox/inbox-navigation.feature:109` — "the save
  prompt keys are cyan-blue and bold".
- Approved original: the prompt keys use `theme::key()`
  (`tui.rs:2014-2019`), and `theme::key()` is `accent_bold` (`theme.rs:26-28`).

Consequence: read literally, line 79 renders the keys with normal `accent`, so
the asserted bold step cannot pass against the design system. This is the same
defect class as R2-1, on a different element.
Fix: `design-system.md:79` → "accent bold keys". The same wording error exists
for the settings hints (`:75` "hint keys accent"; original
`tui.rs:2064-2066` uses `theme::key()`) and the picker footer (`:78` "footer
keys accent"); align them with the `accent_bold` key-hint row in the same edit.

**R3-3 — Minor: the shared role phrases are listed twice in the design's step
section.** `design.md:213` says the six shared phrases (`the selected row is
gold and bold`, both table-header phrases, `the focused pane title is green and
bold`, both metadata phrases) "are bound once in `givn/steps/common.py`; the
capability files hold overlay- and state-specific phrases". `design.md:215-225`
then repeats the same six inside the "Activity inbox" bullet. A literal
reading binds them in both `common.py` and `activity_inbox_steps.py`, which
behave rejects as `AmbiguousStep`. Remove the six from the capability bullet so
the list matches the stated rule.

**R3-4 — Minor: T24 and T27 sit under the other capability's row.**
`design.md:285` (activity-inbox) lists T24 (pull-request detail), but its
evidence scenario `The pull request detail shows its metadata` is in
`inbox-navigation.feature:39`. `design.md:286` (inbox-navigation) lists T27
(ignored view), but its evidence scenario `An ignored row is dim and marked` is
in `activity-inbox.feature:55`. Swap the two rule IDs.

**R3-5 — Minor: the `--keys` whitelist is missing `y` and `n`.**
`passuff/tui/app.py:424` accepts a fixed key set that contains `q`, `esc`,
`tab`, `s`, and `comma` but not `y` or `n`; `--keys tab,s,q,y` exits 2 with
"invalid key" before `apply_key` runs. `design.md:91-94` says app.py adds "`y`,
`n`, and `Esc` handling for the save prompt" but not that the snapshot key set
must accept them. Name the whitelist change; otherwise the three save-prompt
scenarios fail for a runner reason rather than the styling behaviour.

**R3-6 — Minor: the ANSI and role evidence capture is not wired to a
mechanism.** `design.md:105-108` says `tests/support/harness.py` writes
`transcript.txt`, `transcript.ansi`, and the role record, and
`design-system.md:103-105` requires them per `@e2e` scenario. The current world
keeps only the last plain result (`tests/support/world.py:480-513`), and the
harness writes only `world.last.text` (`tests/support/harness.py:53-57`). Name
how the styled and role runs are produced (an extra
`--snapshot --ansi` / `--snapshot --roles` run in the harness or new `World`
fields) so the tasks do not improvise it.

**R3-7 — Minor (carried): no tie-break when several change directories
exist.** `design.md:102-108,153-164` resolves the delta from
`givn/changes/*/specs/*/*.feature`; with one active change the result is
unambiguous, but the rule for two directories is unstated.

**R3-8 — Minor (carried): the `☺` people column is promised but unasserted.**
`proposal.md:49-50` and `design-system.md:51-52` include the column; no scenario
asserts its presence or role.

**R3-9 — Minor (carried): `empty` and `error` remain in the Visual Design
Contract with no palette assertion.** `design.md:175-176` lists both states;
the permanent empty/error scenarios stay text-only and the design system gives
them no distinct role. Drop them from the states list or add the palette
assertion.

**R3-10 — Minor (carried): the structural-model assessment is not recorded.**
`docs/domain-model.md` needs no edit (the Terminal UI role and the picker
relationship already exist at `docs/domain-model.md:137,140,146`; `Role`/
`Span`/`StyledLine` are presentation types inside the adapter), but the design
does not state that assessment.

**R3-11 — Minor (carried): chapter 5 still describes `ScopePicker` as the
terminal screen.** `docs/arc42/05-building-block-view.md:65` says "Terminal
screen for the scope mode…"; `design.md:78-79` splits state/interaction
(`ScopeEditor`, `TuiApp`) from rendering (`TerminalPresentation`). Chapter 5
also lists both `ScopePicker` and `TerminalPresentation`
(`docs/arc42/05:65,73`) without stating the split.

**R3-12 — Minor (carried): the opposite interface classification is not
argued.** `design.md:168` declares `Interface: terminal`. The Web-UI
disqualification (no browser rendering, no browser driver, evidence is a
transcript and a role record) is implicit, not recorded.

**R3-13 — Minor: the design system narrows the persisted setting set.**
`design-system.md:80` says "the current sort/filter changes persist only on
`y`"; proposal `:62-66`, `design.md:46-48`, T22 (`usecase.md:69`), and R-038
(`docs/arc42/11:44`) name sort, direction, filters, ignored view, and
notification settings. Point the bullet at T22 or use the full list.

**R3-14 — Minor: the PR-detail overlay title is not uniformly accent bold in
the approved original.** `tui.rs:2216-2219` renders `" PR"` with
`bright_bold`, `" - "` dim, and the key with `accent_bold`;
`design-system.md:65` states the PR-detail title is accent bold without that
split. No scenario distinguishes the two, so the visual review could accept a
fidelity drift.

**R3-15 — Minor: arc42 row 1 says "No" although chapter 1 gained the
refinement note.** `arc42.md:7` says chapter 1 is unaffected; the chapter
contains the visual-contract refinement note (`docs/arc42/01:42-44`). The
content is right; record the chapter as refined or mark the row "Yes" with the
note.

**R3-16 — BLOCKING: the no-change quit branch has no observable assertion.**
`design.md:48-49` says "Without a change, `q` quits without a prompt", and the
round-2 review treated the permanent quit outline as that branch's evidence.
The outline (`givn/specs/review-activity-inbox/activity-inbox.feature:280-290`)
opens the TUI with `--snapshot` (`givn/steps/common.py:189-191`) and its only
Then is `the terminal UI is closed`, which checks `world.last.code == 0`
(`givn/steps/activity_inbox_steps.py:159-162`). In snapshot mode the process
prints one frame and exits 0 whether `q` quit or opened the prompt, so an
implementation that prompts on every `q` passes the whole suite. No other
scenario presses `q` in the main view (the four `q` presses at
`givn/specs/review-activity-inbox/inbox-navigation.feature:187-200` close
overlays). Fix: add a delta entry (a modified quit scenario or a new scenario)
that opens the TUI with `--keys "q"` with no settings change and asserts the
TUI does not show `Save sort and filter settings?`; update the design's
fourteen-entry count and the traceability row. The scenario count becomes
fifteen. Secondary observation: `--snapshot` cannot distinguish "quit" from
"cancel" for `y` or `n` either — the `y`/`n` scenarios prove the persisted value
and the closed prompt, not the exit. That limitation is pre-existing (the same
outline has it), and the material missing guard is the no-change prompt
absence; if the plan wants to prove the exit itself, the existing pty driver
(`tests/support/pty_driver.py`, used by the e2e steps) can observe process
termination, but that is a larger change than this review requires.

**R3-17 — Minor (carried): the composition root is not named.**
`design.md:91-92` has `TuiApp` import `ROLE_STYLES` from the implementing
component, and `design.md:76-80` lists `TuiApp` as a consumer. The
architecture-principles exception covers the composition root and implementation
tests; the design never names `passuff.tui.app:main` as that root, so the import
cannot be classified under the exception.

**Lint claim.** `givn lint --change restore-tui-styling` exits 2 with 14 `@wip`
findings plus `[LONG]`, `[SUBST]`, and one `[SHAPE]` advisory ("The developer
confirms saving settings on quit" and "The developer discards settings on quit
share a shape"). The operator summary named only the `@wip` and long/subset
advisories; the shape advisory is expected here (the two branches differ only
in the key and the persisted value) and does not block.

### Grilling branches (round 3)

**Scope.** The proposal's screens are all in the delta: main frame, activity
detail, PR detail, help, settings, save prompt (`y`/`n`/`Esc`), picker, stack
column, ignored row, status line, 80x24. Nothing beyond the proposal and D1 is
added. The remaining scope gaps are the `☺` column (R3-8), the states list
(R3-9), and the no-change quit assertion (R3-16).

**Tech choices.** D1 ("both" the styled transcript and the role record) remains
the right evidence mechanism; the alternatives are real and the cost is one
extra projection (`design.md:29-36`, `design-system.md:100-102`). One palette
owner (`frame.py`) with `app.py` importing `ROLE_STYLES` is sound
(`design.md:84-87,91-92`). `curses` stays the adapter; no new dependency. The
256-colour fallback is now defined (`design-system.md:30-34`).

**Missing scenarios.** R3-8 (`☺`), R3-9 (empty/error palette), and R3-16 (the
no-change quit never asserts that the prompt is absent). The `y`, `n`, and
`Esc` branches are asserted; the no-change branch is not. No other behaviour
from T22/T25 is unasserted.

**Testability.** Strict mode: `behave` undefined-step failure plus
`scripts/check_steps.py` (`design.md:303-305`); stub pattern
`raise NotImplementedError("<phrase>")` (`design.md:243`); single-scenario
command `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "..."` and the
`--e2e` variant (`design.md:301-302`). Every new assertion is a concrete role,
symbol, or persisted value and can fail RED with the stubs in place. The
remaining GREEN risks are R3-1, R3-2 (contradictory expected roles), R3-5 (the
`--keys` gate), and R3-6 (the ANSI/role evidence capture).

**E2E fidelity.** Real driver is the `passuff-tui` subprocess with
`--snapshot`, `--keys`, `--picker`, and `--observe-for`; the visual steps read
the public snapshot output, never `passuff.tui.frame`
(`design.md:273-279`). The `@e2e` tags stay on the four modified E2E scenarios
(`activity-inbox.feature:5,31`, `inbox-navigation.feature:4`,
`stacked-inbox.feature:4`), and the browser-opener, help, and PR-detail
permanent scenarios are regular, so the delta correctly adds no `@e2e` tag.
The added scenarios are variants of existing inventory actions, not a duplicate
action. The terminal classification holds (R3-12 is the missing rationale).

**Interaction Coverage.** Inventory: `usecase.md:103-111` (7 entries),
`review-pull-request-stacks/usecase.md:84` (1), `change-watch-scope/usecase.md:87-89`
(3) — 11. Matrix: `design.md:252-264` — 11 rows, one per entry, each with a
real driving mechanism. The save prompt modifies the quit action, which is not
an inventory entry; no extra row is required.

**Risk.** The most likely failure is R3-1/R3-2: the role table drives
`ROLE_STYLES`, so a literal implementation renders normal gold/accent and the
unread-count and save-prompt-key steps fail. Next are R3-5 (the `--keys` gate
rejects `y`/`n`), R3-6 (no ANSI/role artifact), and R3-16 (a
prompt-on-every-quit regression stays green). The design's risk list
(`design.md:331-349`) names palette drift, size, the save prompt, and the
runner, but not these four. R3-1/R3-2 are corrected by two design-system lines,
R3-5 by the app.py bullet, R3-6 by the harness bullet, and R3-16 by one delta
scenario.

**Use-case and Persona context.** The owning use cases keep identity, goal,
guarantees, rules, and typed relationships; T25 matches the design system; T22
is the rule the prompt restores. `review-pull-request-stacks` extends
`review-activity-inbox`; `change-watch-scope` stays separate.
`multi-account-developer` is confirmed 2026-11-02
(`givn/personas/multi-account-developer.md:3-5`) and is used as a review lens,
not a Gherkin actor. The scope-rejection criteria pass.

**Decision ledger.** The proposal has one question, D1; it is decided at
`design.md:29-36` (no fallback; a missing role record is a failure). D2 is
labelled a design decision (`design.md:38`) and D3 an operator decision
(`design.md:44`); neither pretends to be a proposal question. No proposal
`D<n>` is unresolved, and no decision is carried with missing default,
trigger, switch cost, or owner.

**Contracts and components.** One contract `TerminalPresentation`
(`design.md:76`) with its three consumer use cases, consumers `TuiApp` and
`--snapshot`, and one implementing component `passuff.tui.frame`. `ScopeEditor`
owns the scope edit; `TuiApp` composes it; `TerminalPresentation` renders the
picker. The visual step definitions are not consumers. The one open point is
that `TuiApp` imports `ROLE_STYLES` from the component (`design.md:91-92`), and
the design still does not name the CLI entry point as the composition root that
makes that import an accepted exception; carry it as R3-17.

**Decision timing.** All decisions are made now; no scenario depends on a
carried answer; contracts are current requirements.

**Visual design contract.** `design.md:166-194` has `Interface: terminal`,
viewports (80x24 minimum, 120x40 recommended, 100x30 snapshot), the design
system pointer, tokens, primitives, states, screens, the rendered reference,
the capture mechanism, and motion. The approved direction is recorded at
`design.md:12-13` and `proposal.md:39-41`. R3-1/R3-2 break the contract's role
consistency; the states list overreaches (R3-9).

**Deferrals.** None. No `@e2e` tag removed. No mandatory check deferred.

**Persona disposition.** `multi-account-developer`: relevance is the developer
scanning the inbox in a terminal (palette, focus, selection, overlays); evidence
is the three committed reference frames, the role record, and the final visual
review; the design lists the covered screens (`design.md:290-293`). The final
review must record the per-Persona visual outcome. No invented Persona.

**Interface classification.** `Interface: terminal` is correct; the opposite
(Web UI) fails because no HTML/CSS/JS is rendered and no browser driver exists.
The rationale is not recorded in the design (R3-12).

**ADR qualification.** No candidate qualifies. The change adds a presentation
component inside the existing terminal adapter; the visual contract is a
quality constraint durable in `docs/design/design-system.md`; the alternatives
are evidence mechanisms; reversal changes the presentation layer and its tests,
not a system boundary. Cheaper homes: `design.md` §Contracts and Components,
`docs/arc42/05:72-73`, and `docs/design/design-system.md`. ADR-0003 (core/
adapter) and ADR-0002 (local delivery) are unchanged. No `NEW_ADR`,
`AMEND_ADR`, or `SUPERSEDE_ADR`.

**Architecture documentation.** The 12-row verdict was derived before
re-opening `arc42.md`:

| # | Chapter | My verdict | `arc42.md` | Match? | Content check |
|---|---|---|---|---|---|
| 1 | Introduction and Goals | No (refined) | No | Yes | Goals/stakeholders unchanged; the refinement note is present (`docs/arc42/01:42-44`); the "No" label understates the edit (R3-15) |
| 2 | Architecture Constraints | No | No | Yes | Python, curses, Gherkin, palette are existing; the fallback is a risk |
| 3 | Context and Scope | No | No | Yes | Same terminal interface; the save prompt is a state of the recorded terminal inbox row (`docs/arc42/03:50`) |
| 4 | Solution Strategy | Yes | Yes | Yes | Styled frame with plain/ANSI/role projections (`docs/arc42/04:77`) |
| 5 | Building Block View | Yes | Yes | Yes | `TuiApp` and `TerminalPresentation` updated (`docs/arc42/05:72-73`); `ScopePicker` wording is R3-11 |
| 6 | Runtime View | Yes | Yes | Yes | Styled snapshot step 2 and the save-prompt flow (`docs/arc42/06:344-353`) |
| 7 | Deployment View | No | No | Yes | No dependency, service, port, or deployment change |
| 8 | Cross-cutting Concepts | Yes | Yes | Yes | Palette, composition, accessibility, evidence path, save prompt (`docs/arc42/08:125-142`) |
| 9 | Architecture Decisions | No | No | Yes | No ADR; register unchanged (`docs/arc42/09:7-16`) |
| 10 | Quality Requirements | Yes | Yes | Yes | QS-034 added (`docs/arc42/10:66`) |
| 11 | Risks and Technical Debt | Yes | Yes | Yes | R-037 with the fallback and R-038 with the prompt (`docs/arc42/11:43-44`) |
| 12 | Glossary | Yes | Yes | Yes | Four terms in `docs/glossary.md:63-67`; chapter 12 links (`docs/arc42/12`) |

Chapter-file integrity: all twelve files exist with substantive content; no
box-drawing characters (`┌┐└┘│─├┤┬┴┼╔╗╚╝║═╠╣╦╩╬╭╮╰╯`) were found; no
`+---`/`===` pseudo-diagrams; the chapters with diagrams use Mermaid.

**Ubiquitous language.** `Watn palette`, `Palette role` (all eleven roles),
`Styled frame`, `Role record`, and `Rendered snapshot` are in `glossary.md:63-67`;
the design and delta use them. T25 says "dim metadata values" and
"rule-colored (240) separators in the overlays". No invented term without a
glossary entry; no term reused inconsistently.

**Structural model.** No new aggregate, entity, or relationship; `Role`,
`Span`, and `StyledLine` are presentation types inside the terminal adapter;
`docs/domain-model.md` remains accurate (`:137,140,146`). The design should
state that assessment (R3-10).

### Delta checks (round 3)

- Fourteen entries: 3 activity-inbox (scenarios at `activity-inbox.feature:6,32,55`),
  9 inbox-navigation (scenarios at
  `inbox-navigation.feature:6,25,39,61,74,88,100,116,128`), 1 stacked-inbox
  (`stacked-inbox.feature:6`), 1 watch-scope (`watch-scope.feature:6`). Matches `design.md:306-310` and the 14 `@wip`
  findings.
- Modified scenarios preserve every permanent step and only add visual, prompt,
  or exit-status assertions: "Developer opens the inbox TUI"
  (`givn/specs/review-activity-inbox/activity-inbox.feature:315-323`),
  "Developer opens an activity's details" (`:326-340`), "Developer navigates
  the inbox panes" (`givn/specs/review-activity-inbox/inbox-navigation.feature:5-16`),
  "The pull request detail shows its metadata" (`:229-242`; only the
  description gains the Markdown marker and the heading step),
  "The developer opens the key reference" (`:219-226`), "A missing browser
  opener…" (`:509-520`), "Developer reviews a stacked inbox"
  (`givn/specs/review-pull-request-stacks/stacked-inbox.feature:5-19`).
- Step binding map: shared role phrases in `givn/steps/common.py` (the e2e
  scope reaches them through `givn/e2e_steps/common.py:3`), capability phrases
  in the four regular files, and the four e2e files import their capability
  module (`givn/e2e_steps/activity_inbox_e2e.py:19`,
  `inbox_navigation_e2e.py:19`, `stacked_inbox_e2e.py:3`,
  `watch_scope_e2e.py:3`). No phrase is bound twice today; the design's
  duplication is R3-3.
- Every new phrase in the delta appears in the design's step list
  (`design.md:213-241`), and every listed phrase except the parameter examples
  is used by a delta scenario.

### Save-prompt consistency check (round 3)

Proposal `:62-66,88-91`; design `:44-49,144-145,234-236,286,345-346`; T22
(`usecase.md:69`); design system `:79-81`; R-038 (`docs/arc42/11:44`); chapter
6 (`docs/arc42/06:350-353`); chapter 8 (`docs/arc42/08:140-142`). All agree on
the trigger set, `y` persists, `n` discards, `Esc` cancels, and the no-change
quit. The two residual wording issues are the key role (R3-2) and the
"sort/filter" shorthand (R3-13).

### Test-infrastructure fix check (round 3)

Followable. `design.md:102-108` names the two edits: `scripts/verify.sh`
resolves the delta from the active change instead of
`givn/changes/python-rebuild` (`scripts/verify.sh:51` today) and exports
`GIVN_CHANGE_ID`; `tests/support/harness.py` resolves the active change id and
writes the three evidence files instead of `python-rebuild`
(`tests/support/harness.py:54` today). The verification commands are correct
(`design.md:297-305`), and the E2E scope runs the delta's `@e2e` scenarios. The
open points are the multi-change tie-break (R3-7) and the ANSI/role capture
mechanism (R3-6).

### Sign-off (round 3)

The operator decisions stand and are recorded in the plan: the original
main-branch composition and indexed palette are the approved visual direction,
and the save-on-quit prompt is in scope (Option 2), both 2026-11-08. No operator
question is open. R2-2, R2-3, R2-4, R2-5, R2-7, and R2-8 are resolved;
R2-1 is resolved for the overlay titles; R2-6 and R2-9 are partially resolved.

The plan is not ready for tasks: the durable design system assigns two asserted
elements roles that contradict the design, the delta, and the approved original,
and the save prompt's no-change branch is not observably covered:

- R3-1 (unread count: `gold` normal vs `gold_bold`) — `design-system.md:20-21,44`;
  `activity-inbox.feature:13`; `tui.rs:1785`.
- R3-2 (save-prompt keys: "accent keys" vs `accent_bold`) —
  `design-system.md:79` vs `design-system.md:16`, `design.md:145,236`,
  `inbox-navigation.feature:109`, `tui.rs:2014-2019`, `theme.rs:26-28`.
- R3-16 (no-change quit): `design.md:48-49` vs
  `activity-inbox.feature:280-290` and the exit-0-only step
  `givn/steps/activity_inbox_steps.py:159-162`.

R3-1 and R3-2 are one-line corrections to `docs/design/design-system.md`;
R3-16 adds one delta scenario and updates the count. No design decision or
operator choice changes. The minors R3-3 to R3-15 and R3-17 do not block on
their own but must be carried; R3-3 and R3-5 should be fixed before tasks
because they can produce a double binding and a misleading RED failure.

Earlier round verdict: FAIL (superseded)

---

## Round 4 re-review (independent)

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-2

Fourth independent pass, after the round-3 FAIL (R3-1, R3-2, R3-16 blocking;
R3-3 to R3-15 and R3-17 carried). The operator decisions stand and were not
revisited: the save-on-quit prompt is in scope (Option 2) and the original
main-branch/GIF composition is the approved visual direction, both recorded
2026-11-08.

Re-read in this pass: `proposal.md`, the four delta `.feature` files,
`design.md`, `arc42.md`, the three owning use cases and their interaction
tables, the permanent specs, `docs/design/design-system.md`, `docs/glossary.md`,
`docs/domain-model.md`, all twelve `docs/arc42/` chapters and the ADR register,
`givn/steps/common.py`, `givn/steps/support.py`, `givn/e2e_steps/*`,
`tests/support/world.py`, `tests/support/harness.py`, `scripts/verify.sh`,
`passuff/tui/app.py`, `passuff/tui/frame.py`, and the approved original
(`/home/buster/projects/gh-notify/crates/passuff-cli/src/{theme,tui,scope}.rs`)
plus the three committed reference frames. No git history was used. Only this
file was written.

### Operator decisions (recorded, not invented)

**Save-prompt scope — Option 2, answered 2026-11-08.** `q` opens
`Save sort and filter settings?` only when the sort, direction, filters,
ignored view, or notification settings changed; `y` persists and quits, `n`
discards and quits, `Esc` cancels; with no change `q` quits without a prompt.
Recorded in `proposal.md:39-41,62-66,88-91`, `design.md:12-15,44-49`, T22
(`givn/specs/review-activity-inbox/usecase.md:69`), `docs/design/design-system.md:79-82`,
`docs/arc42/06-runtime-view.md:350-353`, `docs/arc42/08-crosscutting-concepts.md:139-142`,
and R-038 (`docs/arc42/11-risks-and-technical-debt.md:44`).

**Visual direction — original main-branch composition and indexed palette,
approved 2026-11-08.** Recorded in `proposal.md:39-41`, `design.md:12-13,17-25`,
and the three frames under `givn/changes/restore-tui-styling/evidence/reference/`.
No rendered option is outstanding.

### Claimed round-4 fixes checked against the artifacts

| Claim | Result | Evidence |
|---|---|---|
| R3-1 unread count assigned to `gold_bold` | Correct | `design-system.md:21` ("selected row text, the `▶` marker, and the unread count"), `:44` ("unread count gold bold"), `activity-inbox.feature:13`, original `tui.rs:1785` (`theme::gold_bold()`) |
| R3-2 settings hints, picker footer, save keys `accent_bold`; save title `accent` | Correct | `design-system.md:75,78,79`; `:16` key-hint row; original `tui.rs:2013-2023` (question bright, `y`/`n`/`Esc` `theme::key()`, title `theme::accent()`), `:2064-2066`, `scope.rs:381`; `theme.rs:26-28` (`key()` = `accent_bold`) |
| R3-16 no-change quit scenario, delta 16 | Correct | `inbox-navigation.feature:141-148`; `design.md:307-312`; `givn lint` reports 16 `@wip` findings |
| Shared phrases listed once in `givn/steps/common.py` | Correct | `design.md:216` vs the capability lists at `:218-242`; the six shared phrases appear nowhere in the bullets |
| T24 moved to `inbox-navigation`, T27 to `activity-inbox` | Correct | `design.md:286` (T27), `:287` (T24) |
| `--keys` whitelist adds `y`, `n` | Correct (plan) | `design.md:94`; code to change is `passuff/tui/app.py:424` |
| `roles.json` named; harness plan names it | Partial | `design-system.md:104-106` names it; `design.md:105-109` still says "the role record" and does not name how the ANSI/role runs are produced (R3-6) |
| Multi-change tie-break | Present, imprecise | `design.md:161-163`; see R4-8 |
| `☺` people column + empty-state assertion | Correct | `activity-inbox.feature:23,72`; `design.md:221,225-226` |
| Empty state documented in the design system | **Not supported** | `docs/design/design-system.md` contains no empty-state entry; only `design.md:225-226` names the step (R4-4) |
| PR detail asserts `Pull request acme/api#42` | Present | `inbox-navigation.feature:53`; see R3-14/R4-3 for the text mismatch with the approved reference |
| Chapter 5 `ScopePicker` rendered by `TerminalPresentation` over `ScopeEditor` | Correct | `docs/arc42/05-building-block-view.md:65` |
| Visual contract excludes Web UI/Playwright/viewports | Correct | `design.md:170-171` |
| Save-prompt persistence list complete | Correct | `design-system.md:80-82`; T22 `usecase.md:69`; R-038 `docs/arc42/11:44` |
| arc42 chapter 1 row Yes with refinement note | Correct | `arc42.md:7`; `docs/arc42/01-introduction-and-goals.md:42-44` |
| Composition root is named | **Not supported** | no "composition root" / `passuff.tui.app:main` text exists in `design.md` or the chapters; `design.md:91-92` still plans the `ROLE_STYLES` import (R3-17) |
| ADR qualification no longer says "adds no lasting component" | Correct | `arc42.md:24-32` |

### Prior finding re-check (round 3)

| ID | Status | Evidence |
|---|---|---|
| R3-1 | Resolved | `design-system.md:21,44`; `activity-inbox.feature:13`; `tui.rs:1785` |
| R3-2 | Resolved | `design-system.md:75,78,79`; `design.md:145,235-237`; `inbox-navigation.feature:32,108-110`; `tui.rs:2013-2023,2064-2066`; `scope.rs:381` |
| R3-3 | Resolved | `design.md:216` lists the six shared phrases once; `:218-242` does not repeat them |
| R3-4 | Resolved | `design.md:286` (T27 with the ignored-row evidence), `:287` (T24 with the PR-detail evidence) |
| R3-5 | Resolved in the plan | `design.md:94`; current gate `passuff/tui/app.py:424` lacks `y`/`n` |
| R3-6 | Partially resolved | `design-system.md:104-106` names `roles.json`; the capture mechanism (an extra `--snapshot --ansi` / `--snapshot --roles` run, or new `World` fields) is still unnamed; `tests/support/harness.py:53-57` still writes only the plain text |
| R3-7 | Resolved, wording imprecise | `design.md:161-163`; see R4-8 |
| R3-8 | Resolved | `activity-inbox.feature:23`; `design.md:221`; design system `:51-52` documents `☺` |
| R3-9 | Partially resolved | empty state asserted (`activity-inbox.feature:72`); error state is the gold status line (`inbox-navigation.feature:85`); the design system still documents neither (R4-4) |
| R3-10 | Still open | no structural-model statement in `design.md`; `docs/domain-model.md` remains accurate (`:137,140,146`) |
| R3-11 | Resolved | `docs/arc42/05-building-block-view.md:65,72-73` |
| R3-12 | Resolved | `design.md:170-171` |
| R3-13 | Resolved | `design-system.md:79-82` |
| R3-14 | Still open | `design-system.md:65,71-72`; original `tui.rs:2216-2219` splits ` PR`/` - `/key/bright-bold/dim/accent-bold; the delta now also pins a title text (R4-3) |
| R3-15 | Resolved | `arc42.md:7`; `docs/arc42/01:42-44` |
| R3-16 | Resolved | `inbox-navigation.feature:141-148`; `design.md:48-49,307-312` |
| R3-17 | Still open | `design.md:91-92`; no composition-root name in any plan artifact |

### New findings (round 4)

**R4-1 — Minor: the traceability Evidence column covers 14 of the 16 delta
scenarios.** `design.md:286` lists three activity-inbox scenarios but the delta
has four (`The empty inbox uses the primary text`, `activity-inbox.feature:66`),
and `design.md:287` lists nine inbox-navigation scenarios but the delta has ten
(`The developer quits without a save prompt when settings are unchanged`,
`inbox-navigation.feature:141`). The counts at `design.md:307-312` are correct
(16), so the two artifacts disagree. Fix: add both scenario titles to the
Evidence column.

**R4-2 — Minor: the `accent` Use column names an element that is not accent.**
`design-system.md:15` says `accent` is used for "labels, **the watch-scope
login**, the controls title". The composition says the header renders account
logins in accent and the watch-scope value in bright
(`design-system.md:42-43`), and the approved original does exactly that
(`tui.rs:1777-1779`: login `theme::accent()`, `watch scope: ` dim, scope label
`theme::bright()`). No scenario asserts the scope value, so no test fails, but
the role table is the source for `ROLE_STYLES` and the Use column contradicts
the composition. Fix: "labels, account logins, the controls title".

**R4-3 — Minor (with R3-14): the PR-detail title text is pinned to the rebuilt
string, not the approved reference.** `inbox-navigation.feature:53` asserts
`the TUI shows "Pull request acme/api#42"`; the design system states only the
title role (`:65`), never the text, and the approved original renders
` PR - acme/api#42` (`tui.rs:2216-2219`). The activity detail's title text is
recorded in the design system (`:68`), the PR detail's is not, so the text
lives only in the delta and diverges from the reference. Fix: record the
PR-detail title text in `design-system.md:71-72` and either accept the
divergence explicitly or assert the reference form. Note also that the plain
`the TUI shows` phrase cannot distinguish the overlay title from the main-frame
summary line that the current renderer prints (`passuff/tui/frame.py:151`);
the role step is the real guard.

**R4-4 — Minor (with R3-9): the empty state is not documented in the design
system.** `activity-inbox.feature:72` asserts `the empty state text is bright`,
`design.md:225-226` lists the step, and the palette row `bright` is "primary
text" (`design-system.md:17`), but the composition never names the empty
watched/activity strings (`no watched pull requests`, `no activity yet`). The
round-4 claim that it is documented is unsupported. Fix: add one line to the
composition, as R3-13 did for the save prompt.

**R4-5 — Minor: the PR-detail description body role has no assertion.**
`design-system.md:71-72` assigns the PR-detail body `body` (bright gray), T25
names "bright-gray description and activity body text"
(`givn/specs/review-activity-inbox/usecase.md:72`), and `proposal.md:57` names
the same. The delta asserts the activity body twice
(`activity-inbox.feature:27,52`) and the PR-detail heading, metadata, border,
and title, but never the description body
(`inbox-navigation.feature:54-58`). An implementation that leaves the
description in the default foreground passes all 16 scenarios. Fix: add a body
assertion to the PR-detail scenario (or rename the shared phrase to `the body
text is bright gray` and use it in both overlays).

**R4-6 — Minor: no module owns the `--snapshot --ansi` and `--snapshot
--roles` flags.** `design-system.md:101-103`, `design.md:278-279`, and
`docs/arc42/06:344-345` require these projections and the visual steps consume
them, but the `passuff/tui/app.py` bullet (`design.md:91-95`) names only the
curses pairs, the save prompt, and the `--keys` whitelist, and `frame.py` owns
only `render_ansi`/`render_roles`. The current parser
(`passuff/tui/app.py:357-371`) rejects both flags and the snapshot branch prints
`render(...)` only (`:432-435`). Fix: name the flag and dispatch change in the
app.py bullet.

**R4-7 — Minor: the save must be observable from the `--snapshot --keys`
run.** The save scenarios drive `apply_key` through the snapshot path and then
assert `the saved sort is ...`, but the only quit-path save today runs after
the interactive loop (`passuff/tui/app.py:439`), and the snapshot branch
returns at `:432-435` before it. The settings toggle already shows the correct
pattern (save inside the key handler, `:272-280`). Fix: state in the app.py
bullet that the save prompt's `y` persists in the key-handling path so the
snapshot run observes it; otherwise the three save scenarios fail for a runner
reason rather than a styling reason.

**R4-8 — Minor: the tie-break sentence would fail on the current repository.**
`design.md:161-163` fails the runner "if more than one active change exists".
`givn status` currently reports two active changes (`python-rebuild` [empty]
and `restore-tui-styling`), so a literal implementation breaks the default
`./scripts/verify.sh` that `design.md:298-303` tells the implementer to run.
Fix: name the candidate set as the change directories that contain
`specs/*/*.feature` (the same glob as `design.md:103-105`), so an empty active
change does not trigger the guard.

**R4-9 — Minor: the lint claim omits two `[SHAPE]` advisories.** The round-4
summary lists only the `@wip` and long/subset advisories. `givn lint --change
restore-tui-styling` exits 2 with 16 `@wip` findings, 2 `[SHAPE]` advisories
(key reference vs no-change quit; save-y vs save-n) and the long/subset
advisories. Both shape advisories are expected for deliberate branch pairs and
do not block; record them so the count is honest.

**R4-10 — Minor: arc42 row 3's reason understates the change.**
`arc42.md:9` says "The same terminal surfaces are restyled", but the save
prompt is an added screen inside the recorded terminal inbox interface. "No"
stays correct (no external system, actor, or scope boundary changes; the
terminal interface row at `docs/arc42/03:50` already covers it), but the reason
should say so explicitly.

**R4-11 — Minor: `Visual fidelity` is a quality attribute with no home in the
quality goals.** QS-034 (`docs/arc42/10-quality-requirements.md:66`) uses the
attribute "Visual fidelity"; the chapter-10 quality tree (`:3-27`) and the
chapter-1 quality-goals table (`docs/arc42/01:46-57`) do not contain it, while
the chapter-1 note says the contract "adds no new goal"
(`docs/arc42/01:42-44`). Fix: add the tree node/goal, or state in QS-034 that
it refines Inspectability.

**R4-12 — Minor: the focused-pane green differs from the approved original.**
The original renders the focused pane border and title with
`Style::new().fg(Color::Green)` (`tui.rs:1119-1132`, ratatui basic green), while
`design-system.md:22-23` maps `green`/`green_bold` to indexed 114 and the
scenario asserts the role. The role, not the index, is asserted, so no test
fails, but the ANSI transcript will not match the reference shade. Fix: record
the normalization to 114 in the design system, or keep basic green for the
focus border/title.

### Grilling branches (round 4)

**Scope.** Every proposal screen is in the delta: main frame, activity detail,
PR detail, help, settings, save prompt (`y`/`n`/`Esc`, no-change), picker, stack
column, ignored row, status line, 80x24, empty state. Nothing beyond the
proposal and D1. Remaining gaps are the description body role (R4-5), the
PR-detail title text (R4-3), and the empty-state documentation (R4-4).

**Tech choices.** D1 ("both") remains the right evidence mechanism: one styled
artifact for humans, one role record for assertions; the cost is one extra
projection. One palette owner (`frame.py`, `ROLE_STYLES`) with the adapter
importing it is sound (`design.md:84-95`). `curses` stays the adapter; no new
dependency (`design.md:267-272`). The 256-colour fallback is defined
(`design-system.md:30-34`).

**Missing scenarios.** R4-5 (PR-detail body) is the only named element left
unasserted; the no-change branch, `☺` column, empty state, error status, 80x24,
key reference, settings, picker, stack column, ignored row, and all four save
branches now have scenarios. No scenario depends on an unrecorded decision.

**Testability.** Strict mode: `behave` undefined-step failure plus
`scripts/check_steps.py` (`design.md:304-306`); the stub pattern is
`raise NotImplementedError("<phrase>")` (`design.md:244`); the single-scenario
command is `GIVN_RESULT_SCOPE=change ./scripts/verify.sh --name "..."` and the
`--e2e` variant (`design.md:301-303`). Every new assertion is a concrete role,
symbol, or persisted value. `World.press` re-runs the snapshot with the
accumulated `--keys` (`tests/support/world.py:506-513`), so the save-prompt
scenarios are executable once the whitelist and the in-handler save land
(R3-5, R4-7); the 80x24 step can set `COLUMNS`/`LINES` through
`World._env_tokens` (`:418-419`). The remaining GREEN risks are R4-6 and R4-7.

**E2E fidelity.** Real driver is the `passuff-tui` subprocess with
`--snapshot`, `--keys`, `--picker`, and `--observe-for`. The visual steps read
the public snapshot output, never `passuff.tui.frame`
(`design.md:275-280`). The `@e2e` tags stay on the four modified E2E scenarios
(`activity-inbox.feature:5,32`, `inbox-navigation.feature:5`,
`stacked-inbox.feature:5`); the added scenarios are regular variants, not new
inventory actions. The terminal classification is correct: no HTML/CSS/JS, no
browser driver, evidence is a transcript and a role record
(`design.md:170-171`).

**Interaction Coverage.** Inventory: `review-activity-inbox/usecase.md:105-111`
(7 entries), `review-pull-request-stacks/usecase.md:84` (1),
`change-watch-scope/usecase.md:87-89` (3) — 11. Matrix: `design.md:253-265` has
11 rows, one per entry, each with a real driving mechanism (CLI subprocess,
snapshot subprocess, fake opener, fake stack metadata, picker). 11/11. The
quit/save flow is not an inventory entry and adds none (it modifies T2's quit
action inside the existing screen), consistent with round 3.

**Risk.** The most likely failure mode is now an invisible role gap: the
PR-detail description body (R4-5) and the PR-detail title text (R4-3) can be
implemented wrongly and stay green. Next are the two unreachable interfaces
(R4-6, R4-7) and the tie-break that would stop the suite on the current
repository (R4-8). Each is one line in the plan or one assertion; none changes
the design direction.

**Use-case and Persona context.** The three owning use cases keep identity,
goal, guarantees, rules, typed relationships, and capability ownership.
`review-pull-request-stacks` extends `review-activity-inbox`;
`change-watch-scope` stays separate. T22 (settings persistence), T24 (PR
detail), T25 (palette and composition), and T27 (ignored view) are the rules the
delta restores. `multi-account-developer` is confirmed 2026-11-02
(`givn/personas/multi-account-developer.md:3-5`) and is used as a review lens,
not a Gherkin actor. The scope-rejection criteria pass.

**Decision ledger.** The proposal has one question, D1; it is decided at
`design.md:29-36` (no fallback; a missing role record is a failure). The
palette-durability decision (`design.md:38-42`) and the operator save-prompt
decision (`design.md:44-49`) are correctly labelled and carry trigger, switch
cost, and owner where they are carried. No proposal `D<n>` is unresolved.

**Contracts and components.** One contract `TerminalPresentation`
(`design.md:76`) with its three consumer use cases, consumers `TuiApp` and
`--snapshot`, and one implementing component `passuff.tui.frame`; split trigger
named. `ScopeEditor` owns the scope edit, `TuiApp` composes it,
`TerminalPresentation` renders the picker overlay. The visual steps are not
consumers. The one unresolved point is the planned `ROLE_STYLES` import from
`TuiApp` (`design.md:91-92`) with no named composition root (R3-17); it is a
labelling gap, one sentence to fix, and does not change the component split.

**Decision timing.** All decisions are made now; no scenario depends on a
carried answer; the contract is a current requirement.

**Visual design contract.** `design.md:168-197` has `Interface: terminal`, the
viewports (80x24 minimum, 120x40 recommended, 100x30 snapshot), the design
system pointer, tokens, primitives, states, screens, the rendered reference,
the capture mechanism, and motion. The approved direction is recorded. The
remaining defects are internal consistency: R4-2 (accent Use column), R4-3
(PR title text), R4-4 (empty state), R4-12 (green shade).

**Deferrals.** None. No `@e2e` tag was removed. No mandatory check is deferred
with an operator decision.

**Persona disposition.** `multi-account-developer`: relevance is the developer
scanning the inbox in a terminal (palette, focus, selection, overlays, save
prompt); evidence is the three committed reference frames, the role record,
and the final visual review; the covered screens are listed
(`design.md:291-294`). No invented Persona.

**Interface classification.** `Interface: terminal` is correct. The opposite
(Web UI) fails: nothing renders HTML/CSS/JS, no browser driver exists, and the
evidence is a transcript plus a role record (`design.md:170-171`).

**ADR qualification.** No candidate qualifies. The change adds a presentation
component inside the existing terminal adapter; the visual contract is a
quality constraint durable in `docs/design/design-system.md`; the alternatives
are evidence mechanisms; reversal changes the presentation layer and its tests,
not a system boundary or persisted data. Cheaper homes: `design.md`
§Contracts and Components, `docs/arc42/05:72-73`, and
`docs/design/design-system.md`. ADR-0002 and ADR-0003 are unchanged
(`docs/arc42/09:12-13`); no `NEW_ADR`, `AMEND_ADR`, or `SUPERSEDE_ADR`.
`arc42.md:22-39` records the qualification once.

**Architecture documentation.** The 12-row verdict was re-derived before
re-opening `arc42.md`:

| # | Chapter | My verdict | `arc42.md` | Match? | Content check |
|---|---|---|---|---|---|
| 1 | Introduction and Goals | Yes (refinement) | Yes | Yes | Note present (`docs/arc42/01:42-44`); quality tree/goal lacks visual fidelity (R4-11) |
| 2 | Architecture Constraints | No | No | Yes | Python, curses, Gherkin, palette pre-exist |
| 3 | Context and Scope | No | No | Yes | Interface row already covers the terminal inbox (`docs/arc42/03:50`); reason wording (R4-10) |
| 4 | Solution Strategy | Yes | Yes | Yes | Styled frame with plain/ANSI/role projections (`docs/arc42/04:77`) |
| 5 | Building Block View | Yes | Yes | Yes | `TuiApp` and `TerminalPresentation` (`docs/arc42/05:72-73`); `ScopePicker` split (`:65`) |
| 6 | Runtime View | Yes | Yes | Yes | Snapshot projections and the save-prompt flow (`docs/arc42/06:344-353`) |
| 7 | Deployment View | No | No | Yes | No dependency, service, port, or deployment change |
| 8 | Cross-cutting Concepts | Yes | Yes | Yes | Palette, composition, accessibility, evidence, save prompt (`docs/arc42/08:133-142`) |
| 9 | Architecture Decisions | No | No | Yes | Register unchanged (`docs/arc42/09:10-18`) |
| 10 | Quality Requirements | Yes | Yes | Yes | QS-034 (`:66`); tree gap (R4-11) |
| 11 | Risks and Technical Debt | Yes | Yes | Yes | R-037 and R-038 (`:43-44`) |
| 12 | Glossary | Yes | Yes | Yes | Four terms in `docs/glossary.md:63-67`; chapter links |

Chapter-file integrity: all twelve files exist with substantive content; no
box-drawing characters (`┌┐└┘│─├┤┬┴┼╔╗╚╝║═╠╣╦╩╬╭╮╰╯`) were found; the only
`|---`/`===` hits are Markdown table separators; no placeholder or scaffold
text; the diagrams use Mermaid.

**Ubiquitous language.** `Watn palette`, `Palette role` (all eleven roles),
`Styled frame`, `Role record`, and `Rendered snapshot` are in
`docs/glossary.md:63-67`; the design, specs, and chapters use them
consistently. T25 names the palette and composition, not the indexed values,
and points at the design system. No invented term without a glossary entry.

**Structural model.** No new aggregate, entity, or relationship; `Role`,
`Span`, and `StyledLine` are presentation types inside the terminal adapter;
`docs/domain-model.md` remains accurate (`:137,140,146`). The design still does
not record that assessment (R3-10).

### Delta checks (round 4)

- Sixteen entries: 4 activity-inbox (`activity-inbox.feature:6,33,56,67`),
  10 inbox-navigation (`inbox-navigation.feature:6,25,39,62,75,89,101,117,129,142`),
  1 stacked-inbox (`stacked-inbox.feature:6`), 1 watch-scope
  (`watch-scope.feature:6`). Matches `design.md:307-312` and the 16 `@wip`
  findings.
- Tags: 4 `@e2e` preserved on modified scenarios
  (`activity-inbox.feature:5,32`, `inbox-navigation.feature:5`,
  `stacked-inbox.feature:5`); 8 `@givn.added`, 8 `@givn.modified`; every
  scenario carries `@wip`; none removed.
- Modified scenarios preserve every permanent step; the only dropped step is
  the PR-detail description fixture, deliberately changed to
  `# Adds the live inbox` to give the heading step a target
  (`inbox-navigation.feature:42,51`); `markdown_lines` strips the marker
  (`passuff/tui/frame.py:27`).
- Step list: all 49 phrases in `design.md:216-242` are used by a delta
  scenario, and every delta step is either new-listed or a pre-existing generic
  step (`the TUI shows`, `the TUI does not show`, `the developer presses`,
  `the exit status is 0`) or a parameter of a listed phrase
  (`the saved sort is "updated"`).
- Shared bindings: the six shared phrases appear only in the `common.py` note
  (`design.md:216`); the four capability bullets contain none of them, so no
  `AmbiguousStep` is planned.
- Target checks: PR-detail heading has a Markdown target
  (`inbox-navigation.feature:42,51,56`); the rule separator has a target in
  T14 (`usecase.md:61`) and the design system (`:68-70`); the detail overlay
  exists in the current renderer (`passuff/tui/frame.py:213-300`), so the
  separator is the only element to add.

### Role-consistency matrix (round 4)

| Element | Design system | Design (layout or step list) | Delta | Approved original |
|---|---|---|---|---|
| Unread count | `:21,44` gold_bold | `design.md:218` (step; layout delegates `:149`) | `activity-inbox.feature:13` | `tui.rs:1785` gold_bold |
| Focus label / pane name | `:45-46` dim / accent_bold | `design.md:219-220` (steps) | `:14-15` | `tui.rs:1796-1797` |
| Outer frame + title | `:40-41` dim / bright_bold + dim | `design.md:134-135` | `:17-20` | `tui.rs:1744-1749` |
| Account login | `:42` accent; `:15` see R4-2 | `design.md:136` | `:21` | `tui.rs:1777` accent |
| Table headers | `:52,57-58` accent_bold | `design.md:139-142` | `:22,24` | `tui.rs` header style |
| Selected row | `:21` gold_bold + `▶` | `design.md:140,142` | `:26`, `inbox-navigation:21`, `stacked:22` | `tui.rs:1135-1137` |
| Focused border/title | `:22-23` green / green_bold | `design.md:227-228` (steps) | `inbox-navigation:17-18` | `tui.rs:1119-1133`; shade R4-12 |
| Metadata labels/values | `:68-72` accent / dim | `design.md:147-149` | `activity-inbox:49-50`, `inbox-navigation:57-58` | `tui.rs:831-835` |
| Rule separator | `:24,68-70` rule 240 | `design.md:149` | `activity-inbox:51` | `tui.rs:884` separator |
| PR detail title | `:65` accent_bold | `design.md:147` | `inbox-navigation:55`; text R4-3 | `tui.rs:2216-2219` split |
| PR detail heading | `:23` green_bold | `design.md:149` | `inbox-navigation:56` | `tui.rs:831` title style |
| Settings row | `:74` gold_bold / green / dim | `design.md:148` (delegates) | `inbox-navigation:33-35` | `tui.rs:896-916` |
| Settings/picker/help keys | `:73,75,78` accent_bold | `design.md:147-149` | `inbox-navigation:32,68-69`; picker `:17` | `tui.rs:2064-2066`, `scope.rs:381` |
| Save prompt | `:79` accent title, bright question, accent_bold keys, dim labels | `design.md:145-146` | `inbox-navigation:107-110` | `tui.rs:2013-2023` |
| Picker | `:76-78` bright_bold / dim / accent_bold / gold | `design.md:148-149` | `watch-scope:14-18` | `scope.rs:310,376-384` |
| Stack column/positions | `:51` accent_bold; `:17` bright | `design.md:137-138` (columns; roles delegated) | `stacked:20-21` | `tui.rs` stack cells |
| Ignored row | `:54-55` dim + `⊘` | `design.md:140` | `activity-inbox:62-63` | `tui.rs` ignored row |
| Status line | `:61-62` gold | `design.md:143-144` (delegates) | `inbox-navigation:85` | `tui.rs:1988` gold |
| Empty state | not documented (R4-4) | `design.md:225-226` | `activity-inbox:72` | default foreground |
| Body text | `:19,71-72` body 252 | `design.md:149` | activity only (R4-5) | `tui.rs:29-31` |

### Save-prompt consistency check (round 4)

Proposal `:62-66,88-91`; design `:44-49,145-146,235-237,287,345-349`; T22
(`usecase.md:69`); design system `:79-82`; R-038 (`docs/arc42/11:44`); chapter
6 (`docs/arc42/06:350-353`); chapter 8 (`docs/arc42/08:139-142`). All agree on
the trigger set (sort, direction, filters, ignored view, notification
settings), `y` persists, `n` discards, `Esc` cancels, and no-change `q` quits
without a prompt. The no-change branch is observable
(`inbox-navigation.feature:141-148`). The residual issues are implementation
reachability (R4-7) and the `--keys` gate (R3-5, fixed in the plan).

### Test-infrastructure fix check (round 4)

Followable. `design.md:103-105,156-166` names both edits: `scripts/verify.sh`
resolves the delta from the active change (today
`scripts/verify.sh:51` hardcodes `givn/changes/python-rebuild/specs`) and
exports `GIVN_CHANGE_ID`; `tests/support/harness.py` resolves the active change
id and writes `transcript.txt`, `transcript.ansi`, and `roles.json` under it
(today `tests/support/harness.py:54` hardcodes `python-rebuild`). The
verification commands are correct (`design.md:298-303`). The two open points
are the tie-break candidate set (R4-8) and the ANSI/role capture mechanism
(R3-6).

### Sign-off (round 4)

The operator decisions stand and are recorded in the plan. No operator
question is open. All three round-3 blockers are resolved: the design system,
the design, the delta, and the approved original agree on `gold_bold` for the
unread count and `accent_bold` for the settings hints, picker footer, and save
prompt keys (with the save title `accent` normal); the no-change quit branch is
asserted; the delta is 16 entries and the design, the lint output, and the
traceability rule IDs agree.

The plan is ready for tasks. The remaining findings are minors: one missing
role assertion (R4-5), one unowned pair of snapshot flags (R4-6), one
reachability note for the save (R4-7), doc-consistency fixes (R4-1, R4-2,
R4-3, R4-4, R4-10, R4-11, R4-12), the tie-break wording (R4-8), the lint claim
(R4-9), and the carried items R3-6, R3-10, R3-17, and R3-14 (via R4-3). Two round-4 claims are
not supported by the artifacts and must not be treated as done: the composition
root is not named (R3-17) and the empty state is not documented in the design
system (R4-4). None changes a role, a scenario outcome, or the architecture
verdict, and each is a single localized edit that the task list can carry.

Grilling branches: scope, tech choices, missing scenarios, testability, E2E
fidelity, interaction coverage (11/11), risk, use-case and Persona context,
decision ledger, contracts and components (one contract, one component), and
decision timing all pass; the visual design contract, interface classification,
Persona disposition, ADR qualification, and the 12-chapter assessment agree
with the plan. No `@e2e` tag was removed and no mandatory check is deferred.

DESIGN-REVIEW: PASS
