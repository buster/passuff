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
- [x] Pre-trial artifacts committed: proposal, design, `arc42.md`, design
      review, this task list, and `.givn-skip`; the commit is the trial
      revision.
- [x] COMMIT: `6c5d223` -- Batch: setup

## Batch: trial

Design constraints: the copy removes every path under the four declared
`source:` roots and keeps the 21 Specification files; the builder sees only the
copy and runs every cargo command with `--offline`; the frozen run uses the
checkout package with `PASSUFF_BIN_DIR` set to the copy's `target/debug`; the
gap table classifies every difference with one outcome and names the wrong
artifact; the neighbour record names every added path outside the
Specification.

- [x] `givn ise rebuild-all --at d63cdc84d89ac729d4460b63da181d180e209d0b
      --into /tmp/passuff-s12-full --manifest /tmp/passuff-s12-full.yaml`
      writes the copy and the manifest: 21 Specification files retained, 0 source
      files kept, 62 removed, 12 e2e files not copied, both checksums.
      `prepare.txt` and `manifest.yaml` are saved under `evidence/ise/`.
- [x] `givn ise compare --manifest /tmp/passuff-s12-full.yaml --copy
      /tmp/passuff-s12-full` reports `Specification files: match` and
      `End-to-end tests: match` before the builder.
- [x] The published `passuff*` crate sources are absent from the Cargo registry
      and both scans are recorded in `sandbox.txt`. The `S11` trial's
      quarantine at `/tmp/passuff-s11-registry-quarantine` holds them; `S12`
      moved nothing and its own quarantine directory is empty.
- [x] One fresh builder rebuilt the whole product in the copy from the copy's
      `givn/specification.yaml`, `givn/specs/**`, `docs/**`, and `README.md`:
      the workspace root, the three crates, the component step harness and
      support, `scripts/verify.sh`, and the bundled sound. It wrote
      `builder-notes.md` with the inputs, every assumption, every authored
      file, every crate version, every attempt, the clean-room statement, and
      the gap list; `authored-files.txt` groups the files by source root.
- [x] The rebuilt product builds offline in the copy:
      `cargo build --offline -p passuff --bins` and
      `cargo build --offline --locked -p passuff --bins` both succeed; the
      hashes and the `--help`, `--version`, and no-terminal `passuff-tui`
      invocations are recorded in `binaries.txt`.
- [x] The rebuilt component check recorded both surfaces: the direct
      `GIVN_FEATURES=/tmp/passuff-s12-full/givn/specs` invocation selected 0
      scenarios (`component-result.json`, `component-run.txt`); the copy's
      declared `./scripts/verify.sh` selected 215 and passed 215
      (`component-run-declared-surface.txt`). The difference is gap G2.
- [x] The frozen acceptance run ran against the rebuilt binaries:
      `PASSUFF_BIN_DIR=/tmp/passuff-s12-full/target/debug
      GIVN_RESULT_FILE=<change>/evidence/ise/e2e-result.json
      ./scripts/verify.sh --e2e --package crates/passuff-e2e` reported 29
      scenarios, 28 passed, 1 failed, 0 skipped, 237 steps; the failure is the
      activity detail overlay body (`e2e-run.txt`).
- [x] `givn ise compare` after the build still reports both subjects `match`.
- [x] `evidence/ise/gap-table.md` holds one row per difference with exactly one
      outcome, the wrong artifact, and the route: 1 `implementation defect`, 4
      `specification defect`, 23 `acceptable undocumented change`, no `E2E test
      defect`.
- [x] `evidence/ise/neighbour-record.md` names the three added build-root files
      with the reason, the 5 invented paths inside a source root, the 17
      product files not recreated, the 23 missing script paths, and the rebuilt
      file count per source root.
- [x] The registry scans before and after the run are both empty; `S12` made no
      registry change, so the environment needs no restore, and `sandbox.txt`
      records that.
- [x] COMMIT: `525bb7a` -- Batch: trial

## Closeout

- [x] `givn check tasks --change full-product-dry-run` passes; the change
      verification is not applicable.
- [x] `./scripts/verify.sh` and `./scripts/verify.sh --e2e --package
      crates/passuff-e2e` are green at the reviewed tree; `./measure-coverage.sh`
      and `./merge-coverages.sh` write the regular and E2E reports.
- [x] `review.md` records the change strategy, the gap table with each wrong
      artifact, the neighbour table, the rebuilt binary hashes, the frozen
      result, the rebuilt component result, and the environment restore.
- [x] `givn check review --change full-product-dry-run` passes.
- [x] `givn archive --change full-product-dry-run` passes the archive gate and
      moves the change directory.
