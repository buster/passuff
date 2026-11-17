# Tasks: full-product-dry-run

Behavior change: none. `specs` is skipped via `.givn-skip`. The change records
the `S12` whole-product `Dry-Run Ise` trial. No product behavior, scenario,
declaration entry, or frozen test changes.

## Setup

Design constraints: `Decides D1` to `D8`; the trial revision is the pre-trial
commit (`Decides D1`); the result is a runnable binary plus the frozen suite
(`Decides D2`); one outcome per difference with the wrong artifact (`Decides
D3`); a missing contract stops the affected reconstruction only (`Decides D4`);
neighbours are every added path outside the Specification (`Decides D5`).

- [x] Baseline recorded: `./scripts/verify.sh` exits 0 with 215 scenarios and
      1731 steps, `./scripts/verify.sh --e2e --package crates/passuff-e2e`
      exits 0 with 29 scenarios and 240 steps, and both result files reconcile
      four counts.
- [x] Strict mode confirmed: `crates/passuff-e2e/tests/e2e_runner.rs` and
      `crates/passuff-cli/tests/features_runner.rs` call `.fail_on_skipped()`;
      the frozen runner counts parsing errors as failed and exits non-zero on
      a zero total.
- [x] No delta specs: the proposal records `Behavior change: none`, `.givn-skip`
      lists `specs`, and `givn check design-review --change
      full-product-dry-run` passes.
- [ ] Pre-trial artifacts committed: proposal, design, `arc42.md`, design
      review, this task list, and `.givn-skip`; the commit is the trial
      revision.
- [ ] COMMIT: `<setup>` -- Batch: setup

## Batch: trial

Design constraints: the copy removes every path under the four declared
`source:` roots and keeps the 21 Specification files; the builder sees only the
copy and runs every cargo command with `--offline`; the frozen run uses the
checkout package with `PASSUFF_BIN_DIR` set to the copy's `target/debug`; the
gap table classifies every difference with one outcome and names the wrong
artifact; the neighbour record names every added path outside the
Specification.

- [ ] `givn ise rebuild-all --at <trial-sha> --into /tmp/passuff-s12-full
      --manifest /tmp/passuff-s12-full.yaml` writes the copy and the manifest;
      the report records the removed roots, the retained Specification count,
      both checksums, and the end-to-end set size. `prepare.txt` and
      `manifest.yaml` are saved under `evidence/ise/`.
- [ ] `givn ise compare --manifest /tmp/passuff-s12-full.yaml --copy
      /tmp/passuff-s12-full` reports `Specification files: match` and
      `End-to-end tests: match` before the builder.
- [ ] The published `passuff*` crate sources are quarantined to
      `/tmp/passuff-s12-registry-quarantine` and both scans are recorded in
      `sandbox.txt`.
- [ ] One fresh builder rebuilds the whole product in the copy from the copy's
      `givn/specification.yaml`, `givn/specs/**`, `docs/**`, and `README.md`:
      the workspace root, the three crates, the component step harness and
      support, the scripts, and the bundled sound. It writes
      `builder-notes.md` with the inputs, every assumption, every authored
      file, every crate version, every attempt, the clean-room statement, and
      the registry scan; `authored-files.txt` groups the files by source root.
- [ ] The rebuilt product builds offline in the copy:
      `cargo build --offline -p passuff --bins` then
      `cargo build --offline --locked -p passuff --bins`; the hashes and the
      `--help`, `--version`, and no-terminal `passuff-tui` invocations are
      recorded in `binaries.txt`.
- [ ] The rebuilt component check runs in the copy:
      `GIVN_FEATURES=/tmp/passuff-s12-full/givn/specs GIVN_RESULT_SCOPE=change
      GIVN_RESULT_FILE=<change>/evidence/ise/component-result.json cargo test
      --offline --locked -p passuff --features test-support --test
      features_runner`; the counts are recorded, including a red result.
- [ ] The frozen acceptance run runs against the rebuilt binaries:
      `PASSUFF_BIN_DIR=/tmp/passuff-s12-full/target/debug
      GIVN_RESULT_FILE=<change>/evidence/ise/e2e-result.json
      ./scripts/verify.sh --e2e --package crates/passuff-e2e`; the four counts
      are recorded, including a red result.
- [ ] `givn ise compare` after the build still reports both subjects `match`.
- [ ] `evidence/ise/gap-table.md` holds one row per difference with exactly one
      outcome (`E2E test defect`, `implementation defect`, `specification
      defect`, or `acceptable undocumented change`), the wrong artifact, and
      the corrective route.
- [ ] `evidence/ise/neighbour-record.md` names every added path outside the
      declared Specification with the reason, and the rebuilt file count per
      source root.
- [ ] The quarantined registry sources are restored and the post-restore scan
      is recorded in `sandbox.txt`.
- [ ] COMMIT: `<trial>` -- Batch: trial

## Closeout

- [ ] `givn check tasks --change full-product-dry-run` passes; the change
      verification is not applicable.
- [ ] `./scripts/verify.sh` and `./scripts/verify.sh --e2e --package
      crates/passuff-e2e` are green at the reviewed tree; `./measure-coverage.sh`
      and `./merge-coverages.sh` write the regular and E2E reports.
- [ ] `review.md` records the change strategy, the gap table with each wrong
      artifact, the neighbour table, the rebuilt binary hashes, the frozen
      result, the rebuilt component result, and the environment restore.
- [ ] `givn check review --change full-product-dry-run` passes.
- [ ] `givn archive --change full-product-dry-run` passes the archive gate and
      moves the change directory.
