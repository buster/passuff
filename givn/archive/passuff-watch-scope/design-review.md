# Design Review: passuff-watch-scope

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent design reviewer (fresh context)

## Verdict

DESIGN-REVIEW: PASS

The revision fixes the false state-writer claim and removes the `TomlConfigStore`
residue. The reload Then is now watcher-produced, but the held runtime still
does not reload under the design's own rule. The empty-retained scenario is
present, but it neither kills the auto-select mutation the prior review named
nor names the config read/sync its steps need. The contract table, the
mode-switch e2e evidence, the missing proposal/use-case scenarios, and the
structural model remain blockers. The 100x30 frame is exact and no longer a
finding.

> Closure: the operator directed one design review with all findings fixed in the
> same context. B1-B6 and N1-N27 are addressed; the original findings are kept
> below as the review record.

## Verification of the four claimed fixes

| # | Claim | What the files show | Result |
|---|---|---|---|
| 1 | Reload Then is watcher-produced; a comment on the newly selected repository is read as unread by the second cycle; no status/config read | `specs/change-watch-scope/watch-scope.feature:59-72`: the second cycle's `Then the activity "New comment from @bob" is unread` (`:72`); `design.md:171-177` says the step reads watcher-produced state; no status/config Then. The Then is fixed. The held runtime is not: `design.md:115-116` says the first cycle uses the config passed to `run_watch_with`, while `:171-174` calls `run_watch_with` twice on the same runtime. B1. | Partial: Then fixed; held runtime still wrong |
| 2 | False "TUI never writes state" claim removed; R-024 records the pre-existing mark-read/watcher save race; D4 records the deferral | `design.md:20` now says "The scope edit writes no state; the pre-existing mark-read/watcher save race remains R-024"; `design.md:291-294` repeats the position; `docs/arc42/11-risks-and-technical-debt.md:30` R-024 records the race; `proposal.md:97-108` D4 is answered "defer". | Fixed; D4 is not in the design's `## Decisions` ledger (minor) |
| 3 | Empty-retained round trip scenario added: all→selected with no retained set yields selected empty and `watching nothing`; R7 evidence cites it | `feature:86-94`; `design.md:153` cites it for R7. The scenario exists. It starts with only a configured account (`:88`), so there is no known repository; an implementation that auto-selects every known repository on empty→selected still passes. Step support is also unnamed (B2). | Added; B3's stated mutation not killed; step support missing |
| 4 | Leftover `TomlConfigStore` in the Decisions ledger removed | `design.md:135-137` now names `passuff-config`, `ScopeEditor`, and ADR-0006. A repo-wide search finds `TomlConfigStore` only in the prior review text, not in the design or chapters. | Fixed |

## Blockers

### B1 — The reload scenario's held runtime does not reload under the design's own rule

`specs/change-watch-scope/watch-scope.feature:59-72` is the reload scenario. The
Then is now watcher state (`:72`; `design.md:176-177`), which is correct. The
`When` is not executable as designed:

- `design.md:113-116` — "`run_watch_with` reloads the path before every cycle
  after the first; ... The first cycle uses the config passed to
  `run_watch_with`."
- `design.md:171-174` — "`a watcher is running with a configuration path`
  writes the config, builds the runtime with that path, and runs one cycle;
  `the watcher runs another cycle` calls the same runtime, which reloads the
  path before the cycle."

The harness calls `run_watch_with` twice. Under the stated rule, the second
call's first cycle is the first cycle of that call and uses the config passed
to that call, not the reloaded path. The design never says the runtime tracks a
cycle counter across `run_watch_with` calls, and it never says what config the
second call passes.

Consequences:

- If the harness passes the pre-edit config, the second cycle keeps the old
  scope; `acme/web` is still out of scope and no activity is recorded. The
  scenario is RED even when the reload works.
- If the harness passes a freshly loaded config, the second cycle uses the new
  scope without the runtime's reload. The scenario is GREEN even when the
  reload is broken. That is the same config-derived false GREEN in a new
  shape.
- The current `run_watch_with` takes the config as an argument and builds
  `PassuffCore` once (`crates/passuff-cli/src/lib.rs:204-231`). Reloading a
  config variable without rebuilding the core leaves the core's watch filter
  stale. The design's "reloads the path" does not state the core rebuild.

The named unit test `runtime_reloads_scope_between_cycles` (`design.md:117-118`,
`:155`) can be made unambiguous; the Gherkin scenario as specified cannot.
Fix: state that the held runtime tracks cycles across calls (or use one
`run_watch_with(StopAfter(2))` call with a control that writes the config
between cycles), name the config the second call passes (the pre-edit config),
and state that the reloaded config rebuilds the core. Then the scenario's
`When` matches its `Then`.

### B2 — The empty-retained scenario does not close the auto-select gap, and the scope steps' config reads are unnamed

The scenario at `feature:86-94` is:

```
Given a configured account "personal" with login "octo-personal"
And the watch scope mode is "all"
When the developer switches the watch scope to "selected"
Then the watch scope mode is "selected"
And the watch scope is empty
When the developer opens the inbox TUI
Then the TUI shows "watching nothing"
```

It has no authored pull request and no known repository. The prior review's B3
named the exact mutation: `set_mode(Selected)` that selects every known
repository when the retained set is empty. With an empty known set, that
mutation produces the same empty selected set and passes this scenario. R7
(`givn/specs/change-watch-scope/usecase.md:50`, `:58`) says the retained
selected set is restored, not populated from known repositories. The scenario
must seed a known but unselected repository (for example an authored pull
request plus `the watcher has already observed`) and assert that repository is
not in the scope after the all→selected switch.

The step support is also unnamed:

- The regular TUI/status/inbox steps build their view from `World::config_text`
  or `World::build_config` (`crates/passuff-cli/tests/support/mod.rs:762-779`,
  `:723-734`, `:736-758`, `:528-565`), not from the config file the scope action
  writes (`:832-846`; the design's `ScopeEditor` writes the file at
  `design.md:83-89`).
- The design names `World::e2e_state_json` for the e2e state path
  (`design.md:239-241`), but no regular or e2e config read/sync helper.
- The new `the watch scope is empty` step (`feature:92`) and the TUI
  `watching nothing` step (`feature:94`) will read the pre-switch mode-all
  config unless the scope step updates `World::config_text`/`build_config` or
  the TUI step reads the file. The same gap affects the other scope assertions:
  `feature:11-22`, `:31-35`, `:43-49`, `:56-57`, `:79-84`, `:91-92`, `:111-118`,
  `:126`, `:137-141`, `:149-151`.
