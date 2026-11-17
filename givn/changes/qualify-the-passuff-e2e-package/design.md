# Design: qualify-the-passuff-e2e-package

## Scope

`S10` of `givn/ideation/ise-rebuildability/plan.md` needs a declared limit
document, two more seeded defects, retained evidence, and a recorded gate
result. No product behavior, scenario, tag, or step definition changes.

## Decisions

**Decides D1: `limits` keeps both entries.** `givn/specification.yaml` declares
`docs/limits.md` and `docs/arc42/02-architecture-constraints.md`. `docs/limits.md`
owns the terminal budgets, the exit codes, and the runner result-file contract;
the arc42 chapter keeps the stack, platform, API, legal, and organisational
constraints that are already declared. Replacing the chapter would drop those
constraints from the Specification and let a rebuild choose, for example, an
inbound webhook receiver without tripping a declared limit. Realized in
`givn/specification.yaml`.

**Decides D2: change strategy is `incremental modification`.** The change edits
the existing declaration, one existing qualification script, and the existing
documentation in place. No component is rebuilt, so no `Ise Test` mode is
recorded. Affected consumers: none; the product's CLI, TUI, and configuration
are untouched. Persisted state: unchanged; `state.json` and `config.toml` keep
the version and keys they have. Rebuild boundary: none.

## Deliverable 1: `docs/limits.md`

A new declared limit document with these sections:

| Section | Content |
|---|---|
| Terminal budgets | The 40x12 minimum and the two tested budgets. |
| What fits at 80x24 | The header, one watched row, the activity count line and rows, the footer hints; long titles clip at the pane edge. |
| What fits at 120x40 | The same panes with more rows, and a centered detail overlay of at most 100 columns by 30 rows that scrolls. |
| Clipping and paging | One line per watched row and per activity row; the panes clip at the frame edge; the activity window keeps the selected row in view; an overlay scrolls instead of growing. |
| Product exit codes | The four codes and the per-surface rows from `docs/reference.md`. |
| Runner result file | `GIVN_RESULT_FILE`, `GIVN_RESULT_SCOPE`, the four counts, and the acceptance rules below. |
| Sources | `docs/reference.md`, `scripts/verify.sh`, `givn/commands.yaml`, `crates/passuff-e2e/tests/e2e_runner.rs`. |

The runner contract stated there, from `src/proof.rs` in Givn 0.9.20 and the
project runner:

- The file must exist after the command exits 0.
- It is JSON with `total`, `passed`, `failed`, `skipped`; `scope` is optional
  and, when present, must equal `change` or `e2e` as the invocation selected.
- `total` is greater than zero.
- `failed` and `skipped` are zero.
- `passed + failed + skipped == total`.
- The runner removes the previous file before the run; a stale file is not a
  result.
- `./scripts/verify.sh --e2e` always runs the frozen package and ignores
  `GIVN_FEATURES`; the component scope honours both variables.

Terminal facts the document records, from the current renderer:

| Budget | Rows | What the frame shows |
|---|---|---|
| 40x12 minimum | below it | The single line `Terminal too small. Resize to at least 40x12.` |
| 80x24 | 3 header rows, 9 watched rows, 9 activity rows, 3 footer rows | Two watched rows, the count line, two activity rows; titles clip (`Add` instead of `Add retry loop`). |
| 120x40 | 3 header rows, 17 watched rows, 17 activity rows, 3 footer rows | Two watched rows, the count line, two activity rows; full titles; a detail overlay clamps to 100 columns by 30 rows. |

## Deliverable 2: declaration

`givn/specification.yaml` `limits` becomes:

```yaml
limits:
  - docs/limits.md
  - docs/arc42/02-architecture-constraints.md
```

`e2e`, `behavior`, `structure`, `contracts`, and `source` do not change.

## Deliverable 3: two seeded defects

Both are added to `scripts/qualify/` and to the `defect_names` array in
`scripts/qualify-e2e.sh`. The count becomes twenty.

### `key-parser-unbinds-quit`

Patched file: `crates/passuff-cli/src/tui/mod.rs`, in `map_key`.

```diff
         KeyCode::Char(character) => Some(match character {
             'j' => Key::Down,
             'k' => Key::Up,
-            'q' => Key::Quit,
+            'q' => Key::Char('q'),
```

Scenario that must fail: `The inbox row is visible at the narrow terminal size`
at its final step, `Then the TUI exits with code 0`. The quit binding no longer
produces `Key::Quit`, so `q` never closes the frame and the session never
exits. The other key bindings and the frame content stay correct, so only the
exit assertion fails.

### `inbox-paging-skips-selected-page`

