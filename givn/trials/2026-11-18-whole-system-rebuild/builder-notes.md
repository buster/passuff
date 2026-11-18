# Builder notes

## Clean-room statement

This tree was authored from the 21 declared Specification files in the copy:
`givn/specification.yaml`, the six `givn/specs/**/usecase.md` files, the seven
`givn/specs/**/*.feature` files, `README.md`, `docs/glossary.md`,
`docs/domain-model.md`, `docs/reference.md`, `docs/limits.md`,
`docs/arc42/02-architecture-constraints.md`, and
`docs/arc42/03-context-and-scope.md`.

No source, archive, evidence, builder note, `.git` directory, or previous
implementation was read. No outbound network request was made. The only external
resources used were the Rust toolchain, the local Cargo registry cache, and a
loopback HTTP mock server written outside this tree for manual checks.

The declared documents were not modified.

## Layout

```text
Cargo.toml                     workspace, resolver 2
rust-toolchain.toml            stable channel
Cargo.lock                     locked offline resolution
crates/passuff-config/         TOML model, validation, atomic save
crates/passuff-core/           model, state, GraphQL, classifier, refresh, read model
crates/passuff-cli/            package `passuff`; `passuff` and `passuff-tui` bins
  assets/notification.wav      bundled 1.0 s, three-beep PCM WAV
scripts/verify.sh              change/e2e verification entry point
scripts/qualify-e2e.sh         frozen-package qualification entry point
measure-coverage.sh            non-E2E/E2E Cobertura entry point
merge-coverages.sh             merged Cobertura entry point
```

`passuff-core` does not depend on clap, crossterm, ratatui, notify-rust,
terminal formatting, or delivery. `passuff-config` holds configuration and
credential-file validation. The CLI owns transcripts, delivery, browser
opening, setup, and the TUI.

## Files authored

- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
- `crates/passuff-config/Cargo.toml`, `crates/passuff-config/src/lib.rs`
- `crates/passuff-core/Cargo.toml`
- `crates/passuff-core/src/lib.rs`
- `crates/passuff-core/src/model.rs`
- `crates/passuff-core/src/state.rs`
- `crates/passuff-core/src/github.rs`
- `crates/passuff-core/src/classifier.rs`
- `crates/passuff-core/src/core.rs`
- `crates/passuff-core/src/view.rs`
- `crates/passuff-cli/Cargo.toml`
- `crates/passuff-cli/src/lib.rs`
- `crates/passuff-cli/src/util.rs`
- `crates/passuff-cli/src/paths.rs`
- `crates/passuff-cli/src/render.rs`
- `crates/passuff-cli/src/runtime.rs`
- `crates/passuff-cli/src/setup.rs`
- `crates/passuff-cli/src/delivery.rs`
- `crates/passuff-cli/src/sound.rs`
- `crates/passuff-cli/src/browser.rs`
- `crates/passuff-cli/src/ui_prefs.rs`
- `crates/passuff-cli/src/tui/mod.rs`
- `crates/passuff-cli/src/tui/render.rs`
- `crates/passuff-cli/src/bin/passuff.rs`
- `crates/passuff-cli/src/bin/passuff-tui.rs`
- `crates/passuff-cli/assets/notification.wav`
- `scripts/verify.sh`, `scripts/qualify-e2e.sh`
- `measure-coverage.sh`, `merge-coverages.sh`
- `builder-notes.md`

## Dependencies resolved

Direct workspace dependencies (the lockfile is attached):

| Crate | Version | Use |
|---|---|---|
| clap | 4.6.8 | CLI and TUI arguments |
| crossterm | 0.29.0 | raw terminal input and size |
| chrono | 0.4.45 | RFC 3339 timestamps, UTC display |
| serde | 1.0.229 | JSON and TOML models |
| serde_json | 1.0.154 | state, ui preferences, notification log |
| toml | 0.8.23 | configuration |
| ureq | 3.5.2 | GraphQL POST over HTTP/HTTPS (`rustls`, `json`) |
| directories | 6.0.0 | XDG/platform config and state paths |
| ctrlc | 3.5.2 | watch-loop SIGINT and TUI signal exit |
| thiserror | 2.0.21 | error types |

Transitive highlights: `rustls` 0.23.46, `ring` 0.17.14, `chrono`,
`passuff-config` 0.1.0, `passuff-core` 0.1.0.

Choices:

- **No ratatui.** The frame is rendered into a character canvas and written
  with crossterm. This keeps layout, clipping, and snapshot output under one
  renderer and avoids a large dependency tree.
- **No notify-rust.** Linux delivery invokes `notify-send`; macOS invokes
  `osascript`. The notification-log seam bypasses both. This departs from the
  arc42 wording that names notify-rust; see gaps.
- **Sound is a child process.** Detection order is `paplay`, `pw-play`,
  `aplay`, `canberra-gtk-play` on Linux and `afplay` on macOS;
  `PASSUFF_SOUND_PLAYER` replaces detection.
- **No async runtime.** GraphQL calls are synchronous ureq requests with
  proxy detection disabled and bounded timeouts.

## Behavior implemented

- Commands: default watch loop, `check`, `status`, `inbox [--all]`,
  `read <id>`, `read --all`, `setup`, `notify-test`, `passuff-tui`.
- Exit codes 0/1/2/3 per `docs/reference.md` and `docs/limits.md`.
- TOML configuration keys, defaults, validation, credential modes, token-file
  owner-only `0600`/`0400` check, loopback-only plain HTTP, environment
  overrides `PASSUFF_GRAPHQL_URL`, `PASSUFF_NOTIFICATION_LOG`,
  `PASSUFF_SOUND_PLAYER`.
- Versioned JSON state, version-1 migration, atomic replacement, quarantine of
  corrupt/unknown-version files, 500-record cap, 30-day terminal retirement.
- GraphQL operations `PassuffViewer`, `PassuffProbe`, `PassuffAuthored`,
  `PassuffDiscovery`, `PassuffObserve`; shared node selection; stack-field
  retry; discovery partial-error tolerance; error envelope mapping.
- Classification for comments, review comments, reviews, new pull requests,
  CI failure/recovery, merge/close, reopen, short-lived pull requests,
  baseline suppression, own-action read records, bot filtering, event-family
  and focus suppression, deduplication, catch-up suppression.
- Grouped optional transcript, one digest notification per pass, digest body
  cap at five pull requests, JSON notification log, sound once per digest.
- TUI at 40x12, 80x24, and 120x40; Tab/BackTab; movement, paging, Home/End;
  watched and activity panes; filters; sort modes; ignored rows; settings;
  help; scope picker; save prompt; pull-request and activity detail overlays
  with scrolling; live probe/observe and reconciliation scheduling; API
  allowance pacing; `--snapshot`, `--keys`, `--observe`, `--observe-for`,
  scope/focus flags; `ui.json`.
- Bundled sound: mono 16-bit PCM, 44.1 kHz, 1.0 s, three beeps.

## Build

```sh
cargo build --offline --locked -p passuff --bins
```

Result: success. Binaries:

```text
target/debug/passuff
target/debug/passuff-tui
```

`cargo fmt --all -- --check` and
`cargo clippy --offline --all-targets -p passuff -p passuff-core -p passuff-config`
both pass with no warnings.

## Manual verification

A temporary `HOME`, loopback GraphQL mock, fake `gh`, fake sound player, and a
pty driver were used outside the copy. Verified:

- `--help` 0, `--version` 0, no-terminal TUI exact line and exit 3.
- Baseline check output, comment notification transcript, one-line JSON
  notification log, dedupe on repeat check, inbox rows, read one/all, unknown
  id 2, read argument errors 2.
- Status before/after check, v1 migration to version 2, corrupt and invalid
  UTF-8 state quarantine, state path directory exit 3.
- Environment token, token-file 0600/0644, empty/missing token file, viewer
  login mismatch, invalid configuration messages and exit 2.
- Unreachable endpoint and HTTP 500 account failures, discovery partial
  errors, stack retry path.
- Merge/close/reopen, post-terminal silence, new and short-lived pull
  requests, CI failure/pending/recovery, own comment read, bot default and
  enabled, focus suppression and widening, selected watch scope counts.
- Two accounts against two loopback endpoints: both baselines, a two-PR
  digest, both account rows, a selected scope hiding one account's PR from
  status/inbox while retaining it in state.
- `notify-test` sent/disabled/unavailable surfaces and fake sound player.
- Watch loop baseline, comment pass, SIGINT exit 0.
- Setup token_env and gh_oauth, replacement decline, invalid repository
  re-prompt, generated configuration load.
- TUI pty frames at 40x12, 80x24, 120x40, 80x10; tab/backtab; live probe
  observation; activity detail enter and scroll; settings, help, picker,
  save prompt; snapshot keys; `ui.json`.

