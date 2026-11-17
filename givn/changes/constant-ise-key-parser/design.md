# Design: constant-ise-key-parser

## Scope

`S11` of `givn/ideation/ise-rebuildability/plan.md` is a component-level
`Constant Ise` trial. The product behavior, the Specification corpus
(`givn/specs/**`), the declared documents, and the frozen
`crates/passuff-e2e` package stay unchanged. The trial rebuilds one file,
`crates/passuff-cli/src/tui/keys.rs`, in a spec copy and grades the rebuilt
binary with the frozen suite. Nothing is promoted.

The key parser has one consumer contract: the `Key` enum and `parse_keys` are
consumed by `crates/passuff-cli/src/tui/mod.rs` and
`crates/passuff-cli/src/tui/app.rs`, by the component test harness
`crates/passuff-cli/tests/steps/tui.rs`, and by the documented `--keys`
surface and TUI keymap in `docs/reference.md`. All of them stay.

## Decisions

**Decides D1: the replaced unit is `crates/passuff-cli/src/tui/keys.rs`
alone.** The plan names the file and any internal parser unit tests outside it.
There are none: `keys.rs` has no `#[cfg(test)]` module, and the key tests in
`crates/passuff-cli/src/tui/mod.rs` exercise the app, not the parser. The
component test harness stays as the unchanged consumer that pins the parser's
input vocabulary. Realized in the `--path` argument and the manifest's
`removed_paths`.

**Decides D2: the trial revision is the pre-trial commit.** It contains the
proposal, the design, the design review, the task list, and the `.givn-skip`
marker. `givn ise rebuild-component --at <sha>` reads that revision and records
it in the manifest. The evidence commit follows the trial revision; it cannot
be the revision the copy reads. Realized in the manifest's `revision`.

**Decides D3: green is the frozen `e2e` suite.** The trial result is
`29 scenarios, 240 steps, 0 failed, 0 skipped` from
`crates/passuff-e2e` run against the copy's rebuilt binaries. The component
suite in the copy is a supporting contract check: it exercises the unchanged
test harness and the rebuilt parser together. It is not the declared
acceptance authority. Realized in the evidence result file and the run log.

**Decides D4: a missing contract stops the rebuild attempt.** When the builder
cannot derive a required parser rule from the Copy's Specification or the kept
consumers, it records exactly one `Spec gap: specification defect` against
`docs/reference.md` (the keymap and `--keys` contract), names the missing rule,
and stops writing the parser. It does not invent the rule. The trial record is
archived with the gap; the fix is a later change. Realized in the builder notes
and `evidence/ise/spec-gap.md` when a gap occurs.

**Decides D5: neighbours are every changed path outside the replaced unit.**
The record names the root workspace manifest, the lockfile, the toolchain file,
and any kept file the builder edits, with the reason. Build output under
`target/`, the builder notes, and the manifest are evidence, not components.
Realized in `evidence/ise/neighbour-record.md` and the review table.

**Decides D6: change strategy is `Constant Ise` with `Ise Test` mode
`Dry-Run Ise`.** The Specification and the frozen `@e2e` package stay frozen;
the key parser is replaced behind them and graded before any cutover.
Consumers: the developer (the keymap they use is unchanged) and the independent
verifier (repeats the prepare, compare, build, and frozen run). Persisted
state: none; the trial reads and writes no `state.json`, `ui.json`, or
`config.toml`, and the product's state version and keys stay. Rebuild
boundary: exactly `crates/passuff-cli/src/tui/keys.rs`. No promotion. Realized
in the manifest and the trial procedure.

## Architecture Impact

The declaration retains the behavior corpus, the structural documents, the
contracts, the limits, and the frozen package. It replaces `keys.rs` and keeps
the rest of the three production crates and `scripts/**`. The copy therefore
contains the kept product source but not the repository build root:

| Omitted path | Why the copy omits it | Consequence |
|---|---|---|
| `Cargo.toml` | Outside every declared role and source root | The builder recreates the workspace manifest so the kept crate manifests resolve `workspace = true`. |
| `Cargo.lock` | Outside every declared role and source root | Cargo generates it during the first build; the verifier records the versions. |
| `rust-toolchain.toml` | Outside every declared role and source root | The builder uses the installed stable toolchain and records its version. |
| `crates/passuff-e2e` | Declared `e2e`, frozen and not copied | The frozen suite runs from the checkout against the copy's binaries. |

The root workspace manifest is reconstructable from the kept crate manifests:
three members (`crates/passuff-config`, `crates/passuff-core`,
`crates/passuff-cli`), the e2e package excluded, resolver 2, and the
`[workspace.package]` and `[workspace.dependencies]` fields the crate
manifests reference. The trial records whether the builder reconstructed it
correctly; a wrong reconstruction is an implementation defect in the trial, not
a Specification gap.