Patched file: `crates/passuff-cli/src/tui/render.rs`, in `activity_lines`.

```diff
-    let start = app
-        .activity_selection
-        .saturating_add(1)
-        .saturating_sub(rows_available);
+    let start = app.activity_selection.saturating_add(rows_available);
```

Scenario that must fail: `The activity detail opens at the wide terminal size
and quits cleanly` at `Then the screen shows "comment by @reviewer"`. At
120x40 the activity pane has fourteen rows, so the offset skips past the single
activity row. The watched pane does not print the phrase, so the assertion
fails before Enter is pressed. The 80x24 scenario still passes because its
watched row contains the key it asserts, which is why the wide scenario is the
named detection.

The remaining eighteen defects keep their patches; their named scenarios are
pinned in the same table as the two new ones.

## Deliverable 4: qualification script

`scripts/qualify-e2e.sh` gains a `declare -A expected_scenario` map with one
entry per defect name and verifies containment, not just any failure:

```sh
scenarios="$(failing_scenarios "$logs/$name.suite.log")"
if ! grep -Fxq -- "${expected_scenario[$name]}" <<<"$scenarios"; then
  echo "DEFECT NOT DETECTED IN ITS SCENARIO: $name" >&2
  ...
  status=1
else
  echo "detected: $name [\"${expected_scenario[$name]}\"]"
  detected=$((detected + 1))
fi
```

`failing_scenarios` replaces `failing_scenario`: it collects every scenario
title whose body has a failing step, sorts and de-duplicates them, so a defect
that fails several scenarios is accepted when its named one is among them. A
defect with no expected entry, or with a failing scenario outside the
expectation, fails the run. The script continues after a mismatch so one run
reports every wrong expectation, and the final status is non-zero if any
mismatch occurred. The exit code, the clean-tree hash check, the revert check,
and the `N/M defects detected; clean tree green` line stay. Bash 5.2 is
available; the map uses no feature newer than associative arrays.

Expected scenarios for the existing eighteen:

| Defect | Scenario that must fail |
|---|---|
| `narrow-size-blind` | The inbox row is visible at the narrow terminal size |
| `status-unread-count` | A first check baselines the account and status reports the scope |
| `delivery-silent-ok` | A notification is delivered as one JSON line |
| `observe-empty-payload` | The activity detail opens at the wide terminal size and quits cleanly |
| `server-error-success` | A server error fails the check |
| `transport-failure-skipped` | An unreachable endpoint fails with a diagnostic |
| `setup-drops-accounts` | Setup writes a usable configuration |
| `credentials-token-file-mode` | A token file readable by other users is refused |
| `read-argument-guard` | Read with no id and with an id and --all exits 2 |
| `migration-not-persisted` | A version-1 state file migrates on the next status |
| `quarantine-not-moved` | A corrupt state file is quarantined and replaced |
| `ci-recovery-unseen` | A failing check notifies and a recovery notifies |
| `own-comment-not-read` | The viewer's own comment is recorded read and does not notify |
| `dedupe-replays-comment` | The same activity delivered twice produces one record |
| `scope-filter-dropped` | Two accounts report only the watched scope |
| `narrow-size-threshold` | A terminal too short prints the minimum size |
| `tab-does-not-switch` | Tab and back-tab switch the active pane |
| `watch-interval-capped` | The default command watches and delivers one notification |

## Deliverable 5: evidence under the change

| Path | Content | Produced by |
|---|---|---|
| `evidence/cargo-tree-passuff-e2e.txt` | The package graph and the absence of a production crate | `cargo tree --manifest-path crates/passuff-e2e/Cargo.toml -e normal,build,dev --prefix none` |
| `evidence/qualify-20-of-20.txt` | Every defect line and the closing summary | `./scripts/qualify-e2e.sh` |
| `evidence/verify-e2e-clean.txt` | The 29-scenario pass and the summary | `./scripts/verify.sh --e2e` |
| `evidence/verify-e2e-result.json` | The four counts the runner wrote | The same run's `GIVN_RESULT_FILE` |
| `evidence/defect-narrow-size.txt` | The failing scenario block | The suite run with `narrow-size-blind.patch` applied |
| `evidence/defect-key-parser.txt` | The failing scenario block | The suite run with `key-parser-unbinds-quit.patch` applied |
| `evidence/defect-inbox-paging.txt` | The failing scenario block | The suite run with `inbox-paging-skips-selected-page.patch` applied |
| `evidence/transcripts/80x24.txt` | One parsed screen from a real pty | `passuff-tui` at 80x24 |
| `evidence/transcripts/120x40.txt` | One parsed screen from a real pty | `passuff-tui` at 120x40 |
| `evidence/s10-gate.md` | Each `G9` row with its command and observed value | Hand-written from the files above |

