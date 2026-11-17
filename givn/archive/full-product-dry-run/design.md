# Design: full-product-dry-run

## Scope

`S12` of `givn/ideation/ise-rebuildability/plan.md` is a whole-product
`Dry-Run Ise` trial. The product behavior, the Specification corpus
(`givn/specs/**`), the declared documents, the frozen `crates/passuff-e2e`
package, and `givn/specification.yaml` stay unchanged. The trial rebuilds every
declared `source:` root in a spec copy, builds the product there, and grades the
rebuilt binaries with the frozen suite. Nothing is promoted.

The trial answers one question: can a fresh builder rebuild the whole product
from its own Specification? A green frozen run is the strongest answer; a
runnable binary with every difference classified is the accepted answer.

## Decisions

**Decides D1: the trial revision is the pre-trial commit.** It contains the
proposal, the design, the design review, the task list, `arc42.md`, and
`.givn-skip`. `givn ise rebuild-all --at <sha>` reads that revision and records
it in the manifest. The evidence commit follows the revision it describes and
cannot be the revision the copy reads. Realized in the manifest's `revision`.

**Decides D2: the trial result is a runnable rebuilt binary plus the frozen
suite result.** Both binaries must exist and run (`passuff --help`,
`passuff --version`, `passuff-tui` refuses without a terminal with exit 3), and
`./scripts/verify.sh --e2e --package crates/passuff-e2e` must run the frozen
suite against them and write four reconciling counts. A red frozen run is a
result when every failing scenario is classified by `D3`. A copy that does not
build has no trial result and the design records the build defect instead.
Realized in `evidence/ise/binaries.txt`, `build.txt`, `e2e-run.txt`, and
`e2e-result.json`.

**Decides D3: every difference gets exactly one outcome and names the wrong
artifact.** The four admissible outcomes are the ise vocabulary, applied as:

| Outcome | When it is the right call | Wrong artifact named |
|---|---|---|
| `E2E test defect` | The scenario asserts something the Specification does not state, or asserts it through an interface the product does not own | The scenario title and its feature file |
| `implementation defect` | The Specification states the rule and the rebuilt product contradicts it | The rebuilt file under a source root |
| `specification defect` | The rebuilt product needs a rule the Specification does not state and the rebuild must not invent | The declared document that should state it (`docs/reference.md`, `docs/limits.md`, `README.md`, or the declaration) |
| `acceptable undocumented change` | The Specification leaves the detail free and the rebuild's choice is compatible with every scenario | The free detail, recorded in the corpus by a later change |

Each row of the gap table names one difference. A difference that fits two
outcomes is split into two rows. Realized in `evidence/ise/gap-table.md` and
the review table.

**Decides D4: a missing contract stops the affected reconstruction and records
one `specification defect`.** The builder does not invent the rule. It leaves
the affected behavior unimplemented behind a compiling interface, records the
missing rule and the named document, and continues the independent parts so the
trial still produces a runnable binary and classifies the remaining differences.
"Compiling" is a hard constraint: the builder iterates until
`cargo build --offline -p passuff --bins` succeeds, and an unimplemented
behavior shows up as a failing frozen scenario, not as a guessed rule.
Stopping the whole rebuild at the first gap would leave no binary, no frozen
result, and no evidence about the other source roots, which contradicts `D2`;
inventing the rule is forbidden by the ise instruction. Realized in the gap
table and the builder notes.

**Decides D5: neighbours are every added path outside the declared
Specification.** For a whole-product rebuild the rebuilt source roots are the
rebuild itself, not neighbours, so the table records (a) every file the trial
adds outside a declared `source:` root, with its reason, and (b) the rebuilt
file count per declared source root. The build root (`Cargo.toml`,
`Cargo.lock`, `rust-toolchain.toml`) is the expected (a) set; any other file is
a finding. `target/`, `builder-notes.md`, and the manifest are evidence, not
components. Realized in `evidence/ise/neighbour-record.md` and the review
table.

**Decides D6: change strategy is `whole-product rebuild` with `Ise Test` mode
`Dry-Run Ise`.** The Specification and the frozen `@e2e` package stay frozen;
the whole declared source set is replaced behind them and graded before any
cutover. Consumers: the developer (every command and the terminal interface
they use) and the independent verifier (repeats the prepare, compare, build,
and frozen run). Persisted state: none; the trial reads and writes no product
`config.toml`, `state.json`, or `ui.json`, and the rebuild must carry the
documented version-2 state contract if it implements state at all. Rebuild
boundary: every path under `crates/passuff-cli`, `crates/passuff-core`,
`crates/passuff-config`, and `scripts/**`. No promotion. Realized in the
manifest and the trial procedure.

