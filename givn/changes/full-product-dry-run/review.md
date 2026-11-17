# Review: full-product-dry-run

Review kind: self-review
Author: deepseek/deepseek-flash via omp, 2026-11-17
Reviewer context: deepseek/deepseek-flash via omp, 2026-11-17

PERSONAS: multi-account-developer
VISUAL-REVIEW: NOT APPLICABLE

## Preflight

| Check | Command | Result |
|---|---|---|
| Component scope | `./scripts/verify.sh` | exit 0; 7 features, 215 scenarios, 1731 steps; `{"failed":0,"passed":215,"skipped":0,"total":215}` |
| Frozen acceptance scope | `./scripts/verify.sh --e2e --package crates/passuff-e2e` | exit 0; 6 features, 29 scenarios, 240 steps; `{"failed":0,"passed":29,"skipped":0,"total":29}` |
| Coverage | `GIVN_FEATURES=givn/specs ./measure-coverage.sh`, `./merge-coverages.sh` | regular 2877/4293 lines, 582/1858 branches; E2E 0/4293 (TD-008) |
| Tasks gate | `givn check tasks --change full-product-dry-run` | verify passed; changed-scenario verification not applicable (Behavior change: none) |
| Static lint | `givn lint --change full-product-dry-run` | 0 delta files; clean |
| Receipt freshness | `givn status --change full-product-dry-run` | verification not applicable; both scopes green at the reviewed tree |

## Implementation evidence check

- **`@e2e` tag integrity:** no delta spec exists. `.givn-skip` carries `specs`;
  the 29 frozen `@e2e` scenarios are unchanged and `givn ise compare` reports
  `End-to-end tests: match` before and after the build. No tag was removed or
  added.
- **Empty step bodies:** 0 added by this change. The change adds no Gherkin
  step definition; the only step code is inside the trial copy, which is not
  archived and not promoted. The rebuilt component runner has real step
  dispatchers and a runner that panics on a zero total and on any failed
  scenario.
- **Task evidence:** `6c5d223` holds the proposal, design, design review,
  `arc42.md`, the task list, and `.givn-skip`; `525bb7a` holds the trial
  evidence, the design correction, and the updated task list. Every checked
  task names the evidence file that proves it, and the two `COMMIT:` lines name
  those revisions.
- **Design promises:** `evidence/ise/` holds `prepare.txt`, `manifest.yaml`,
  `compare-baseline.txt`, `compare-after-build.txt`, `sandbox.txt`,
  `environment.txt`, `builder-notes.md`, `authored-files.txt`, `build.txt`,
  `binaries.txt`, `component-run.txt`, `component-result.json`,
  `component-run-declared-surface.txt`, `e2e-run.txt`, `e2e-result.json`,
  `gap-table.md`, and `neighbour-record.md` at the paths `design.md` names. The
  design's counts (21 Specification files, 62 removed, 12 frozen) match
  `prepare.txt`; its component command produced `total: 0` and the design now
  records that observation (post-trial correction).
- **Design versus implementation:** the copy removed every declared `source:`
  root and kept only the 21 Specification files; the builder authored the build
  root, 27 source/script files, and the bundled sound. The rebuilt product
  builds offline, both binaries run, and the frozen suite passes 28 of 29
  scenarios. The single failure is the activity detail body (G1), an
  `implementation defect` in `crates/passuff-cli/src/tui/render.rs` inside the
  copy, not a corpus defect.
- **Strict mode:** the frozen runner calls `.fail_on_skipped()`, counts parse
  errors as failed, and exits non-zero on a zero total. The rebuilt component
  runner writes the four counts, fails on any failed scenario, and fails when
  the selection is empty.
- **`@e2e` Then steps:** every frozen Then asserts CLI output, a JSON log line,
  an exit code, the state file, or a parsed pty frame. None asserts only on a
  repository or database. The package is unchanged and outside the copy.
- **Driving mechanism:** the frozen steps spawn the rebuilt binary from
  `PASSUFF_BIN_DIR` and drive the TUI through a real pty (`portable-pty`) parsed
  by `vt100`. No HTTP client is substituted for a terminal interaction.
