# Review: qualify-the-passuff-e2e-package

Review kind: self-review
Author: deepseek/deepseek-flash via omp, 2026-11-17
Reviewer context: deepseek/deepseek-flash via omp, 2026-11-17

## Preflight

| Check | Command | Result |
|---|---|---|
| Component scope | `./scripts/verify.sh` | exit 0; 215 scenarios, 1731 steps; `{"failed":0,"passed":215,"skipped":0,"total":215}` |
| Frozen acceptance scope | `./scripts/verify.sh --e2e --package crates/passuff-e2e` | exit 0; 6 features, 29 scenarios, 240 steps; `{"failed":0,"passed":29,"skipped":0,"total":29}` |
| Qualification | `./scripts/qualify-e2e.sh` | exit 0; `20/20 defects detected; clean tree green`; the clean-tree hash is unchanged |
| Coverage | `./measure-coverage.sh`, `./merge-coverages.sh` | regular 2877/4293 lines, 582/1858 branches; E2E 0/4293 (TD-008) |
| Tasks gate | `givn check tasks --change qualify-the-passuff-e2e-package` | verify passed; changed-scenario verification not applicable |
| Static lint | `givn lint --change qualify-the-passuff-e2e-package` | 0 delta files; no blocking finding |
| Interaction inventory | `givn lint` | 7 files checked; the 29 declared titles equal the 29 `@e2e` scenarios; no blocking finding |
| Receipt freshness | `givn status --change qualify-the-passuff-e2e-package` | change verification not applicable; both scopes green at the reviewed tree |

## Implementation evidence check

- **Empty step bodies:** 0 added by this change. Four pre-existing bodies
  remain in the component step files: two `#[given]` defaults
  (`account_has_no_pull_requests`, `no_prior_activity`) whose empty body states
  the new world's default, and two `#[allow(dead_code)]` helpers
  (`_unused`). None is bound by this change and none is evidence for it.
- **`@e2e` tag integrity:** no delta spec exists. The 29 scenarios moved from
  `crates/passuff-e2e/features/boundary.feature` into the seven capability
  feature files under `givn/specs`; each scenario block is byte-identical
  apart from the scenario-level `@e2e` tag, and the component runner still
  excludes `@e2e`. The package runner selects `@e2e` from the corpus.
- **Task evidence:** every batch records a revision: `96d2be0` (declared
  limits), `38b09ce` (seeded defects), `40b37b4` (S10 evidence), `9b5c8ad`
  (e2e boundary), and the corpus-alignment commit that contains the move. Each
  commit contains the files its batch names.
- **Design promises:** `docs/limits.md`, the `limits` declaration, the two
  patch files, the expected-scenario map, the evidence paths, and the seven
  interaction tables exist at the paths `design.md` names. `Decides D4` is
  realized by the moved scenarios, the corpus-selecting runner, the tables, and
  TD-009.
- **Strict mode:** `crates/passuff-e2e/tests/e2e_runner.rs` keeps
  `.fail_on_skipped()`; the setup task records it; the qualification proves a
  defect turns the suite red. The runner adds an `@e2e` filter over
  `givn/specs`; a zero total still exits 1.
- **E2E scope:** `verify.e2e_command` is
  `./scripts/verify.sh --e2e --package crates/passuff-e2e`; it builds the
  product, the package selects the 29 `@e2e` scenarios from `givn/specs`, and
  the component command is unchanged.
- **Design versus implementation:** the commands, the package path, the defect
  files, the evidence layout, and the corpus location match `design.md` as
  amended under `Decides D1`, `D2`, `D3`, and `D4`.
- **Interaction coverage:** the seven tables list exactly the 29 moved titles;
  `givn lint` reports no blocking finding. Eight declared actions have no
  acceptance scenario: toggling a repository in the watch scope, switching the
  watch scope mode, narrowing the focus, opening a pull request from an
  activity, live inbox observation, the stacked inbox, `--version`, and the
  command-line notification test. TD-009 records each with the scenario it
  needs; the tables do not claim coverage for them.

## Change strategy and Spec gap

`Decides D2`: `incremental modification`; no `Ise Test` mode. The corpus move
changes where the 29 acceptance scenarios live, not their text or behavior. No
corrective change, no observed difference between the suite and the
Specification, and no `Spec gap` outcome. The change records a limit and
qualifies the suite.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Declared limits and source boundary | `02-architecture-constraints.md` `Declared limits` | Row 2 Yes | `Decides D1`, `D3` | Batch `declared-limits`, `e2e-boundary` | `givn/specification.yaml`, `givn/commands.yaml` | Yes |
| Two test scopes and the twenty-defect qualification | `08-crosscutting-concepts.md` `Testing and coverage` | Row 8 Yes | Deliverables 3 and 4, `Decides D4` | Batch `seeded-defects`, `corpus-alignment` | `scripts/verify.sh`, `scripts/qualify-e2e.sh`, the moved corpus | Yes |
| PTY coverage, suite limit, coverage scope, inventory debt | `11-risks-and-technical-debt.md` R-019, TD-007, TD-008, TD-009 | Row 11 Yes | Evidence, coverage, `Decides D4` | Batches `s10-evidence` and `corpus-alignment` | `evidence/transcripts/`, coverage run, `givn lint` | Yes |

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language and structural model

No product domain term is added; "budget", "qualification", and "seeded
defect" stay test-tooling vocabulary and are defined by `docs/limits.md` and
the scripts. `docs/domain-model.md` is unchanged because no entity, boundary,
or relationship changed.

## Coverage

The change adds no production code. The component scope covers 2877/4293 lines
and 582/1858 branches. The E2E scope measures 0/4293: after the move, the
component runner cannot bind the package's `@e2e` steps, so
`measure-coverage.sh` selects no scenario for that scope and records the
zero-line report. That is TD-008, not a gap in this change's code. No
uncovered region belongs to this change.

## Findings

| ID | Finding | Disposition |
|---|---|---|
| R1 | The E2E coverage report is 0/4293; the frozen package's production coverage is not measured. | Non-critical measurement gap, pre-existing after the package move. The component runner cannot bind the package steps, so the script records the zero-line report; instrumenting the binaries and the package remains the fix. Recorded as TD-008; the S10 gate is scenario sensitivity, which the qualification proves. |
| R2 | The seven interaction tables declared eight consumer actions with no acceptance scenario. | Non-critical, pre-existing. The gate exposed the mismatch when the scenarios moved into the corpus. The rows were removed and TD-009 records each action with the missing scenario; no scenario was invented and no table claims unverified coverage. |
| R3 | Four pre-existing empty bodies and `#[allow(dead_code)]` helpers in the component step files. | Non-critical, not introduced by this change; the given steps express the default world. Cleanup is for a later change, not this qualification. |
| R4 | `limits` keeps the arc42 entry beside `docs/limits.md`, while the task text names one entry. | Accepted design decision `D1`: dropping the chapter would remove the stack, platform, API, and license constraints from the Declaration. |

No finding blocks the change.

## Overlap dispositions

| Scenario A | Scenario B | Disposition |
|---|---|---|

## Split-or-keep

| Scenario | Decision |
|---|---|

README-IMPACT: updated - Development

REVIEW: PASS