**Decides D7: the exact commands and the builder sandbox are fixed before the
trial.** The copy is written by `givn ise rebuild-all --into
/tmp/passuff-s12-full --manifest /tmp/passuff-s12-full.yaml --at <trial-sha>`;
a path that exists is removed first so a rerun cannot read stale work. The
builder workspace is the copy only: no checkout, no `crates/passuff-e2e`, no
`.git`, no network (`cargo --offline` for every command), and no published
`passuff` source from the Cargo registry. The published `passuff*` crate
sources the registry holds are moved to `/tmp/passuff-s12-registry-quarantine`
before the builder starts and restored after the frozen run; the scan is
recorded before and after. Realized in `evidence/ise/sandbox.txt` and the
builder notes.

**Decides D8: the builder authors the build root and the withheld assets.** The
copy omits every file outside the declared roles, including the root workspace
manifest, the lockfile, the toolchain file, `crates/passuff-cli/assets`
(the bundled sound), and `crates/passuff-e2e`. The builder reconstructs the
workspace from the kept Declaration and the crate contracts, generates the
asset, and records each choice in `builder-notes.md` with the crate versions
the offline registry resolved. It gains no network access and adds no crate the
Declaration does not name. Realized in `builder-notes.md` and `Cargo.lock`.

## Architecture Impact

The copy holds the 21 declared Specification files and nothing else:

| Role | Declared paths | Files in the copy | Source of the count |
|---|---|---|---|
| behavior | `givn/specs/**` | 13 | 7 features and 6 use-case documents |
| structure | `docs/glossary.md`, `docs/domain-model.md` | 2 | declaration |
| contracts | `docs/reference.md`, `README.md`, `docs/arc42/03-context-and-scope.md` | 3 | declaration |
| limits | `docs/limits.md`, `docs/arc42/02-architecture-constraints.md` | 2 | declaration |
| implicit | `givn/specification.yaml` | 1 | `givn instructions ise` |
| e2e | `crates/passuff-e2e` | 0, not copied | frozen, outside the copy |
| source | four roots | 62 removed | `git ls-files` at the trial revision |

The 62 removed files: `crates/passuff-cli` 26 (including its manifest and
`assets/notification.wav`), `crates/passuff-core` 10, `crates/passuff-config` 2,
`scripts/**` 24 (including `scripts/qualify/*.patch`).

The copy therefore has no build root. The builder must author:

| Omitted path | Why omitted | What the builder must author |
|---|---|---|
| `Cargo.toml` | Outside every declared role and source root | The workspace manifest: three members, the e2e package absent, `[workspace.package]`, `[workspace.dependencies]`, and profiles |
| `Cargo.lock` | Outside every declared role and source root | Generated by the offline build; the resolved versions are recorded |
| `rust-toolchain.toml` | Outside every declared role and source root | A channel pin, or none; the environment file records the toolchain used |
| `crates/passuff-cli/assets/notification.wav` | Inside a removed source root | A valid WAV the bundled-sound path can play; the Specification states the delivery contract, not the waveform |
| `crates/passuff-e2e` | Declared `e2e`, frozen and not copied | Nothing; the frozen run uses the checkout package |

A wrong build-root reconstruction is an `implementation defect` in the rebuild,
not a `specification defect`: the Specification does not own the build root, and
the declaration may or may not retain it (`D5`).

### Contracts and Components

Every product contract is rebuilt in this trial. Each row names the consumer use
case, the consumers, the single implementing component today, and the trigger
that would split it.

