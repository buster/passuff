# Design Review: passuff-stacked-inbox

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent design reviewer (fresh context)

## Verdict

The blocker is closed and no new blocker was found. The unread scenario now comments on the declared standalone key `acme/api#40`, executes without the `pr_mut` panic, and its two Then steps hold under the design's aggregation rule. The four reported advisory fixes are present in the files: retry cost stated as an account-observation attempt (`design.md:94`), child profile output `e2e-child-%p-%m.profraw` (`design.md:253`), the read-only unit test named in both the ledger and the test plan (`design.md:181`, `:200-201`), and a frame that is exactly 30 lines by 100 columns (`design.md:121-152`). The plan is implementation-ready; nine non-blocking advisories remain for the author to apply before archive.

## Blocker closure

**B8 (closed) — the unread scenario comments on an undeclared pull request.**

`givn/changes/passuff-stacked-inbox/specs/review-pull-request-stacks/stacked-inbox.feature:106` now reads
`And "carol" comments "Standalone" on "acme/api#40"`. `acme/api#40` is declared at `:100` and watched at `:101`; `#42` and `#43` are declared at `:98-99` and watched at `:102-103`. `design.md:178` cites this scenario for R5 and the citation is now sound.

Empirical verification, run through the real runner (`GIVN_FEATURES=/tmp/drprobe/... GIVN_SCENARIO=... cargo test -p passuff-cli --features test-support --test features_runner`, probe files outside the repository):

1. Byte-for-byte copy of `feature:96-113` minus the two stack Givens: PASSES, `total 1, passed 1, failed 0, skipped 0`, transcript contains `2 unread of 3`.
2. Same probe plus three assertions on record state after `marks the newest activity read`: PASSES with alice on `acme/api#43` read, bob on `acme/api#42` unread, carol on `acme/api#40` unread. The newest record is therefore alice on the stack member `#43`, exactly as the scenario intends, and the stack group holds one unread record (bob on `#42`) while carol's standalone `#40` record stays outside it. `1 unread in stack` and `2 unread of 3` (`feature:112-113`) follow from the design's rule at `design.md:108-109`, and `1 != 2` discriminates the aggregated count from the total.

The three references in the prior blocker still hold and are unchanged: the When step is `crates/passuff-cli/tests/steps/pr_activity_notification.rs:162-165`, `pr_mut` panics on an undeclared key at `crates/passuff-cli/tests/support/mod.rs:520-522`, and `newest_activity_id` takes `state.activity.last()` (`:865-868`), so key order is what decides which record the read applies to.

## Closure verification

