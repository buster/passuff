# Proposal: full-product-dry-run

## Release note

Rebuild the whole passuff product from the Specification in a clean-room copy
and grade the rebuilt binaries with the frozen acceptance suite, without
changing product behavior.

## Behavior Change

Behavior change: none

The change records the `S12` whole-product `Dry-Run Ise` trial. It adds no
product behavior, edits no scenario, changes no declaration entry, and promotes
no rebuilt file.

## Use-Case Context

- Use-case ID: none, the change adds no capability and edits no use case.
- Ideation topic: `ise-rebuildability`
- Seed path: none, the work is phase `S12` of
  `givn/ideation/ise-rebuildability/plan.md`, not an ideation change seed.
- Confirmed Personas: `multi-account-developer`, the developer whose whole
  terminal product the trial rebuilds; the change adds no new interaction.

## Review Context

- Review kind: self-review
- Reason: the change adds no product behavior, public contract, persisted
  state, screen, or dependency. It records a clean-room trial whose facts are
  produced by `givn ise rebuild-all`, `givn ise compare`, the builder notes,
  the product build, and the frozen suite. An independent context would read
  the same commands and the same result file; it would not add a distinct
  judgment.

## Problem / Opportunity

`S12` requires a whole-product `Dry-Run Ise` trial on a real product. The
repository records a component-level trial (`S11`) and an E2E qualification
(`S10`), but never the full replacement: every declared `source:` root removed
at once, the whole product rebuilt from the Specification by a fresh builder,
and the frozen acceptance suite run against the rebuilt binaries behind the
unchanged corpus.

Without this trial the project cannot tell:

- whether the declared Specification is enough to rebuild the whole product,
  not one component behind kept consumers;
- which files outside the declared Specification a fresh builder must
  reconstruct to make the product build and run;
- whether the frozen suite can grade a whole rebuilt product, and what it
  cannot see;
- which defects belong to the Specification, the E2E suite, the rebuild, or to
  no artifact at all.

`S13` (Givn self-rebuild) depends on this trial's record, its gap table, and its
neighbour table.

## Proposed Solution

- **Declared strategy.** The design records `whole-product rebuild` with `Ise
  Test` mode `Dry-Run Ise`, the affected consumers, no persisted state, and the
  rebuild boundary: every path under the declared `source:` roots
  (`crates/passuff-cli`, `crates/passuff-core`, `crates/passuff-config`,
  `scripts/**`).
- **Spec copy rule.** `givn ise rebuild-all` prepares a copy at the recorded
  trial revision. The copy carries the declared Specification
  (`givn/specification.yaml`, `givn/specs/**`, `docs/**`, `README.md`) and
  nothing from the source roots. The frozen `crates/passuff-e2e` package stays
  outside the copy and is the acceptance authority.
- **Fresh builder.** One builder that has not read the checkout authors the
  whole product in the copy: both production crates, the CLI crate, the
  component step harness and fakes, the scripts, the bundled sound asset, and
  the build root the copy withholds. It records `builder-notes.md` with every
  file it authored, every crate version it chose, every assumption, and the
  clean-room statement.
- **Frozen run.** The kept product is built in the copy and `./scripts/verify.sh
  --e2e --package crates/passuff-e2e` runs the frozen suite from the checkout
  against the rebuilt binaries.
- **Gap rule.** Every difference between the rebuilt product and the declared
  Specification is classified with exactly one outcome: `E2E test defect`,
  `implementation defect`, `specification defect`, or `acceptable undocumented
  change`, and the wrong artifact is named. A missing contract stops the
  affected reconstruction and is recorded as a `specification defect`; the
  rebuild does not invent the contract.
- **Neighbour record.** The review names every file the trial added outside the
  declared Specification, with the reason. This is one review question, not a
  gate.
- **No promotion.** The rebuilt product stays in the trial copy. The checkout
  keeps its original source and declaration.

## Capability Routing

`givn spec route --change full-product-dry-run` reported no signal: the top
candidates tie at 12.00 and no ranked list is printed.

| Proposed capability | Decision | Rationale |
|---|---|---|
| none | No delta specs | The change records a trial, not a capability change. `givn/specs/**` and the frozen package are the Specification under test and stay frozen. `.givn-skip` carries `specs`. |

## Out of Scope

- Changing product behavior, the keymap, the commands, the documented exit
  codes, or any `givn/specs/**` scenario.
