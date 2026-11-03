# Review: passuff-stacked-inbox

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent design reviewer (fresh context)

PERSONAS: multi-account-developer

The same independent fresh context covers the design review and this final
review; `design-review.md:5` records the identical context. This final pass is
the independent review analyst role (`independent review analyst (fresh
context)`) of that one context.

This pass ran in a fresh context with no author conversation history and wrote
only this file. It read the proposal, delta feature, owning use case, ideation
seed, `design.md`, `design-review.md`, `tasks.md`, `arc42.md`, the durable
arc42 chapters, glossary, domain model, merged coverage report, verification
receipt, and the committed visual evidence. It did not re-run the suites; the
receipt is the execution evidence (`verification/tasks-verification.json`:
`status: passed`, `runner_exit: 0`, `total 7, passed 7, failed 0, skipped 0`,
tested head `83a3150`, reused for the documentation-only revision `5389d77`).

Result: **PASS**. No implementation-evidence, arc42, language, structural,
use-case, Persona, coverage, or visual blocker remains. The only findings are
the recorded design-review advisories and the residual observations below; none
hides a behavioural gap.

## Implementation evidence check

### `@e2e` tag integrity

| Feature file | `@e2e` scenarios | Change |
|---|---|---|
| `givn/changes/passuff-stacked-inbox/specs/review-pull-request-stacks/stacked-inbox.feature` | 1 (`:5-6`, `Developer reviews a stacked inbox`) | Added with the delta; the only tag edit in `57c4652` removed `@wip` (`:5`, `:25`, `:38`, `:54`, `:70`, `:81`, `:95`), never `@e2e`. `git show 57c4652 -- …/stacked-inbox.feature` shows `- @givn.added @e2e @wip` → `+ @givn.added @e2e`. |
| `givn/specs/review-activity-inbox/activity-inbox.feature` | 3 (`:170`, `:182`, `:303`) | Unchanged by this change. |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | Unchanged. |
| `givn/specs/inspect-watch-status/watch-status.feature` | 1 | Unchanged. |

No `@e2e` tag was removed or moved to evade the verify-e2e gate, and no `@wip`
remains in the delta or permanent specs (`grep -rn "@wip"` returns nothing).

### Step-body audit

0 empty step bodies across 6 files (116 annotated definitions:
`activity_inbox.rs` 12, `activity_inbox_tui.rs` 11,
`pr_activity_notification_e2e.rs` 6, `pr_activity_notification.rs` 71,
`stacked_inbox.rs` 3, `watch_status.rs` 13). The only body without its own
assertion is the delegating Given `pull_request_state`
(`crates/passuff-cli/tests/steps/pr_activity_notification.rs:187-189`), which
calls `set_state` (`:513-530`); it is not empty. No `unimplemented!`, `todo!`,
or `#[ignore]` remains anywhere under `crates/`.

### Scenario commit evidence

Seven scenario tasks, seven `COMMIT:` entries, every one naming
`Batch: stack-grouping`:

| Task | Line | Revision | Production/test content in the commit |
|---|---|---|---|
| The stack orders members by position | `tasks.md:39` | `57c4652` | `tui.rs` projection (254 lines), `domain.rs`, `cycle.rs`, `inbox.rs`, `github.rs` |
| An unwatched stack member leaves a position gap | `tasks.md:45` | `57c4652` | same batch; `tui.rs` grouping |
| A pull request without stack metadata stays standalone | `tasks.md:50` | `57c4652` | same batch; blank-cell projection |
| A merged stack member remains in the group | `tasks.md:55` | `57c4652` | same batch; state label plus group membership |
| The stack unread count aggregates its members | `tasks.md:62` | `57c4652` | same batch; per-group unread count |
| Stack membership updates after a change | `tasks.md:68` | `57c4652` | same batch; existing-record copy path `cycle.rs:187` |
| Developer reviews a stacked inbox | `tasks.md:94` | `57c4652` | same batch; e2e step file, `support/e2e.rs`, feature, transcript |

`git show --stat 57c4652` confirms the shared batch touches `crates/passuff-cli/src/tui.rs`,
`crates/passuff-core/src/github.rs`, `domain.rs`, `cycle.rs`, `inbox.rs`,
`state.rs`, `lib.rs`, `crates/passuff-cli/tests/features_runner.rs`,
`tests/steps/stacked_inbox.rs`, `tests/support/e2e.rs`,
`tests/support/mod.rs`, and the delta feature. No scenario commit is
comment-, whitespace-, tag-, task-, feature-, or transcript-only. This is a
normal change; no `CHARACTERIZATION` marker is used.

