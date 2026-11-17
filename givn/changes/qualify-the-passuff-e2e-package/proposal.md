# Proposal: qualify-the-passuff-e2e-package

## Release note

Record the passuff terminal budgets and exit-code contract, and qualify the
frozen acceptance suite with twenty seeded defects and the viewport evidence.

## Behavior Change

Behavior change: none

The change records a declared limit, qualifies an existing test asset, and
retains the S10 evidence. It adds no product behavior and changes no scenario.

## Use-Case Context

- Use-case ID: none, the change adds no capability and edits no use case.
- Ideation topic: `ise-rebuildability`
- Seed path: none, the work is phase `S10` of
  `givn/ideation/ise-rebuildability/plan.md`, not an ideation change seed.
- Confirmed Personas: `multi-account-developer`, the documented budgets are
  the terminal sizes that person's workstation must meet; the change adds no
  new interaction.

## Review Context

- Review kind: self-review
- Reason: the change adds one contract document, one declaration entry, two
  test-asset patches, and four evidence files, and moves 29 scenarios into the
  Specification corpus. It changes no production behavior and no scenario
  text, so an independent review context adds a second reader for the same
  mechanically checked facts.

## Problem / Opportunity

`S10` requires that the frozen `passuff-e2e` package prove it can fail, that
the terminal budgets and exit-code contract are declared, and that the
qualification evidence is retained. Four parts are missing:

- The terminal budgets (80x24 and 120x40) and the exit-code contract are
  described only in prose in `docs/reference.md` and arc42; no declared
  `limits` document states what must fit at each size or how a runner result
  is accepted.
- The suite has no seeded defect for the key parser or for the inbox
  activity-list paging. A wrong key binding or a wrong page offset can pass
  the whole suite.
- The `cargo tree` result, the qualification run, the clean pass, and the
  viewport transcripts live only in `/tmp`, so the gate cannot cite them later.
- The `S10` gate result is not recorded anywhere.

Without this change the maintainer cannot tell whether the acceptance suite is
sensitive to the key parser and paging surfaces, and `S14` has no recorded
`S10` result to read.

## Proposed Solution

- **Declared limits.** `docs/limits.md` states the 40x12 minimum, the 80x24
  and 120x40 budgets with what must fit at each, the product exit codes, and
  the runner result-file contract. `givn/specification.yaml` declares it under
  `limits`.
- **Two more seeded defects.** `scripts/qualify-e2e.sh` seeds one defect in
  the terminal key mapping and one in the inbox activity-list paging, and
  asserts the scenario that must fail for every seeded defect. The count rises
  from eighteen to twenty, with the clean scratch tree green after every
  revert.
- **Retained evidence.** The change retains the `cargo tree` output, the
  twenty-defect qualification output, the clean end-to-end pass and its result
  file, and one parsed-screen transcript per viewport size.
- **Recorded gate.** The `S10` gate condition (`G9`) is recorded with the
  command and the observed value for each row.
- **Corpus alignment.** The 29 acceptance scenarios move from the package's
  own feature file into the capability feature files under `givn/specs`, and
  the frozen package selects `@e2e` from that corpus. The seven interaction
  tables list the moved titles. Eight declared actions have no acceptance
  scenario; their rows are removed and recorded as TD-009.

## Capability Routing

`givn spec route --change qualify-the-passuff-e2e-package` reported no signal:
the top candidates tie and no ranked list is printed. `.givn-skip` carries
`specs`, so no delta scenario is authored. The permanent capability files gain
the moved `@e2e` scenarios with unchanged text, and the interaction tables are
rewritten to the moved titles.

| Proposed capability | Route's recommendation | Decision | Rationale |
|---|---|---|---|
| the seven existing capabilities' `@e2e` inventory | no signal | `EXTEND` the seven existing capability files and their interaction tables; no new capability and no delta spec | The 29 scenarios move with unchanged text, so the behavior is already specified. The move makes the inventory visible to `givn lint`; TD-009 records the actions with no acceptance scenario. |

## Out of Scope

- Product behavior, CLI output, TUI rendering, and the 40x12 minimum.
- The text of the 29 scenarios, their step definitions, and their step
  helpers. The scenarios move into `givn/specs` with their titles, steps, and
  docstrings unchanged; the feature-level `@e2e` tag becomes a scenario tag.
- The component tier and its fakes.
- The viewport sizes: 80x24 and 120x40 are recorded, not changed.
- The observation, delivery, and persistence code.
- The `givn-harness` campaign and the `S11` trial.
- Any arc42 chapter other than chapter 2 (constraints) and the testing
  subsection of chapter 8.

## Open Questions

### D1, Does `docs/limits.md` replace the arc42 constraints entry or join it?

- Question: `givn/specification.yaml` currently declares
  `docs/arc42/02-architecture-constraints.md` as its only `limits` entry.
  Should the new `docs/limits.md` replace that entry, or should both be
  declared?
- Options:
  1. **Replace it**, `limits` becomes exactly `["docs/limits.md"]`, matching
     the wording of plan section `S10`. Gain: one limits authority. Cost: the
     Rust, platform, GitHub API, and license constraints leave the declared
     Specification, so a rebuild can omit them. In practice: a rebuilt product
     could choose a webhook receiver without tripping a declared limit.
  2. **Keep both**, `limits` lists `docs/limits.md` and the arc42 chapter.
     Gain: the budgets and exit-code contract gain a home while every existing
     constraint stays declared. Cost: two files to read and keep consistent.
     In practice: `givn ise rebuild-all` retains both, and a reader finds the
     budgets in `docs/limits.md` and the stack constraints beside them.
- Disposition: carried, owner: the design's `## Decisions` ledger; deciding
  artifact: `design.md`.