### Contracts and Components

| Contract | Consumer use case | Consumers | Implementing component today | Split trigger |
|---|---|---|---|---|
| `TerminalKeyMap`: the `--keys` token vocabulary and the key-to-action mapping | Review the activity inbox and navigate it | `tui/mod.rs` (`--keys` and `map_key`), `tui/app.rs`, `tests/steps/tui.rs`, the documented `--keys` surface | `crates/passuff-cli/src/tui/keys.rs` | The parser gains a configurable or locale-aware keymap, or a second interface needs the mapping |

The contract is consumer-shaped: it exposes the `Key` values the two TUI
modules match, and the string vocabulary the harness and the CLI pass. It does
not expose the parser's internal match structure.

## Use-case traceability

No use case, guarantee, rule, event, or capability changes. The trial proves
the unchanged behavior survives the replacement.

| Use-case fact | Technical decision | Evidence |
|---|---|---|
| The inbox keymap in `docs/reference.md` maps keys to actions | D1, D3 | Frozen `e2e` suite drives the real pty; component suite binds `parse_keys` for the key phrases |
| The `--keys` option accepts the documented vocabulary | D1, D4 | Component suite scenario phrases and the CLI contract; a missing token would be a `Spec gap` |
| The TUI consumes the `Key` enum exhaustively | D1, D5 | The kept `app.rs` and `mod.rs` compile against the rebuilt enum; a missing or extra variant fails the build |
| No persisted state changes | D6 | The trial copy starts without `state.json`, `ui.json`, or `config.toml`; the verification runs create no product state |

The confirmed Persona is `multi-account-developer`, the reader of the unchanged
terminal keymap. No Persona biography is used as an actor.

## Technology and versions

| Choice | Version | Resolution and check |
|---|---|---|
| Rust | `stable` channel | `rust-toolchain.toml` in the checkout pins `channel = "stable"`, `profile = "minimal"`; the trial records `rustc --version` and `cargo --version` in `evidence/ise/environment.txt` |
| Rust edition | 2021 | `Cargo.toml` `[workspace.package] edition = "2021"` at the trial revision |
| Minimum Rust | 1.88 | `Cargo.toml` `[workspace.package] rust-version = "1.88"` at the trial revision |
| Givn | 0.9.20 | `givn --version` on the installed binary; it owns `ise rebuild-component` and `ise compare` |
| Cucumber (frozen suite) | 0.23 | `crates/passuff-e2e/Cargo.toml`, declared, not changed |
| Cucumber (component suite) | 0.23 | `crates/passuff-cli/Cargo.toml` dev-dependency, declared, not changed |

The trial adds no dependency to the product or to either suite.

## Strict mode

- **Frozen suite:** `crates/passuff-e2e/tests/e2e_runner.rs` calls
  `.fail_on_skipped()`, adds parsing errors to `failed`, exits non-zero when
  `failed > 0` or `total == 0`, and writes `total`, `passed`, `failed`, and
  `skipped` to `GIVN_RESULT_FILE`. An undefined or skipped step is a failure.
- **Component suite:** `crates/passuff-cli/tests/features_runner.rs` also calls
  `.fail_on_skipped()` and counts skipped scenarios as failed.
- **Step bodies:** the trial adds no step definition. The frozen and component
  steps are kept source and are not edited.
- **Single-scenario command:** neither runner filters by scenario name. The
  frozen runner selects every `@e2e` scenario and the component runner accepts
  `GIVN_FEATURES` as a file or directory, not a name. A failure names the
  scenario in the transcript; the trial runs the whole 29-scenario frozen suite
  and the whole component suite once.

## Trial procedure

The trial runs in four stages. The exact commands and their outputs are the
evidence.

### 1. Pre-trial revision

Commit the proposal, the design, the design review, the task list, and
`.givn-skip`. That commit is the trial revision.

### 2. Spec copy

```sh
givn ise rebuild-component crates/passuff-cli/src/tui/keys.rs \
  --at <trial-sha> \
  --into /tmp/passuff-s11-keys \
  --manifest /tmp/passuff-s11-keys.yaml
```

The report must show one removed path, the retained Specification count, the
kept source count, the outside count, the sibling count, both checksums, and
`End-to-end tests: not copied (12 files)`. Save the output as
`evidence/ise/prepare.txt` and the manifest as `evidence/ise/manifest.yaml`.

### 3. Baseline and builder

```sh
givn ise compare --manifest /tmp/passuff-s11-keys.yaml --copy /tmp/passuff-s11-keys
```

The baseline must report `Specification files: match` and
`End-to-end tests: match`. Save it as `evidence/ise/compare-baseline.txt`.