| Prior finding | Status | Evidence |
|---|---|---|
| B1 aggregate-unread scenario cannot pass | Closed as B8 | `feature:96-113`; probes 1-2 above. |
| B2 unreachable rejection scenario | Closed | No rejection scenario in the delta; R7 is the adapter unit test named at `design.md:180`, `:199-202` (wiremock two-response retry). |
| B3 stub pattern missing | Closed | `design.md:197-198`: `unimplemented!("<phrase>")` bodies, RED fails on the stub, no empty step body. |
| B4 visual contract unapproved / no real frame | Closed as recorded | `design.md:208-227` records interface, viewport, approval, capture path; the frame at `design.md:121-152` is 32 fenced lines, of which 30 content lines are each exactly 100 characters (`awk '{print length}' ` → 30x100). Remaining fidelity gap is A1. |
| B5 arc42 chapter 01 stale | Closed | `docs/arc42/01-introduction-and-goals.md:26`, `:34`: requirement line and four use cases including `review-pull-request-stacks`. |
| B6 glossary stack definition | Closed | `docs/glossary.md:40-42`. |
| B7 ordering cannot distinguish key order | Closed | `feature:30-33`: `#43` is position 1, `#42` position 2; key order is `#42`, `#43`, so a key-only sort fails `feature:36`. |
| B8 unread scenario unreachable | Closed | `feature:106` retargeted to `#40`; probes 1-2. |
| N1 chapter 02/04 constraint | Applied | `docs/arc42/02-architecture-constraints.md:15-16`; `docs/arc42/04-solution-strategy.md:5-8`. |
| N2 chapter 05 grouping wording | Applied | `docs/arc42/05-building-block-view.md:68`. |
| N3 chapter 12 neutral link | Applied | `docs/arc42/12-glossary.md:3-10`. |
| N4 refresh/from_baseline copy | Applied | `design.md:62-64` names both paths; `crates/passuff-core/src/cycle.rs:169-190`, `crates/passuff-core/src/domain.rs:170-199`. |
| N5 explicit delta e2e command | Applied | `design.md:189-192`, `:230-231`; `scripts/verify.sh:20-41`; `givn/commands.yaml` `verify.e2e_command: "./scripts/verify.sh --e2e"`. |
| N6 surface-less projection contract | Applied | `design.md:27-30` has two rows; the projection is `TuiApp`'s responsibility (`design.md:17`, `:99-117`), and `TuiApp` already renders the read model. |
| N7 both InboxService consumers | Applied | `design.md:30` lists `TuiApp` and `InboxCli` (flat, unchanged); `crates/passuff-cli/src/tui.rs:3` imports `InboxService`, not `Inbox`. |
| N8 activity Stack column unasserted | Remains, advisory A2 | `design.md:107-109`, `:202-203`; the `@e2e` assertion at `feature:17` is satisfied without the column. |
| N9 frame not a render | Partly applied, advisory A1 | Alignment fixed (30x100), but `design.md:128` puts `Position` at column 38 while `design.md:129` puts `position 1` at column 33, and the activity row at `design.md:136` shows a 10-character `2026-11-02` where the column is 27 wide and the recorded value is RFC3339 (`crates/passuff-core/src/github.rs:214`); no widths are named for the new columns. `design.md:119` declares the renderer's snapshot authoritative. |
| N10 structural model | Applied | `docs/domain-model.md:113`, `:127`, `:133-138`. |
| N11 ADR-0005 distinction | Applied with a misattributed reason, advisory A7 | `design.md:161-165`; ADR-0005 rejected version 1 because readers cannot distinguish old from new files (`docs/arc42/adr/ADR-0005-state-version-2-migration.md:56-59`), not because "the migration reinterpreted existing records"; the chosen optional field on version 2 still does not reverse that decision. |
| N12 GraphQL field source | Applied | `design.md:76-78` records the reference and the 2026-11-03 verification; `PullRequest.baseRefName` mapping already exists at `crates/passuff-core/src/github.rs:281`. |
| N13 missing scenarios | Remains, advisory A3 | `givn/specs/review-pull-request-stacks/usecase.md:39` (2c), `:41` (3a), `:54` (R8), and the cross-repository boundary (`design.md:268-269`) have no scenario. |
| N14 standalone scenario not RED | Remains as intended, advisory A4 | `feature:70-79` passes against today's TUI; a valid R1/R6 regression guard, not RED evidence. |
| N15 widget structure | Applied | `design.md:112-114`. |
| N16 activity grouping wording | Applied | `design.md:105-117`. |
| N17 retry cost wording | Applied, one residual, advisory A5 | `design.md:94` now says "one extra observation attempt"; `docs/arc42/02-architecture-constraints.md:16` and `givn/changes/passuff-stacked-inbox/arc42.md:10` say "one retry of the account observation". `design.md:282` still says "one extra account request per cycle". |
| N18 coverage process row | Applied | `design.md:253` now names `e2e-child-%p-%m.profraw`; matches `crates/passuff-cli/tests/support/e2e.rs:591-598`; starter `./measure-coverage.sh` and the merge step exist (`measure-coverage.sh:27-60`). |
| N19 e2e mock stack branch | Applied | `design.md:232-234`; the sibling helper style is `crates/passuff-cli/tests/support/e2e.rs:255-293`. |
| N20 duplicate R8 row | Applied | One R8 row at `design.md:181`. |
| N21 glossary preamble | Applied | `docs/glossary.md:9-12` names `review-pull-request-stacks` with `change-watch-scope` context. |
| N22 manual-citation cells | Remains, advisory A6 | `design.md:183` cites "Visual review of the stacked snapshot". |

