# Clean-room rebuild notes — passuff key parser (S11 Constant Ise trial)

Component: `crates/passuff-cli/src/tui/keys.rs`
Copy: `/tmp/passuff-s11-keys`

## Clean-room statement

I read only under `/tmp/passuff-s11-keys`. I did not read
`/home/buster/projects/gh-notify`, the givn worktree, any git object store, any
other copy of the product source, the frozen `crates/passuff-e2e` package, or
any `passuff*` crate in the user's Cargo registry. I did not edit any
Specification file (`givn/**`, `docs/**`, `README.md`) or any kept product
source. The only product file I wrote is `crates/passuff-cli/src/tui/keys.rs`.

## Inputs read

Specification (read-only):

- `givn/specification.yaml`
- `docs/reference.md` (the `passuff-tui options`, `TUI keymap`, `Overlays`,
  `Terminal output`, and `Files` sections)
- `docs/limits.md`
- `docs/arc42/02-architecture-constraints.md`
- `README.md`
- `givn/specs/review-activity-inbox/activity-inbox.feature`
- `givn/specs/review-activity-inbox/inbox-navigation.feature`
- `givn/specs/review-pull-request-stacks/stacked-inbox.feature`
- `givn/specs/watch-pull-requests/pr-activity-notification.feature`
  (key-phrase and `--version` scan)

Kept source and manifests (read-only):

- `crates/passuff-cli/Cargo.toml`, `crates/passuff-core/Cargo.toml`,
  `crates/passuff-config/Cargo.toml`
- `crates/passuff-cli/src/tui/mod.rs` (`parse_keys`/`Key` use, `map_key`)
- `crates/passuff-cli/src/tui/app.rs` (`handle_key`, `handle_overlay_key`,
  `handle_picker_key` — the exhaustive `Key` matches)
- `crates/passuff-cli/src/args.rs` (`TuiCli.keys`)
- `crates/passuff-cli/src/lib.rs`
- `crates/passuff-cli/tests/steps/tui.rs` (`parse_keys`, `Key::Down`,
  `Key::OpenPicker`)
- `crates/passuff-cli/tests/features_runner.rs`, `crates/passuff-cli/tests/steps/mod.rs`
- `crates/passuff-core/src/github.rs` (dependency feature use only)
- `scripts/verify.sh`, `scripts/bundle-macos.sh` (build commands, version)

Environment facts used (not product source): installed toolchain
(`rustc 1.99.1`, `stable`), cached Cargo registry contents (version choice
only, see Assumptions), openssl availability.

## Contract derived

`Key` must be a `Copy` enum with exactly the variants the kept exhaustive
matches in `app.rs` name, because `app.rs` matches `Key` without a wildcard and
`tui/mod.rs` derives `Clone` on `TuiOptions` and uses `.iter().copied()`:

`Tab`, `BackTab`, `Down`, `Up`, `PageDown`, `PageUp`, `Home`, `End`, `Enter`,
`Esc`, `Quit`, `MarkRead`, `MarkAllRead`, `ToggleUnread`, `ClearFilter`,
`ResetView`, `SortNext`, `SortReverse`, `FilterPullRequest`, `FilterStack`,
`ToggleIgnore`, `ToggleIgnoredVisibility`, `Left`, `Right`, `OpenSettings`,
`OpenPicker`, `Help`, `ToggleMode`, `Space`, `Char(char)`.

`pub fn parse_keys(value: &str) -> Result<Vec<Key>, String>` is called by
`tui/mod.rs` for `--keys` and by the test harness for every key phrase in the
corpus.

Token mapping chosen (each token → `Key`):