The setup and unit-test tasks also point to `57c4652` (`tasks.md:33`, `:81`,
`:86`). One evidence-precision observation: `tasks.md:76-81` lists
`nodes_stack_error_triggers_retry` in the unit-test task whose `COMMIT:` line is
`57c4652`, but that test body was added in `c57420e`; the other four named
tests are in `57c4652`. `c57420e` is a real test commit and touches
`github.rs`, so this is a stale task citation, not missing work (advisory A2).

### Promised components

| `design.md` promise | As built |
|---|---|
| `PullRequestStack` value object (`:50-57`) | `crates/passuff-core/src/domain.rs:106-111` |
| `AuthoredPullRequest.stack` (`:59`) | `domain.rs:129` |
| `WatchedPullRequest.stack` with `#[serde(default)]` (`:60-61`) | `domain.rs:171-173` |
| `from_baseline` records stack (`:62-64`) | `domain.rs:200` |
| `refresh` copies stack on the existing-record path (`:62-64`) | `crates/passuff-core/src/cycle.rs:187` |
| `PullRequestView.stack` (`:65`) | `crates/passuff-core/src/inbox.rs:24`, filled at `:81` |
| `GqlStack` / `GqlStackEntry` optional wire fields (`:66-68`) | `crates/passuff-core/src/github.rs:607-616` |
| `GqlNodesResponse.errors` (`:90-91`) | `github.rs:527-530` |
| Validated mapping and retry (`:68-95`) | `github.rs:376-390`, `:123-146`, `:231-233` |
| `TuiApp` stack projection (`:99-117`) | `crates/passuff-cli/src/tui.rs:54-95`, `:233-291`, `:306-344` |
| Step file `tests/steps/stacked_inbox.rs`, registered (`:193-196`) | file exists; `tests/features_runner.rs:11-12` |
| E2E mock branch in `tests/support/e2e.rs` (`:232-234`) | `e2e_set_stack` at `support/e2e.rs:281-293` |

Contract use: `tui.rs:5` and `inbox_cli.rs:5` import `InboxService`, not
`Inbox`; the TUI passes `&dyn InboxService` into `load_view`, `reload`, and
`handle_key` (`tui.rs:23`, `:123`, `:136`). The only places that name the
implementation are the composition roots (`main.rs:47`, `tui_main.rs:52`) and
the implementation's own tests (`support/mod.rs`, `tui.rs` test module), which
`givn instructions architecture-principles` names as exceptions. The TUI uses
the exported `passuff_core` surface (`lib.rs:12-21`), not module internals. No
consumer imports an implementation.

### Strict-mode proof

Present with output at `tasks.md:10-21`: an undefined stacked-inbox step in
`/tmp/stacked-strict.feature` reports
`total 1, passed 0, failed 1, skipped 0` and cargo exits 1. The runner sets
`.fail_on_skipped()` (`tests/features_runner.rs:58`) and filters the scope
explicitly (`:35`, `:60-71`).

### E2E integrity (not downgraded)

Read every Then step of the delta `@e2e` scenario (`stacked-inbox.feature:17-23`):

- `the TUI shows "stack 7"` / `"base main"` / `"size 2"` / `"position 1"` /
  `"position 2"` / `"New comment from @alice"` read `last_transcript`
  (`steps/activity_inbox_tui.rs:31-37`), which `e2e_run_tui_snapshot` fills from
  the spawned binary's **stdout** (`support/e2e.rs:393-413`).
- `the exit status is 0` reads the child's real process status
  (`steps/pr_activity_notification.rs:394-402`).

The scenario's `passuff runs one check` also spawns the real
`CARGO_BIN_EXE_passuff` against the wiremock GitHub twin
(`support/e2e.rs:370-391`), and `the developer opens the inbox TUI` spawns the
real `CARGO_BIN_EXE_passuff-tui --snapshot` (`support/e2e.rs:401-409`). No
Then asserts only on the state file or a repository, and no step sends HTTP or
calls `fetch`/`XMLHttpRequest` in place of driving the interface. This is a
terminal interface, not a Web UI: the opposite classification does not hold
because the product renders ratatui/crossterm text (`tui.rs:356-364`), with no
HTML, CSS, JavaScript, or DOM, so a browser driver would be the wrong interface
and the real subprocess is the correct one.

### Exact e2e command, parallel implementation, and explicit scope

- `givn/commands.yaml` sets `verify.e2e_command: "./scripts/verify.sh --e2e"`.
  Reading the command (`scripts/verify.sh:19-23`) shows `--e2e` forces
  `scope="e2e"` and exports `GIVN_RESULT_SCOPE`; `:53` runs
  `cargo test -p passuff-cli --features test-support --test features_runner`.
  The invoked test file is therefore
  `crates/passuff-cli/tests/features_runner.rs`, which binds
  `tests/steps/stacked_inbox.rs` (`:11-12`) plus the shared
  `activity_inbox_tui.rs` and `pr_activity_notification.rs` steps and
  `support/e2e.rs`. A file with "e2e" in its path is not assumed to be the
  command; this is the file the command actually compiles.