| Contract | Consumer use case | Consumers | Implementing component today | Split trigger |
|---|---|---|---|---|
| `Configuration`: the TOML schema, defaults, validation, and accounts | `set-up-passuff`, `change-watch-scope` | `passuff setup`, every command, the TUI | `passuff-config` | A second config format, or per-account validation that the CLI cannot own |
| `WatchScope`: watched accounts, repositories, focus modes | `change-watch-scope` | setup, status, check, watch, TUI | `passuff-config` schema + `passuff-cli` scope editing | A server-side scope, or a scope shared across machines |
| `PullRequestObservation`: the GraphQL reads, classifier, and stack membership | `watch-pull-requests` | watch loop, `check`, `status`, `inbox` | `passuff-core` (`github`, `probe`, `classifier`) | A second provider, or an event stream replacing polling |
| `ActivityInbox`: activity records, read state, grouping, ordering, the 500-record cap | `review-activity-inbox`, `review-pull-request-stacks` | `inbox`, `read`, `status`, TUI | `passuff-core` (`inbox`, `state`, `model`) | A remote or multi-device inbox |
| `WatchStatus`: accounts, scope, watched rows, unread count, approximate time | `inspect-watch-status` | `status`, TUI header | `passuff-core` (`core`) + `passuff-cli` (`render`) | A machine-readable status surface diverging from the terminal one |
| `NotificationDelivery`: desktop notification, sound, notification log | `watch-pull-requests` | watch loop, `notify-test` | `passuff-cli` (`delivery`, `sound`) | A second delivery surface needing its own retry policy |
| `TerminalInbox`: frame composition, keymap, overlays, paging, clipping | `review-activity-inbox` (T1 to T3), `inspect-watch-status` | `passuff-tui` | `passuff-cli` (`tui`, `render`) | A web or remote terminal client |

No split is triggered by this change. The trial proves each contract survives a
replacement behind the unchanged corpus.

## Use-case traceability

No use case, guarantee, rule, event, or capability changes. The trial proves the
unchanged behavior survives the replacement.

| Use-case fact | Technical decision | Evidence |
|---|---|---|
| A first run creates an account configuration and validates it | D2, D6 | Frozen scenarios under `set-up-passuff` drive the real prompt; a rebuilt `passuff` must exit 2 on invalid input and 3 on I/O failure |
| The watch scope narrows to authored pull requests and focus modes | D2, D6 | Frozen `change-watch-scope` scenarios drive the real CLI; the rebuilt configuration schema must accept the documented keys |
| One pass observes authored pull requests and groups new activity | D2, D3 | Frozen `watch-pull-requests` scenarios use the mock GitHub server and the notification log |
| The inbox lists, orders, caps, and marks activity read | D2, D3 | Frozen `review-activity-inbox` scenarios assert the real `status` and `inbox` output and the persisted state |
| The status command reports accounts, scope, watched rows, and the unread count | D2, D3 | Frozen `inspect-watch-status` scenarios assert the real output, including the approximate-time marker |
| The terminal interface renders the frame, keymap, overlays, and paging at 40x12, 80x24, and 120x40 | D2, D3 | Frozen `@e2e` scenarios drive a real pty through `vt100` at both budgets |
| Stack membership and the stacked inbox group rows | D2, D3 | Frozen `review-pull-request-stacks` scenarios assert grouping and stack position |
| State persists across restart with the documented version-2 contract | D6 | Frozen scenarios restart the real binary against the same state file |

The confirmed Persona is `multi-account-developer`, the reader of the rebuilt
terminal product. No Persona biography is used as an actor.

## Technology and versions

| Choice | Version | Resolution and check |
|---|---|---|
| Rust | `stable` channel | `rust-toolchain.toml` at the trial revision pins `channel = "stable"`, `profile = "minimal"`; the design records the installed `rustc`/`cargo` in `evidence/ise/environment.txt`, checked by running `rustc --version` and `cargo --version` |
| Rust edition | 2021 | `Cargo.toml` `[workspace.package] edition = "2021"` at the trial revision |
| Minimum Rust | 1.88 | `Cargo.toml` `[workspace.package] rust-version = "1.88"` at the trial revision; the builder may choose another value and must record it |
| Product dependency versions | Resolved from the offline registry | `Cargo.lock` at the trial revision; the builder records every crate version it chose |
| Cucumber (frozen suite) | 0.23 | `crates/passuff-e2e/Cargo.toml` at the trial revision, declared and unchanged |
| Cucumber (component suite) | 0.23 | `crates/passuff-cli/Cargo.toml` at the trial revision, a dev-dependency of the replaced source; the builder chooses its version and records it |
| Givn | 0.9.20 | `givn --version` on the installed binary; it owns `ise rebuild-all` and `ise compare` |

The trial adds no dependency to the product or to either suite.

## Strict mode