| Token(s) | Key | Source |
|---|---|---|
| `tab` | `Tab` | `docs/reference.md` `--keys` list + keymap |
| `backtab` | `BackTab` | same |
| `down`, `j` | `Down` | `--keys` list; keymap `j`, `↓`; `map_key` maps `'j'` to `Down` |
| `up`, `k` | `Up` | same |
| `pageup` | `PageUp` | same |
| `pagedown` | `PageDown` | same |
| `home` | `Home` | same |
| `end` | `End` | same |
| `enter` | `Enter` | same |
| `esc` | `Esc` | same |
| `q` | `Quit` | keymap (`q`, `Esc` quit); `map_key` maps `'q'` to `Quit` |
| `left` | `Left` | `--keys` list; keymap `←` hides ignored |
| `right` | `Right` | same |
| `space` | `Space` | `--keys` list; keymap settings/picker toggle |
| `comma` | `OpenSettings` | `--keys` list; keymap `,` opens settings; `map_key` maps `','` to `OpenSettings` |
| `r` | `MarkRead` | keymap `r`; `map_key` |
| `a` | `MarkAllRead` | keymap `a`; `map_key` |
| `u` | `ToggleUnread` | keymap `u`; `map_key` |
| `c` | `ClearFilter` | keymap `c`; `map_key` |
| `C` | `ResetView` | keymap `C`; `map_key` |
| `s` | `SortNext` | keymap `s`; `map_key` |
| `S` | `SortReverse` | keymap `S`; `map_key` |
| `x` | `ToggleIgnore` | keymap `x`; `map_key` |
| `f` | `FilterPullRequest` | keymap `f`; `map_key` |
| `F` | `FilterStack` | keymap `F`; `map_key` |
| `h`, `H` | `Help` | keymap `h`, `H`, `F1`; `map_key` |
| `p` | `OpenPicker` | keymap `p`; `map_key` |
| any other single character | `Char(c)` | `map_key`'s `other => Key::Char(other)` fallback |
| any other name | `Err("unknown key: <token>")` | nothing else is documented |

`ToggleIgnoredVisibility` and `ToggleMode` are required by the exhaustive
matches but are not produced by `map_key` or by any documented key; they are
kept as matchable variants and are not produced by `parse_keys`.

## Assumptions

1. **Build root is omitted from the copy, so I reconstructed it.** Versions and
   features are not in the Specification; I chose them from the kept manifests
   plus the constraint docs and the crate versions present in the local Cargo
   registry cache (version choice is environmental, not product source):
   `chrono 0.4` (serde), `clap 4.6` (derive), `crossterm 0.29`,
   `directories 6`, `notify-rust 4`, `ratatui 0.30`, `reqwest 0.12`
   (blocking, json), `serde 1` (derive), `serde_json 1`, `thiserror 2`,
   `tokio 1` (macros, rt, rt-multi-thread, signal, time), `toml 0.8`,
   `toml_edit 0.22`. The `toml 0.8` / `toml_edit 0.22` pair is what the kept
   `passuff-config` code compiles against (`DocumentMut`, `toml::from_str`,
   `toml::to_string_pretty`). No dependency is added that a kept manifest does
   not name.
2. **Package metadata.** `version = "0.1.3"` comes from
   `scripts/bundle-macos.sh`; `license = "GPL-3.0-or-later"` from the legal
   constraints in `docs/arc42/02-architecture-constraints.md`; `edition = 2021`
   and `rust-version = "1.85"` are not stated anywhere, so I chose a conservative
   stable combination. The corpus only asserts that `--version` output contains
   `passuff`.
3. **`--keys` under-enumerates.** The `--keys` paragraph lists 20 tokens, but
   the executable corpus presses `s`, `S`, `f`, `F`, `h`, `u`, `c` through the
   same `parse_keys` step definition. Each of those has exactly one mapping in
   the `TUI keymap` table and in `map_key`, so the rule is derivable; I treat the
   shorter `--keys` list as an enumeration that is not exhaustive rather than a
   defect. No mapping was invented.
4. **Whitespace and empty segments.** The Specification does not say whether
   segments are trimmed or how an empty phrase behaves. I trim each segment,
   skip empty segments, and return an empty sequence for an empty phrase. This
   is permissive and cannot change any scenario phrase, all of which are exact
   lowercase names or single characters.
5. **Error text** for an unknown token is not specified or asserted; I used
   `unknown key: <token>`, which `TuiOptions::from_cli` surfaces as exit 2,
   matching the documented "configuration, usage, or argument error" code.
6. **Single-character fallback.** `parse_keys` maps unlisted single characters
   through the same table as `map_key` and then to `Char(c)`. This is required so
   `y`, `n` (save prompt) and `m` (scope-picker mode) behave like terminal
   input, and it matches `map_key`'s fallback arm.
