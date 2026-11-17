# S12 gap table

Trial revision: `d63cdc84d89ac729d4460b63da181d180e209d0b`
Copy: `/tmp/passuff-s12-full`
Frozen result: 29 scenarios, 28 passed, 1 failed, 0 skipped, 237 steps.
Rebuilt component scope: 215 scenarios, 215 passed through the copy's declared
`./scripts/verify.sh`; 0 selected through a direct `GIVN_FEATURES` absolute
directory (G24).

Every difference between the rebuilt product and the retained Specification
carries exactly one outcome and the artifact that is wrong. The four outcomes
are the ise vocabulary: `E2E test defect`, `implementation defect`,
`specification defect`, `acceptable undocumented change`.

Totals: 1 `implementation defect`, 4 `specification defect`, 23 `acceptable
undocumented change`. No `E2E test defect`: every frozen scenario asserts a rule
the retained corpus states, and the one failure is a rebuilt rendering defect.

## Rebuilt-product differences

| ID | Difference | Outcome | Wrong artifact | Evidence | Route |
|---|---|---|---|---|---|
| G1 | The activity detail overlay opens at 120x40 with an empty body; `Event: comment by @reviewer` never renders. `docs/reference.md` "Activity detail overlay" lists the content lines. | `implementation defect` | `crates/passuff-cli/src/tui/render.rs` (rebuilt) | `e2e-run.txt` scenario `The activity detail opens at the wide terminal size and quits cleanly`; screen capture shows the overlay frame with no content lines | Repair the rebuilt renderer in a later `Constant Ise` attempt; no corpus change |
| G2 | The rebuilt component runner selects zero scenarios when `GIVN_FEATURES` is an absolute directory; the declared scope command (`./scripts/verify.sh`) selects 215 and passes them. | `acceptable undocumented change` | `docs/limits.md` ("Honours `GIVN_FEATURES`" does not state the accepted forms) | `component-run.txt` (`total: 0`), `component-run-declared-surface.txt` (`215/215`) | State the accepted `GIVN_FEATURES` forms in `docs/limits.md` |
| G3 | Rebuilt source layout differs: 17 revision product files are not recreated as separate files (`src/args.rs`, `paths.rs`, `setup.rs`, `sound.rs`, `tui/app.rs`, `tui/keys.rs`, `tui/markdown.rs`, seven `tests/**` files, `core/credentials.rs`, `inbox.rs`, `probe.rs`) and 5 new paths appear (`cli.rs`, `credential.rs`, `tests/steps/{dispatch,specs,world}.rs`). | `acceptable undocumented change` | `givn/specification.yaml` (declares roots, not a file layout) | `neighbours-scan.txt`; frozen suite 28/29 grades the behavior, not the layout | None; the declaration intentionally owns roots |
| G4 | Product version is `0.2.0`; the revision declares `0.1.3`. `--version` only has to contain `passuff`. | `acceptable undocumented change` | `docs/reference.md` (`-V, --version`; no version value stated) | `binaries.txt`; frozen scenarios assert only the `passuff` substring | None |

## Specification defects

| ID | Missing rule | Outcome | Wrong artifact | Evidence | Route |
|---|---|---|---|---|---|
| G5 | `measure-coverage.sh` and `merge-coverages.sh` behavior: arguments, scopes, report paths, and outputs. `README.md` names both commands; no declared document states what they must do. | `specification defect` | `README.md` (lines 156, 157, 191, 192) | `builder-notes.md` "Spec gaps"; the rebuilt `scripts/` holds only `verify.sh` | State the coverage script contract in `README.md` or `docs/limits.md` in a later change |
| G6 | `scripts/qualify-e2e.sh` and the twenty seeded-defect patches: the defect set, pinning rule, and summary contract. Only `README.md` names the script; the defect list lives in TD-007 and the `S10` archive, neither declared. | `specification defect` | `README.md` (line 170) | `builder-notes.md` "Spec gaps"; 21 of the 23 missing script paths | Move the seeded-defect contract into a declared document |
| G7 | `scripts/bundle-macos.sh` and `scripts/json_to_cobertura.py` behavior. No declared document describes either, yet `scripts/**` is a replaceable root. | `specification defect` | `givn/specification.yaml` (`source:` declares `scripts/**`) | `builder-notes.md` "Spec gaps"; `neighbours-scan.txt` missing paths | Either declare these scripts' contract or narrow the `scripts/**` source root |
| G8 | The page size and paging rule for `PassuffAuthored` and `PassuffDiscovery`. `docs/reference.md` names the operations and selections but states `first: 100` only for `PassuffProbe`; an account with more than one page of authored pull requests would be observed incompletely. | `specification defect` | `docs/reference.md` (GitHub GraphQL contract) | `builder-notes.md` "Spec gaps"; frozen corpus uses one page | State the connection arguments and the paging rule |

## Accepted undocumented choices

Each row is a detail the Specification leaves free and the rebuild's choice
contradicts no scenario. The artifact named is the document that leaves it free.