- **E2E scope selection:** `verify.e2e_command` is
  `./scripts/verify.sh --e2e --package crates/passuff-e2e`, which sets
  `GIVN_RESULT_SCOPE=e2e` and runs every `@e2e` scenario in `givn/specs`. The
  trial run used that exact command with `PASSUFF_BIN_DIR` set to the copy's
  binaries, so the scenarios are the checkout's frozen corpus and the binaries
  are the rebuilt ones.
- **Design deviation:** the only design correction after review is the
  post-trial note on the component selection and the completed Interaction
  Coverage Matrix; both are recorded in `design-review.md` "Post-trial
  re-confirmation" and do not change a decision.

### Interaction coverage cross-reference

| Capability | Inventory entries | Matrix rows in `design.md` | `@e2e` titles matched | Driver evidence |
|---|---|---|---|---|
| `watch-scope` | 1 | 1 | 1/1 | `givn/specs/change-watch-scope/watch-scope.feature`; CLI step spawns the rebuilt binary |
| `watch-status` | 1 | 1 | 1/1 | `givn/specs/inspect-watch-status/watch-status.feature`; CLI step |
| `activity-inbox` | 9 | 9 | 9/9 | `activity-inbox.feature`; CLI steps and the 80x24/120x40 pty steps |
| `inbox-navigation` | 1 | 1 | 1/1 | `inbox-navigation.feature`; pty sends `Tab`/`BackTab` |
| `review-pull-request-stacks` | none | none | 0/0 | The use case records `none` |
| `initial-setup` | 3 | 3 | 3/3 | `initial-setup.feature`; scripted stdin against the rebuilt binary |
| `pr-activity-notification` | 14 | 14 | 14/14 | `pr-activity-notification.feature`; CLI and watch-loop steps |

The 29 matrix rows and the 29 frozen titles are the same set; the title diff is
empty. No inventory entry is unmapped and no matrix row is fictional.

## Change strategy and Spec gap

`Decides D6`: change strategy `whole-product rebuild`, `Ise Test` mode `Dry-Run
Ise`. The Specification and the frozen package stay frozen; every declared
`source:` root is replaced in `/tmp/passuff-s12-full` and graded against the
frozen suite before any cutover. The copy is not promoted.

The gap table records one outcome per difference and names the wrong artifact:
**1 `implementation defect`** (G1, the activity detail body in the rebuilt
renderer), **4 `specification defect`** (G5 coverage scripts, G6 qualification
suite and seeded defects, G7 the undeclared packaging/conversion scripts, G8 the
GraphQL paging rule), **23 `acceptable undocumented change`** (G2 to G4 and G9
to G28), and **no `E2E test defect`**. Because no `E2E test defect` exists, the
freeze holds and the trial is not stopped: the one red scenario is a defect in
the rebuilt copy, not in the frozen corpus.

The four Specification defects are recorded, not fixed. This change does not
edit a Specification file, per its proposal; a later change must state those
rules before another whole-product rebuild can recreate the paths.

## Neighbour record

The review question: which files changed outside the replaced unit, and why?
The full record is `evidence/ise/neighbour-record.md`; the scan is
`evidence/ise/neighbours-scan.txt`.

| Path | Kind | Reason |
|---|---|---|
| `Cargo.toml` | Recreated build root | The declaration retains no root workspace manifest, so the copy omits it. Rebuilt from the declaration and the crate contracts: `0.2.0`, `rust-version 1.80`, `ureq`/`crossterm` instead of `reqwest`/`ratatui`/`tokio`, no e2e exclude, no dev profiles. |
| `Cargo.lock` | Generated | 1,611 lines and 177 locked packages from the offline resolution, against the revision's 3,925 lines. |
| `rust-toolchain.toml` | Recreated build root | `stable` with `rustfmt` and `clippy`; no `profile = "minimal"`. |
| `builder-notes.md` | Evidence | The builder's record; not a component. |
| 17 revision product files | Not recreated as separate files | The rebuild consolidated argument parsing, sound/delivery, the key parser/app, and the inbox/probe into fewer files. Layout is free (G3); the frozen suite grades behavior (28/29). |
| 23 revision script files | Not recreated | `measure-coverage.sh`, `merge-coverages.sh`, `qualify-e2e.sh`, `bundle-macos.sh`, `json_to_cobertura.py`, and 20 seeded-defect patches. The declared Specification does not state their behavior (G5 to G7). |