7. **`comma` → `OpenSettings`**, not `Char(',')`, because `map_key` maps `','`
   to `OpenSettings`; the `Char(',')` arm in the settings overlay is a
   secondary close path that no token needs to reach.

## Files authored or edited

| Path | Reason |
|---|---|
| `crates/passuff-cli/src/tui/keys.rs` | The rebuilt component (enum, `parse_keys`, unit tests). |
| `Cargo.toml` | Workspace root omitted by the copy; reconstructed from the kept crate manifests, with `[workspace.package]` and `[workspace.dependencies]`. |
| `Cargo.lock` | Generated by Cargo while resolving the reconstructed workspace. |
| `rust-toolchain.toml` | Omitted build root; pins the channel to the installed `stable` toolchain. |
| `builder-notes.md` | This record. |
| `target/**` | Cargo build output. |

No Specification file, no kept product source, and no file listed under the
manifest's `specification_files` was added or modified.

## Build and check attempts

1. `cargo build -p passuff --bins` — **success** (both `passuff` and
   `passuff-tui`; 21s, dependency download included).
2. `cargo build --locked -p passuff --bins` — **success** (lockfile stable,
   no changes).
3. `cargo check -p passuff --all-targets --features test-support` — **success**;
   the kept Gherkin runner and step harness compile against the rebuilt parser.
4. `cargo test -p passuff --lib keys` — **success**: 4 parser unit tests passed
   (documented names, scenario characters, empty segments, unknown token).

## Registry scan

Command:

```sh
find "$HOME/.cargo/registry/src" "$HOME/.cargo/registry/cache" "$HOME/.cargo/registry/git" -maxdepth 2 -name 'passuff*' -print
```

Output (verbatim):

```text
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-config-0.1.1
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-0.1.1
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-config-0.1.2
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-config-0.1.0
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-core-0.1.1
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-core-0.1.2
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-0.1.0
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-0.1.2
/home/buster/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/passuff-core-0.1.0
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-0.1.0.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-core-0.1.0.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-config-0.1.0.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-0.1.1.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-config-0.1.1.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-core-0.1.1.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-0.1.2.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-config-0.1.2.crate
/home/buster/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/passuff-core-0.1.2.crate
```

`find` also reported `.../registry/git: No such file or directory`, so that root
does not exist. The scan is **not empty**: the machine's shared user Cargo
registry already holds published `passuff` 0.1.0–0.1.2 sources, which contain
product source including this component. This is a trial-hygiene problem in the
environment, not a build dependency: the reconstructed workspace resolves
`passuff-config` and `passuff-core` as path members and compiled only
`/tmp/passuff-s11-keys/crates/*` plus third-party crates. I did not open, list
the contents of, or read any of the printed paths, and none of the mapping rules
above depends on them. The trial operator should decide whether to move those
cache entries aside and re-run the scan before recording an empty result; I did
not delete or move another project's registry cache.

## Spec gap

None. Every parser rule needed by the kept consumers and by the corpus is
derivable from `docs/reference.md` (`--keys` paragraph, `TUI keymap`,
`Overlays`) together with `tui/mod.rs::map_key` and the exhaustive `Key`
matches in `tui/app.rs`. The only under-specification found is that the
`--keys` token list omits characters the corpus uses; their mapping is stated
unambiguously by the keymap table and `map_key`, so it is not a specification
defect.

## Registry scan after quarantine

After the verifier moved the published passuff crates out of the Cargo registry
(to `/tmp/passuff-s11-registry-quarantine`), the same scan was repeated. I did
not read any registry source.

Command:

```sh
find "$HOME/.cargo/registry/src" "$HOME/.cargo/registry/cache" "$HOME/.cargo/registry/git" -maxdepth 2 -name 'passuff*' -print
```

Output: no `passuff*` path was printed. Standard output was empty; standard
error carried only the absent-root notice below (exit status 1):

```text
Error: /home/buster/.cargo/registry/git: No such file or directory (os error 2)
```

The registry roots now hold no `passuff*` entry, so the scan requirement is
satisfied.