The transcripts are captured with the package's own PTY and `vt100` harness in
a throwaway copy outside the repository: spawn `passuff-tui` with a fixture
`config.toml` and `state.json`, wait for the count line, sleep 700 ms so the
frame settles, and write `PtySession::screen()`. The fixture has one account,
two watched pull requests, and two activity records. No test file, manifest, or
configuration in the repository changes for the capture.

The defect blocks come from the same suite runner with the patch applied to a
scratch copy; `awk` extracts the named scenario block from the cucumber
transcript.

## Decisions already fixed by the project

| Concern | Decision | Existing artifact |
|---|---|---|
| Runner | `cucumber-rs`, custom `tests/e2e_runner.rs` with `.fail_on_skipped()` | `crates/passuff-e2e/tests/e2e_runner.rs` |
| E2E command | `./scripts/verify.sh --e2e`; scope `e2e`; runs the frozen package only | `givn/commands.yaml`, `scripts/verify.sh` |
| Component command | `./scripts/verify.sh`; scope `change`; honours `GIVN_FEATURES` | `givn/commands.yaml` |
| Strict-mode proof | Undefined steps fail the run; skipped scenarios fail via `.fail_on_skipped()`; `GIVN_RESULT_FILE` counts a parse error as failed and a zero total as a failure | `crates/passuff-e2e/tests/e2e_runner.rs` |
| Single-scenario run | Not available: the frozen runner executes the whole feature file. A seeded defect is observed by the whole suite, and its named scenario is extracted from the transcript. | `crates/passuff-e2e/tests/e2e_runner.rs` |
| E2E step location | `crates/passuff-e2e/tests/steps/` (`cli.rs`, `tui.rs`), separate from the component steps | package tree |
| GitHub boundary | `wiremock` server claimed per `Authorization` token | `crates/passuff-e2e/tests/harness/github.rs` |
| Terminal boundary | Real pty through `portable-pty`, parsed by `vt100` | `crates/passuff-e2e/tests/harness/pty.rs` |
| Coverage | `./measure-coverage.sh`, `./merge-coverages.sh`; two scopes, two reports | `givn/commands.yaml` |

## Contracts and components

No contract is added or changed. The product's contracts stay as declared in
the permanent use cases; the change edits a limit document, a declaration, and
test assets.

## Visual design contract

None. The change adds no screen and changes no rendering. It records the
viewport sizes the existing terminal screen is qualified at. The confirmed
Persona `multi-account-developer` is the reader of those budgets; no
interaction or visible string changes.

## Interaction coverage

No capability, inventory entry, scenario, or step definition is added or
changed. The 29 frozen scenarios and the use-case interaction tables stay as
they are, and `verify.e2e_command` still selects the package. The design does
not re-map the permanent inventory because this change touches none of it.

## External boundary cases

Unchanged and already covered by the frozen suite: wrong viewer identity (`A
token for another login fails the account`), malformed payload (`A server error
fails the check`), transport failure (`An unreachable endpoint fails with a
diagnostic`), corrupt persistence (`A corrupt state file is quarantined and
replaced`), and replay (`The same activity delivered twice produces one
record`). The two new seeded defects are sensitivity checks for the terminal
boundary, not new boundary cases.

## Local runnability

```sh
cargo build --locked -p passuff --bins
./scripts/verify.sh          # component scope
./scripts/verify.sh --e2e    # frozen package against the built binaries
./scripts/qualify-e2e.sh     # twenty seeded defects
```

No service, container, or network dependency is added. The mock GitHub server
and the pty are started by the package per scenario.

## Coverage process boundaries

No process changes; the rows stay as measured before this change.

| Process | Started by | Instrumented artifact | Profile output | Merge step | Non-zero production probe |
|---|---|---|---|---|---|
| Component runner | `measure-coverage.sh` (`cargo llvm-cov ... --test features_runner`) | `passuff-config`, `passuff-core`, `passuff` with `test-support` | `target/llvm-cov-target` profraw | `cargo llvm-cov report`, then `scripts/json_to_cobertura.py` | `crates/passuff-cli/src/render.rs` status lines |
| Library tests | `measure-coverage.sh` (`cargo llvm-cov ... --lib`) | The same three crates | The same profraw set | The same report | `crates/passuff-core/src/inbox.rs` view counts |

Measured: application source under `crates/`, excluding the frozen `e2e`
package. Excluded: tests, fakes, and the package. The populations do not change
with this change.

## Version freshness

No version is introduced or changed. The package versions come from the
existing `Cargo.toml` and `Cargo.lock`; the toolchain is pinned by
`rust-toolchain.toml`.
