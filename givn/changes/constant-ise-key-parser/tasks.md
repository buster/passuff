# Tasks: constant-ise-key-parser

Behavior change: none. `specs` is skipped via `.givn-skip`. The change records
the `S11` component-level `Constant Ise` trial. No product behavior, scenario,
declaration entry, or frozen test changes.

## Setup

Design constraints: `Decides D1` to `D6`; the trial revision is the pre-trial
commit (`Decides D2`); the frozen suite is the acceptance authority (`Decides
D3`); a missing contract stops the rebuild (`Decides D4`); neighbours are every
changed path outside `keys.rs` (`Decides D5`).

- [ ] Baseline recorded: `./scripts/verify.sh --e2e --package
      crates/passuff-e2e` exits 0 with 29 scenarios and 240 steps, and the
      result file holds `{"failed":0,"passed":29,"skipped":0,"total":29}`.
- [ ] Strict mode confirmed: `crates/passuff-e2e/tests/e2e_runner.rs` and
      `crates/passuff-cli/tests/features_runner.rs` call `.fail_on_skipped()`;
      the frozen runner counts parsing errors as failed and exits non-zero on
      a zero total.
- [ ] No delta specs: the proposal records `Behavior change: none`, `.givn-skip`
      lists `specs`, and `givn check design-review --change
      constant-ise-key-parser` passes.
- [ ] Pre-trial artifacts committed: proposal, design, design review,
      `arc42.md`, this task list, and `.givn-skip`; the commit is the trial
      revision.
- [ ] COMMIT: <setup> -- Batch: setup

## Batch: trial

Design constraints: the copy removes exactly
`crates/passuff-cli/src/tui/keys.rs`; the builder sees only the copy and the
Specification; the frozen run uses the checkout package with
`PASSUFF_BIN_DIR` set to the copy's `target/debug`; the neighbour record names
every changed path outside `keys.rs`.

- [ ] `givn ise rebuild-component crates/passuff-cli/src/tui/keys.rs --at
      <trial-revision> --into /tmp/passuff-s11-keys --manifest
      /tmp/passuff-s11-keys.yaml` writes the copy and the manifest; the report
      records one removed path, both checksums, and the end-to-end set size.
- [ ] `givn ise compare --manifest /tmp/passuff-s11-keys.yaml --copy
      /tmp/passuff-s11-keys` reports `Specification files: match` and
      `End-to-end tests: match` before the builder.
- [ ] One fresh builder rebuilds `crates/passuff-cli/src/tui/keys.rs` in the
      copy from the copy's `givn/specification.yaml`, `givn/specs/**`,
      `docs/**`, and the kept consumers; it writes `builder-notes.md` with the
      inputs, every assumption, every authored or edited file, every attempt,
      the clean-room statement, and the registry scan.
- [ ] The kept product builds in the copy:
      `cargo build --locked -p passuff --bins`.
- [ ] The component contract check runs in the copy:
      `GIVN_FEATURES=<copy>/givn/specs GIVN_RESULT_SCOPE=change
      GIVN_RESULT_FILE=<change>/evidence/ise/component-result.json cargo test
      --locked -p passuff --features test-support --test features_runner`; the
      counts are recorded.
- [ ] The frozen acceptance run runs against the rebuilt binary:
      `PASSUFF_BIN_DIR=<copy>/target/debug GIVN_RESULT_FILE=<change>/evidence/ise/e2e-result.json
      ./scripts/verify.sh --e2e --package crates/passuff-e2e`; it exits 0 with
      29 scenarios and 240 steps.
- [ ] `givn ise compare` after the build still reports both subjects `match`.
- [ ] `evidence/ise/neighbour-record.md` names every path changed outside
      `crates/passuff-cli/src/tui/keys.rs` and the reason (the build root, the
      lockfile, the toolchain file, any kept file).
- [ ] The trial evidence is saved under
      `givn/changes/constant-ise-key-parser/evidence/ise/`: the manifest, both
      compare outputs, the builder notes, the environment, the build log, the
      component result, the frozen log and result, and the neighbour record.
- [ ] No `Specification defect` occurred. If one occurred, the trial stops the
      rebuild attempt and `evidence/ise/spec-gap.md` names exactly one outcome
      and `docs/reference.md`; no rule is invented.
- [ ] COMMIT: <trial> -- Batch: trial

## Closeout

- [ ] `givn check tasks --change constant-ise-key-parser` passes; the change
      verification is not applicable.
- [ ] `./scripts/verify.sh` and `./scripts/verify.sh --e2e --package
      crates/passuff-e2e` are green at the reviewed tree; `./measure-coverage.sh`
      and `./merge-coverages.sh` write the regular and E2E reports.
- [ ] `review.md` records the neighbour table, the frozen result, the component
      result, the `Spec gap` outcome (or none), and the change strategy.
- [ ] `givn check review --change constant-ise-key-parser` passes.
- [ ] `givn archive --change constant-ise-key-parser` passes the archive gate
      and moves the change directory.