- **Frozen suite:** `crates/passuff-e2e/tests/e2e_runner.rs` calls
  `.fail_on_skipped()`, adds parsing errors to `failed`, exits non-zero when
  `failed > 0` or `total == 0`, and writes `total`, `passed`, `failed`, and
  `skipped` to `GIVN_RESULT_FILE`. An undefined or skipped step is a failure.
  The package is frozen and is not rebuilt.
- **Rebuilt component runner:** the builder authors
  `crates/passuff-cli/tests/features_runner.rs` inside the replaced source root.
  The rebuild must call `.fail_on_skipped()` and count skipped scenarios as
  failed. The trial checks the rebuilt runner's text and the four counts; a
  rebuilt runner that cannot fail on a skipped step is an
  `implementation defect`.
- **Step bodies:** the rebuilt step definitions are inside the replaced root and
  may be rewritten; the trial does not accept an empty step body. A step whose
  body only records that it ran is an `implementation defect` for that file.
- **Single-scenario command:** neither runner filters by scenario name. The
  frozen runner selects every `@e2e` scenario and the component runner accepts
  `GIVN_FEATURES` as a file or directory, not a name. A failure names the
  scenario in the transcript; the trial runs the whole frozen suite once.

## Trial procedure

### 1. Pre-trial revision

Commit the proposal, the design, `arc42.md`, the design review, the task list,
and `.givn-skip`. That commit is the trial revision.

### 2. Spec copy

```sh
rm -rf /tmp/passuff-s12-full /tmp/passuff-s12-full.yaml
givn ise rebuild-all \
  --at <trial-sha> \
  --into /tmp/passuff-s12-full \
  --manifest /tmp/passuff-s12-full.yaml
```

The report must show the removed source roots, the retained Specification
count, the removed file count, both checksums, and the end-to-end set size.
The command refuses on a dirty checkout or a path that exists. Save the output
as `evidence/ise/prepare.txt` and the manifest as
`evidence/ise/manifest.yaml`.

### 3. Baseline and builder

```sh
givn ise compare --manifest /tmp/passuff-s12-full.yaml \
  --copy /tmp/passuff-s12-full
```

The baseline must report `Specification files: match` and
`End-to-end tests: match`. Save it as `evidence/ise/compare-baseline.txt`.

Before the builder starts, move the published `passuff*` crate sources out of
the Cargo registry and record the scan:

```sh
mkdir -p /tmp/passuff-s12-registry-quarantine
find "$HOME/.cargo/registry/src" "$HOME/.cargo/registry/cache" \
  -maxdepth 2 -name 'passuff*' -print
# move each printed path under /tmp/passuff-s12-registry-quarantine
find "$HOME/.cargo/registry/src" "$HOME/.cargo/registry/cache" \
  -maxdepth 2 -name 'passuff*' -print
```

Both scans and the quarantine path go to `evidence/ise/sandbox.txt`.

One fresh builder works only inside `/tmp/passuff-s12-full`. It reads the copy's
`givn/specification.yaml`, `givn/specs/**`, `docs/**`, and `README.md`. It must
not read the checkout's `crates/**`, `scripts/**`, or `givn/archive/**`, the
givn worktree, the git object store, `crates/passuff-e2e`, or the quarantine
directory. It writes the whole product: the workspace manifest, the three crate
manifests, every production source file, the component step harness and its
support, the scripts, the bundled sound, and `builder-notes.md`.

`builder-notes.md` records:

- the inputs it read, with the exact paths;
- every file it authored, grouped by declared source root;
- every contract it had to assume, and why the Specification did not state it;
- every crate version it resolved, and the `cargo --offline` output that
  resolved it;
- every build or check attempt and the result;
- the clean-room statement: it did not read the checkout, the frozen package,
  the archive, or a published `passuff` source, and it did not edit any
  Specification file;
- the registry scan result.

The builder runs every cargo command with `--offline`. It never edits a
Specification file; adding a file under a declared Specification path is
refused by the spec copy rule.

### 4. Build and grade

The verifier, not the builder, builds the kept product in the copy:

```sh
cd /tmp/passuff-s12-full
cargo build --offline -p passuff --bins
cargo build --offline --locked -p passuff --bins
sha256sum target/debug/passuff target/debug/passuff-tui
```

Both build attempts are recorded; the second must succeed without changing the
lockfile. Save the log as `evidence/ise/build.txt` and the hashes plus
`--version`, `--help`, and the no-terminal `passuff-tui` exit as
`evidence/ise/binaries.txt`.

The rebuilt component check runs in the copy:

