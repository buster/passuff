# Review: constant-ise-key-parser

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
| Tasks gate | `givn check tasks --change constant-ise-key-parser` | verify passed; changed-scenario verification not applicable (Behavior change: none) |
| Static lint | `givn lint --change constant-ise-key-parser` | 0 delta files; no blocking finding |
| Receipt freshness | `givn status --change constant-ise-key-parser` | verification not applicable; both scopes green at the reviewed tree |

## Implementation evidence check

- **`@e2e` tag integrity:** no delta spec exists. The 29 frozen `@e2e`
  scenarios are unchanged; `givn ise compare` reports `End-to-end tests: match`
  before and after the build. No tag was removed or added.
- **Empty step bodies:** 0 added by this change. The change adds no Gherkin
  step definition. The rebuilt `keys.rs` adds four parser unit tests, each with
  a real assertion.
- **Task evidence:** `b23d060` contains the pre-trial design, design review,
  task list, proposal, and `.givn-skip`; `9c99766` contains the trial evidence
  and the design corrections; `6e59bcd` records the task list. The trial commit
  contains the prepare manifest, both compare outputs, the builder notes, the
  build log, the component result, the frozen result, and the neighbour record.
- **Design promises:** the manifest, `prepare.txt`, `compare-baseline.txt`,
  `compare-after-build.txt`, `builder-notes.md`, `build.txt`,
  `component-run.txt`, `component-result.json`, `e2e-run.txt`,
  `e2e-result.json`, and `neighbour-record.md` exist at the paths `design.md`
  names. The design's frozen-file count was corrected from 13 to 12 after the
  prepare.
- **Design versus implementation:** the spec copy removed exactly
  `crates/passuff-cli/src/tui/keys.rs`; the builder wrote that file plus the
  omitted build root. No kept product source, test harness, Specification file,
  or frozen test changed. The component suite (215/215) and frozen suite
  (29/29) pass against the rebuilt binaries.
- **Strict mode:** `crates/passuff-e2e/tests/e2e_runner.rs` and
  `crates/passuff-cli/tests/features_runner.rs` call `.fail_on_skipped()`. The
  frozen runner counts parsing errors as failed and exits non-zero on a zero
  total. Both result files report 0 skipped.
- **E2E scope:** the frozen package is outside the copy. The frozen run uses the
  checkout package with `PASSUFF_BIN_DIR=/tmp/passuff-s11-keys/target/debug`, so
  the scenarios read the unchanged corpus and execute the rebuilt binaries. The
  rebuilt binary hashes are
  `3dd5562ce4bfab65062d3fa4a30a253f4e6f2acf85349c53cf610333ef332d4a` and
  `f29e793c5ca5c3ad12448585953c3cdd4e3e9bc1f2154771b95f5995f2a7bc0f`.
- **Interaction coverage:** no capability, use case, interaction inventory
  entry, or `@e2e` scenario changes. No Interaction Coverage Matrix row is
  required.

## Change strategy and Spec gap

`Decides D6`: change strategy `Constant Ise`, `Ise Test` mode `Dry-Run Ise`.
The Specification and the frozen package stay frozen; the key parser is
replaced behind them in `/tmp/passuff-s11-keys` and graded against the frozen
suite before any cutover. The copy is not promoted.

No `Spec gap` occurred. The builder found every parser rule it needed in
`docs/reference.md` (the `--keys` paragraph and the TUI keymap), the scenarios
under `givn/specs/**`, `tui/mod.rs::map_key`, and the exhaustive `Key` matches
in `tui/app.rs`. The `--keys` list under-enumerates the characters the corpus
uses (`s`, `S`, `f`, `F`, `h`, `u`, `c`); each has exactly one mapping in the
keymap table and `map_key`, so the builder recorded an assumption, not a
`Specification defect`. The missing build root is a neighbour, not a parser
contract gap.

## Neighbour record