- The e2e config assertions (`feature:11-14`, `:20`, `:80`, `:84`) must read
  `e2e_config_dir` (`crates/passuff-cli/tests/support/e2e.rs:209-217`), not
  `World::config_text`; the design names no helper. The prior review's N21
  remains.

Fix: seed a known but unselected repository in the empty scenario and assert it
is not selected; name the config read/sync helper for regular and e2e scope
steps (for example `World::reload_config_text` / `World::e2e_config_text`) and
state that every scope action refreshes it before the next assertion.

### B3 — The contract table still fails the contract checks

- `design.md:29` — `WatchScope` returns `Result<ScopeSnapshot, ScopeError>`.
  `ScopeSnapshot` appears nowhere else in the design. The data model defines
  `WatchScopeState` (`design.md:58-61`) with mode/repositories, `WatchConfig`
  (`:53-56`), and `ScopeError` (`:88-89`). The contract's return type is
  undefined for its consumers: `ScopePicker` (`:17`, `:122-124`) and the
  interactive loop (`:129-130`) need the post-edit mode/set/label to re-render.
  The design's "the caller loads the watcher state ... and passes the known
  repositories" (`:85-87`) does not define the snapshot. A declared contract
  with an undefined return type does not pass the consumer-shaped contract
  check.
- `design.md:30` — `InboxService`'s consumer use case is
  `change-watch-scope (display)`. That is not a use-case id. The filtered read
  model is consumed by `review-activity-inbox` (inbox) and
  `inspect-watch-status` (status); `change-watch-scope` owns the scope editing,
  not the inbox/status read model. The table must name the real consumer use
  cases.
- `design.md:31` — `ConfigLoader`'s consumer use case is "all use cases"; that is
  not a consumer use case. The `Consumers` column lists `passuff-tui` and
  `CliRuntime` but omits `ScopeEditor`, which `design.md:83` says reads the
  config through `ConfigLoader`. A current consumer is missing from the
  contract row.

The `WatchScope` → `ScopeEditor` one-to-one, the `ScopePicker` internal helper
(`design.md:87`), the removed `TomlConfigStore`, and the `ScopeError` shape
pass. The `ScopeSnapshot` and the consumer-use-case labels are the remaining
failures. Define `ScopeSnapshot` (fields and relationship to
`WatchScopeState`/`InboxView.scope_label`) or return `WatchScopeState`;
correct the two consumer-use-case rows and add `ScopeEditor` as a
`ConfigLoader` consumer.

### B4 — The mode-switch `@e2e` scenario does not exercise the picker its matrix row promises

`design.md:253` says the `switch the watch scope mode` inventory action is
driven by `--snapshot --picker --scope-mode all/selected` and asserts "config
and stdout, exit 0." The feature (`feature:74-84`) never opens the picker:

```
When the developer switches the watch scope to "all"
Then the watch scope mode is "all"
When the developer opens the inbox TUI
Then the TUI shows "watch scope: all authored pull requests"
When the developer switches the watch scope to "selected"
Then the watch scope contains "acme/api"
```

The scenario asserts the config (`:80`, `:84`) and the inbox (`:82`), not the
picker. The picker's mode display (all/selected), the `m` key path
(`design.md:123`), and the all-mode picker state are untested; the scenario's
primary assertion is on the inbox, not the picker named by the matrix.

The visual contract's `Capture mechanism` (`design.md:230-232`) names only
`.../developer-toggles-a-repository-in-the-watch-scope/`; the mode scenario's
`.../developer-switches-the-watch-scope-mode/` transcript is not named. The
design instruction requires every `@e2e` scenario to commit a transcript under
its own scenario slug. The picker frame (`design.md:198-228`) is selected-mode
only; there is no all-mode frame/transcript.

The 100x30 frame itself is exact: `design.md:199-228` is 30 lines and every
row is 100 columns (verified). No blocker on dimensions. Fix the evidence and
the scenario/matrix mismatch: make the mode scenario open `--picker`, assert the
picker's mode line (and the config), commit the mode scenario's transcript, and
add the all-mode picker frame/state; or change the matrix to the inbox and
record the picker's mode display as a visual-review gap.

### B5 — The spec still omits proposal/use-case behaviours and error paths the design itself lists

- **R2 status/activity hiding.** `givn/specs/change-watch-scope/usecase.md:45`
  (R2), main flow `:29`, and example `:59` say a deselected repository's
  watched pull requests **and activity** are hidden from the **inbox and
  status**. `feature:5-23` and `:37-49` assert the transcript, state, and TUI
  (inbox); no scenario asserts status or hidden activity for a deselected
  repository. `design.md:148` cites those two scenarios for R2; neither asserts
  status or activity. The proposal (`proposal.md:33-35`) says "hides its watched
  pull requests and activity from the inbox and status"; `docs/arc42/10:52`
  QS-019 says the same.
- **Extension 4a.** `usecase.md:40` — a selected repository with no retained
  records has no catch-up and the next observation records a baseline. The
  design's `External boundary cases` (`design.md:269`) says "Reselect with no
  retained records: baseline, no catch-up notification." `feature:128-141` only
  covers the retained case.