```sh
cd /tmp/passuff-s12-full
GIVN_FEATURES=/tmp/passuff-s12-full/givn/specs \
GIVN_RESULT_SCOPE=change \
GIVN_RESULT_FILE=<change>/evidence/ise/component-result.json \
  cargo test --offline --locked -p passuff --features test-support \
    --test features_runner
```

The frozen acceptance run runs from the checkout against the copy's binaries:

```sh
PASSUFF_BIN_DIR=/tmp/passuff-s12-full/target/debug \
GIVN_RESULT_FILE=<change>/evidence/ise/e2e-result.json \
  ./scripts/verify.sh --e2e --package crates/passuff-e2e
```

`verify.sh` builds the checkout product first, then runs the frozen package
with the overridden `PASSUFF_BIN_DIR`; the package reads its scenarios from the
checkout's `givn/specs` and the binary from the copy. The copy cannot run
`./scripts/verify.sh` in its e2e scope: the script requires
`crates/passuff-e2e`, which the copy does not hold. That is a recorded
difference, not a reason to copy the package. Save the full log as
`evidence/ise/e2e-run.txt` and the four counts as
`evidence/ise/e2e-result.json`.

**Post-trial correction.** The rebuilt runner selected 0 scenarios when
`GIVN_FEATURES` was this absolute directory (`component-result.json` reports
`total: 0`). The copy's declared `./scripts/verify.sh` selected 215 and passed
them; that run is `component-run-declared-surface.txt`. The gap table records
the difference as G2 (`acceptable undocumented change`: `docs/limits.md` does
not state the accepted `GIVN_FEATURES` forms).

### 5. Triage

Classify every difference between the rebuilt product and the retained
Specification with `D3`. The sources of differences are, in order:

1. every scenario that fails in `e2e-result.json` and its transcript;
2. every declared behavior with no working implementation in the copy;
3. every file the builder authored outside a declared `source:` root;
4. every Specification rule the builder recorded as an assumption or a gap.

Write `evidence/ise/gap-table.md` with one row per difference: the observable
difference, the outcome, the wrong artifact, the evidence path, and the
corrective route. Write `evidence/ise/neighbour-record.md` with the `D5`
tables. A `specification defect` names the document and the missing rule; it is
not fixed in this change.

### 6. Compare after the build

Run `givn ise compare` again and save
`evidence/ise/compare-after-build.txt`; both subjects must match. A mismatch is
a Specification edit inside the copy and fails the trial.

### 7. Restore the environment

Move the published `passuff*` sources back from the quarantine and record the
post-restore scan in `evidence/ise/sandbox.txt`. The trial copy under `/tmp` is
not archived and never promoted.

## Evidence layout

| Path | Content |
|---|---|
| `evidence/ise/manifest.yaml` | The prepare manifest, copied from `/tmp` |
| `evidence/ise/prepare.txt` | `givn ise rebuild-all` output |
| `evidence/ise/compare-baseline.txt` | Compare before the builder |
| `evidence/ise/compare-after-build.txt` | Compare after the build |
| `evidence/ise/sandbox.txt` | Registry scans, quarantine path, and restore |
| `evidence/ise/environment.txt` | `givn`, `rustc`, `cargo`, and platform versions |
| `evidence/ise/builder-notes.md` | The builder's inputs, assumptions, files, versions, attempts, and clean-room statement |
| `evidence/ise/authored-files.txt` | The builder's file list, grouped by declared source root |
| `evidence/ise/build.txt` | The copy's product build log |
| `evidence/ise/binaries.txt` | Binary hashes and the three smoke invocations |
| `evidence/ise/component-run.txt` and `component-result.json` | The rebuilt component check |
| `evidence/ise/e2e-run.txt` and `e2e-result.json` | The frozen acceptance run |
| `evidence/ise/gap-table.md` | One outcome per difference, with the wrong artifact |
| `evidence/ise/neighbour-record.md` | Every added path outside the Specification and the per-root rebuild counts |

## Visual Design Contract

- Interface: none
- Viewports: none, the change adds or visibly changes no screen
- Design system: none under `docs/design`; the unchanged terminal frame is
  specified by `docs/reference.md` and `docs/limits.md`

No screen is introduced or changed. The rebuilt product renders the same
terminal interface behind the unchanged frozen scenarios, and the confirmed
Persona `multi-account-developer` is its reader.

## E2E policy

