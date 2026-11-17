# Tasks: constant-ise-key-parser

Behavior change: none. `specs` is skipped via `.givn-skip`. The change records
the `S11` component-level `Constant Ise` trial. No product behavior, scenario,
declaration entry, or frozen test changes.

## Setup

Design constraints: `Decides D1` to `D6`; the trial revision is the pre-trial
commit (`Decides D2`); the frozen suite is the acceptance authority (`Decides
D3`); a missing contract stops the rebuild (`Decides D4`); neighbours are every
changed path outside `keys.rs` (`Decides D5`).

- [x] Baseline recorded: `./scripts/verify.sh --e2e --package
      crates/passuff-e2e` exits 0 with 29 scenarios and 240 steps, and the
      result file holds `{"failed":0,"passed":29,"skipped":0,"total":29}`.
- [x] Strict mode confirmed: `crates/passuff-e2e/tests/e2e_runner.rs` and
      `crates/passuff-cli/tests/features_runner.rs` call `.fail_on_skipped()`;
      the frozen runner counts parsing errors as failed and exits non-zero on
      a zero total.
- [x] No delta specs: the proposal records `Behavior change: none`, `.givn-skip`
      lists `specs`, and `givn check design-review --change
      constant-ise-key-parser` passes.
- [x] Pre-trial artifacts committed: proposal, design, design review,
      `arc42.md`, this task list, and `.givn-skip`; the commit is the trial
      revision.
- [x] COMMIT: `b23d060` -- Batch: setup

## Batch: trial

Design constraints: the copy removes exactly
`crates/passuff-cli/src/tui/keys.rs`; the builder sees only the copy and the
Specification; the frozen run uses the checkout package with
`PASSUFF_BIN_DIR` set to the copy's `target/debug`; the neighbour record names
every changed path outside `keys.rs`.

- [x] `givn ise rebuild-component crates/passuff-cli/src/tui/keys.rs --at
      b23d060d1c8a408d695b0e526414471bcad4b6a9 --into /tmp/passuff-s11-keys
      --manifest /tmp/passuff-s11-keys.yaml` writes the copy and the manifest;
      the report records one removed path, both checksums, and the end-to-end
      set size (12 files).
- [x] `givn ise compare --manifest /tmp/passuff-s11-keys.yaml --copy
      /tmp/passuff-s11-keys` reports `Specification files: match` and
      `End-to-end tests: match` before the builder.
- [x] One fresh builder rebuilds `crates/passuff-cli/src/tui/keys.rs` in the
      copy from the copy's `givn/specification.yaml`, `givn/specs/**`,
      `docs/**`, and the kept consumers; it writes `builder-notes.md` with the
      inputs, every assumption, every authored or edited file, every attempt,
      the clean-room statement, and the registry scan.
- [x] The kept product builds in the copy:
      `cargo build --locked -p passuff --bins`.
- [x] The component contract check runs in the copy:
      `GIVN_FEATURES=/tmp/passuff-s11-keys/givn/specs GIVN_RESULT_SCOPE=change
      GIVN_RESULT_FILE=<change>/evidence/ise/component-result.json cargo test
      --locked -p passuff --features test-support --test features_runner`; it
      passes 215 scenarios and 1731 steps.
- [x] The frozen acceptance run runs against the rebuilt binary:
      `PASSUFF_BIN_DIR=/tmp/passuff-s11-keys/target/debug
      GIVN_RESULT_FILE=<change>/evidence/ise/e2e-result.json
      ./scripts/verify.sh --e2e --package crates/passuff-e2e`; it exits 0 with
      29 scenarios and 240 steps.
- [x] `givn ise compare` after the build still reports both subjects `match`.
- [x] `evidence/ise/neighbour-record.md` names every path changed outside
      `crates/passuff-cli/src/tui/keys.rs` and the reason: the reconstructed
      build root (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`), with no
      kept product source change.
- [x] The trial evidence is saved under
      `givn/changes/constant-ise-key-parser/evidence/ise/`: the manifest, both
      compare outputs, the builder notes, the environment, the build log, the
      component result, the frozen log and result, the binary hashes, the
      registry quarantine, the rebuilt-original diff, and the neighbour record.
- [x] No `Specification defect` occurred. The builder found every parser rule
      in `docs/reference.md`, the keymap table, `tui/mod.rs::map_key`, and the
      exhaustive `Key` matches; it recorded its build-root assumptions.
- [x] COMMIT: `9c99766` -- Batch: trial

## Closeout

- [x] `givn check tasks --change constant-ise-key-parser` passes; the change
      verification is not applicable.
- [x] `./scripts/verify.sh` and `./scripts/verify.sh --e2e --package
      crates/passuff-e2e` are green at the reviewed tree; `./measure-coverage.sh`
      and `./merge-coverages.sh` write the regular and E2E reports.
- [x] `review.md` records the neighbour table, the frozen result, the component
      result, the `Spec gap` outcome (none), and the change strategy.
- [x] `givn check review --change constant-ise-key-parser` passes.
- [x] `givn archive --change constant-ise-key-parser` passes the archive gate
      and moves the change directory.