Not verified against a real desktop notification daemon, a real audio device,
macOS, or the frozen acceptance package (it is not in the copy).

## Assumptions and deviations

- **Desktop transport:** `notify-send` / `osascript` instead of notify-rust.
  The log seam is the tested surface.
- **Quarantine name:** `<state-path>.corrupt` (for example
  `state.json.corrupt`).
- **Catch-up storage:** a `catch_up` array in state version 2 holds lower-cased
  `owner/repo` names awaiting the next silent observation. The configuration
  schema has no documented key for this marker.
- **Disabled event families:** the event is still recorded as unread history;
  only delivery is suppressed. The Specification says "suppresses that family"
  without saying whether history is retained.
- **Terminal retirement:** a v1 terminal record has no terminal timestamp, so
  its 30-day window starts at migration time.
- **TUI rendering:** text-only frame; the watn palette and muted/color styling
  are not implemented. The exact title-clipping examples in `docs/limits.md`
  are not reproduced because they conflict with the column layout in
  `docs/reference.md`.
- **TUI repository add:** `--scope-select` adds an arbitrary repository and
  validates `owner/repo`; the picker toggles known repositories and focus
  relations. There is no documented text-entry key for an unknown repository.
- **`PASSUFF_SOUND_PLAYER`:** a value with whitespace is split into program and
  arguments; the sound file is appended. A value naming an existing file is
  used as the program directly.
- **Sound failure wording:** no player is `sound unavailable`; a player that
  starts and exits non-zero is `sound failed`.
- **Browser URL:** pull requests open at
  `https://github.com/<owner>/<repo>#<number>`, matching the activity-inbox
  scenario. The GraphQL `url` is retained in state but not used for opening.
- **Snapshot size:** terminal size when stdout is a terminal, otherwise
  `COLUMNS`/`LINES`, otherwise 80x24.
- **Frozen package placement:** `crates/passuff-e2e` is excluded from this
  workspace. If the package is placed under `crates/`, its manifest builds as
  its own workspace; the product lockfile is not changed by it.
- **No component runner:** `crates/passuff-cli/tests/features_runner.rs` and
  `crates/passuff-e2e` are not rebuilt. `scripts/verify.sh` runs formatting,
  clippy, and Rust tests for the default scope and delegates `--e2e` to the
  frozen package when it is present. The scripts do not write a
  `GIVN_RESULT_FILE`.

## Specification gaps found

| Missing or contradictory rule | Document that should state it |
|---|---|
| Quarantine destination filename is not named. | `docs/reference.md` Files / state failure section |
| No documented state field for pending catch-up repositories. | `docs/reference.md` state section or `change-watch-scope` rules |
| Event-family suppression does not say whether activity history is retained. | `docs/reference.md` `[events]` or R23 |
| Version-1 migration does not say how seen identities or terminal timestamps are reconstructed; replay behavior after migration is undefined. | `docs/reference.md` version-1 migration section |
| Linux notification transport is named as notify-rust in a constraint, but the reference contract does not require that dependency and the environment has no daemon. | `docs/arc42/02-architecture-constraints.md` or `docs/reference.md` notification delivery |
| TUI repository-add interaction has no documented key or prompt, yet the watch-scope feature mentions selecting an invalid `acme` in the TUI. | `docs/reference.md` TUI keymap / scope picker |
| Exact watched-row column widths and title-clipping examples are inconsistent between limits and reference. | `docs/limits.md` and `docs/reference.md` watched row |
| `read --all` output when no record changes is not specified. | `docs/reference.md` `passuff read` |
| `--snapshot` terminal size selection is not documented. | `docs/reference.md` `passuff-tui` options |
| "One typed query per account per cycle" conflicts with the three-operation observation table (`Viewer`, `Authored`, `Discovery`). | `docs/arc42/02-architecture-constraints.md` and `docs/reference.md` GraphQL contract |
| Sound failure wording other than player absence is not specified. | `docs/reference.md` notification delivery |
| State version-2 field names beyond `version` are not specified, so the black-box suite can only assert the version and retained strings. | `docs/reference.md` state section |
| The glossary says Activity detail "never contains a comment or review body", while T14 and the reference example require the full body in the detail. | `docs/glossary.md` and `docs/reference.md` activity detail |
| The glossary lists a pull request state and CI result in Activity detail, but the reference example omits those lines; T14 requires them. | `docs/glossary.md` and `docs/reference.md` activity detail |
