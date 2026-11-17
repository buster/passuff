# Design review: constant-ise-key-parser

Review kind: self-review
Author: deepseek/deepseek-flash via omp, 2026-11-17
Reviewer context: deepseek/deepseek-flash via omp, 2026-11-17

The proposal records why independence is not requested for a trial with no
product behavior change.

## Grilling

| Branch | Question | Answer and evidence |
|---|---|---|
| Scope | Does the plan cover exactly the proposal? | Yes. One spec copy, one fresh builder, one rebuilt file, the product build, the frozen suite, the component check, the neighbour record, and the gap rule. No product file, scenario, declaration entry, or frozen test is edited. |
| Scope | Is "done" unambiguous? | Yes: the manifest, both compare outputs, the builder notes, the frozen result (29 scenarios, 240 steps, 0 failed, 0 skipped), the component result, and the neighbour record exist. A `Specification defect` stops the rebuild and is recorded instead. |
| Tech choices | Is `givn ise rebuild-component` the right command? | Yes. The plan names it, the installed givn 0.9.20 provides it, and it removes only the named path. `rebuild-all` would remove every source root and no longer test the component boundary. |
| Tech choices | Why not copy `crates/passuff-e2e` into the trial? | The command does not copy the declared `e2e` set; it is frozen and stays in the checkout. The frozen runner is invoked from the checkout with `PASSUFF_BIN_DIR` pointing at the copy's binaries. That is the acceptance authority, not a trial copy of it. |
| Tech choices | Why does the design accept a rebuilt build root? | The root `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml` are outside every declared role and source root, so the copy omits them. The kept crate manifests reference `workspace = true`, so the builder must reconstruct the workspace root to build. This is recorded as a neighbour, not hidden. |
| Missing scenarios | Does any behavior need a delta scenario? | No. `Behavior change: none` and `.givn-skip` carry `specs`. The trial grades existing `@e2e` scenarios and existing component scenarios; it adds none. |
| Testability | Can the frozen suite fail on a wrong rebuilt parser? | Yes for the key bindings the pty scenarios drive (`q`, `esc`, `tab`, `enter`, `j`, `k`, and the detail/quit paths). It does not drive `--keys` directly, so the component suite is the supporting check for the string vocabulary. The design states both limits. |
| Testability | Is the strict-mode mechanism named? | Yes: both runners call `.fail_on_skipped()`; the frozen runner counts parsing errors as failed and exits non-zero on a zero total. The design names both files. |
| Testability | Is a single-scenario command named? | No single-scenario filter exists in either runner. The design records that a failure names the scenario in the transcript and that the trial runs each suite once. |
| E2E fidelity | Does the design redefine the interface? | No. The frozen package drives the real binaries over a pty and wiremock; the design does not change the driver, the assertions, or the corpus. |
| Interaction coverage | Does the plan touch an inventory entry? | No. No use case, action, or `@e2e` scenario changes, so no matrix row is added. The design records that. |
| Risk | What is the most likely way this trial fails? | The builder reconstructs the parser with a narrower input vocabulary than `parse_keys` actually has, because `docs/reference.md` lists fewer tokens than the parser accepts. Mitigation: the kept `tui/mod.rs` `map_key` carries the full mapping, the component suite binds the scenario phrases through `parse_keys`, and D4 stops the trial if a required rule is absent instead of inventing it. The trial records the actual assumption. |
| Risk | Could the builder read the original parser? | The checkout, the git history, and every other copy are readable on the host. Mitigation: the builder is spawned with the copy as its workspace and an explicit instruction not to read the checkout's `crates/**`, another copy, or the git object store; its notes carry the clean-room statement. The copy itself has no `.git` and no `keys.rs`. |
| Use-case and Persona | Are the use case and Persona dispositioned? | No use case changes. `multi-account-developer` is named as the reader of the unchanged keymap; no biography is used as an actor. |
| Decision ledger | Is every `D<n>` resolved? | Yes. D1 to D5 are decided in the design; D6 records the change strategy. No carried decision remains. |
| Contracts and components | Does the change touch a contract? | The `TerminalKeyMap` contract is replaced behind unchanged consumers. The design names the contract, its consumers, the implementing file, and the split trigger. No consumer imports the parser's internals. |
| Decision timing | Is any question answered by usage or carried? | No. The trial commands, the trial revision, the green definition, the gap rule, and the neighbour scope are decided now. |
| change strategy | Is exactly one strategy recorded? | Yes. `Decides D6`: `Constant Ise` with `Ise Test` mode `Dry-Run Ise`, consumers, no persisted state, and the one-file boundary. |
| Visual design contract | Does the change add or change a screen? | No. The design records `Interface: none`, no viewports, and no design system; the terminal frame is unchanged. |
| Interface classification | Could a browser interface hide here? | No. The rebuilt parser is a terminal input mapper; no HTML, CSS, or JavaScript is involved. |
| Deferrals | Is any mandatory check deferred? | No. The trial is a dry run; no promotion or cutover is deferred. |
| ADR qualification | Does anything qualify as an ADR? | No. The commitment is none, the alternatives are promote or keep, and the durable consequence is none. `arc42.md` records `ADR action: NONE`. |
| arc42 | Does the claim match an independent re-derivation? | Yes. Re-reading the proposal and design: no goal, limit, context, strategy, building block, runtime flow, deployment, test strategy, decision, quality scenario, risk, or glossary term changes. All 12 rows are No. All 12 chapter files exist, have content, and contain no box-drawing art. |
| Ubiquitous language | Does the change introduce a glossary term? | No. "spec copy", "manifest", "trial", and "neighbour" are process vocabulary; `docs/glossary.md` owns product vocabulary and is unchanged. |
| Structural model | Does `docs/domain-model.md` need an edit? | No. No entity, boundary, or relationship changes. |

## Findings and hardening

| ID | Finding | Disposition |
|---|---|---|
| F1 | The copy omits the build root, so the trial cannot build without a neighbour change. | Accepted and recorded: `Decides D5` and the architecture table name `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml` as expected neighbour changes. The builder reconstructs them; the neighbour record gives the reason. |
| F2 | The frozen suite does not exercise `--keys` directly, so it cannot catch a narrowed string vocabulary by itself. | Accepted and mitigated: the component suite in the copy binds the scenario phrases through `parse_keys`, the kept `map_key` carries the full mapping, and the design records the limit. The frozen suite remains the acceptance authority. |
| F3 | The design names the exact evidence paths and the copy directory under `/tmp`; a rerun would collide. | Hardened: the evidence command uses a fixed trial directory that must not exist; a rerun removes or renames the old copy first. The manifest and logs record the revision and the copy path. |
| F4 | A `Specification defect` would leave no frozen result, which could be mistaken for a trial failure. | Hardened: the design and tasks state that a gap stops the rebuild attempt and is archived as the trial result, with the gap record naming the document. No invented rule is graded. |

## Re-confirmation

No post-review design change has been made. The design's decisions, the
technology table, the strict-mode section, the trial commands, the evidence
layout, and the arc42 assessment were re-read against the served design and
design-review instructions. The verdict stands.

## Verdict

Every branch is walked, the four findings are resolved or explicitly accepted,
and the trial commands and evidence paths are in `design.md`.

DESIGN-REVIEW: PASS
