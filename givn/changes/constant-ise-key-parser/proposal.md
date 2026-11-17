# Proposal: constant-ise-key-parser

## Release note

Reconstruct the passuff terminal key parser from the Specification in a
clean-room copy and prove it with the frozen acceptance suite, without changing
product behavior.

## Behavior Change

Behavior change: none

The change records the S11 component-level `Constant Ise` trial. It adds no
product behavior, changes no scenario, and promotes no rebuilt file.

## Use-Case Context

- Use-case ID: none, the change adds no capability and edits no use case.
- Ideation topic: `ise-rebuildability`
- Seed path: none, the work is phase `S11` of
  `givn/ideation/ise-rebuildability/plan.md`, not an ideation change seed.
- Confirmed Personas: `multi-account-developer`, the key parser is the
  terminal interface that person uses; the change adds no new interaction.

## Review Context

- Review kind: self-review
- Reason: the change adds no product behavior, public contract, persisted
  state, screen, or dependency. It records a clean-room trial whose facts are
  produced by `givn ise rebuild-component`, `givn ise compare`, the builder
  notes, and the frozen suite. An independent context would read the same
  commands and the same result file; it would not add a distinct judgment.

## Problem / Opportunity

`S11` requires a component-level `Constant Ise` trial on a real product. The
repository has no record of `givn ise rebuild-component` against a passuff
component, no evidence that the frozen suite can grade a rebuilt parser, and no
neighbour record of the files that must change outside the replaced unit to
build the kept product.

Without this trial the project cannot tell:

- whether a one-component rebuild transfers to a real product;
- whether the frozen suite catches a rebuilt key parser behind its unchanged
  consumers;
- which files outside the component must be recreated because the declaration
  does not retain them;
- whether the Specification states enough for the rebuild to avoid inventing a
  contract.

`S12` (whole-product `Dry-Run Ise`) depends on this trial's record, its
neighbour table, and any `Spec gap` it exposes.

## Proposed Solution

- **Declared strategy.** The design records `Constant Ise` with `Ise Test`
  mode `Dry-Run Ise`, the affected consumers, no persisted state, and the
  rebuild boundary: `crates/passuff-cli/src/tui/keys.rs` only.
- **Spec copy.** `givn ise rebuild-component` prepares a copy at the recorded
  trial revision. The copy removes the key parser and keeps the rest of the
  product, the Specification corpus, and the declared documents. The frozen
  `e2e` package stays outside the copy and is the acceptance authority.
- **Fresh builder.** One builder that has not read the checkout rebuilds the
  parser in the copy from the Specification and the unchanged consumers. It
  records the files it read and every contract it had to assume.
- **Frozen run.** The kept product is built in the copy and the frozen `e2e`
  suite runs against the rebuilt binary. The component suite runs in the copy
  as a contract check against the unchanged consumer.
- **Neighbour record.** The review names every path changed outside the
  replaced unit and the reason. This is one review question, not a gate.
- **Gap rule.** A parser contract absent from the Specification and the kept
  consumers is one `Spec gap: specification defect` against the named
  document. The trial stops the rebuild attempt and does not invent the
  contract. The gap is recorded, not fixed, in this change.
- **No promotion.** The rebuilt parser stays in the trial copy. The product
  checkout keeps its original parser.

## Capability Routing

`givn spec route --change constant-ise-key-parser` reported no signal: the top
candidates tie at 12.00 and no ranked list is printed.

| Proposed capability | Decision | Rationale |
|---|---|---|
| none | No delta specs | The change records a trial, not a capability change. `givn/specs/**` and the frozen package are the Specification under test and stay frozen. `.givn-skip` carries `specs`. |

## Out of Scope

- Changing product behavior, the keymap, the documented `--keys` vocabulary,
  or any `givn/specs/**` scenario.
- Editing the frozen `crates/passuff-e2e` package.
- Promoting the rebuilt parser into the checkout.
- The whole-product trial (`S12`) and any corrective change for a discovered
  gap.
- Fixing the build-root omission in `givn/specification.yaml`; the trial
  records it as a neighbour change.

## Open Questions

### D1: Which paths form the replaced unit?

- Question: Does the trial replace only `crates/passuff-cli/src/tui/keys.rs`,
  or also the component tests and fixtures that exercise the parser?
- Options:
  1. **The parser file only**; gain: the trial tests exactly the component
     boundary the plan names, and every test harness stays as an unchanged
     consumer; cost: the parser must satisfy the harness without edits. In
     practice: `--path crates/passuff-cli/src/tui/keys.rs`.
  2. **The parser and the test harness**; gain: the builder may also rewrite
     the tests; cost: the trial no longer proves the parser against unchanged
     consumers, and the frozen suite does not read those tests. In practice:
     add `crates/passuff-cli/tests/**`.
- Disposition: answered by `S11` and `Decides D1`: the parser file only. The
  plan allows parser unit tests outside that file; none exist.

### D2: Which revision does the trial read?

- Question: Which commit is the trial revision recorded in the manifest?
- Options:
  1. **The pre-trial commit**; gain: the design, tasks, and skip marker are
     in the revision the copy reads; cost: the evidence is committed after the
     revision it describes. In practice: `--at <pre-trial-sha>`.
  2. **The evidence commit**; gain: one revision holds the artifacts and the
     result; cost: the copy would read evidence it is supposed to produce.
     In practice: commit evidence first, then prepare.
- Disposition: answered by `Decides D2`: the pre-trial commit.

### D3: How is a green trial measured?

- Question: Which result counts as the trial's acceptance result?
- Options:
  1. **The frozen `e2e` suite**; gain: the declared acceptance authority
     grades the rebuilt binary and reports four reconciling counts; cost: the
     suite covers 29 scenarios, not the whole component corpus (TD-007). In
     practice: `PASSUFF_BIN_DIR=<copy>/target/debug ./scripts/verify.sh --e2e
     --package crates/passuff-e2e`.
  2. **The component suite**; gain: more scenarios; cost: it runs in process
     and is not the declared acceptance authority. In practice: a supporting
     check, not the trial result.
- Disposition: answered by `Decides D3`: the frozen suite is the result, and
  the component suite is a supporting contract check.

### D4: What happens when the Specification lacks a required contract?

- Question: When the builder cannot reconstruct the parser's consumer
  contract from the Specification and the kept consumers, what stops?
- Options:
  1. **Record one `Specification defect` and stop the rebuild**; gain: the
     trial exposes the gap instead of hiding it; cost: no rebuilt binary and no
     frozen result for that attempt. In practice: name `docs/reference.md`,
     record the missing rule, and archive the trial record.
  2. **Let the builder invent the missing rule**; gain: the trial still
     produces a binary; cost: the trial grades the builder's invention, not the
     Specification. In practice: forbidden by `S11`.
- Disposition: answered by `Decides D4`: option 1.

### D5: Which paths count as neighbours?

- Question: Which changes outside the replaced unit must the neighbour record
  name?
- Options:
  1. **Every changed path outside `keys.rs`**, including build scaffolding
     the copy omitted; gain: the record shows the real cost of a component
     rebuild; cost: the table includes files that are not product components.
     In practice: root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and
     any kept file the builder edits.
  2. **Only product source files**; gain: a short table; cost: the trial hides
     the build-root reconstruction that the declaration forces. In practice:
     the root workspace files are outside every declared role.
- Disposition: answered by `Decides D5`: option 1.