- **Reload failure.** The proposal's D2 (`proposal.md:74-83`) asks what happens
  when the reload fails; the design answers (`design.md:264`: "Config file
  missing/unparseable on reload: previous config kept; watcher continues").
  No scenario covers it. `feature:143-152` is a write failure, not a reload
  failure.
- **Account failure during catch-up.** `design.md:271` — `last_scope` is not
  advanced and the marker survives. No scenario covers it.
- **R5 global scope.** `usecase.md:48` says the scope is global across
  configured accounts; every scenario uses one account. The design's R5
  evidence (`design.md:151`) is "Config shape", not a scenario. A two-account
  scenario is needed to prove the global scope at the read model/observation
  level.

These are current rules/error paths, not future scope: the design's own
`External boundary cases` table (`design.md:263-273`) names them. Add the
scenarios (or narrow the use case/design and record the disposition); tag them
`@wip` until the steps exist. The `@e2e` tags must stay.

### B6 — The structural model is stale and the change records no structural assessment

- `docs/domain-model.md:13-14` says the model covers "the implemented watch,
  status, activity-review, and stack-review capabilities through the
  `passuff-stacked-inbox` change." The new `change-watch-scope` use case and its
  picker/watch-scope configuration are not mentioned.
- `docs/domain-model.md:109` puts "configuration editing" explicitly outside
  the Activity Review context. The design's picker (`design.md:120-131`) is a
  screen in the Terminal UI (`TuiApp`), which the model's relationship
  `Inbox → Terminal UI` (`docs/domain-model.md:130`) describes as "renders the
  read model and marks records read." The TUI now also opens the scope picker
  and edits the configuration; the relationship and the glossary's `Terminal UI`
  definition (`docs/glossary.md:50`) are incomplete.
- The change's design "Architecture impact" and `arc42.md` do not record a
  domain-model assessment; `docs/domain-model.md` is unmodified. The
  design-review instruction's structural-model branch requires the model to
  reflect the design's boundaries and relationships and the change to assess
  its structural impact; a no-op document edit is not evidence of maintenance.

The structural shape (two contexts, two aggregates) does not need a new
context; the watch scope is configuration. But the model must record the
picker's TUI relationship and the `change-watch-scope` capability; the Activity
Review boundary's "configuration editing" sentence should distinguish the
context from the TUI adapter. Update the model's scope/status, the TUI
relationship, and the glossary's Terminal UI; record the assessment in
`design.md`/`arc42.md`. The glossary preamble (`docs/glossary.md:7-12`) still
lists four use cases and calls `change-watch-scope` "planned" while its use
case and rules exist (N13).

## Grilling log

| # | Branch | Question | Disposition |
|---|---|---|---|
| Q1 | Scope | Does the delta contain exactly the proposal's behaviours, nothing missing, and is "done" unambiguous? | Mode, select/deselect, hide, catch-up, retention, persistence, manual entry, invalid entry, and write failure are represented. Missing: R2 status/activity hiding, extension 4a, reload failure, account failure during catch-up, R5 two-account evidence (B5). The empty round trip is present but not the known-but-unselected case (B2). |
| Q2 | Tech choices | Is the stack right, with no simpler alternative? | `WatchMode` + the existing list is the narrowest schema and matches ADR-0006. Reload each cycle is reasonable. Scripted flags are the recorded D3 surface. `toml_edit` is sound, but `design.md:16` still gives the false reason "already in `Cargo.lock` via `toml`" (N1). `WatchScopeState` duplicates `WatchConfig` while `passuff-core` already depends on `passuff-config` (N12). The `TomlConfigStore` residue is gone (fixed). |
| Q3 | Missing scenarios | Which observable behaviours implied by the proposal lack a scenario? | B5: R2 status/activity, extension 4a, reload failure, account failure during catch-up, R5; also legacy R10 (unit tests only, `design.md:156`), case-insensitive duplicate entry, extension 1a empty known list, R8 duplicate (N10). The empty round trip is present but weak (B2). |
| Q4 | Testability | Can every scenario fail RED; are strict mode, the stub pattern, and a single-scenario command named; are Thens concrete? | Strict mode `.fail_on_skipped()` (`design.md:161`; `crates/passuff-cli/tests/features_runner.rs:58`), `unimplemented!("<phrase>")` (`design.md:167`), and `GIVN_SCENARIO` (`design.md:164`; runner `:37`) are named. The reload held runtime is not RED-able as specified (B1); the empty scenario's config step source is unnamed (B2); `the TUI reports "scope"` is substring-vacuous (N9). |
| Q5 | E2E fidelity | Real action, real interface, primary assertion on output, no duplicate coverage? | Two inventory actions, two `@e2e` scenarios, real `passuff-tui` children, primary assertions on stdout for the toggle. The mode-switch scenario asserts the inbox, not the picker its matrix row names (B4); the mode's e2e config read helper is unnamed (N21); the matrix row 1 promises exit 0 and the scenario never asserts it (N7). |
| Q6 | Interaction Coverage | Every inventory entry has a matrix row and one matching `@e2e`? | 2/2 structurally: `usecase.md:83-86` → `design.md:250-253` → `feature:6` and `:75`. The trigger's "opens the repository picker" action (`usecase.md:17`) is still absent from the inventory (N10). The mode row's `--picker` and stdout are not exercised (B4). |
| Q7 | Risk | Most likely implementation failure and mitigation? | The catch-up/reload state machine. The unit test mitigates reload; the Gherkin held runtime does not (B1). The second risk is the state lost-update window (`last_scope` vs. mark-read); R-024 records it and D4 defers it (fixed). The picker clipping is recorded (R-023). The reload-failure path has no scenario (B5). |
| Q8 | Use-case and Persona context | Do identity, contract, capability ownership, typed relationships, preconditions, and Persona disposition hold? | Goal (`usecase.md:13`), primary actor (`:9`, `:17`), capability placement (`:79`), extensions (`:35-40`), and `Includes`/`Extends` (`:90-94`) pass. R11 is recorded (`:28`, `:54`) but not traced in the design (N24). The success guarantee (`:71`) lists three clauses; the prior review treated them as facets; note N17. The "persisted atomically" vocabulary in the main flow (`:28`) is mechanism-adjacent; note N17. |
| Q9 | Decision ledger | Do D1-D3 resolve; does D4 resolve; do seed-sourced questions keep their reference? | D1-D3 resolve at `design.md:135-141`. D4 is answered in the proposal (`proposal.md:97-108`) and repeated in the design's Risks (`:291-294`), not in the design's `## Decisions` ledger; minor. No carried question remains. No proposal question carries a seed reference; the seed's questions are unnumbered and Q5 is still `Status: open` in `questions.md:142-144` although the seed resolves it (N16). |
| Q10 | Contracts and components | One contract, one component, consumer-shaped, replaceable, no implementation imports, ports for boundaries? | `WatchScope` → `ScopeEditor` is one-to-one and consumer-shaped; `ScopeSnapshot` is undefined (B3). `InboxService` → `Inbox` passes; the consumer-use-case label "change-watch-scope (display)" is not a use-case id (B3). `ConfigLoader` → `passuff-config` passes; "all use cases" is not a consumer use case and the row omits `ScopeEditor` (B3). `ScopePicker` is correctly an internal helper with no contract. `TomlConfigStore` is gone (fixed). |
| Q11 | Decision timing | Any foundational question carried, speculative abstraction, or contract treated as open? | No carried question remains. No speculative abstraction beyond `WatchScopeState` duplicating `WatchConfig` (N12). Contracts are current requirements; `ScopeSnapshot` is an incomplete current contract (B3), not a carried decision. |
| Q12 | Visual design contract | Complete, approved, and backed by a rendered reference? | Interface `terminal`, viewport 100x30, screens, target frame, motion, and accessibility are present (`design.md:186-235`). The frame at `:199-228` is exactly 100 columns on every row and 30 lines. Approval rests on the archived `passuff-tui` 100x30 approval (2026-11-02) and the 2026-11-03 instruction; the prior review accepted the precedent. The mode-switch picker state and its transcript are missing (B4). |
| Q13 | Deferrals | Was a mandatory check deferred? | The state lost-update fix is deferred by D4 (`proposal.md:97-108`) with the deferral recorded in `design.md:291-294` and R-024 (`docs/arc42/11:30`). No `@e2e` tag was removed; the e2e runner is configured (`givn/commands.yaml:37`). No other mandatory check is deferred. |
| Q14 | Persona disposition | Is a confirmed Persona recorded and relevant? | Yes: `multi-account-developer` is confirmed (`givn/personas/multi-account-developer.md:3`; running record 2026-11-03), relevant, and recorded at `usecase.md:75` and `design.md:157`. No invented Persona was found. The Gherkin actor stays `developer`. |
| Q15 | Interface classification | Test the opposite before accepting "terminal". | No issue. The surface is ratatui/crossterm with `TestBackend::new(100, 30)` (`crates/passuff-cli/src/tui.rs:356-373`) and a real child process; there is no HTML, CSS, JavaScript, or DOM, so a browser driver would be the wrong interface. |
| Q16 | ADR qualification | Is commitment, alternatives, consequence, existing-decision search, and a cheaper home recorded once? | Yes: `arc42.md:35-47` records all five and routes to `NEW_ADR`; ADR-0006's three alternatives match the seed's three options; chapter 09 registers it (`docs/arc42/09-architecture-decisions.md:16`). The ADR directory row is outside its table (`docs/arc42/adr/README.md:14-17`) (N14); the ADR's writer-preservation consequence has no chapter-11 entry (`ADR-0006:45-48`; `docs/arc42/11:28-30`) (N15); the ADR Confirmation cites the reload scenario for the legacy shapes (`ADR-0006:50-55`) (N16). |
| Q17 | Architecture documentation | Independent 12-row derivation, content, integrity, ADR cross-check. | My derivation matches `arc42.md` row for row: rows 1-6 and 8-11 Yes, rows 7 and 12 No (table below). Chapters 01-06, 08-11 match the design, including the `ScopeEditor` ownership in 05/06/08/11. All 12 chapter files exist, contain project-specific content, and contain no box-drawing or `+`/`|` pseudo-diagrams. The ADR cross-check leaves N14/N15/N16. |
| Q18 | Ubiquitous language | Are the terms in the glossary and used consistently? | `watch scope`, `known repository`, `catch-up`, and `selected set` are in `docs/glossary.md:49-52` and used consistently in the use case, feature, design, and chapters. The preamble still lists four use cases and calls `change-watch-scope` "planned" (`docs/glossary.md:7-12`); the `Terminal UI` row (`:50`) still describes the TUI as a read-model consumer only. No invented term; the glossary sweep is N13/B6. |
| Q19 | Structural model | Did the change alter structure, and is the model maintained? | No artifact records the structural assessment. `docs/domain-model.md:13-14` covers through `passuff-stacked-inbox`; `:109` puts configuration editing outside Activity Review; `:130` describes the Terminal UI as rendering and marking read only. The structure (two contexts, two aggregates) does not need a new context, but the boundary/TUI relationship needs an edit (B6). |
| Q20 | Focus: new scenarios | Are the new scenarios implementable with the current harness and able to fail RED? | The toggle is RED-able (new flags, picker, filter, prune removal; state assertion bound to the e2e helper). The picker, mode, last-deselection, invalid-entry, write-failure, and switching-to-all scenarios are RED-able through new steps or the new filter. The reselect scenario is now RED/GREEN-capable. The reload Then is watcher-produced but the held runtime is not (B1). The empty scenario has no known repo and no config step source (B2). |
| Q21 | Focus: reload path | Is "a watcher with a configuration path" supported by the design and test support? | `CliRuntime` gains the path and the reload rule (`design.md:113-116`); the harness is named (`:171-177`); the unit test is named (`:117-118`, `:155`). The second `run_watch_with` call's first cycle uses the passed config under the stated rule; the design does not state a runtime-scoped counter or what config the second call passes (B1). The current `run_watch_with` builds one core (`crates/passuff-cli/src/lib.rs:204-231`); the core rebuild is not stated. |
| Q22 | Focus: e2e actions | Do the e2e actions drive the scripted real binary and assert output, config, and state? | Yes for stdout and state; the e2e state helper is named (`design.md:239-241`). The e2e config read helper is unnamed (N21). The mode-switch e2e asserts the inbox, not the picker (B4). The toggle's `the state retains` after deselect (no check) is weak, but `A deselected repository stops observation` runs the check and carries the RED. |
| Q23 | Focus: `scope_label` | Are `scope_label` and filtering consistent across TUI, status, and tests? | The intent is consistent (`design.md:80-82`, `:100-102`), but the design does not say that `InboxView.scope` is removed or remapped. The renderers and existing tests read `scope` (`crates/passuff-cli/src/tui.rs:205-211`, `crates/passuff-cli/src/transcript.rs:84-91`), so the `watch scope: all authored pull requests` assertion depends on an unstated renderer change (N4/N25). |
| Q24 | Focus: ADR/chapter 09 | Are ADR-0006 and chapter 09 consistent? | The register row, date, status, and title match (`docs/arc42/09:16`; `ADR-0006:1-9`). The ADR directory index row is malformed (N14). The ADR Confirmation cites `The watcher reloads the scope without a restart` for the legacy shapes (`ADR-0006:50-55`), but that scenario asserts the reload, not legacy resolution (N16). |
| Q25 | Focus: catch-up derivation | Can `last_scope` distinguish "deselected" from "never selected"? | Yes: the reselect scenario runs a cycle while the repository is deselected, `last_scope` records the excluded scope, and the comparison is mode-aware via `allows_repository` for previous and current (`design.md:104-108`; `feature:128-141`). The `None` first-cycle case and previous-`All` case are stated (`:108-109`). The previous-`Selected` to current-`All` transition is not explicitly stated; the `Switching to all observes every repository` scenario does not add a gap comment, so the catch-up path for the reverse transition is unproven (minor). |
| Q26 | Focus: select-from-all / all-mode picker | Does the design record what `select` does in all mode, and what the picker shows? | The use case records the transition (`usecase.md:28`, `:54` R11); the design does not trace R11 (`design.md:145-157`) (N24). The picker's all-mode checkbox semantics and toggle behaviour are still undefined: `design.md:122-124` gives the keys unconditionally and never says whether known repositories show `[x]`/`[ ]` in all mode or what `space` does there; `usecase.md:50` says only that the list is "informational" (N22). |
| Q27 | Focus: 100x30 frame | Is the picker frame dimensionally exact and consistent with the contract? | Yes dimensionally: every row of `design.md:199-228` is 100 columns and the frame is 30 lines, matching `TestBackend::new(100, 30)` (`crates/passuff-cli/src/tui.rs:356`). Content covers mode, known repositories, add buffer, and keys. The error line the design says the picker renders (`design.md:122-123`) is not in the frame; the all-mode state has no frame (N22/B4); character input/backspace are not in the key map (N5). |
| Q28 | Focus: contract shape | Is `WatchScope` shaped enough for its consumers? | The method list, return type, and error shape are named (`design.md:29`, `:88-89`), but `ScopeSnapshot` is not defined anywhere; the consumer cannot know whether it carries the mode, the set, the known list, or a display label (B3). The `ConfigLoader` consumer list omits `ScopeEditor` and the `InboxService` consumer use case is not a use-case id (B3). |

## Independent arc42 re-derivation

| # | Chapter | My derivation | `arc42.md` claim | Result |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | Yes — new requirement; the use-case list must include `change-watch-scope` | Yes | Match (`docs/arc42/01:27`, `:31-36`) |
| 2 | 02 architecture-constraints | Yes — the terminal UI may write the config atomically; the watcher reloads | Yes | Match (`docs/arc42/02:15`) |
| 3 | 03 context-and-scope | Yes — configuration becomes read/write; the picker is a new terminal surface; per-account scope stays outside | Yes | Match (`docs/arc42/03:43`, `:53-55`) |
| 4 | 04 solution-strategy | Yes — explicit mode, atomic edit, reload, hide, catch-up | Yes | Match (`docs/arc42/04:23-26`) |
| 5 | 05 building-block-view | Yes — `WatchMode`, `ScopeEditor`, `ScopePicker`, `TuiApp` picker, `last_scope` | Yes | Match (`docs/arc42/05:63-65`); the level-1 diagram omits the picker/editor but the building-block table carries them (minor) |
| 6 | 06 runtime-view | Yes — picker edit, atomic write, reload, filtering, catch-up | Yes | Match (`docs/arc42/06:208-224`) |
| 7 | 07 deployment-view | No — no process, service, or packaging change | No | Match |
| 8 | 08 crosscutting-concepts | Yes — scope editing, atomicity, hide, catch-up, reload failure | Yes | Match (`docs/arc42/08:93-104`) |
| 9 | 09 architecture-decisions | Yes — ADR-0006 is a new decision | Yes | Match: ADR exists and is registered (`docs/arc42/09:16`); qualification recorded (`arc42.md:35-47`) |
| 10 | 10 quality-requirements | Yes — QS-022 for picker, hide, catch-up | Yes | Match (`docs/arc42/10:54`) |
| 11 | 11 risks-and-technical-debt | Yes — config write, picker clipping, state race, ADR consequences | Yes | Mostly: R-022/23/24 exist (`docs/arc42/11:28-30`); the ADR's writer-preservation consequence has no entry (N15) |
| 12 | 12 glossary | No — no new term; the glossary needs a sweep, not a new definition | No | Match on new terms; the sweep is N13 |

Chapter file integrity: all 12 files under `docs/arc42/` exist, contain
project-specific content, and contain no Unicode box-drawing characters and no
`+`/`|` pseudo-diagrams. The only `+`/`---`/`===` matches are Markdown table
separators. The ADR directory index row is outside its table (N14).

## Contracts and components check

| Contract | Consumer use case | Consumers | Implementing component | Checks |
|---|---|---|---|---|
| `WatchScope` (`select`, `deselect`, `set_mode`, `add`; each `Result<ScopeSnapshot, ScopeError>`) | change-watch-scope | `ScopePicker`, scripted `passuff-tui` flags, interactive loop | `ScopeEditor` (`passuff-cli::scope`) | Pass on one contract/one component and consumer-shaped methods; `ScopeSnapshot` undefined (B3) |
| `InboxService` | change-watch-scope (display) | `TuiApp`, `InboxCli` | `Inbox` (`passuff-core::inbox`) | Pass on the split; the consumer-use-case label is not a use-case id; the touched `review-activity-inbox`/`inspect-watch-status` are not named (B3) |
| `ConfigLoader` | "all use cases" | `passuff-tui`, `CliRuntime` | `passuff-config` | Pass on one component; the consumer-use-case label is invalid and `ScopeEditor` (which reads through it, `design.md:83`) is missing from the consumer list (B3) |
| `ScopePicker` | — | Internal helper of the terminal adapter | — | Correct as an internal helper (`design.md:87`) |
| `TomlConfigStore` | — | — | — | Removed; no occurrence outside the prior review (fixed) |

No consumer is planned to import another component's implementation except the
composition root (`tui_main` constructs `ScopeEditor`) and the implementation
tests. `ScopePicker` holds the contract, not the implementation, per
`design.md:17`. The design does not state the `&dyn WatchScope` injection for
the interactive loop; `design.md:129-130` calls "the editor" directly. That is
a wiring detail for B3's contract-shape fix, not a second component.

## Decision ledger

| ID | Proposal disposition | Design resolution | Verdict |
|---|---|---|---|
| D1 | Carried to design (`proposal.md:63-72`) | `design.md:135-137`: `WatchMode` + list; legacy absent mode resolves by list presence; explicit selected empty means none; `passuff-config`, `ScopeEditor`, ADR-0006 | Resolved; no seed reference (N16) |
| D2 | Carried to design (`proposal.md:74-83`) | `design.md:138-139`: reload each cycle; previous config on error; unit test `runtime_reloads_scope_between_cycles` | Resolved; the Gherkin held runtime is not (B1); the reload failure has no scenario (B5) |
| D3 | Carried to design (`proposal.md:85-95`) | `design.md:140-141`: scripted flags + `--picker`; same `ScopeEditor` | Resolved; the mode-switch e2e does not use the picker (B4); the e2e config helper is unnamed (N21) |
| D4 | Answered — defer (`proposal.md:97-108`) | `design.md:291-294` and `docs/arc42/11:30` R-024; the scope edit adds no state writer | Recorded in the proposal and Risks, not in the design's `## Decisions` ledger (minor); the line 20 claim is accurate |

No decision remains carried, so no default/trigger/switch-cost/owner is owed.
The proposal's D1-D4 carry no seed reference; the seed's open questions are
unnumbered and Q5 is still `Status: open` in `questions.md:142-144` although the
seed resolves it (N16).

## Use-case scope rejection criteria

| Criterion | Evidence | Result |
|---|---|---|
| Goal conjunction | `usecase.md:13` — one verb phrase; the success guarantee's clauses (`:71`) are facets of one scope outcome | Pass with note (N17) |
| Missing primary actor | `usecase.md:9`; the trigger names the developer (`:17`) | Pass |
| Placement failure | `watch-scope` advances "change which repositories passuff observes" (`:13`, `:79`) | Pass |
| Mutation under an observing goal | The goal is a scope change; the scenarios assert config, read-model, and state outcomes | Pass |
| Extension without a step reference | 1a→1, 2a-2d→2, 4a→4 (`:35-40`) | Pass |
| Inconsistent preconditions | `usecase.md:22` says the terminal UI can read; the flow writes; the minimal guarantee covers the failure | Pass with note (N17) |
| Mechanism vocabulary | Main flow `:28` says "persists ... atomically"; the minimal guarantee `:67` says "no partial scope is persisted"; the success guarantee `:71` says "is persisted" | Minor (N17); the guarantees describe an all-or-nothing observable state, but the wording is mechanism-adjacent |
| Component- or channel-shaped name | `change-watch-scope`, `watch-scope` are goal-shaped | Pass |
| Oversized leaf | Five steps, one actor goal (`:25-31`) | Pass |
| Relationship misuse | `Includes: none`, `Extends: none` (`:90-94`) | Pass |
| Rule consistency | R7 and the Example agree (`:50`, `:58`); R11 exists (`:54`); the new empty scenario is present but weak (B2); R2 status/activity and extension 4a are not covered (B5) | Partial (B2/B5) |

Persona: `multi-account-developer` is confirmed, relevant, and recorded at
`usecase.md:75` and `design.md:157`. No invented Persona was found. The
Persona remains a review lens; the Gherkin actor stays `developer`.

## Scenario testability

| Scenario | RED source | Concrete assertions | Gaps |
|---|---|---|---|
| `Developer toggles a repository in the watch scope` (`feature:5-23`) | New flags, picker, filter, prune removal | mode, contains/not-contains, picker title, `[x]`, TUI shown/hidden, state retained | State Then uses the e2e helper; e2e config read unnamed (N21); no exit assertion (N7); multi-action/long (N19/N27) |
| `The picker shows the known repositories` (`feature:25-35`) | New picker step | title, `[x]`, mode | No empty-known list (N10); all-mode state undefined (N22); the regular TUI config source is unnamed (B2) |
| `A deselected repository stops observation` (`feature:37-49`) | New scope/state steps | transcript absence, state retained, TUI absence | No hidden activity/unread or hidden status assertion (B5) |
| `Adding a repository by owner/name` (`feature:51-57`) | New add step | contains, mode | Duplicates the e2e add (`feature:15-16`) (N19); the config read is unnamed (B2) |
| `The watcher reloads the scope without a restart` (`feature:59-72`) | New config-path step | activity unread | Held runtime does not reload under the rule (B1); the second call's config argument and the core rebuild are unspecified |
| `Developer switches the watch scope mode` (`feature:74-84`) | New flags | mode, TUI label, retained set | No picker; matrix says `--picker` (B4); no exit (N7); config read unnamed (N21) |
| `Switching to selected with no retained set observes nothing` (`feature:86-94`) | New mode | mode selected, empty, TUI `watching nothing` | No known repository, so the auto-select mutation passes (B2); the TUI config source is unnamed (B2) |
| `The last deselection observes nothing` (`feature:96-103`) | New mode | mode selected, status `watching nothing` | `Then status says` is a Then with no When (N8); the config read is unnamed (B2) |
| `Switching to all observes every repository` (`feature:105-118`) | New mode/check | both rows | Lint subset advisories; the selected-to-all catch-up rule is not stated (Q25); the config read is unnamed (B2) |
| `An invalid repository entry is rejected` (`feature:120-126`) | New add step | "owner/name" substring, not-contains | Existing message says `owner/repo` (`crates/passuff-config/src/lib.rs:344`); stream unspecified (N9); the config read is unnamed (B2) |
| `Reselecting catches up silently` (`feature:128-141`) | New select/catch-up steps | unread, transcript absence, inbox row | The eager baseline must be written before the scope restriction, which changes the current `pending_baseline` step (N26); the inbox config read is unnamed (B2) |
| `A scope write failure keeps the previous scope` (`feature:143-152`) | Failing-write fixture | "scope" substring, previous/absent set | `"scope"` matches the picker title (N9); no exit-code assertion; the config read is unnamed (B2) |

Strict mode, the stub pattern, and the single-scenario command are named
(`design.md:161-167`). The gaps above are fixture/semantic and step-support,
not runner gaps.

## Visual design contract check

Interface: terminal — correct; the opposite classification was tested (Q15).
Viewports: 100x30 — correct and matches `TestBackend::new(100, 30)`
(`crates/passuff-cli/src/tui.rs:356`). Screens: picker and inbox with current
scope (`design.md:190`). The target frame at `design.md:199-228` is exactly 100
columns on every row and 30 lines; verified. This is no longer a finding.

Approval: the operator approved the 100x30 layout in the archived `passuff-tui`
design on 2026-11-02 (`givn/archive/passuff-tui/design.md:140-142`); this change
adds a picker in the same style and the 2026-11-03 instruction authorizes the
work. The prior review accepted the same basis for a new terminal variant
(`givn/archive/passuff-stacked-inbox/design.md:211-218`). Accepted.

Motion: none. Accessibility: keyboard, `[x]`/`[ ]`, mode and error text, no
color-only meaning; the key map omits character input and backspace, which the
manual-entry buffer needs (N5).

Capture: `design.md:230-232` promises "`@e2e` transcripts for both screens" under
one slug. The evidence writer writes one `transcript.txt` per scenario and each
spawn overwrites the previous one (`crates/passuff-cli/tests/support/e2e.rs:566-601`);
the toggle's last spawn is an inbox, so the picker transcript is lost. The
mode-switch `@e2e` does not retain a picker transcript and its scenario slug is
not named at all. The all-mode picker state has no frame. B4/N20/N22.

## Non-blocking findings

- **N1 — `toml_edit` rationale is false.** `design.md:16` says it is "already in
  `Cargo.lock` via `toml`". `toml` 1.1.8 depends on `toml_parser`/`toml_writer`,
  not `toml_edit` (`Cargo.lock:3624-3632`); `toml_edit` enters only through
  `proc-macro-crate` (`Cargo.lock:2472`, `:3644`). `design.md:284` states the
  accurate version.
- **N2 — local runnability is wrong.** `design.md:277` says
  `cargo run -p passuff-cli -- watch ...`; there is no `watch` subcommand
  (`crates/passuff-cli/src/cli.rs:26-44`). Continuous watch is the no-subcommand
  path (`crates/passuff-cli/src/main.rs:62-65`).
- **N4 — `InboxView.scope` migration is unstated.** `design.md:69-72` adds
  `scope_label` and `:80-82` says both renderers carry it, but the design does
  not say `scope: Vec<String>` is removed or remapped. The renderers and the
  existing TUI test read `scope` (`crates/passuff-cli/src/tui.rs:205-211`;
  `crates/passuff-cli/src/transcript.rs:84-91`).
- **N5 — the picker key map omits text entry.** `design.md:123` lists
  `j`/`k`, `space`, `m`, `enter`, `esc`/`q`, but the manual-entry buffer needs
  character keys and backspace. Add them to the key map and the accessibility
  line.
- **N7 — matrix and feature disagree on exit status.** `design.md:252-253` says
  both e2e scenarios assert exit 0; `feature:5-23` and `:74-84` never assert an
  exit status.
- **N8 — `Then status says` has no When.** `feature:103` performs the status
  action inside a Then. Existing status scenarios use
  `When the developer asks for status`
  (`givn/specs/inspect-watch-status/watch-status.feature:7`).
- **N9 — error assertions are substring-vacuous.** `feature:150` asserts
  `the TUI reports "scope"`, which matches the picker title "Watch scope" even
  when the write failure is not reported. `feature:125` asserts
  `the TUI reports "owner/name"`, while the existing validation message is
  `must be owner/repo` (`crates/passuff-config/src/lib.rs:344`). Name the exact
  error copy and stream.
- **N10 — remaining missing scenarios.** Legacy mode resolution (unit tests only,
  `design.md:156`), case-insensitive duplicate entry, extension 1a (empty known
  list), R8 duplicate entry. B5 covers the higher-impact omissions.
- **N11 — no `@wip` on unimplemented steps.** Every new scenario is
  `@givn.added`/`@e2e` without `@wip` (`feature:5,25,37,51,59,74,86,96,105,120,128,143`),
  and no `watch_scope.rs` step file exists. The specs instruction requires
  `@wip` until the steps exist.
- **N12 — `WatchScopeState` duplicates `WatchConfig`.** `design.md:53-61`
  defines two identical shapes while `passuff-core` already depends on
  `passuff-config` (`crates/passuff-core/Cargo.toml:10`). Use one type or
  record why the state shape must stay independent.
- **N13 — glossary sweep.** `docs/glossary.md:7-12` still lists four use cases
  and calls `change-watch-scope` "planned"; `:50` describes the Terminal UI as a
  read-model consumer only. B6 covers the structural-model edit.
- **N14 — the ADR directory row is outside its table.**
  `docs/arc42/adr/README.md:14-17` appends the ADR-0006 row after the
  historical-ADR paragraph.
- **N15 — ADR consequences are only partly reflected in chapter 11.** R-022/23/24
  cover atomicity, clipping, and the state race; the ADR's "Bad, because a
  configuration writer must preserve the field" consequence has no entry
  (`ADR-0006:45-48`; `docs/arc42/11:28-30`).
- **N16 — seed traceability.** D1-D4 carry no seed reference and the seed's
  questions are unnumbered (`proposal.md:63-108`; seed `:124-147`); Q5 is still
  `Status: open` in `questions.md:142-144` although the seed resolves it.
  ADR-0006's Confirmation cites the reload scenario for the legacy shapes
  (`ADR-0006:50-55`).
- **N17 — success guarantee and mechanism wording.** `usecase.md:71` lists three
  clauses; `:28` and `:67` use "persisted"/"atomically"; `:22` says the terminal
  UI can read while the flow writes. The prior review treated these as minor;
  they remain minor.
- **N18 — reload error floods.** `design.md:113-114` appends a transcript line
  per failed reload; a persistent malformed config prints once per poll
  interval. Report once per distinct error or rate-limit.
- **N19 — lint advisories and the bundled e2e workflow.** `givn lint` exits 0
  with shape/subset advisories; the toggle scenario has more than 14 steps and
  bundles select, add, deselect, and two inbox opens under one inventory entry.
  The regular add scenario repeats the e2e add assertion.
- **N20 — capture says "both screens" for one slug.** `design.md:230-232`
  promises picker and inbox transcripts under the toggle slug; the writer
  writes one `transcript.txt` per scenario and overwrites it per spawn
  (`crates/passuff-cli/tests/support/e2e.rs:566-601`). The mode-switch `@e2e`
  does not retain a picker transcript. B4.
- **N21 — the e2e config read helper is unnamed.** `design.md:239-241` names
  only `World::e2e_state_json`. The `@e2e` Thens `the watch scope mode is ...`
  and `the watch scope contains ...` must read the config the spawned TUI child
  wrote (`e2e_config_dir`, `crates/passuff-cli/tests/support/e2e.rs:209-217`),
  not `World::config_text`; the design names no helper. B2.
- **N22 — the all-mode picker state is undefined.** `design.md:122-124` gives
  the picker keys unconditionally and never says whether known repositories show
  `[x]`/`[ ]` in all mode or what `space` does there; `usecase.md:50` says only
  that the list is "informational". The visual contract has no all-mode frame.
  B4.
- **N23 — `ScopeSnapshot` is undefined.** `design.md:29` returns it from every
  `WatchScope` method; no field list, no relationship to `WatchScopeState` or
  `InboxView.scope_label`. The interactive loop also needs the post-edit config
  or snapshot to reload the inbox view; `design.md:129-130` says only "reloads
  the inbox view". B3.
- **N24 — the design's traceability omits R11.** `usecase.md:54` records that
  selecting in all mode switches to selected with that repository; the design's
  action list (`design.md:90-91`) and traceability table (`:145-157`) do not.
  The toggle's first Then depends on it.
- **N25 — `scope_label` renderer change unstated.** The `watch scope: all
  authored pull requests` assertion (`feature:82`) depends on the renderer
  reading `scope_label`; `design.md:80-82` does not say how the TUI and status
  renderers change from `view.scope` (`crates/passuff-cli/src/tui.rs:205-211`;
  `crates/passuff-cli/src/transcript.rs:84-91`).
- **N26 — the eager-baseline wording does not match the harness.** `design.md:178-182`
  says `the watcher has already observed` "writes the baseline state
  immediately"; today the step sets `pending_baseline` and the baseline runs on
  the next action that calls `ensure_baseline`
  (`crates/passuff-cli/tests/support/mod.rs:992-1001`;
  `crates/passuff-cli/tests/steps/pr_activity_notification.rs:38-45`). The
  reselect scenario's Green depends on the change being made explicit.
- **N27 — the toggle bundles another action's e2e coverage.** `feature:17-22`
  opens the inbox twice inside the toggle; the matrix row (`design.md:252`)
  names only the picker spawn. This is not the watch-scope action's e2e evidence
  and the matrix does not say so.

## Hardening outstanding

The author must apply these before the task list is written:

1. Fix the reload held runtime (B1): state a runtime-scoped cycle counter (or
   use one `run_watch_with(StopAfter(2))` call with a control that writes the
   config between cycles), name the config the second call passes (the pre-edit
   config), and state that the reloaded config rebuilds the core.
2. Fix the empty-retained scenario (B2): seed a known but unselected repository
   and assert it is not selected after all→selected; name the regular and e2e
   config read/sync helpers and state that every scope action refreshes them.
3. Fix the contract table (B3): define `ScopeSnapshot` (or return
   `WatchScopeState`); name `review-activity-inbox` and `inspect-watch-status`
   as the `InboxService` consumer use cases; replace `ConfigLoader`'s "all use
   cases" and add `ScopeEditor` as a consumer.
4. Fix the mode-switch e2e evidence (B4): open the picker, assert its mode line,
   commit the mode scenario's transcript, and add the all-mode picker frame;
   or change the matrix to the inbox and record the picker mode display as a
   visual-review gap. Keep the 100x30 frame as is.
5. Add the missing scenarios (B5): R2 status/activity hiding, extension 4a,
   reload failure, account failure during catch-up, R5 two-account; or narrow
   the use case/design and record the disposition. Tag `@wip` until the steps
   exist.
6. Update the structural model (B6): `docs/domain-model.md` scope/status, the
   `Inbox → Terminal UI` relationship, the Activity Review boundary sentence,
   and the glossary's Terminal UI; record the structural assessment in
   `design.md`/`arc42.md`.
7. Sweep the non-blocking findings, especially N1, N2, N4/N25, N9, N10, N11,
   N14, N15, N16, N21, N22, N24, N26, N27.
8. Run `givn lint --change passuff-watch-scope` after the spec edits; do not
   remove any `@e2e` tag.

## Verification performed

- Ran `givn instructions design-review --change passuff-watch-scope`,
  `givn instructions specs --change passuff-watch-scope`,
  `givn instructions architecture-principles`, `givn instructions review
  --change passuff-watch-scope`,
  `givn instructions arc42-docs --change passuff-watch-scope`,
  `givn instructions design --change passuff-watch-scope`, and
  `givn status --change passuff-watch-scope`.
- Ran `givn lint --change passuff-watch-scope`: exit 0, advisories only.
- Ran `givn check design-review --change passuff-watch-scope`: exit 0.
- Read `proposal.md`, `design.md`, `arc42.md`, the delta feature, the permanent
  `change-watch-scope` use case, the ideation seed, `questions.md`, `frame.md`,
  the persona, the glossary, the domain model, all twelve arc42 chapters,
  ADR-0006, both ADR indexes, and the archived `passuff-tui` and
  `passuff-stacked-inbox` designs.
- Verified the harness and product code: `run_watch_with`
  (`crates/passuff-cli/src/lib.rs:204-240`), `StopAfter` (`:117-147`),
  `run_watch_cycles` (`crates/passuff-cli/tests/support/mod.rs:628-690`),
  `run_tui_snapshot` (`:762-779`), `build_config` (`:528-565`), `run_status`
  (`:723-734`), `run_inbox` (`:736-758`), `state_json` (`:987-993`), the e2e
  config/state paths (`crates/passuff-cli/tests/support/e2e.rs:209-217`), the
  evidence writer (`:566-601`), the config validation
  (`crates/passuff-config/src/lib.rs:333-355`), the TUI/status scope renderers
  (`crates/passuff-cli/src/tui.rs:205-211`,
  `crates/passuff-cli/src/transcript.rs:84-91`), the TUI mark-read writes
  (`crates/passuff-cli/src/tui.rs:151-162`, `crates/passuff-core/src/inbox.rs:103-142`),
  the out-of-scope skip and prune (`crates/passuff-core/src/cycle.rs:147-152`,
  `:213`), and the absent `watch` subcommand
  (`crates/passuff-cli/src/cli.rs:26-44`).
- Verified `toml_edit` enters through `proc-macro-crate`
  (`Cargo.lock:2472`, `:3644`) and is not a `toml` dependency
  (`Cargo.lock:3624-3632`).
- Verified all 12 arc42 chapter files exist, contain project content, and
  contain no box-drawing or pseudo-diagrams.
- Verified the picker frame is exactly 100 columns per row and 30 lines
  (`design.md:199-228`).
- Verified no `@e2e` tag was removed: the delta has two `@e2e` scenarios
  (`feature:5`, `:74`).
- No file other than this review was changed.

DESIGN-REVIEW: PASS

## Closure (operator-directed)

| Finding | Fix |
|---|---|
| B1 reload runtime | `CliRuntime::run_watch_cycle(path)` reloads, rebuilds the core, and runs one cycle; the harness uses it for both cycles; the reload scenario asserts a watcher-produced activity. |
| B2 empty-retained/config sync | The empty scenario seeds a known, observed repository and asserts it stays unselected; `World::sync_scope_from_file`, `World::e2e_config_text`, and `World::e2e_state_json` are named. |
| B3 contracts | `ScopeSnapshot` is defined; `InboxService` names `review-activity-inbox`/`inspect-watch-status`; `ConfigLoader` lists `ScopeEditor`. |
| B4 mode e2e/picker | The mode scenario opens the picker and asserts `Mode: all`/`Mode: selected`; the mode transcript slug is named; the all-mode picker state is defined. |
| B5 missing scenarios | Status/activity hiding, baseline-no-records, reload failure, and two-account scenarios were added; account-failure catch-up is a named unit test with its disposition. |
| B6 structural model | The domain model covers `change-watch-scope`, adds the Watch Configuration context/relationships, and the Terminal UI/glossary include `ScopeEditor`; `arc42.md` records the assessment. |
| N1-N27 | `toml_edit` rationale; run command; `scope_label`; picker text keys; exit-status assertions; status When; exact error copy; legacy/duplicate unit tests; `@wip`; state-shape rationale; glossary preamble; ADR index placement; R-025; seed references and Q5 closure; ADR confirmation; reload-error rate limit; capture slugs; e2e config helper; all-mode picker; R11 traceability; renderer change; eager baseline; capture/evidence notes. |

The long toggle scenario is kept as one end-to-end action; its split-or-keep
disposition is recorded in the final review.

DESIGN-REVIEW: PASS
