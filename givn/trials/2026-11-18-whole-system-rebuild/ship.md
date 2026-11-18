# Ship record, 2026-11-18

Ise rebuild decision: `Whole-system rebuild`.
Rebuilt code: `ship`.
Trial revision: `372cd83ac33f05d11b108476b4d18bbc4a4e1ff3`.
Ship branch: `ise/whole-system-ship`.
Source copy: `/tmp/passuff-ise-whole`.

This branch adopts the verified replacement that the trial at
`ise/whole-system-rebuild` kept outside the project. The trial report remains
the record of the copy and its result.

## Replaced

| Path | Change |
|---|---|
| `crates/passuff-cli` | Replaced by the rebuilt crate |
| `crates/passuff-core` | Replaced by the rebuilt crate |
| `crates/passuff-config` | Replaced by the rebuilt crate |
| `scripts/**` | Replaced by `scripts/verify.sh` and `scripts/qualify-e2e.sh` |
| `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | Rebuilt workspace root |
| `measure-coverage.sh`, `merge-coverages.sh` | Rebuilt coverage entry points |

The declared Specification, the declared documents, the frozen
`crates/passuff-e2e` package, and the project files are unchanged. The builder
record is kept at `givn/trials/2026-11-18-whole-system-rebuild/builder-notes.md`.

## Verification on this branch

| Check | Command | Result |
|---|---|---|
| Offline build | `cargo build --offline --locked -p passuff --bins` | exit 0 |
| Component gate | `GIVN_FEATURES="$PWD/givn/specs" ./scripts/verify.sh` | exit 0; fmt, clippy, 0 unit tests |
| Frozen acceptance | `GIVN_RESULT_FILE=/tmp/ship-e2e-result.json ./scripts/verify.sh --e2e --package crates/passuff-e2e` | exit 0; 29 passed, 0 failed, 0 skipped |
| CI Clippy with `test-support` | `cargo clippy --offline --locked -p passuff --all-targets --features test-support -- -D warnings` | exit 0 |

Accepted result file: `{"failed":0,"passed":29,"skipped":0,"total":29}`.

## Shipping changes

| Path | Change | Reason |
|---|---|---|
| `scripts/verify.sh` | Runs the frozen runner with `--test e2e_runner` and writes `GIVN_RESULT_FILE` | The rebuilt script passed `--nocapture` to a custom harness, which rejects it |
| `scripts/qualify-e2e.sh` | Uses the same frozen runner invocation | Same harness contract |
| `crates/passuff-cli/Cargo.toml` | Adds an empty `test-support` feature | The existing CI Clippy job names the feature; the rebuilt crate has no test-support code |

## Known limits

- The 14 Specification defects recorded in `report.md` remain.
- The rebuild authored no component unit tests; the frozen acceptance package
  is the executable gate.
- The 20 seeded-defect patch files, `scripts/bundle-macos.sh`, and
  `scripts/json_to_cobertura.py` from the previous implementation are removed
  because the whole `scripts/**` root is replaced.
- The release workflow still expects the previous `passuff --version` string and
  release version; this branch prints `passuff 0.1.0`. Release work is not part
  of this ship.