- The tree, including untracked files, was searched for a parallel
  implementation (`grep -rn "CARGO_BIN_EXE_passuff-tui"`): it appears at
  `support/mod.rs:838` (the regular in-process harness's binary guard) and
  `support/e2e.rs:401` (the e2e harness). Both spawn the real binary; the e2e
  path is the stronger one and is the one the `@e2e` scenario uses. There is no
  second, weaker e2e implementation for this capability, and no untracked test
  file (`git status --porcelain` shows only the one documentation edit noted
  below).
- The E2E scope is selected explicitly, not inferred: `GIVN_RESULT_SCOPE` is
  set by the script and applied by `features_runner.rs:62-66`
  (`"e2e" => is_e2e`, `"regular" => !is_e2e`). Recorded executions:
  change scope `7/7` (`verification/tasks-verification.json`), delta regular
  `6/6` (`target/coverage-regular-result.json`), delta e2e `1/1`
  (`target/coverage-e2e-result.json`), permanent e2e `8/8`
  (`target/coverage-e2e-permanent-result.json`), permanent regular `123/123`
  (`target/givn-verify-result.json`); every one reports zero failed and zero
  skipped. The change is not over-covered: one distinct inventory action, one
  `@e2e` scenario, six regular boundary/variant scenarios.

### Design conformance

| `design.md` names | As built | Match |
|---|---|---|
| GraphQL `stack { number size baseRefName }` and `stackEntry { position }` in both queries (`:78-83`) | `github.rs:424-470` (`MAIN_QUERY` and `NODES_QUERY`) | yes |
| Optional wire fields, complete-object validation `1 <= position <= size`, `size >= 1` (`:66-71`) | `github.rs:376-390`, `:607-616` | yes |
| One retry from a fresh accumulator when an error names `stack`/`stackEntry`; both queries; nodes errors propagate (`:85-95`) | `github.rs:123-146`, `:231-233`; `without_stack_fields` `:264-272` | yes |
| State version 2, optional `stack` with serde default, no migration (`:16`, `:59-61`) | `domain.rs:171-173`; `state.rs:73`, `:89-91` | yes |
| `refresh` and `from_baseline` both copy stack (`:62-64`) | `cycle.rs:187`; `domain.rs:200` | yes |
| `PullRequestView.stack` (`:65`) | `inbox.rs:24`, `:81` | yes |
| Group by `(repository, stack.number)`, sort by position, summary line, Position/Stack columns, activity Stack column, blank standalone cells, nested layout (`:99-117`) | `tui.rs:54-95`, `:233-291`, `:306-344` | yes |
| Step file `tests/steps/stacked_inbox.rs` registered (`:193-196`) | file present; `features_runner.rs:11-12` | yes |
| Commands `./scripts/verify.sh`, delta feature path, `--e2e`, `GIVN_SCENARIO` (`:189-192`, `:230-231`) | `scripts/verify.sh:19-53`; `features_runner.rs:32-37`, `:67-70` | yes |
| ratatui 0.30.2 / crossterm 0.29.0 (`:19`) | `Cargo.toml` / `Cargo.lock` | yes |
| Visual contract 100x30, transcript and Alacritty PNG (`:208-227`) | `tui.rs:356-364`; evidence directory below | yes; target-frame column fidelity is the recorded advisory A1, and `design.md:119` makes the renderer authoritative |
| No new decision contradicts a reviewed choice; D1-D3 decided (`:154-168`) | `design.md` unchanged since design-review; implementation follows it | yes |

No silent implementation-vs-design deviation was found. The design's named
framework, step-file layout, runner commands, and driver are the ones built.

### Interaction coverage

Inventory from the owning use case (`givn/specs/review-pull-request-stacks/usecase.md:83-85`),
matrix from `design.md:245-247`, scenario from the delta feature:

| Inventory entry | Design matrix row | `@e2e` scenario | Driving mechanism in the step definitions | Match |
|---|---|---|---|---|
| review a stack and its activity | yes (`design.md:247`) | `Developer reviews a stacked inbox` (`stacked-inbox.feature:5-23`) | Real `CARGO_BIN_EXE_passuff check` against wiremock (`support/e2e.rs:370-391`) then real `CARGO_BIN_EXE_passuff-tui --snapshot` (`:393-413`); asserts stdout and exit 0 | yes |

1 inventory row, 1 matrix row, 1 matching `@e2e` scenario, no unmapped entry and
no extra row. The six regular scenarios are variants, boundaries, or the
degradation rule; they are not additional real user actions, so no action is
over-covered. Marking read stays in the base use case's inventory
(`givn/specs/review-activity-inbox/usecase.md:81-83`) and is exercised as the
setup action inside the unread-aggregation scenario, not as a separate stacked
inbox action.

## Arc42 implementation conformance

ARC42 CONFORMANCE: CLEAN

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Reviewing a stack as one unit is an essential requirement; four use cases | `docs/arc42/01-introduction-and-goals.md:26,34` | row 1 Yes | `:3-8`; use-case context | batch setup `:27-33` | use case and delta feature exist; README Usage describes it (`README.md:107-109`) | yes |
| Stack metadata is preview-gated and read-only; at most one retry of the account observation | `02-architecture-constraints.md:16` | row 2 Yes | `:85-95` | `:70-81` | `github.rs:123-146`; `queries_are_read_only` `:1065-1072` | yes |
| New read-only, preview-gated external input; stack mutation outside | `03-context-and-scope.md:48,54,57` | row 3 Yes | `:3-8`, `:85-95` | `:27-33` | `github.rs:424-470`; no mutation query | yes |
| Read-only enrichment plus terminal projection; flat CLI unchanged | `04-solution-strategy.md:6-7,20-22` | row 4 Yes | `:5-19`, `:97-117` | `:27-33` | `inbox.rs:24,81`; `tui.rs:54-95`; `inbox_cli.rs` unchanged | yes |
| `PullRequestStack`, adapter stack responsibility, `TuiApp` grouping | `05-building-block-view.md:61-62,68` | row 5 Yes | `:50-65`, `:99-117` | `:27-33` | `domain.rs:106-111`; `github.rs`; `tui.rs` | yes |
| Stacked-inbox runtime flow: observation → state → grouping → activity → degradation | `06-runtime-view.md:195-206` | row 6 Yes | `:74-117` | `:27-33` | `github.rs`, `cycle.rs:187`, `tui.rs:233-291` | yes |
| No process, service, or packaging change | `07-deployment-view.md:58-60` | row 7 No | no packaging change | — | existing two binaries in `passuff-cli`; no new process | yes |
| Stack metadata persistence and rejection degradation are crosscutting | `08-crosscutting-concepts.md:92-99` | row 8 Yes | `:16`, `:59-61`, `:85-95` | `:27-33`, `:70-81` | `domain.rs:171-173`; `github.rs:123-146` | yes |
| No new ADR; additive field under ADR-0005, read-only under ADR-0001 | `09-architecture-decisions.md:12-18` | row 9 No | `:154-168` | — | ADR register unchanged; state stays version 2 (`state.rs:73,89-91`) | yes |
| QS-021 covers the stacked snapshot and standalone degradation | `10-quality-requirements.md:53` | row 10 Yes | `:170-183` | `:88-94` | delta feature and transcript | yes |
| R-021 records preview availability and the retry path | `11-risks-and-technical-debt.md:27` | row 11 Yes | `:278-284` | `:70-81` | `github.rs` retry; `incomplete_stack_membership_is_standalone` | yes |
| Three stack terms with `PullRequestStack` symbols | `12-glossary.md:10` → `docs/glossary.md:40-42` | row 12 Yes | `:50-57` | — | `PullRequestStack` exists (`domain.rs:106`) | yes |

Every affected chapter contains decision-specific content, not a placeholder or
a stale contradiction. Chapter 07 is correctly unaffected; chapter 09 is
correctly marked No because the additive field stays inside ADR-0005's version-2
boundary. The retry cost is stated consistently in `design.md:282` ("one extra
observation attempt") and chapter 02 ("one retry of the account observation");
`design.md:161-165` matches ADR-0005's stated reason (`:60-61`). Those two
prior design-review advisories (design-review A5, retry wording, and A7, ADR
reason) are closed in the current design.

## Ubiquitous language and structural model

The glossary owns three new terms and their symbols
(`docs/glossary.md:40-42`): **Pull request stack** (`PullRequestStack`),
**Stack position** (`PullRequestStack.position`), **Stack base**
(`PullRequestStack.base`), with anti-terms that reject "stack of commits" and
confusing position with pull request number. The delta feature, use case,
design, arc42 chapters, README, and code use those terms consistently:
`stack`, `position`, `base`, `size`, and `standalone`; no "chain", "stack of
commits", or alternate "stack id" vocabulary appears. Existing terms are reused
unchanged: **Inbox**, **Activity record**, **Read state**, **Watched pull
request**, **Watch scope**. The glossary preamble (`:9-12`) names
`review-pull-request-stacks` (`stacked-inbox`) and the planned
`change-watch-scope` context, so a reader can find the owning capability.

Structural model: the change adds a value on the existing **Watched Pull
Request** aggregate, not a new aggregate or bounded context. `docs/domain-model.md:113`
adds the GitHub-reported stack membership to that aggregate's consistency
responsibility; `:127` adds the read-only `GitHub → Watched Pull Request (stack
membership)` relationship governed by `review-pull-request-stacks` R1-R4; and
`:133-138` records the explicit no-open-question assessment. The structural
overview deliberately stays at aggregate/entity/external-system level and
already omits table-only read-model edges (`Activity Record → Inbox`,
`Watched Pull Request → Watch status`), so the new value does not require a new
node or edge. The model is maintained, not re-stamped; no stale term or second
definition remains.

## Use-case and Persona conformance

Owning use case `givn/specs/review-pull-request-stacks/usecase.md`:

- **Goal:** "The developer reviews a stack of pull requests as one unit."
  (`:14`) — one verb phrase, no conjunction.
- **Primary actor and trigger:** `developer` (`:9`); the trigger names the
  developer opening the inbox and reviewing a stack (`:18`).
- **Placement:** the capability `stacked-inbox` (`:79`) advances the review
  goal directly; "inbox" is the domain read model defined in the glossary, not
  a channel name, and the existing `activity-inbox` uses the same pattern.
- **Main flow and extensions:** steps 1-5 (`:29-33`); extensions 2a-2d, 3a,
  3b, 4a each cite their main-flow step and protect R1, R4, R2, R7, R5, R3, and
  the minimal guarantee (`:37-43`). No extension adds a success outcome.
- **Rules:** R1-R9 (`:47-55`) map to implemented and asserted behaviour:
  R1 `stacked-inbox.feature:70-79`; R2 `:25-36`; R3 `:5-23`, `:54-68`;
  R4 `:38-52`; R5 `:95-113`; R6 `:70-79`; R7 `github.rs:1046-1063` plus the
  display-side standalone scenario; R8 `github.rs:1065-1072`; R9 the
  `Extends` condition below.
- **Typed relationship:** the use case extends `review-activity-inbox`
  (`:94`). The committed form carries the condition inline; the worktree adds
  R9 (`:55`) which states the same condition, so the relationship is not
  unconditional. See advisory A4 about the uncommitted edit.
- **Preconditions and guarantees:** the preconditions are established by prior
  observation, not by the flow; the trigger is an action, not a state or
  desire; the success guarantee's aggregated unread count is the observable
  outcome of the mark-read step inherited from the base use case. No capability
  asserts an unrelated state change.
- **Mechanism vocabulary:** the goal, trigger, and guarantees name no
  mechanism; the flow's `passuff` and the minimal guarantee's "standalone
  rows" are product/interface vocabulary, not implementation instructions.
- **Oversized leaf:** the main flow names one actor goal (review the stack);
  marking read is the concluding action, not a second goal.
- **Persona:** `multi-account-developer` is confirmed
  (`givn/personas/multi-account-developer.md:1-4`) and listed at
  `usecase.md:75`. Its goal — review a stack as one unit — is the scenario's
  subject and is visible in the committed evidence. No Persona was invented or
  silently promoted; actors remain interaction roles.

## Coverage

Measured evidence (`coverage/cobertura-coverage.xml`, branch instrumentation
present):

| Report | Lines | Branches |
|---|---|---|
| merged | 2273/2670 (85.1%) | 240/716 (33.5%) |
| non-e2e | separate report present (`coverage/non-e2e-cobertura.xml`) | branch data present |
| e2e | separate report present (`coverage/e2e-cobertura.xml`) | branch data present |

The README badges (`README.md:4-5`) match the merged report exactly. Both
scopes carry branch data; nothing was downgraded to line-only.

Changed-code coverage was isolated by intersecting the added line ranges of
`57c4652` with the merged report. Every added statement line is hit. The only
added line without a hit is the closing-brace region `github.rs:233`: the
return it closes at `:232` is hit by `nodes_stack_error_triggers_retry`
(`:1106-1123`). All other uncovered lines in the touched files are pre-existing,
not introduced here: `tui.rs` key-dispatch lines (permanent TUI scenarios),
`github.rs` pre-existing error/pagination branches, `cycle.rs` account-failure
paths, and `domain.rs` `key()` (exercised by the permanent suite recorded at
`tasks.md:104-105`). The changed projection, grouping, retry, state-copy, and
view lines are all hit by the delta scenarios and the named unit tests.

Classification of uncovered changed code, in exactly three buckets:

1. **Dead code (delete): none.** `PullRequestStack`, `StackGroup`,
   `stack_groups`, `activity_stack`, `map_stack`, `mentions_stack`, and
   `without_stack_fields` are all used; no `#[allow(dead_code)]` was added.
2. **Missing test coverage (add tests): none for change-introduced rules.**
   Every changed line is covered. The design-named tests exist:
   `maps_stack_membership` (`github.rs:946`),
   `incomplete_stack_membership_is_standalone` (`:980`),
   `rejects_stack_fields_and_retries_without_them` (`:1046`),
   `queries_are_read_only` (`:1065`), `nodes_stack_error_triggers_retry`
   (`:1106`), and the six `tui.rs` projection tests (`:616-695`). The
   `map_stack` guard's `size = 0` boolean outcome is the only unexecuted
   comparison; the incomplete-object rule itself is fault-injected by the four
   malformed cases, and `size = 0` is a data variant that hits the same guard.
   It is recorded as advisory A3, not a separate rule.
3. **Legitimately hard to test (concrete justification): none in the changed
   code.** The interactive terminal loop is pre-existing and outside this
   change; the snapshot path renders through the same `render` code and the
   transcript proves it.

No fourth bucket is used. No redundant unit test was added: the projection
tests exercise the renderer directly, while the feature scenarios exercise the
cycle-to-render path; the two cases without a feature scenario
(`snapshot_labels_activity_with_its_stack`, `snapshot_separates_the_same_stack_number_across_repositories`)
are the only coverage of the activity Stack column and the cross-repository
grouping key.

## Visual review

Interface: terminal (declared at `design.md:208-227`; not a Web UI — no HTML,
CSS, or JavaScript is rendered; the Alacritty frame is text over crossterm).
Evidence under
`givn/changes/passuff-stacked-inbox/evidence/visual/developer-reviews-a-stacked-inbox/`:

| Declared UI state | Evidence seen |
|---|---|
| stacked inbox (group summary, Position/Stack columns, activity stack label) | `transcript.txt` (30 lines of 100 characters plus `exit status: 0`): header `passuff — Inbox`, `personal @octo-personal`, `watch scope: all authored pull requests`, `1 unread of 1`, summary `acme/api stack 7 | base main | size 2 | 1 unread in stack`, table headers `Pull request State CI Position Stack Title`, members `acme/api#42 … position 1 stack 7` and `acme/api#43 … position 2 stack 7`, activity row `▶ a1 … acme/api#43 unread stack 7 New comment from @alice`, controls `j/k move r read a read all q quit`. `alacritty.png` shows the same live Alacritty frame with the same stack summary, positions, stack labels, and activity line (titles `Split the migration` / `Add the schema`, timestamp `2026-11-03T09:00:00Z`). |
| standalone inbox | no separate artifact: the single `@e2e` scenario does not traverse that state; the state is asserted by `A pull request without stack metadata stays standalone` (`feature:70-79`) and `snapshot_renders_a_standalone_row_without_a_stack_summary` (`tui.rs:647-655`). The capture contract fixes one transcript plus one screenshot of the traversed state. |

Baseline: every control has a visible label; no internal identifier, token, or
enum name is visible (human labels `open`, `CI passing`, `unread`, `stack 7`);
the panel titles are real headings; no content is duplicated; the `▶` marker
plus reversed video preserves selection without colour; the screen is English
and motion-free. Persona `multi-account-developer` can see which account and
watch scope it is viewing, which stack it is in, the stack base and size, both
watched positions, and the comment; it can mark the record read and quit. Its
goal — reviewing a stack as one unit — is reachable from what is shown.

Findings, dispositioned:

- The PNG and transcript are separate runs (`1 unread of 1` and the same state,
  but different titles and timestamp). Both show the declared state and layout;
  `design.md:119` makes the renderer snapshot authoritative, and the transcript
  is that snapshot. No contract rule is broken; no action.
- The target frame's column alignment/timestamp differs from the renderer
  (`design.md:128-136`). The committed transcript is the real 100x30 render;
  this is the recorded design-review advisory A1, not a Persona or contract
  finding.
- The standalone screen has no separate capture. The e2e scenario does not
  traverse it and the capture set does not promise one; the regular scenario
  and unit test assert it. No Persona finding.

The evidence is complete for the single `@e2e` scenario and the declared
100x30 viewport. No Persona finding remains.

VISUAL-REVIEW: PASS

## Overlap dispositions

Every shape/subset advisory pair from `givn check review --change
passuff-stacked-inbox` is dispositioned below. None is a duplicate: no pair has
the same rule, the same Given/When path, and the same assertion such that
removing one loses no distinct failure detection.

The gate's one SHAPE finding is dispositioned here; the nineteen SUBST
advisories follow in the subset table, matching the gate's distinction between
shape-match findings and subset advisories.

| Scenario A | Scenario B | Decision | Reason |
|---|---|---|---|
| The terminal UI shows the empty inbox | The stack unread count aggregates its members | boundary | Empty-state rendering (T4, `0 unread of 0`) vs R5 aggregation after a read; different preconditions and outcomes. |

### Subset advisories

| Scenario A | Scenario B | Decision | Reason |
|---|---|---|---|
| An unwatched stack member leaves a position gap | The terminal UI shows the inbox | boundary | R4 gap/size rule vs the base T1 populated frame; the base scenario does not assert the gap. |
| A merged stack member remains in the group | The terminal UI shows the inbox | boundary | 3b terminal-state membership vs base T1 rendering; the merged state is the distinguishing assertion. |
| A pull request without stack metadata stays standalone | The terminal UI shows the inbox | boundary | R1/R6 standalone degradation vs the base inbox frame; the base scenario does not assert the absence of stack summary/columns. |
| Stack membership updates after a change | The terminal UI shows the inbox | boundary | R1/R3 replacement across cycles vs the base frame; the second observation is the distinguishing action. |
| The stack unread count aggregates its members | The terminal UI shows the inbox | boundary | R5 per-stack unread count vs base T1; different rule and count. |
| The stack unread count aggregates its members | Developer opens the inbox TUI | boundary | Delta regular aggregation vs the permanent in-process TUI scenario; different capabilities and different assertion set. |
| A merged stack member remains in the group | Developer reviews a stacked inbox | variant | Same R3 stack-group rendering rule, with the merged member state as the variation; merging would hide the terminal-state case. |
| The stack unread count aggregates its members | Developer reviews a stacked inbox | boundary | R5 aggregation (including the standalone `#40` contrast) vs the e2e group-display frame; the e2e does not assert the counts. |
| A merged stack member remains in the group | An unwatched stack member leaves a position gap | boundary | 3b merged membership vs R4 position gap; different rules and different state. |
| Stack membership updates after a change | An unwatched stack member leaves a position gap | boundary | R1/R3 replacement vs R4 omission/gap; different rule and different data. |
| The stack unread count aggregates its members | An unwatched stack member leaves a position gap | boundary | R5 unread aggregation vs R4 gap; distinct rules. |
| The stack unread count aggregates its members | A merged stack member remains in the group | boundary | R5 aggregation vs 3b terminal-state membership; distinct rules. |
| The stack unread count aggregates its members | Stack membership updates after a change | boundary | R5 aggregation vs R1/R3 replacement; distinct rules. |
| The terminal UI shows the empty inbox | Developer reviews a stacked inbox | boundary | Empty permanent frame vs the populated stacked e2e; different precondition, driver, and outcome. |
| Developer opens the inbox TUI | Developer reviews a stacked inbox | boundary | Base `activity-inbox` e2e vs the `stacked-inbox` e2e; different capability and different renderer state. |
| The terminal UI shows the empty inbox | An unwatched stack member leaves a position gap | boundary | Empty state vs R4 populated gap; no shared outcome beyond the fixture shape. |
| The terminal UI shows the empty inbox | A merged stack member remains in the group | boundary | Empty state vs 3b merged membership; distinct rules. |
| The terminal UI shows the empty inbox | Stack membership updates after a change | boundary | Empty state vs R1/R3 replacement; distinct rules. |
| Developer starts watching and is notified of a new comment | Developer reviews a stacked inbox | boundary | Notification capability (`watch-pull-requests`) vs stack review; different capability and driver. |

Removed+added pairs: none. The gate reports `net delta: 7 added, 0 modified,
0 removed`, so no scenario was replaced and no supersession decision is needed.

## Split-or-keep

| Scenario | Decision | Reason |
|---|---|---|
| Developer reviews a stacked inbox | keep | One `@e2e` interaction is the subject: seed the stack, run the real check, open the real TUI, assert the final frame and exit 0. Splitting would create a second e2e scenario for the same user action and duplicate the wiremock/account fixture; the 15-step length is runtime, not a rule boundary. |
| The stack unread count aggregates its members | keep | One R5 aggregation rule is the subject: the read action creates the discriminating state (`2 unread of 3` overall vs `1 unread in stack`), and the standalone `#40` record proves the group boundary. Splitting would separate the contrast that makes the scenario discriminating. |

No split is chosen, so no split work is outstanding.

## Non-blocking advisories

| ID | Observation | Disposition |
|---|---|---|
| A1 | The target frame (`design.md:128-136`) puts `Position`/`Stack` headers at different columns than the renderer and shows a 10-character `2026-11-02` where the activity column is 27 wide (`tui.rs:187-190`, `:230-234`). `design.md:119` makes the renderer snapshot authoritative; the transcript is that render. | Visual contract stays with the real render. No Persona finding. |
| A2 | `tasks.md:76-81` lists `nodes_stack_error_triggers_retry` under the unit-test task whose `COMMIT:` line is `57c4652`; that test was added in `c57420e`. `queries_are_read_only` is in `57c4652`. | Task citation precision; the test and its commit exist. No scenario commit is weakened. |
| A3 | The `map_stack` guard's `size = 0` comparison is unexecuted; the rule is fault-injected by `incomplete_stack_membership_is_standalone` (`github.rs:980-1044`) with missing entry, `position 0`, and `position > size`. A `size = 0` Examples row would close the branch. | Data variant of a covered rule; the guard returns `None` for it. No behavioural gap. |
| A4 | `givn/specs/review-pull-request-stacks/usecase.md` has an uncommitted worktree edit: `:55` adds R9, `:94` is the bare `Extends` line, and the inline condition/inheritance note is removed. R9 states the same condition, so the typed relationship remains conditional and the inventory/semantics are unchanged. The archive should commit the file. | Documentation state; no implementation or use-case semantic loss. |
| A5 | The README managed `## Coverage` block (`README.md:157-158`) still shows the prior merged values (91%/45%); `givn instructions archive` says the archive gate updates that managed block and it must never be hand-edited. The badges (`README.md:4-5`) already match 85.1%/33.5%. | Generated block; verify after archive. Not a review failure. |
| A6 | The e2e screenshot and transcript are separate runs (different titles/timestamp). Both show the declared state; the transcript is the authoritative snapshot. | Visual evidence observation; no contract or Persona rule is broken. |
| A7 | Extension 2c (standalone → stacked), 3a (zero unread), and the cross-repository boundary have no dedicated scenario; the standalone scenario is a regression guard rather than RED evidence. 2c is a data variant of the existing-record copy path exercised by `Stack membership updates after a change`; 3a renders through the same summary projection; the cross-repository key is covered by `snapshot_separates_the_same_stack_number_across_repositories` (`tui.rs:686-695`). | Design-review A3/A4. No behavioural gap; the relevant rules are implemented and covered at the projection/unit level. |

## README impact decision

README-IMPACT: updated - Usage

- Usage (`README.md:94-109`) now states that the terminal inbox groups
  pull requests GitHub reports as one stack, showing the stack base and size and
  each member's position, and that repositories without the stacked-PRs
  preview show standalone rows. The paragraph was added in `5389d77`.
- The coverage badges (`README.md:4-5`) already match the merged report
  (85.1% line, 33.5% branch). The managed `## Coverage` summary block is
  refreshed by the archive gate, not hand-edited. No placeholder is open.

## Sign-off checklist

- [x] Fabrication audit clean: 0 empty step bodies across 6 files; no
      `unimplemented!`/`todo!`/`#[ignore]`; every checked scenario has a
      `COMMIT:` touching production code or its regression test (7 scenarios,
      7 batch revisions, `Batch: stack-grouping`).
- [x] Every promised component exists and every consumer uses only its declared
      contract (`InboxService`; composition roots and the implementation's own
      tests excepted).
- [x] Strict-mode proof present and passing (`tasks.md:10-21`;
      `.fail_on_skipped()` at `features_runner.rs:58`).
- [x] Scenario execution evidence recorded per scope in the verification
      receipt; the suite was not re-run in this review.
- [x] Coverage classified from the most recent measured evidence (merged
      2273/2670 lines, 240/716 branches); branch instrumentation intact; no
      line-only downgrade.
- [x] Every coverage gap classified and resolved: bucket 1 empty, bucket 2
      empty for change-introduced rules, bucket 3 empty in the changed code.
- [x] Dead code deleted; no redundant unit test added; the two unit-only cases
      cover behaviour without a feature scenario.
- [x] No `@wip`; no `@e2e` removed; no implementation-layer detail in the
      spec.
- [x] Canonical E2E policy read from `givn instructions specs`: one normalized
      inventory action has exactly one `@e2e` scenario; variants and boundaries
      are regular scenarios.
- [x] Every E2E scenario asserts the real interface (spawned binary stdout and
      exit status); no HTTP/fetch substitute; the terminal classification was
      tested against the Web UI opposite and holds.
- [x] The exact file `verify.e2e_command` invokes was identified
      (`scripts/verify.sh --e2e` → `tests/features_runner.rs` with
      `steps/stacked_inbox.rs` plus shared steps); the tree, including
      untracked files, was searched for a parallel implementation — both
      `CARGO_BIN_EXE_passuff-tui` sites spawn the real binary.
- [x] E2E scope selected explicitly through `GIVN_RESULT_SCOPE`; both required
      executions report zero failures and skips (delta e2e 1/1, permanent e2e
      8/8; regular 6/6 and 123/123).
- [x] Implementation (commands, file layout, framework/driver, state schema)
      matches what `design.md` explicitly named; no silent deviation.
- [x] Interaction coverage verified: 1 inventory row → 1 matrix row → 1
      matching `@e2e` scenario → real spawn driver.
- [x] Run declaration passes the gate (`product_type: cli`; `givn check review`
      reports `run declaration passed`); not re-executed in this pass, which
      reuses the execution receipt.
- [x] No finding was excused with a classification outside the three coverage
      buckets; the recorded advisories name concrete mechanical facts, not
      invented scope exceptions.

REVIEW: PASS
