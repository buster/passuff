# S12 neighbour record

Trial revision: `d63cdc84d89ac729d4460b63da181d180e209d0b`
Copy: `/tmp/passuff-s12-full`
Manifest: `evidence/ise/manifest.yaml`
Scan: `evidence/ise/neighbours-scan.txt`

Method: every file in the copy was compared with the trial revision. The copy's
51 files (excluding `target/**` and `builder-notes.md`) are the 21 declared
Specification files, the 3 build-root files, and 27 rebuilt source files. The
neighbour question is D5: which added paths lie outside every declared `source:`
root, and what did the whole-product rebuild have to recreate that the
declaration withholds?

## Added outside the Specification and every source root

| Path | Kind | Reason |
|---|---|---|
| `Cargo.toml` | Recreated build root | Outside every declared role, so the copy omits it. The rebuild reconstructed the workspace from the Declaration and the crate contracts. It differs from the revision: `version = "0.2.0"` instead of `0.1.3`; `rust-version = "1.80"` instead of `1.88`; a `repository` field instead of `authors`; no `exclude = ["crates/passuff-e2e"]` (the copy holds no e2e package); dependencies `ureq 3` and raw `crossterm` instead of `reqwest`/`ratatui`/`tokio`; no `[profile.dev]`/`[profile.test]`. |
| `Cargo.lock` | Generated | The copy omits the lockfile. Cargo generated it offline: 1,611 lines and 177 locked packages, against the revision's 3,925 lines. |
| `rust-toolchain.toml` | Recreated build root | The copy omits it. The rebuild pinned `channel = "stable"` with `rustfmt` and `clippy`, without `profile = "minimal"`. |
| `builder-notes.md` | Evidence | The builder's record. Not a component and not product source; archived as evidence. |

No other path was added outside the Specification and the source roots.

## Rebuilt files per declared source root

| Root | Files rebuilt | Revision files | New paths | Revision paths not recreated |
|---|---|---|---|---|
| `crates/passuff-cli` | 16 | 26 | `src/cli.rs`, `tests/steps/{dispatch,specs,world}.rs` | `src/{args,paths,setup,sound}.rs`, `src/tui/{app,keys,markdown}.rs`, `tests/steps/{commands,setup,tui}.rs`, `tests/support/{clock,fake,mod,world}.rs` |
| `crates/passuff-core` | 8 | 10 | `src/credential.rs` | `src/{credentials,inbox,probe}.rs` |
| `crates/passuff-config` | 2 | 2 | none | none |
| `scripts` | 1 (`verify.sh`) | 24 | none | `bundle-macos.sh`, `json_to_cobertura.py`, `qualify-e2e.sh`, `scripts/qualify/*.patch` (20) |

The 17 product paths not recreated as separate files are a layout difference,
not a behavior difference: the rebuild consolidated argument parsing into
`src/cli.rs`, sound and delivery into `src/delivery.rs`, the key parser and app
into `src/tui/mod.rs`, and the inbox/probe into `crates/passuff-core/src/core.rs`
and `model.rs`. The declaration owns roots, not a file layout, and the frozen
suite grades behavior: 28 of 29 pass. The layout difference is G3 in the gap
table.

The 23 missing script paths are the trial's largest omission. `verify.sh` was
derivable from `docs/limits.md` and produced the declared 215-scenario scope; the
coverage, qualification, packaging, and Cobertura-conversion scripts were not,
because no declared document states their behavior. They are G5, G6, and G7 in
the gap table, all `specification defect`.

## Differences that stayed inside the rebuilt unit

| Path group | Kind | Result |
|---|---|---|
| The rebuilt production source | Rebuilt | The frozen suite drives the real binaries: 28/29 scenarios, 236/237 steps. The single failure is the activity detail body (G1, `implementation defect`). |
| The rebuilt component step harness | Rebuilt | The copy's declared `./scripts/verify.sh` runs 215 scenarios, 215 passed. A direct invocation with an absolute `GIVN_FEATURES` directory selects none (G2, `acceptable undocumented change`). |
| `crates/passuff-cli/assets/notification.wav` | Recreated asset | The declaration removes the asset with its source root. The rebuild authored a 44,144-byte WAV; the Specification states only "three short beeps within one second" (G19). |

## What did not change

| Path group | Role | Why it is unchanged |
|---|---|---|
| The 21 Specification files | declaration, `givn/specs/**`, `docs/**`, `README.md` | `givn ise compare` reports `Specification files: match` before and after the build. The rebuild edited no Specification file. |
| The 12 frozen e2e files | `crates/passuff-e2e` | Not copied; `givn ise compare` reports `End-to-end tests: match`, and the frozen run used the checkout package against the copy's binaries. |

## Result

The only additions outside the declared Specification and source roots are the
three build-root files the declaration withholds; the only asset the rebuild had
to recreate inside a root is the bundled sound. No Specification file, frozen
test, or checkout source changed. The rebuilt binaries are
`target/debug/passuff` sha256
`9818ef6d494777a456d7fb331e3d0a6df3379f50eddedc382c3d642152db142d` and
`target/debug/passuff-tui` sha256
`7ff117fd2bfd9ef03a101e62139af83577c05d38908acf354017cd8c4437e89c`.
The copy is not promoted.
