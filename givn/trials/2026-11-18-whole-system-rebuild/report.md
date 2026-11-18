# Whole-system rebuild trial, 2026-11-18

Ise rebuild decision: `Whole-system rebuild`.
Rebuilt code: `discard`.
Trial revision: `372cd83ac33f05d11b108476b4d18bbc4a4e1ff3`.
Branch: `ise/whole-system-rebuild`.
Spec copy: `/tmp/passuff-ise-whole`, created by
`givn ise rebuild-all --at 372cd83 --into /tmp/passuff-ise-whole`.
The rebuilt tree is outside the project and is not promoted.

## Result

| Check | Command | Result |
|---|---|---|
| Spec copy | `givn ise rebuild-all --at 372cd83 --into /tmp/passuff-ise-whole` | 21 Specification files kept, 62 source files removed, 117 paths withheld, 0 source files kept |
| Specification unchanged | `givn ise compare --manifest /tmp/passuff-ise-whole.yaml --copy /tmp/passuff-ise-whole` | `Specification files: match`, `End-to-end tests: match` |
| Offline build | `cargo build --offline --locked -p passuff --bins` in the copy | exit 0; `passuff` and `passuff-tui` in `target/debug` |
| Frozen acceptance | `PASSUFF_BIN_DIR=/tmp/passuff-ise-whole/target/debug ./scripts/verify.sh --e2e --package crates/passuff-e2e` | exit 0; 6 features, 29 scenarios, 240 steps, `{"failed":0,"passed":29,"skipped":0,"total":29}` |
| Component unit tests | `cargo test --offline --locked` in the copy | 0 tests authored; the frozen suite is the acceptance authority |

Binary identities:

| Binary | SHA-256 |
|---|---|
| `passuff` | `b04e1b18a61167010d3e7aa08280be99d9005f4c1cb8bf52613b633b010addef` |
| `passuff-tui` | `a35efb7eb0dec18c7fc5b8d9b9e8b393929d020ed5e515bf9336ed13718cff10` |

`passuff --version` exits 0 (`passuff 0.1.0`), `passuff --help` exits 0, and
`passuff-tui` without a terminal exits 3 with the documented refusal.

## Rebuild inventory

| Role | Files removed | Files authored in the copy |
|---|---|---|
| `crates/passuff-cli` | 26 | 16 |
| `crates/passuff-core` | 10 | 8 |
| `crates/passuff-config` | 2 | 2 |
| `scripts/**` | 24 | 2 |
| Build root and root scripts | not in the declaration | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `measure-coverage.sh`, `merge-coverages.sh` |
| Bundled sound | 1 (inside `passuff-cli`) | `crates/passuff-cli/assets/notification.wav` |

The builder used Rust 1.99.1, `ureq`, `crossterm`, `serde`, `toml`, `clap`,
`chrono`, `directories`, `ctrlc`, and `thiserror`. The rebuilt core does not
depend on the CLI, terminal, or delivery crates.

## Neighbour record

Files outside the declared Specification and the replaced source roots:

| Path | Kind | Reason |
|---|---|---|
| `Cargo.toml` | Recreated build root | The declaration does not retain the workspace manifest |
| `Cargo.lock` | Generated | Offline resolution for the rebuilt workspace |
| `rust-toolchain.toml` | Recreated build root | Stable channel pin |
| `builder-notes.md` | Evidence | Builder record; not a component |
| `measure-coverage.sh` | Root script | Outside `scripts/**`; the reference names a coverage command but the declaration does not place it |
| `merge-coverages.sh` | Root script | Outside `scripts/**`; same gap |

No Specification file, declared document, or frozen e2e file changed. The
21 Specification files are byte-identical to the trial revision.

## Specification defects

Each row is a rule the builder needed and the declared Specification does not
state, or states inconsistently. The builder recorded each one and chose a
compatible behavior; none caused a frozen scenario to fail.

| Missing or contradictory rule | Document that should state it |
|---|---|
| Quarantine destination filename | `docs/reference.md` state failure section |
| Pending catch-up repository state field | `docs/reference.md` state section |
| Whether event-family suppression retains activity history | `docs/reference.md` `[events]` |
| Version-1 migration reconstruction of seen identities and terminal timestamps | `docs/reference.md` version-1 migration |
| Linux notification transport names `notify-rust` but the reference does not require the dependency | `docs/arc42/02-architecture-constraints.md` or `docs/reference.md` |
| TUI repository-add key and prompt | `docs/reference.md` TUI keymap |
| Watched-row column widths and title clipping differ between limits and reference | `docs/limits.md` and `docs/reference.md` |
| `read --all` output when no record changes | `docs/reference.md` `passuff read` |
| `--snapshot` terminal size selection | `docs/reference.md` `passuff-tui` |
| One query per account per cycle conflicts with `Viewer`, `Authored`, and `Discovery` | `docs/arc42/02-architecture-constraints.md` and `docs/reference.md` |
| Sound failure wording other than player absence | `docs/reference.md` notification delivery |
| State version-2 field names beyond `version` | `docs/reference.md` state section |
| Glossary says activity detail never contains a comment or review body, while T14 requires the body | `docs/glossary.md` and `docs/reference.md` |
| Glossary lists pull request state and CI result in activity detail, while the reference example omits them | `docs/glossary.md` and `docs/reference.md` |

## What the trial proves

- A builder with only the 21 declared Specification files produced a runnable
  Rust product that passed all 29 frozen black-box scenarios behind an unchanged
  corpus.
- The declared `source:` roots are replaceable as a set: all 62 tracked files
  under them were absent from the copy and 34 files were authored in their
  place, including the build root and the sound asset.
- The frozen package can grade a whole rebuilt product from the checkout while
  the product runs from the copy.

## What the trial does not prove

- The builder process had access to the checkout and declared that it did not
  read it. This is not an audited sandbox.
- Passing 29 scenarios proves the exercised behavior. It does not prove the
  Specification is complete for the 14 defects above.
- The rebuilt tree is not on the branch and is not proposed for adoption.

## Evidence

Temporary evidence from this run is under `/tmp/ise-evidence/`:

| File | Content |
|---|---|
| `prepare.txt` | `givn ise rebuild-all` counts |
| `manifest.yaml` | `ise-manifest/v1` for the copy |
| `compare.txt` | Specification and e2e compare result |
| `builder-notes.md` | Builder clean-room statement, layout, dependencies, gaps |
| `authored-files.txt` | Every file in the copy after the rebuild |
| `build.txt` | Frozen offline build |
| `binaries.txt` | Binary hashes, version, help, terminal refusal |
| `e2e-run.txt` | Full frozen acceptance output |
| `e2e-result.json` | Four reconciling counts |
| `environment.txt` | Toolchain, platform, revision, copy path |