Nothing else changed outside the replaced unit. The 21 Specification files and
the 12 frozen e2e files are unchanged (`givn ise compare` reports `match` for
both before and after the build). No production code in the checkout changed.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| No goal, limit, context, strategy, building block, runtime flow, deployment, test strategy, decision, quality scenario, risk, or glossary change | Chapters 01 to 12 | All 12 rows No | Scope, D1 to D8, and the Architecture Impact table | Setup, trial, and closeout batches | The trial changes no durable doc; the copy is outside the repository; `givn ise compare` matches both subjects | Yes |
| The declaration's `source:` roots are the replaceable set | `docs/arc42/02-architecture-constraints.md` | No change | D6 names the four roots | Trial batch names them | `prepare.txt`: 62 removed, 0 kept | Yes |
| The core stays UI-agnostic and delivery-free | `docs/arc42/02-architecture-constraints.md` | No change | Contracts table names `passuff-core` as the observation component | Builder task | `crates/passuff-core/Cargo.toml` in the copy depends only on config, serde, chrono, and `ureq` | Yes |

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language and structural model

No product domain term is added. "Spec copy", "manifest", "gap table", "trial",
and "neighbour" are process vocabulary defined by `givn instructions ise`.
`docs/glossary.md` and `docs/domain-model.md` are unchanged because no entity,
boundary, or relationship changes. The trial copy's vocabulary is the corpus's
and the reference's; no new term enters the product.

## Coverage

The change adds no production code to the checkout. The component scope covers
2877/4293 lines and 582/1858 branches; the E2E scope measures 0/4293: the
component runner cannot bind the frozen package's `@e2e` steps, so
`measure-coverage.sh` records the zero-line report. That is TD-008, not a gap in
this change. The trial copy is outside the checkout and is not measured. No
uncovered region belongs to this change.

## Findings

| ID | Finding | Disposition |
|---|---|---|
| R1 | The rebuilt activity detail overlay renders no body, so one frozen scenario fails (G1). | Non-critical for the change record: the defect is inside the trial copy, which is not promoted. The frozen run is recorded as 28/29 with the failure named, and the gap table routes it to a later `Constant Ise` repair. |
| R2 | Four Specification defects: the coverage scripts, the qualification suite and seeded defects, two undeclared scripts, and the GraphQL paging rule (G5 to G8). | Recorded, not fixed: this change edits no Specification file. The gap table names the document and the missing rule for each. |
| R3 | The rebuilt component runner selects 0 scenarios for an absolute `GIVN_FEATURES` directory; the copy's declared `./scripts/verify.sh` selects 215 and passes (G2). | Accepted as an undocumented choice: `docs/limits.md` does not state the accepted `GIVN_FEATURES` forms, and the declared scope surface is green. |
| R4 | The published `passuff` 0.1.0 to 0.1.2 sources remain quarantined from the `S11` trial instead of the registry. | Recorded in `sandbox.txt`: both scans are empty, `S12` moved nothing, and the builder did not read the quarantine. No product change. |
| R5 | The rebuild swapped `reqwest` for `ureq` and `ratatui` for raw `crossterm`. | Accepted: the declared constraints name only the `notify-rust`/`user-notify` compatibility and the core's no-UI rule, and both hold. Recorded as G26. |
| R6 | The E2E coverage report is 0/4293 lines. | Pre-existing measurement scope, TD-008. The change adds no production code. |

No finding blocks the change. No critical finding invalidates the trial.

## Overlap dispositions

| Scenario A | Scenario B | Disposition |
|---|---|---|

## Split-or-keep

| Scenario | Decision |
|---|---|

README-IMPACT: none

REVIEW: PASS