One fresh builder works only inside `/tmp/passuff-s11-keys`. It reads the
copy's `givn/specification.yaml`, `givn/specs/**`, `docs/reference.md`,
`docs/limits.md`, `docs/glossary.md`, `docs/domain-model.md`, and the kept
consumers (`tui/mod.rs`, `tui/app.rs`, `tests/steps/tui.rs`, the crate
manifests). It must not read the checkout's `crates/**` or any other copy of
the product source. It writes `crates/passuff-cli/src/tui/keys.rs`, the omitted
build root, and `builder-notes.md`.

`builder-notes.md` records:

- the inputs it read, with the exact paths;
- every contract it had to assume, and why the Specification or the kept
  consumer did not state it;
- every file it authored or edited outside `keys.rs`, with the reason;
- every build or check attempt, and the result;
- the clean-room statement: it did not read the checkout or another copy of
  the original parser, and it did not edit any Specification file;
- the registry scan result for published `passuff` sources.

### 4. Build and grade

The verifier, not the builder, builds the kept product in the copy:

```sh
cd /tmp/passuff-s11-keys
cargo build --locked -p passuff --bins
```

Then the component contract check, in the copy:

```sh
GIVN_FEATURES=/tmp/passuff-s11-keys/givn/specs \
GIVN_RESULT_SCOPE=change \
GIVN_RESULT_FILE=<change>/evidence/ise/component-result.json \
  cargo test --locked -p passuff --features test-support --test features_runner
```

Then the frozen acceptance run, from the checkout, against the copy's binaries:

```sh
PASSUFF_BIN_DIR=/tmp/passuff-s11-keys/target/debug \
GIVN_RESULT_FILE=<change>/evidence/ise/e2e-result.json \
  ./scripts/verify.sh --e2e --package crates/passuff-e2e
```

`verify.sh` rebuilds the checkout product first, then runs the frozen package
with the overridden `PASSUFF_BIN_DIR`; the package reads its scenarios from the
checkout's `givn/specs` and the binary from the copy. Save the full log as
`evidence/ise/e2e-run.txt` and the four counts as
`evidence/ise/e2e-result.json`.

### 5. Compare and neighbours

Run `givn ise compare` again and save
`evidence/ise/compare-after-build.txt`; both subjects must match. Compare every
kept file in the copy with the same path at the trial revision and list every
difference outside `keys.rs` in `evidence/ise/neighbour-record.md`:

| Neighbour path | Kind | Reason |
|---|---|---|
| (recorded per trial) | added, changed, or removed | (the build-root omission, a builder repair, or a consumer edit) |

`target/`, `builder-notes.md`, and the manifest are not components.

### 6. Result

The trial passes when the manifest, both compare outputs, the builder notes,
the frozen result, and the neighbour record exist, and the frozen result is
green. A `Spec gap: specification defect` stops the rebuild attempt and is
recorded instead of a rebuilt parser; the trial record and the gap are
archived, and no rule is invented.

## Evidence layout

| Path | Content |
|---|---|
| `evidence/ise/manifest.yaml` | The prepare manifest, copied from `/tmp` |
| `evidence/ise/prepare.txt` | `givn ise rebuild-component` output |
| `evidence/ise/compare-baseline.txt` | Compare before the builder |
| `evidence/ise/compare-after-build.txt` | Compare after the build |
| `evidence/ise/builder-notes.md` | The builder's inputs, assumptions, files, attempts, and clean-room statement |
| `evidence/ise/environment.txt` | `givn`, `rustc`, `cargo`, and platform versions |
| `evidence/ise/build.txt` | The copy's product build log |
| `evidence/ise/component-run.txt` and `component-result.json` | The component contract check |
| `evidence/ise/e2e-run.txt` and `e2e-result.json` | The frozen acceptance run |
| `evidence/ise/neighbour-record.md` | Every changed path outside the replaced unit |
| `evidence/ise/spec-gap.md` | One gap record, only when the Specification is missing a contract |

The spec copy stays under `/tmp`; it is not archived and never promoted.

## Visual Design Contract

- Interface: none
- Viewports: none, the change adds or visibly changes no screen
- Design system: none under `docs/design`; the unchanged terminal frame is
  specified by `docs/reference.md` and `docs/limits.md`

No screen is introduced or changed. The rebuilt parser sits behind the
existing terminal interaction, and the confirmed Persona
`multi-account-developer` is the reader of the unchanged keymap.

## E2E policy

No `@e2e` scenario is added, modified, or removed. The frozen package is the
acceptance authority and is not edited. The trial runs it against the rebuilt
binary, and the component suite is a supporting check. No interaction
inventory entry changes, so no Interaction Coverage Matrix row is added.