The review question is: which components changed outside the replaced unit, and
why? The full comparison is `evidence/ise/neighbour-record.md`; the scan is
`evidence/ise/neighbours-scan.txt`.

| Path | Kind | Reason |
|---|---|---|
| `Cargo.toml` | Recreated build root | The declaration retains no root workspace manifest, so the copy omitted it. The builder reconstructed it from the kept crate manifests. It differs from the checkout in dependency versions and features: `toml 0.8`/`toml_edit 0.22` instead of `1`/`0.23`, `clap` without `env`, `reqwest` without `default-features = false` and `rustls-tls`, `tokio` without `sync`, `rust-version 1.85` instead of `1.88`, and no `[profile.dev]`/`[profile.test]`. |
| `Cargo.lock` | Generated | The copy omitted the lockfile; Cargo generated it from the reconstructed workspace. |
| `rust-toolchain.toml` | Recreated build root | The copy omitted it; the builder pinned the installed `stable` channel without `profile = "minimal"`. |
| `crates/passuff-cli/src/tui/keys.rs` | Replaced unit | The only removed path; rebuilt by the builder with the same `Key` enum and token mapping, plus four parser unit tests. |
| Every kept product file | Unchanged | No product source, test harness, Specification file, or frozen test changed; the four TUI siblings (`app.rs`, `mod.rs`, `render.rs`, `markdown.rs`) and `tests/steps/tui.rs` are unchanged consumers. |

The build-root differences did not prevent the component suite (215/215) or the
frozen suite (29/29). The copy is not promoted.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| No goal, limit, context, strategy, building block, runtime flow, deployment, test strategy, decision, quality scenario, risk, or glossary change | Chapters 01 to 12 | All rows No | Scope and `Decides D1` to `D6` | Setup and trial batches | The trial changes no durable doc; the copy is outside the repository | Yes |

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language and structural model

No product domain term is added. "Spec copy", "manifest", "trial", and
"neighbour" are process vocabulary defined by `givn instructions ise`.
`docs/glossary.md` and `docs/domain-model.md` are unchanged because no entity,
boundary, or relationship changes.

## Coverage

The change adds no production code. The component scope covers 2877/4293 lines
and 582/1858 branches. The E2E scope measures 0/4293: the component runner
cannot bind the frozen package's `@e2e` steps, so `measure-coverage.sh` selects
the placeholder scenario and records the zero-line report. That is TD-008, not
a gap in this change. The trial's copy is outside the checkout and is not
measured. No uncovered region belongs to this change.

## Findings

| ID | Finding | Disposition |
|---|---|---|
| R1 | The reconstructed build root chose different dependency versions and features from the checkout (see the neighbour table). | Non-critical. The build root is outside every declared role, so the copy omits it; the builder reconstructed it from the kept manifests and the trial records the exact differences. Both suites are green and the copy is not promoted. A future change that needs exact dependency parity should declare the build root as a retained input. |
| R2 | The frozen suite does not exercise `--keys` directly; it drives key presses through the pty. | Non-critical. The component suite in the copy binds every scenario key phrase through `parse_keys` (215/215) and the frozen suite remains the acceptance authority (29/29). No scenario was added or weakened. |
| R3 | The Cargo registry held published `passuff` 0.1.0 to 0.1.2 sources, a contamination risk for a clean-room builder. | Quarantined to `/tmp/passuff-s11-registry-quarantine`; the post-quarantine scan is clean. The builder states it did not read any registry source; the reconstructed workspace resolves the product crates by path. No product change. |
| R4 | The regular coverage report is 2877/4293 lines and the E2E report is 0/4293. | Pre-existing measurement scope, TD-008. The change adds no production code and no uncovered change region. |

No finding blocks the change.

## Overlap dispositions

| Scenario A | Scenario B | Disposition |
|---|---|---|

## Split-or-keep

| Scenario | Decision |
|---|---|

README-IMPACT: none

REVIEW: PASS
