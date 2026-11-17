# Design review: full-product-dry-run

Review kind: self-review
Author: deepseek/deepseek-flash via omp, 2026-11-17
Reviewer context: deepseek/deepseek-flash via omp, 2026-11-17

The proposal records why independence is not requested for a trial with no
product behavior change.

## Grilling

| Branch | Question | Answer and evidence |
|---|---|---|
| Scope | Does the plan cover exactly the proposal? | Yes. One spec copy, one fresh builder, every declared source root rebuilt, the product build, the frozen suite, the gap table, the neighbour record, and the environment restore. No product file, scenario, declaration entry, or frozen test is edited. |
| Scope | Is "done" unambiguous? | Yes: both rebuilt binaries run, the frozen suite produced four reconciling counts, every difference carries exactly one outcome and a named artifact, both compare outputs match, and the neighbour table names every added path outside the Specification. A copy that does not build has no trial result. |
| Tech choices | Is `givn ise rebuild-all` the plan's `--full` command? | Yes. The plan's text says "with `--full`"; the installed givn 0.9.20 `rebuild-all --help` exposes `--at`, `--into`, `--manifest`, and `--dry-run`, and `givn instructions ise` states that `rebuild-all` removes every declared source root. There is no `--full` flag to pass, so the design records the actual command and the boundary it removes. |
| Tech choices | Why not copy `crates/passuff-e2e` into the trial? | The command does not copy the declared `e2e` set; it is frozen and stays in the checkout. The frozen runner is invoked from the checkout with `PASSUFF_BIN_DIR` pointing at the copy's binaries. That is the acceptance authority, not a trial copy of it. |
| Tech choices | Why does the design accept a rebuilt build root and a rebuilt sound asset? | The root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and `crates/passuff-cli/assets/notification.wav` sit outside every declared role or inside a removed source root, so the copy omits them. The rebuild must produce them to build and run; the table names each one and the neighbour record gives the reason. |
| Tech choices | Can the copy run the component scope? | Not through `scripts/verify.sh`: its non-e2e path runs `cargo tree --manifest-path crates/passuff-e2e/Cargo.toml`, and the copy holds no such package. The design runs the rebuilt component corpus directly through `cargo test --test features_runner` and records the script gap as a difference. |
| Tech choices | Is the frozen run still measuring the rebuilt binary? | Yes. `verify.sh --e2e` builds the checkout product first, then exports `PASSUFF_BIN_DIR` for the package; the package spawns the binary from that directory. The design states the incidental checkout build so the record is not misread. |
| Sandbox | How is a clean room enforced? | The copy lies under `/tmp` with no `.git`; the builder is told to read only under it, and its notes carry the statement. Published `passuff` crate sources are moved out of the Cargo registry for the trial (S11 found 0.1.0 to 0.1.2 there), scanned before and after, and restored after the frozen run. Every cargo command uses `--offline`. |
| Sandbox | What stops the builder from fetching the product from the network? | `--offline` fails a build that needs an uncached crate, and no declared dependency is missing from the cache. The registry quarantine removes the published product source. The builder cannot read the checkout by path because it is instructed not to and the copy has no link to it. |
| Gap rule | Can the trial still archive when a contract is missing? | Yes. `D4` records one `specification defect`, leaves the behavior unimplemented behind a compiling interface, and continues. The binary must build; an unimplemented behavior fails its frozen scenario. The gap table then classifies it, and the trial meets "a runnable binary with every difference classified". |
| Gap rule | Is "acceptable undocumented change" distinct from the other three? | Yes. It names a detail the Specification leaves free whose rebuilt choice contradicts no scenario; the other three name a wrong corpus scenario, a wrong rebuilt file, or a wrong declared document. |
| Missing scenarios | Does any behavior need a delta scenario? | No. `Behavior change: none` and `.givn-skip` carry `specs`. The trial grades the existing `@e2e` scenarios and the existing component corpus; it adds none and edits none. |
| Testability | Can the frozen suite fail on a wrong rebuilt product? | Yes: it drives the real binaries through a pty and a mock GitHub server and asserts the real output, state, and frames. It cannot see the component step harness or the qualification scripts, and the design records that limit instead of claiming coverage. |
| Testability | Is the strict-mode mechanism named? | Yes: the frozen runner calls `.fail_on_skipped()`, counts parsing errors as failed, and exits non-zero on a zero total. The rebuilt component runner must do the same; a runner that cannot fail on a skipped step is an `implementation defect`. |
| Testability | Is a single-scenario command named? | No single-scenario filter exists in either runner. The design records that a failure names the scenario in the transcript and that each suite runs once. |
| E2E fidelity | Does the design redefine the interface? | No. The frozen package drives the real binaries over a pty and wiremock; the design does not change the driver, the assertions, or the corpus. |
| Interaction coverage | Does the plan touch an inventory entry? | No. No use case, action, or `@e2e` scenario changes, so no matrix row is added. The design records the eight uncovered actions from TD-009. |
| Risk | What is the most likely way this trial fails? | The builder cannot derive the full production surface from the corpus and docs, so the frozen suite fails widely. Mitigation: the gap table classifies each failure, the compiled product still runs, and the trial's value is exactly the list of missing rules. The design forbids inventing a rule to make a scenario pass. |
| Risk | Could the builder read the original product from the Cargo registry? | The registry held published `passuff` 0.1.0 to 0.1.2 in S11. Mitigation: quarantine, scans, `--offline`, the builder instruction, and the recorded clean-room statement. |
| Risk | Could the prepare read a stale copy from an earlier attempt? | The command refuses an existing `--into` path; the design removes the copy and manifest first and records the command output. |
| Risk | Could the trial accidentally edit the Specification? | The copy is the only writable tree, and `givn ise compare` runs after the build; a Specification difference fails the trial. |
| Use-case and Persona | Are the use case and Persona dispositioned? | No use case changes. `multi-account-developer` is named as the reader of the rebuilt terminal product; no biography is used as an actor. |
| Decision ledger | Is every `D<n>` resolved? | Yes. D1 to D8 are decided in the design; the proposal's D1 to D5 resolve to the same ledger entries. No carried decision remains. |
| Contracts and components | Does the change touch a contract? | All seven product contracts are replaced behind unchanged consumers. The table names each contract, its consumer use case, its consumers, its single implementing component, and its split trigger. |
| Decision timing | Is any question answered by usage or carried? | No. The trial revision, the result definition, the outcome vocabulary, the gap rule, the neighbour scope, the strategy, the commands, and the build-root rule are decided now. |
| change strategy | Is exactly one strategy recorded? | Yes. `Decides D6`: `whole-product rebuild` with `Ise Test` mode `Dry-Run Ise`, consumers, no persisted state, and the four-root boundary. |
| Visual design contract | Does the change add or change a screen? | No. The design records `Interface: none`, no viewports, and no design system; the terminal frame is unchanged. |
| Interface classification | Could a browser interface hide here? | No. The rebuilt product is a terminal and CLI product; no HTML, CSS, or JavaScript is involved. |
| Deferrals | Is any mandatory check deferred? | No. The trial is a dry run; no promotion or cutover is deferred. |
| ADR qualification | Does anything qualify as an ADR? | No. The commitment is none, the alternatives are promote or keep, and the durable consequence is none. `arc42.md` records `ADR action: NONE`. |
| arc42 | Does the claim match an independent re-derivation? | Yes. Re-reading the proposal and design: no goal, limit, context, strategy, building block, runtime flow, deployment, test strategy, decision, quality scenario, risk, or glossary term changes. All 12 rows are No. All 12 chapter files exist, have content, and contain no box-drawing art. |
| Ubiquitous language | Does the change introduce a glossary term? | No. "spec copy", "manifest", "trial", "gap table", and "neighbour" are process vocabulary; `docs/glossary.md` owns product vocabulary and is unchanged. |
| Structural model | Does `docs/domain-model.md` need an edit? | No. No entity, boundary, or relationship changes. |