No `@e2e` scenario is added, modified, or removed. The frozen package is the
acceptance authority and is not edited.

- **E2E runner command:** `./scripts/verify.sh --e2e --package
  crates/passuff-e2e`; `GIVN_RESULT_SCOPE=e2e`; every `@e2e` scenario in
  `givn/specs` runs through the frozen package.
- **E2E step definition location:** `crates/passuff-e2e/tests/steps/`, separate
  from the component steps, declared and frozen.
- **Local test infrastructure:** the package starts a mock GitHub server
  (`wiremock`) per scenario and drives the real binary through a pty
  (`portable-pty` + `vt100`). No container or external service.
- **E2E framework choice and justification:** cucumber-rs over the real binary;
  the pty is the real terminal interface, so no in-process double is used.
- **E2E strict-mode proof:** `.fail_on_skipped()`, undefined steps fail, parse
  errors count as failed, and a zero total exits 1.

The rebuilt component runner is a supporting check, not the acceptance
authority.

## Interaction Coverage Matrix

No inventory entry is added, changed, or removed by this change; every existing
entry keeps its recorded row and is graded through the frozen suite unchanged.
The matrix is the cross-reference the review checks; the owning use-case
documents are the inventory source.

| Inventory entry (capability: consumer action) | @e2e scenario title | Real interface | Driving mechanism |
|---|---|---|---|
| watch-scope: report only the watched scope across two accounts | Two accounts report only the watched scope | CLI | cucumber-rs step spawns the rebuilt `passuff`; mock GitHub per `Authorization` token |
| watch-status: show the watch state | A first check baselines the account and status reports the scope | CLI | Spawns the rebuilt `passuff check` then `status`; asserts stdout |
| activity-inbox: list unread activity | New activity reaches the inbox and reading all clears the unread count | CLI | Spawns `check`, `inbox`, `read --all`; asserts stdout and state file |
| activity-inbox: open the inbox in the terminal UI at the narrow budget | The inbox row is visible at the narrow terminal size | Terminal pty | pty at 80x24 through `portable-pty`; `vt100` parses the frame |
| activity-inbox: open an activity's detail at the wide budget | The activity detail opens at the wide terminal size and quits cleanly | Terminal pty | pty at 120x40; `Enter` opens the overlay; `vt100` reads the frame |
| activity-inbox: mark one activity record read | Reading one id marks that record and reports it | CLI | Spawns `read <id>`; asserts stdout and the state file |
| activity-inbox: reject a read with no id or with an id and `--all` | Read with no id and with an id and --all exits 2 | CLI | Spawns `read` with the conflicting arguments; asserts exit 2 |
| activity-inbox: reject an unknown activity id | Reading an unknown id exits 2 | CLI | Spawns `read <unknown>`; asserts exit 2 and stderr |
| activity-inbox: load a version-1 state file | A version-1 state file migrates on the next status | CLI | Writes a version-1 state; spawns `status`; asserts output and the rewritten file |
| activity-inbox: record a repeated activity delivery once | The same activity delivered twice produces one record | CLI | Two `check` passes over the same mock activity; asserts one record |
| activity-inbox: report a terminal below the minimum size | A terminal too short prints the minimum size | Terminal pty | pty below 40x12; `vt100` reads the size warning |
| inbox-navigation: navigate the watched pull requests and activity | Tab and back-tab switch the active pane | Terminal pty | pty; sends `Tab` and `BackTab`; `vt100` reads the focus line |
| initial-setup: set up passuff on first run | Setup writes a usable configuration | CLI | Spawns `setup` with scripted stdin; asserts exit 0 and the written TOML |
| initial-setup: keep an existing configuration without confirmation | Setup leaves an existing configuration alone without confirmation | CLI | Spawns `setup`, answers `n`; asserts the file is unchanged |
| initial-setup: reject an invalid watch repository and ask again | Setup rejects an invalid watch repository and asks again | CLI | Spawns `setup` with an invalid repository then a valid one; asserts the re-ask |
| pr-activity-notification: show the available commands | The CLI lists its commands | CLI | Spawns `passuff --help`; asserts the command list |
| pr-activity-notification: deliver one notification as a JSON line | A notification is delivered as one JSON line | CLI | Spawns `check` with `PASSUFF_NOTIFICATION_LOG`; reads the JSON line |
| pr-activity-notification: fail the check on a server error | A server error fails the check | CLI | mock GitHub returns 500; spawns `check`; asserts exit 1 and stderr |
| pr-activity-notification: fail the check when the endpoint is unreachable | An unreachable endpoint fails with a diagnostic | CLI | Points `graphql_url` at a closed port; asserts exit 1 and the diagnostic |
| pr-activity-notification: authenticate an account with an environment token | An environment token authenticates the account | CLI | Spawns `check` with `token_env` set; asserts the account resolves |
| pr-activity-notification: accept a token file readable only by its owner | A token file readable only by its owner is accepted | CLI | Writes a `0600` token file; spawns `check`; asserts success |
| pr-activity-notification: refuse a token file readable by other users | A token file readable by other users is refused | CLI | Writes a `0644` token file; asserts exit 2 and the refusal |
| pr-activity-notification: fail an account whose token belongs to another login | A token for another login fails the account | CLI | mock GitHub returns a different viewer; asserts exit 1 naming both logins |
| pr-activity-notification: quarantine a corrupt state file | A corrupt state file is quarantined and replaced | CLI | Writes invalid JSON; spawns `check`; asserts the quarantine and the new state |
| pr-activity-notification: notify a CI failure and its recovery | A failing check notifies and a recovery notifies | CLI | mock GitHub changes the rollup state; two `check` passes; asserts both transcripts |
| pr-activity-notification: record the viewer's own comment read without notifying | The viewer's own comment is recorded read and does not notify | CLI | mock GitHub returns the viewer's own comment; asserts the inbox and no notification |
| pr-activity-notification: ignore bot comments by default | A bot comment is ignored by default | CLI | mock GitHub returns a bot comment; asserts no notification |
| pr-activity-notification: record bot comments when they are enabled | A bot comment is recorded when bot comments are enabled | CLI | Enables `bot_comments`; asserts the summary and the unread count |
| pr-activity-notification: watch continuously and deliver one notification | The default command watches and delivers one notification | CLI | Spawns the watch loop; waits for the baseline, adds a comment, intercepts SIGINT; asserts one log line and exit 0 |