## Grilling log

| # | Branch | Question | Disposition |
|---|---|---|---|
| Q1 | Scope | Does the delta contain exactly the proposal's behaviours, nothing extra, and is "done" unambiguous? | Resolved. Proposal bullets map to `feature:6` (record and show number, size, base, positions), `:26` (order), `:39` (unwatched member gap), `:55` (member state), `:71` (standalone), `:82` (membership change), `:96` (aggregated unread). Read-only (proposal `:44`) is a posture plus the named `queries_are_read_only` test rather than a scenario (A3). No scope was invented; out-of-scope items at proposal `:53-60` appear nowhere in the delta. |
| Q2 | Tech choices | Is the stack in `design.md` the right fit, with no simpler alternative? | No issue found. Existing `reqwest` GraphQL adapter, no new dependency; state stays version 2 with a defaulted field; the projection is local to `TuiApp` over `InboxView`. A version-3 migration would add fixtures and migration code for a field that no scenario requires; a separate stack screen would split the review goal (seed rejected it). |
| Q3 | Missing scenarios | Which observable rules implied by the proposal still lack a scenario? | Finding A3, non-blocking: extension 2c (standalone to stacked), 3a (zero unread), R8, and the cross-repository stack-number boundary. Every proposal bullet has an evidence path; the uncovered items are boundary rules that the design states (`design.md:261-269`) and that the planned unit tests touch, so the plan is still implementable and the gap is additive. |
| Q4 | Testability | Can every scenario fail in RED, do Then steps assert concrete values, and are strict mode, stub pattern, and a single-scenario command named? | Resolved. Strict mode `.fail_on_skipped()` (`crates/passuff-cli/tests/features_runner.rs:56`), stub pattern `unimplemented!` (`design.md:197-198`), single-scenario command via `GIVN_SCENARIO` (`design.md:191-192`; filter at `features_runner.rs:35`). Six of seven scenarios fail RED on new steps (`GitHub reports stack ...`, `the TUI shows "position 1" before "position 2"`, the new columns); `feature:70-79` is a regression guard (A4). Every Then asserts a literal string or a literal comparison. B8 is closed, so the seventh scenario now executes. |
| Q5 | E2E fidelity | Is the interaction a real action through the real interface with a primary assertion on the output? | No issue found. One `@e2e` (`feature:5`), one inventory entry (`givn/specs/review-pull-request-stacks/usecase.md:84`), one matrix row (`design.md:246`). The step spawns `CARGO_BIN_EXE_passuff-tui --snapshot` against the seeded config and state (`crates/passuff-cli/tests/support/e2e.rs:379-399`), and the primary assertion is stdout plus exit 0 (`feature:17-23`). `--snapshot` is an output mode of the same action, not a different interface. The strongest existing implementation is the one bound to `verify.e2e_command`, so there is no weaker parallel implementation. |
| Q6 | Interaction Coverage | Every inventory entry has a matrix row and one matching `@e2e`? | No issue found. `usecase.md:80-86` → `design.md:245-247` → `feature:6`: one-to-one, and `usecase.md:86` records that marking read stays in the base use case's inventory (`givn/specs/review-activity-inbox/usecase.md`, "mark activity read"). No extra rows, no unmapped entry. |
| Q7 | Risk | What is the most likely implementation failure and what mitigates it? | The existing-record path dropping `stack` on the second cycle, which no first-cycle scenario would notice. Mitigation is explicit in `design.md:62-64` (both copy paths named) and detectable by `feature:82-93` (stack 7 to stack 8, base main to base release). Second risk: the watched block's summary paragraph eating a table row inside the 8-line block, bounded by `design.md:112-114` and the frame. |
| Q8 | Use-case and Persona context | Do the use-case identity, contract, capability ownership, typed relationships, and the ten scope-rejection criteria hold? | Resolved. Goal is one verb phrase (`usecase.md:14`); the primary actor is named in Trigger (`:18`); capability `stacked-inbox` is goal-shaped (`:78`); `Extends` carries its condition (`:94`); extensions cite main-flow steps (`:37-43`); no mechanism vocabulary in goal, trigger, or guarantees; the main flow names one actor goal; preconditions are state the flow consumes (`:20-25`). Read-state change in the main flow is local acknowledgement inherited from the base use case, not a GitHub mutation. Persona `multi-account-developer` is confirmed (`givn/personas/multi-account-developer.md:3`) and stays a review lens; the Gherkin actor remains `developer`. |
| Q9 | Decision ledger | Do D1-D3 resolve, and does a seed-sourced question keep its seed reference? | Resolved: `design.md:156-168` decides all three; nothing is carried. Seed reference: the seed's two open questions are unnumbered (`givn/ideation/inbox-scope-and-stacks/changes/passuff-stacked-inbox.md:111-127`), so no `seed Q<n>` token exists to carry (A8). The topic-level Q1-Q6 are separate and closed (`givn/ideation/inbox-scope-and-stacks/questions.md:42-76`), and no scenario depends on them. |
| Q10 | Contracts and components | One contract per consumer use case, one implementing component, consumer-shaped, replaceable, no implementation imports? | No issue found. Two rows (`design.md:27-30`), one implementing component and one split trigger each. `GitHubGateway` exposes only `observe` (`crates/passuff-core/src/github.rs:53-60`); `InboxService` exposes `view`, `mark_read`, `mark_all_read` (`crates/passuff-core/src/inbox.rs:41-45`) and `TuiApp` holds `&dyn InboxService` (`crates/passuff-cli/src/tui.rs:21`, `:61`, `:74`). `PullRequestStack` is contract data, not a component. The retry lives inside `GitHubApiAdapter`, so no new contract or port is added. |
| Q11 | Decision timing | Any foundational question carried, any speculative abstraction, any contract treated as open? | No issue found. `design.md:154-168` resolves D1-D3; no `carried` disposition survives into implementation. `PullRequestStack` holds exactly the four displayed fields; no future-use method and no cache for the preview capability (explicitly rejected at `design.md:94-95`). |
| Q12 | Visual design contract | Is the contract complete, approved, and does a real target frame exist? | Recorded: `Interface: terminal` (`design.md:210`), viewport 100x30 (`:211`), screens (`:213`), approval basis (`:214-217`), rendered reference and capture (`:218-224`), motion and accessibility (`:225-227`). The frame is dimensionally exact. A1 covers the remaining column fidelity gap, which `design.md:119` already subordinates to the renderer. The stacked variant rests on the archived 100x30 contract plus the operator's 2026-11-03 direction (`:214-217`) rather than a fresh two-option approval; accepted on that recorded basis, and A1 asks for the frame to be reconciled before review. |
| Q13 | Deferrals | Was any mandatory check deferred? | None. No operator question was needed or recorded, and no `@e2e` tag was removed. |
| Q14 | Persona disposition | Is a confirmed Persona recorded and relevant, and is the disposition explicit? | No issue found. `multi-account-developer` is confirmed (`givn/personas/multi-account-developer.md:1-4`) and is the stacked-inbox viewer; `usecase.md:74` records it; `design.md:183` records the outcome. |
| Q15 | Interface classification | Test the opposite classification before accepting "terminal". | No issue found. The opposite does not hold: the surface is ratatui widgets over crossterm with a `TestBackend` snapshot (`crates/passuff-cli/src/tui.rs:1-10`, `:109-255`, `:257-258`, where the snapshot backend is `TestBackend::new(100, 30)`); there is no HTML, CSS, JavaScript, or DOM, so a browser driver would be the wrong interface. The e2e driver is the real CLI child process, which is the correct boundary. |
| Q16 | ADR qualification | Is "no new ADR" right, and does every candidate have a cheaper home? | No issue found. `arc42.md:17` records the No with its route; the retry rationale lives in `design.md:85-95` and `docs/arc42/08-crosscutting-concepts.md:92-99`; the optional state field is an additive choice under ADR-0005 explained at `design.md:160-165`; grouping is a terminal projection owned by `design.md` and chapter 05. No `NEW_ADR` is proposed, so no cheaper-home gap exists. A7 covers the misattributed sentence, not the route. |
| Q17 | Architecture documentation | Independent 12-row derivation, then diff, content, integrity, and ADR cross-check. | No issue found. My derivation matches `arc42.md:9-20` row for row (table below). All 12 chapter files exist with decision-specific content; no box-drawing characters or pseudo-diagram blocks anywhere under `docs/arc42/` (the only matches for `\|---` patterns are Markdown table separators); chapter 09 has five register rows, correct because no new ADR qualifies; chapters 01, 02, 03, 04, 05, 06, 08, 10, 11, 12 match `design.md`, and R-021 (`docs/arc42/11-risks-and-technical-debt.md:27`) covers the new risk independently of any ADR consequence. |
| Q18 | Ubiquitous language | Is the vocabulary consistent and in the glossary? | No issue found. `docs/glossary.md:40-42` holds Pull request stack, Stack position, and Stack base with `PullRequestStack` symbols and anti-terms. The feature, use case, design, and chapters use those terms; no synonym drift (`stack number`, `position`, `base` are used only as field names, which the glossary's symbols allow). |
| Q19 | Structural model | Did the change alter structure, and is the model maintained rather than re-stamped? | No issue found. `docs/domain-model.md:113` adds stack membership to the Watched Pull Request aggregate; `:127` adds the GitHub-to-watched-member read-only relationship with its governing rules; `:133-138` records the explicit no-open-question assessment. `PullRequestStack` is a value on the aggregate, not a second bounded context, which matches `design.md:50-72`. |

### Independent Arc42 re-derivation

| # | Chapter | My derivation | `arc42.md` claim | Result |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | Yes — new goal line and fourth use case | Yes | Match (`01-introduction-and-goals.md:26`, `:30-34`) |
| 2 | 02 architecture-constraints | Yes — preview-gated, read-only stack metadata; retry bound | Yes | Match (`02-architecture-constraints.md:15-16`) |
| 3 | 03 context-and-scope | Yes — new read-only external input, new terminal surface content | Yes | Match (`03-context-and-scope.md` technical-context table and scope boundaries) |
| 4 | 04 solution-strategy | Yes — read-only enrichment plus projection | Yes | Match (`04-solution-strategy.md:5-8`, `:20-22`) |
| 5 | 05 building-block-view | Yes — `PullRequestStack`, adapter and `TuiApp` responsibilities | Yes | Match (`05-building-block-view.md:62-68`) |
| 6 | 06 runtime-view | Yes — stacked-inbox runtime flow | Yes | Match (`06-runtime-view.md:192-206`) |
| 7 | 07 deployment-view | No — no process, service, or packaging change | No | Match |
| 8 | 08 crosscutting-concepts | Yes — persistence and rejection degradation | Yes | Match (`08-crosscutting-concepts.md:92-99`) |
| 9 | 09 architecture-decisions | No — additive field under ADR-0005, no new boundary | No | Match (register unchanged, `09-architecture-decisions.md:12-18`) |
| 10 | 10 quality-requirements | Yes — QS-021 | Yes | Match (`10-quality-requirements.md:53`) |
| 11 | 11 risks-and-technical-debt | Yes — R-021 | Yes | Match (`11-risks-and-technical-debt.md:27`) |
| 12 | 12 glossary | Yes — three stack terms | Yes | Match (`docs/glossary.md:40-42`; `12-glossary.md:3-10`) |

## Risk verdicts

1. **GraphQL retry — implementable.** `design.md:85-95` covers both `MAIN_QUERY` and `NODES_QUERY`, a fresh accumulator, and the nodes-error propagation that `GqlNodesResponse` lacks today (`crates/passuff-core/src/github.rs:445-447`, whose only field is `data`). The named unit test is realizable with two wiremock mounts in the existing style (`github.rs:742` already exercises an error payload).
2. **State decision — correct.** An optional `stack` on version 2 loads old files as `None`; `WatchedPullRequest` carries no `deny_unknown_fields` (`crates/passuff-core/src/domain.rs:152-163`), so old readers ignore the added field. It does not reverse ADR-0005, which rejected only staying on version 1.
3. **TUI projection — implementable.** Grouping, ordering, gaps, and unread aggregation are deterministic over `InboxView` (`crates/passuff-core/src/inbox.rs:25-31`). At 100x30 the watched block is 8 lines, which fits header, two member rows, and one summary line; the added columns fit the inner width against the existing constraints (`crates/passuff-cli/src/tui.rs:185-191`, `:229-235`). A1 covers the mock frame, not the layout.
4. **`@e2e` plus inventory — correct.** One action, one inventory entry, one scenario, the real binary, an output-based primary assertion, and evidence written to `givn/changes/passuff-stacked-inbox/evidence/visual/developer-reviews-a-stacked-inbox/` (`crates/passuff-cli/tests/support/e2e.rs:552-575` matches the slug).
5. **Scenario discrimination — passes.** Six scenarios fail RED on new steps; the unread scenario now executes and its assertions discriminate `1 unread in stack` from `2 unread of 3`; the standalone scenario is a deliberate regression guard. `givn lint --change passuff-stacked-inbox` exits 2 with advisories only, which the workflow allows.

## Non-blocking advisories (apply before archive)

- **A1 (N9 residual)** — the frame at `design.md:128-136` is dimensionally exact but not a render: header `Position` sits at column 38 while the data cell `position 1` sits at column 33, the same shift applies to `Stack`, and the row shows a 10-character timestamp where the activity column is width 27 (`crates/passuff-cli/src/tui.rs:187-190`, `:230-234`). Name the `Position` and `Stack` widths or regenerate the frame from `--snapshot`.
- **A2 (N8)** — the activity `Stack` column has no `@e2e` assertion; `design.md:202-203` covers the label in `passuff-cli::tui` unit tests.
- **A3 (N13)** — add scenarios for extension 2c, 3a, R8, and the cross-repository stack-number boundary.
- **A4 (N14)** — keep `feature:70-79` as a regression guard and do not cite it as RED evidence.
- **A5 (N17 residual)** — `design.md:282` still says "one extra account request per cycle"; the retry re-issues the account observation, which can issue a main query per page and the nodes query. `design.md:94` and chapter 02 are already correct.
- **A6 (N22)** — `design.md:183` cites a manual visual review; mark it as supporting evidence or replace it with the R8 scenario A3 asks for.
- **A7** — `design.md:161-165` attributes ADR-0005's rejection of version 1 to "the migration reinterpreted existing records"; ADR-0005 rejects it because old and new files become indistinguishable (`docs/arc42/adr/ADR-0005-state-version-2-migration.md:56-59`). The conclusion (optional field on version 2) is unaffected.
- **A8** — the seed's open questions are unnumbered, so D1/D2 cannot carry a `seed Q<n>` token. Cite the seed section by title if traceability is wanted.
- **A9** — `Developer reviews a stacked inbox` has 15 steps (`givn lint` LONG advisory); runtime cost only.

## Hardening

This review was read-only; only this file was written. No blocker requires a design, spec, or arc42 change, so `DESIGN-REVIEW` signs off now. Apply A1-A9 before the review stage: A1 and A5 are one-line edits in `design.md`, A3 and A6 add or re-mark evidence, and the rest are wording. Do not remove `@e2e`; the runner is configured (`givn/commands.yaml` `verify.e2e_command`, `scripts/verify.sh --e2e`) and the tag is required by `design.md:229-241`.

DESIGN-REVIEW: PASS
