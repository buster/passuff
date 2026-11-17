# Design review: qualify-the-passuff-e2e-package

Review kind: self-review
Author: deepseek/deepseek-flash via omp, 2026-11-17
Reviewer context: the author context; the proposal records why independence is
not requested for a documents-and-test-assets change

## Grilling

| Branch | Question | Answer and evidence |
|---|---|---|
| Scope | Does the plan cover exactly the proposal? | Yes. `docs/limits.md`, the declaration, two seeded defects, the evidence set, and the gate record. No product file is edited except the two defect-patch targets, which are only targets, not edits. |
| Scope | Is "done" unambiguous? | Yes: twenty defects each fail their named scenario, the clean tree is green after every revert, `verify.sh --e2e` reports 29/29 and writes the four counts, both transcripts exist, and `cargo tree` shows no production crate. |
| Tech choices | Is bash the right place for the expected-scenario map? | Yes. The script is already bash with a defect-name array, a scratch tree, and a hash check. Bash 5.2 supports associative arrays. A separate checker would duplicate the run loop and its revert checks. |
| Tech choices | Why not extend the runner with an expected-defect mode? | The runner is the frozen acceptance asset. A defect mode inside it would couple the test asset to its qualification and change the file the suite is proving. The qualification script owns this check. |
| Missing scenarios | Does any behavior need a delta scenario? | No. The change records a limit and qualifies an existing suite; `.givn-skip` carries `specs`, and the proposal records `Behavior change: none`. |
| Testability | Can each seeded defect fail RED? | Yes, for the two new ones: dropping the quit binding prevents the pty session from exiting; the paging offset hides the only activity row at 120x40. Both were reasoned against the exact assertions; the qualification run confirms containment. |
| Testability | Is the strict-mode mechanism named? | Yes: `.fail_on_skipped()` in `crates/passuff-e2e/tests/e2e_runner.rs`, undefined steps fail by cucumber-rs default, a parse error counts as failed, and a zero total fails the run. |
| Testability | Is the single-scenario command named? | It does not exist in the frozen runner. The design records that a defect is observed by the whole 29-scenario suite and the named scenario is extracted from the transcript. |
| E2E fidelity | Does the design redefine the interface? | No. The package drives the real CLI, the real pty at both sizes, and wiremock over HTTP; nothing is changed. |
| Interaction coverage | Does the plan touch an inventory entry? | No. The 29 scenarios and every use-case interaction table are unchanged; the design records that the inventory is not re-mapped. |
| Risk | What is the most likely implementation failure? | A wrong expected scenario for one of the eighteen existing defects. Mitigation: the script continues after a mismatch and reports all of them, so one run corrects the table; the final run must be 20/20. A second risk is the quit defect's 15-second exit wait per TUI scenario, bounded by the runner's timeout. |
| Use-case and Persona | Are the use case and Persona dispositioned? | No use case is touched. The confirmed Persona `multi-account-developer` is named because the documented budgets are the sizes that person's terminal must meet; no Persona biography is used as an actor. |
| Decision ledger | Is every `D<n>` resolved? | Yes. `D1` is decided in the design: both limits entries. No carried decision remains. |
| Contracts and components | Does the change touch a contract? | No. No contract is added, changed, or replaced; the design deletes the subsection per the instruction. |
| Decision timing | Is any question answered by usage or carried? | No. `D1` is resolved now, and it decides a declaration list, not a contract. |
| change strategy | Is exactly one strategy recorded? | Yes. `Decides D2`: `incremental modification`; no rebuild, no `Ise Test` mode, no persisted-state change, no boundary. |
| Visual design contract | Does the change add or change a screen? | No. The change records the existing terminal budgets; the design records no visual design contract and names the Persona as the reader. |
| Interface classification | Could a browser interface hide here? | No. The product renders a terminal frame; no HTML, CSS, or JavaScript is involved. |
| Deferrals | Is any mandatory check deferred? | No. |
| ADR qualification | Does anything qualify as an ADR? | No. The recorded commitment is none; the declaration and the limit document are the cheaper canonical home; chapter 11 records the one actionable debt (TD-007). |
| arc42 | Does the claim match an independent re-derivation? | Yes after re-reading the proposal, design, and chapters: rows 2, 8, and 11 are affected; the other nine are not. Chapter 2 gains `Declared limits`; chapter 8's `Testing and coverage` now names both scopes, the package, and the qualification; chapter 11 updates R-019 and adds TD-007. All 12 chapter files exist, have content, and contain no box-drawing art. |
| Ubiquitous language | Does the change introduce a glossary term? | No. "Budget", "qualification", and "seeded defect" are test-tooling vocabulary; the glossary owns product vocabulary, and no product term or code symbol changes. |
| Structural model | Does `docs/domain-model.md` need an edit? | No. No entity, boundary, or relationship changes. |

## Findings and hardening

| ID | Finding | Disposition |
|---|---|---|
| F1 | The task names `limits: ["docs/limits.md"]`; the design keeps the arc42 entry too. | Accepted as a design decision, not an unresolved question: dropping the chapter removes the stack, platform, API, and license constraints from the declared Specification. Both entries satisfy the requirement that `docs/limits.md` be declared. Recorded as `Decides D1`; the deviation is named in the review report. |
| F2 | A wrong expected scenario would make the qualification fail after a long run. | Hardened: the script continues after a mismatch and reports every wrong expectation in one run; the final status is non-zero. |
| F3 | The design did not name a single-scenario command. | Hardened: the design records that the frozen runner has none, and how a defect's named scenario is extracted. |
| F4 | The defect transcripts could be mistaken for new evidence paths. | Hardened: the evidence table names the producing command for every file and confines the PTY capture to a throwaway copy outside the repository. |
| F5 | The clean-tree hash included `givn/history`, which Givn writes on every command, so a concurrent `givn status` would fail a twenty-minute run. | Hardened during implementation: the hash excludes `givn/history` as an ignored local record store; everything under version control and the change artifacts stay covered. Recorded in the design's qualification section. |

## Verdict

Every branch is walked, the four findings are resolved or explicitly accepted,
and the hardening edits are in `design.md`.

DESIGN-REVIEW: PASS