| ID | Detail the rebuild had to choose | Outcome | Wrong artifact | Evidence |
|---|---|---|---|---|
| G9 | Version-2 state JSON schema: field names, nesting, and whether activity bodies persist. Only the version-1 sample and the "kept every watched pull request" rule are stated. | `acceptable undocumented change` | `docs/reference.md` (status/state) | `builder-notes.md`; frozen restart, upgrade, and cap scenarios pass |
| G10 | Corrupt-state quarantine name and location ("set aside" only). | `acceptable undocumented change` | `docs/reference.md` (error envelope) | `builder-notes.md` |
| G11 | The inbox change-probe fingerprint: which fields constitute a change. | `acceptable undocumented change` | `docs/reference.md` (observation model) | `builder-notes.md`; frozen TUI scenarios pass |
| G12 | The rate-limit pacing formula; only "never spends the allowance faster than it is replenished" is observable. | `acceptable undocumented change` | `docs/reference.md` (observation model) | `builder-notes.md` |
| G13 | `ui.json` schema and the exact view fields that survive a restart; the reference names the five saved settings. | `acceptable undocumented change` | `docs/reference.md` (TUI save prompt) | `builder-notes.md` |
| G14 | Footer hints, help, settings, and picker wording beyond the asserted substrings. | `acceptable undocumented change` | `docs/reference.md` (allows wording beyond asserted substrings) | `builder-notes.md`; frozen frame assertions pass except G1 |
| G15 | Default frame size for `--snapshot` without a terminal. | `acceptable undocumented change` | `docs/reference.md` (`passuff-tui options`) | `builder-notes.md` |
| G16 | Acceptance of `s` and `S` in `--keys` although the `--keys` list omits them and the keymap uses them. | `acceptable undocumented change` | `docs/reference.md` (`--keys` list) | `builder-notes.md`; the keymap table and `map_key` give one mapping each |
| G17 | Setup prompt wording beyond the replacement question and the error substrings. | `acceptable undocumented change` | `docs/reference.md` (Setup) | `builder-notes.md`; setup scenarios pass |
| G18 | Sound player command-line forms and whether the player receives a file path. | `acceptable undocumented change` | `docs/reference.md` (Notification delivery) | `builder-notes.md` |
| G19 | Bundled sound asset name, container format, and location; only "three short beeps within one second" is declared. | `acceptable undocumented change` | `docs/reference.md` (notifications) | `crates/passuff-cli/assets/notification.wav` (44,144 bytes, authored); `builder-notes.md` |
| G20 | `gh auth login` argv (host, protocol, scopes, retry check); `--web` is the only stated part. | `acceptable undocumented change` | `docs/reference.md` (`gh_oauth` accounts) | `builder-notes.md` |
| G21 | Whether reading an already-read id reports it. The reference says "one changed record", so the rebuild counts changed records only. | `acceptable undocumented change` | `docs/reference.md` (`passuff read`) | `builder-notes.md`; frozen read scenarios pass |
| G22 | Whether ignoring a pull request in the TUI persists to configuration. | `acceptable undocumented change` | `docs/reference.md` (TUI keymap, save prompt) | `builder-notes.md` |
| G23 | Status watched-row ordering when several pull requests are present. | `acceptable undocumented change` | `docs/reference.md` (`passuff status`) | `builder-notes.md`; frozen single-row status scenarios pass |
| G24 | The initial TUI observation cycle: probe versus reconciliation on open. | `acceptable undocumented change` | `docs/reference.md` (observation model) | `builder-notes.md`; frozen TUI scenarios pass |
| G25 | Transcript composition when one pass records a baseline and a notification; where delivery failures are written. | `acceptable undocumented change` | `docs/reference.md` (terminal output allows wording beyond asserted substrings) | `builder-notes.md` |
| G26 | Dependency and build-root choices: `ureq` instead of `reqwest`, raw `crossterm` instead of `ratatui`, no `tokio`, `tokio`/`toml_edit`/`directories`/`thiserror`/`serde_json` versions and profiles. The declared constraints name only `notify-rust`/`user-notify` compatibility and the core's no-UI rule, and the rebuild satisfies both. | `acceptable undocumented change` | `givn/specification.yaml` (the build root is outside every declared role) | `neighbour-record.md`; `crates/passuff-core/Cargo.toml` has no UI or delivery dependency |
| G27 | The rebuilt root manifest leaves out `exclude = ["crates/passuff-e2e"]` because the copy holds no e2e package; the frozen package declares its own workspace. | `acceptable undocumented change` | `givn/specification.yaml` (build root not declared) | `neighbour-record.md`; the frozen run builds and runs from the checkout package |
| G28 | Rust toolchain pin: `stable` with `rustfmt` and `clippy`, no `profile = "minimal"`; `rust-version = "1.80"` instead of `1.88`. The Specification states no minimum. | `acceptable undocumented change` | `givn/specification.yaml` (build root not declared) | `neighbour-record.md`; build succeeds on the installed stable toolchain |

## No drift left open

- **G1** is the only red frozen scenario; it is an implementation defect inside
  the rebuilt copy, not a corpus defect. The copy is not promoted.
- **G2 to G4** need no corrective change: the declared scope surface is green,
  the layout and version are free.
- **G5 to G8** are the four Specification defects. Each names the document and
  the missing rule; a later change must state the rule before another
  whole-product rebuild can recreate those paths. This change does not edit a
  Specification file, per its proposal.
- **G9 to G28** are accepted choices. No corpus edit follows from them.