Every row has a non-empty driving mechanism. The review re-reads each `.feature`
title and each step body against this table. The 29 rows match the 29 frozen
`@e2e` scenario titles exactly; the counts reconcile with
`evidence/ise/e2e-run.txt`.

## External boundary cases

The trial keeps the existing adversarial cases and adds no new external
boundary. The frozen suite already drives: wrong identity (`A token for another
login fails the account`), malformed payload (`A server error fails the
check`), transport failure (`An unreachable endpoint fails with a diagnostic`),
corrupt persistence (`A corrupt state file is quarantined and replaced`), and
replay (`The same activity delivered twice produces one record`).

The trial's own boundaries are the spec copy, the toolchain, and the Cargo
registry: a dirty checkout refuses, an existing copy path refuses, a
Specification edit inside the copy fails the compare, a network-dependent build
fails under `--offline`, and a published `passuff` source in the registry is a
clean-room violation recorded in `sandbox.txt`.

## Local runnability

```sh
# in the copy, after the builder wrote the product
cd /tmp/passuff-s12-full
cargo build --offline -p passuff --bins
./target/debug/passuff --help
./target/debug/passuff --version

# from the checkout, against the copy's binaries
PASSUFF_BIN_DIR=/tmp/passuff-s12-full/target/debug \
  ./scripts/verify.sh --e2e --package crates/passuff-e2e
```

No service, container, or network dependency is added. The mock GitHub server
and the pty are started by the frozen package per scenario.

## Coverage process boundaries

No process changes; the rows stay as measured before this change.

| Process | Started by | Instrumented artifact | Profile output | Merge step | Non-zero production probe |
|---|---|---|---|---|---|
| Component runner | `measure-coverage.sh` (`cargo llvm-cov ... --test features_runner`) | `passuff-config`, `passuff-core`, `passuff` with `test-support` | `target/llvm-cov-target` profraw | `cargo llvm-cov report`, then `scripts/json_to_cobertura.py` | `crates/passuff-cli/src/render.rs` status lines |
| Library tests | `measure-coverage.sh` (`cargo llvm-cov ... --lib`) | The same three crates | The same profraw set | The same report | `crates/passuff-core/src/inbox.rs` view counts |

Measured: application source under `crates/`, excluding the frozen `e2e`
package. Excluded: tests, fakes, and the package. The trial adds no measured
process; the copy is outside the checkout and is not measured.
