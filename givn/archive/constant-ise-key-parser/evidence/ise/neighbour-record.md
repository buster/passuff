# S11 neighbour record

Trial revision: `b23d060d1c8a408d695b0e526414471bcad4b6a9`
Copy: `/tmp/passuff-s11-keys`
Manifest: `evidence/ise/manifest.yaml`

Method: every file in the copy was compared with the same path at the trial
revision. The start set is the manifest's `specification_files`,
`source_kept_files`, and `removed_paths`. `target/`, `builder-notes.md`, and the
manifest are not components. The scan is `evidence/ise/neighbours-scan.txt`.

## Changed outside the replaced unit

| Path | Kind | Reason |
|---|---|---|
| `Cargo.toml` | Recreated build root | The declaration retains no root workspace manifest, so the copy omits it. The builder reconstructed it from the kept crate manifests. It differs from the checkout: `rust-version = "1.85"` instead of `1.88`; no `authors`; `passuff-config`/`passuff-core` workspace entries without `version = "0.1.3"`; `clap` without the `env` feature; `notify-rust = "4"` instead of `4.18`; `reqwest` without `default-features = false` and `rustls-tls`; `tokio` without `sync`; `toml = "0.8"` and `toml_edit = "0.22"` instead of `1` and `0.23`; no `exclude = ["crates/passuff-e2e"]` (the copy has no e2e package); no `[profile.dev]` or `[profile.test]`. |
| `Cargo.lock` | Generated | The copy omits the lockfile. Cargo generated it from the reconstructed workspace. It resolves the versions above and is not identical to the checkout lock. |
| `rust-toolchain.toml` | Recreated build root | The copy omits it. The builder pinned the installed `stable` channel and omitted the checkout's `profile = "minimal"`. |

No other path was added, changed, or removed. The scan reports `added:` empty
and `removed-from-start-set:` empty.

## Replaced unit

| Path | Kind | Result |
|---|---|---|
| `crates/passuff-cli/src/tui/keys.rs` | Removed by the prepare, rebuilt by the builder | The rebuilt file has the same `Key` enum and the same token mapping as the checkout; it adds four parser unit tests. The diff is `evidence/ise/keys-diff.txt`. |

## Neighbours that stayed unchanged

| Path | Role | Why it was not changed |
|---|---|---|
| `crates/passuff-cli/src/tui/mod.rs` | Consumer: `--keys` and `map_key` | The rebuilt enum and `parse_keys` satisfy its exhaustive mapping; the file is kept source and its content matches the trial revision. |
| `crates/passuff-cli/src/tui/app.rs` | Consumer: exhaustive `Key` matches | The rebuilt enum has every variant the match arms name; the file is unchanged. |
| `crates/passuff-cli/src/tui/render.rs` | TUI renderer | No parser change needed; unchanged. |
| `crates/passuff-cli/src/tui/markdown.rs` | TUI helper | No parser change needed; unchanged. |
| `crates/passuff-cli/tests/steps/tui.rs` | Component test harness calling `parse_keys` | The rebuilt parser accepts every scenario key phrase; the harness is unchanged. |
| The other 57 kept source files | Product crates and scripts | The component suite (215 scenarios, 1731 steps) passes with them unchanged. |
| The 21 Specification files | `givn/specification.yaml`, `givn/specs/**`, `docs/**`, `README.md` | `givn ise compare` reports `Specification files: match` before and after the build. |
| The 12 frozen e2e files | `crates/passuff-e2e` | The package is not in the copy; `givn ise compare` reports `End-to-end tests: match`, and the frozen run uses the checkout package against the rebuilt binary. |

## Result

The only changes outside the replaced unit are the three build-root files the
declaration does not retain. No product source, test harness, Specification
file, or frozen test changed. The component suite passes 215/215 and the frozen
suite passes 29/29 against the rebuilt binaries
(`target/debug/passuff` sha256
`3dd5562ce4bfab65062d3fa4a30a253f4e6f2acf85349c53cf610333ef332d4a`,
`target/debug/passuff-tui` sha256
`f29e793c5ca5c3ad12448585953c3cdd4e3e9bc1f2154771b95f5995f2a7bc0f`).
