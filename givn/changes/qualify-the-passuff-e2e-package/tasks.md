# Tasks: qualify-the-passuff-e2e-package

Behavior change: none. `specs` is skipped via `.givn-skip`; the frozen
acceptance suite and every scenario stay unchanged. The deliverables are one
declared limit document, one declaration edit, two seeded-defect patches with
their expectation table, and the `S10` evidence.

## Setup

- [x] Inherited runner confirmed: `./scripts/verify.sh` is the component scope
      (`change`), `./scripts/verify.sh --e2e` runs the frozen
      `crates/passuff-e2e` package, and both honour `GIVN_RESULT_FILE` and
      `GIVN_RESULT_SCOPE`.
- [x] Strict mode confirmed: `crates/passuff-e2e/tests/e2e_runner.rs` calls
      `.fail_on_skipped()`, counts parse errors as failed, and exits non-zero
      on a zero total.
- [x] Baseline recorded: `./scripts/verify.sh --e2e` exits 0 with 29 scenarios
      and 240 steps; `target/givn-verify-result.json` holds
      `{"failed":0,"passed":29,"skipped":0,"total":29}`.
- [x] No new runner, step file, or fixture: the change adds test assets to the
      existing qualification script only.

## Batch: declared-limits

Design constraints: `docs/limits.md` owns the terminal budgets, the product
exit codes, and the runner result-file contract; `givn/specification.yaml`
declares it and keeps `docs/arc42/02-architecture-constraints.md` (design
`Decides D1`); chapters 2, 8, and 11 are the only affected arc42 chapters.

- [x] `docs/limits.md` states the 40x12 minimum, the 80x24 and 120x40 budgets
      with what fits at each, the clipping and paging rule, the four product
      exit codes with their surfaces, and the runner result-file contract
      (fresh file, four reconciling counts, `total > 0`, zero failed, zero
      skipped, optional scope).
- [x] `givn/specification.yaml` `limits` lists `docs/limits.md` and
      `docs/arc42/02-architecture-constraints.md`; every other role is
      unchanged.
- [x] `docs/arc42/02-architecture-constraints.md` gains `Declared limits`;
      `docs/arc42/08-crosscutting-concepts.md` names both scopes and the
      twenty-defect qualification; `docs/arc42/11-risks-and-technical-debt.md`
      updates R-019 and adds TD-007.
- [x] COMMIT: `96d2be0` -- Batch: declared-limits

## Batch: seeded-defects

Design constraints: `scripts/qualify/key-parser-unbinds-quit.patch` patches
`crates/passuff-cli/src/tui/mod.rs` and must fail `The inbox row is visible at
the narrow terminal size`; `scripts/qualify/inbox-paging-skips-selected-page.patch`
patches `crates/passuff-cli/src/tui/render.rs` and must fail `The activity
detail opens at the wide terminal size and quits cleanly`. The script pins the
named scenario for all twenty defects and continues after a mismatch.

- [x] The key-parser patch maps `'q'` to `Key::Char('q')` instead of
      `Key::Quit`, so the pty session never exits; the rest of the key map is
      unchanged.
- [x] The paging patch replaces the selected-row window start with
      `activity_selection + rows_available`, so the only activity row is
      skipped at 120x40.
- [x] `scripts/qualify-e2e.sh` lists twenty defects, pins one named scenario
      per defect, checks containment in the failing scenario set, and keeps the
      clean-tree hash check, the revert check, and the summary line.
- [x] `./scripts/qualify-e2e.sh` reports `20/20 defects detected; clean tree
      green` and every named scenario matches.
- [x] COMMIT: `38b09ce` -- Batch: seeded-defects

## Batch: s10-evidence

Design constraints: the evidence lives under
`givn/changes/qualify-the-passuff-e2e-package/evidence/`; the transcripts come
from the package's own pty and `vt100` harness in a throwaway copy; the gate
record names every command and observed value.

- [ ] `evidence/cargo-tree-passuff-e2e.txt` shows the package graph and no
      `passuff`, `passuff-core`, or `passuff-config` line.
- [ ] `evidence/qualify-20-of-20.txt` holds the qualification output.
- [ ] `evidence/verify-e2e-clean.txt` holds the 29/29, 240/240 pass and the
      written result file; `evidence/verify-e2e-result.json` holds the four
      counts.
- [ ] `evidence/defect-narrow-size.txt`, `evidence/defect-key-parser.txt`, and
      `evidence/defect-inbox-paging.txt` hold the failing scenario block for
      the 80x24 layout, key-parser, and inbox-paging defects.
- [ ] `evidence/transcripts/80x24.txt` and `evidence/transcripts/120x40.txt`
      hold one parsed screen each from a real pty.
- [ ] `evidence/s10-gate.md` records each `G9` row with its command and
      observed value.
- [ ] COMMIT: <pending> -- Batch: s10-evidence

## Closeout

- [ ] `./scripts/verify.sh` exits 0 for the component scope.
- [ ] `./scripts/verify.sh --e2e` exits 0 with 29 scenarios and writes
      `GIVN_RESULT_FILE`.
- [ ] `cargo fmt --all -- --check` and the package format check are clean.
- [ ] `givn check tasks --change qualify-the-passuff-e2e-package` reports the
      change verification as not applicable and leaves a clean tree.
- [ ] `givn check review --change qualify-the-passuff-e2e-package` passes and
      the receipt is committed.
- [ ] `givn archive --change qualify-the-passuff-e2e-package` passes the
      archive gate and moves the change directory.