## Findings and hardening

| ID | Finding | Disposition |
|---|---|---|
| F1 | The plan says "with `--full`", but givn 0.9.20 `rebuild-all` has no such flag; it removes every declared source root by definition. | Recorded: the design names the actual command and states that `rebuild-all` is the plan's `--full` replacement. No artifact invents a flag. |
| F2 | The copy omits the build root, the bundled sound, and `crates/passuff-e2e`; the first two must be authored, the third must not be copied. | Accepted and recorded in the Architecture Impact table and in `D8`; the neighbour record names the build root, and the frozen run uses the checkout package. |
| F3 | The copy cannot run `scripts/verify.sh` in either scope as written: the e2e path needs `crates/passuff-e2e`, and the non-e2e path runs `cargo tree` against it. | Hardened: the design runs the rebuilt component corpus through `cargo test --test features_runner` directly and the frozen suite from the checkout; the script gap is a recorded difference, not a reason to copy the frozen package. |
| F4 | A `specification defect` could be mistaken for a reason to stop the whole trial, leaving no frozen result. | Hardened in `D4`: the affected behavior stays unimplemented behind a compiling interface, the gap is recorded, and the trial continues. |
| F5 | The design predicts 21 Specification files and 62 removed source files from `git ls-files` at the trial revision. | To be confirmed by `prepare.txt`; if the prepare prints different counts, `design.md` is corrected after the prepare (as `S11` did). |
| F6 | The Cargo registry held published `passuff` 0.1.0 to 0.1.2 sources, a contamination risk for a clean-room builder. | Hardened: `D7` quarantines them before the builder, records both scans, and restores them after the frozen run. |
| F7 | The rebuilt component runner is authored inside the replaced source root, so it is a weak contract check. | Accepted and recorded: the frozen suite is the acceptance authority; the rebuilt runner is a supporting check, and its strict mode is checked for `.fail_on_skipped()`. |

## Re-confirmation

The design's decisions, the trial commands, the boundary, the gap rule, the
evidence layout, and the arc42 assessment were re-read against the served
design and design-review instructions. The counts in the Architecture Impact
table come from `git ls-files` at the reviewed revision and are re-checked by
the prepare report. The verdict stands.

## Verdict

Every branch is walked, the seven findings are resolved or explicitly accepted,
and the trial commands and evidence paths are in `design.md`.

DESIGN-REVIEW: PASS