- Editing the frozen `crates/passuff-e2e` package or weakening any scenario.
- Promoting the rebuilt product into the checkout.
- Fixing any discovered `Specification defect`; the trial records each gap with
  one outcome and names a later corrective change.
- Editing `givn/specification.yaml`; the trial records the declaration's
  coverage as a finding, not a fix.

## Open Questions

### D1 — Which revision does the trial read?

- Question: Which commit is recorded as the trial revision and read by
  `givn ise rebuild-all --at <revision>`?
- Options:
  1. **The pre-trial commit** — the commit holding the proposal, the design,
     the design review, the task list, and `.givn-skip`; gain: the copy reads
     the artifacts that describe the trial; cost: the evidence is committed
     after the revision it describes; in practice: `--at <pre-trial-sha>`.
  2. **The evidence commit** — gain: one revision holds artifacts and result;
     cost: the copy would read evidence it is supposed to produce; in
     practice: commit evidence first, then prepare.
- Disposition: answered by `givn instructions ise` and `S11`: the pre-trial
  commit.

### D2 — What counts as the trial result?

- Question: Which observations decide whether the whole-product trial passed?
- Options:
  1. **A runnable rebuilt binary plus the frozen suite result** — gain: the
     trial proves the product can be built and graded, and every difference is
     classified; cost: the frozen suite may be red, so the record is a
     classified gap table rather than a green run; in practice: both binaries
     present, `--help` and `--version` exit 0, then
     `./scripts/verify.sh --e2e --package crates/passuff-e2e` with
     `PASSUFF_BIN_DIR` set to the copy's `target/debug`.
  2. **The frozen suite alone** — gain: one number; cost: a suite that cannot
     start distinguishes nothing and hides the build and classify work; in
     practice: green 29/29 or nothing.
- Disposition: answered by the plan's exit criteria: a runnable binary with
  every difference classified; the frozen suite is the grading authority.

### D3 — Which artifact is wrong when the rebuild contradicts the corpus?

- Question: How does the trial classify a difference between the rebuilt
  product and the retained Specification?
- Options:
  1. **Exactly one outcome per difference, naming the wrong artifact** — gain:
     the record says whether the corpus, the frozen test, the rebuild, or no
     artifact is wrong; cost: each difference needs a judgment with evidence;
     in practice: `E2E test defect` names the scenario, `specification defect`
     names the document and the missing rule, `implementation defect` names
     the rebuilt file, `acceptable undocumented change` names what the
     Specification leaves free.
  2. **One blanket outcome for the trial** — gain: less writing; cost: the
     record cannot route a corrective change; in practice: "the trial failed".
- Disposition: answered by `givn instructions ise`: option 1.

### D4 — What stops when a required contract is missing?

- Question: When the builder cannot derive a required contract from the
  Specification, does the whole trial stop or only that part?
- Options:
  1. **Stop the affected reconstruction, record one `specification defect`,
     continue the rest** — gain: the trial still produces a runnable binary and
     can classify the remaining differences; cost: the runnable binary is
     incomplete by construction; in practice: record the gap against
     `docs/reference.md` or `docs/limits.md`, leave the affected file absent,
     and keep rebuilding the independent parts.
  2. **Stop the whole rebuild on the first gap** — gain: no invention at all;
     cost: no binary, no frozen result, and no evidence about the other source
     roots; in practice: the trial ends at the first script contract.
  3. **Invent the missing rule** — forbidden by the ise instruction.
- Disposition: answered by the plan's exit criteria (a runnable binary with
  every difference classified) and `S11` D4 (no invention): option 1.

### D5 — Which paths count as neighbours?

- Question: Which additions does the neighbour table record when the replaced
  unit is every declared source root?
- Options:
  1. **Every added path outside the declared Specification, with the reason**
     — gain: the record shows the build-root and non-declared files the trial
     had to author; cost: a long table that includes the rebuilt product
     roots themselves; in practice: the build root (`Cargo.toml`,
     `Cargo.lock`, `rust-toolchain.toml`) plus any file outside a declared
     `source:` root, and per-root file counts for the rebuilt source.
  2. **Only the build root** — gain: short; cost: hides non-declared files the
     trial authored; in practice: a table of three rows.
- Disposition: answered by `S11` D5 and the task's neighbour rule: option 1.
