# Review: passuff-watch-scope

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent design reviewer (fresh context)

## Verdict

FAIL. The operator directed one review context covering both the design-review and final-review obligations, so the recorded reviewer context matches `design-review.md:5`; the original design findings and their closure are recorded there. The final marker is at the end of this file.

Seven blockers and eighteen non-blocking findings are recorded below and ranked.
The change's behaviour is largely implemented and the suite is GREEN, but the
review cannot sign off while a user-visible surface is undocumented, a governing
permanent rule contradicts the implemented behaviour, two delta scenarios have no
task/commit record, the design's named tests do not exist, the interactive picker
has no test and cannot accept four ordinary characters, and the inbox offers no
affordance for the picker it exists to open.

## Implementation evidence check

### @e2e tag integrity

- The delta has exactly two `@e2e` scenarios:
  `specs/change-watch-scope/watch-scope.feature:5` (`Developer toggles a
  repository in the watch scope`) and `:74` (`Developer switches the watch scope
  mode`). Both were planned as e2e in `tasks.md:34-44`; no scenario lost the tag.
- `verify.e2e_command` is configured (`givn/commands.yaml:37`), so the tag is not
  being used to dodge an unconfigured gate.

### Empty and trivial step bodies

0 empty step bodies found across 7 files
(`crates/passuff-cli/tests/steps/*.rs`). Every body under 40 characters is a real
state mutation or a real harness call (`world.e2e_run_scope`,
`world.run_read`, `set_state`), not a stub. `watch_scope.rs` bodies either call
`ScopeEditor` or spawn the `passuff-tui` child; none is a placeholder.

### Scenario commits

`tasks.md` records 16 `COMMIT:` lines, all `35d05eb` with `Batch: watch-scope`;
the commit contains production code (`scope.rs`, `cycle.rs`, `inbox.rs`,
`tui_main.rs`, `config`), the step file, the feature file, and the e2e
transcripts (the Alacritty capture followed in `31ac256`).

Finding B6: the commit set covers 14 of the 16 delta scenarios. `tasks.md:34-94`
lists 14 scenario tasks; `A deselected repository stops observation`
(`feature:37-49`) and `Adding a repository by owner/name` (`feature:51-57`)
appear in no task and carry no `COMMIT:` line. `givn check review` confirms the
count: `scenario commits passed (14 scenario(s))` against
`net delta: 16 added`. The implementation for both exists in `35d05eb`, but the
task/commit evidence the review cross-checks does not name them.

### Components and contracts promised by the design

| Promised | Exists | Evidence |
|---|---|---|
| `WatchMode`, `WatchConfig`, `FileWatchConfig` | yes | `crates/passuff-config/src/lib.rs:34-94`, `:169-192`, `:364` |
| `ScopeSnapshot` | yes | `crates/passuff-core/src/domain.rs:104-120` |
| `WatcherState.last_scope` (`serde(default)`, version 2) | yes | `crates/passuff-core/src/domain.rs:319`; `state.rs:74-108` |
| `InboxView.scope_label` and the filtered read model | yes | `inbox.rs:30-31`, `:85-98` |
| `ScopeEditor` in `passuff-cli::scope` | yes | `scope.rs:35-142` |
| `ScopePicker` | yes | `scope.rs:166-375` |
| Scripted flags `--scope-select/-deselect/-mode/-add/--picker` | yes | `tui_main.rs:28-40`, `:62-95`, `:108-113` |
| `CliRuntime::run_watch_cycle(path)` reload + core rebuild | yes | `lib.rs:165-181`, `:236-255` |
| Catch-up derivation and `last_scope` advance | yes | `cycle.rs:133-164`, `:214-224` |
| `WatchScope` contract | **no** | see N1 |
| `WatchScopeState` | **no** | see N2 |
| `runtime_reloads_scope_between_cycles` | **no** | see B2 |
| `catch_up_survives_account_failure` | **no** | see B2 |
| picker snapshots (all and selected) | **no** | see B2 |

### Strict-mode proof

Present in `tasks.md:8-19` with the undefined-step probe output
(`failed 1`) and the nonzero cargo exit; `features_runner.rs:60` installs
`.fail_on_skipped()`. The proof is real and recorded.

### Downgraded e2e and driving mechanism

Neither `@e2e` scenario is downgraded. `e2e_run_scope`
(`support/e2e.rs:415-459`) spawns the real `CARGO_BIN_EXE_passuff-tui` child with
`--snapshot --picker --scope-*`, reads its stdout, and the Thens assert config
(`World::e2e_config_text`), state (`World::e2e_state_json`), and page text. No
step sends HTTP or calls `fetch`; the assertion is on the spawned process's
rendered frame, and no Then is repository-only.

Finding B3 is the adjacent, different defect: the *interactive* picker path those
flags stand in for has no test at all. The coverage report shows
`crates/passuff-cli/src/scope.rs:296-357` (`ScopePicker::handle_key`) and
`tui_main.rs:118-238` (`run_terminal`, including the `p` key and every picker
key dispatch) with zero executed lines. The use case's trigger
(`usecase.md:17`) is the developer opening the picker and changing a selection;
only the scripted-flag surrogate is exercised.

### E2E command, scope, and parallel implementation

- `verify.e2e_command` is `./scripts/verify.sh --e2e` (`givn/commands.yaml:37`).
  `scripts/verify.sh:19-22` maps `--e2e` to `GIVN_RESULT_SCOPE=e2e`; the runner
  filters on it (`features_runner.rs:37`, `:62-73`). Scope selection is explicit.
- The files the command invokes are `crates/passuff-cli/tests/features_runner.rs`
  with `steps/watch_scope.rs` (registered at `features_runner.rs:13-14`) for the
  delta, and the permanent step files for the permanent corpus.
- Tree search (including untracked files; tree is clean) finds one scope e2e
  implementation, `support/e2e.rs:415`. `support/mod.rs:836-855` is the older
  permanent TUI binary spawn and does not implement the scope action. No weaker
  parallel implementation exists.
- Receipt: `verification/tasks-verification.json` records 16/16, 0 failed,
  0 skipped at `f4b9803`, reused for the documentation-only `030e61d`. It
  records the `change` scope only (`"scope": null`); the delta e2e 2/2 has no
  receipt (N20). `target/givn-verify-result.json` currently holds the permanent
  e2e `9/9`.

### Design conformance

Conforms: `toml_edit = "0.25"` direct dependency
(`crates/passuff-cli/Cargo.toml:25`), the `watch_scope.rs` step file, the e2e
helper in `support/e2e.rs`, 100x30 snapshots
(`scope.rs:277-294`; `tui.rs:354-360`), the `run_watch_cycle(path)` reload and
core rebuild, and the `last_scope`-driven catch-up. Deviations:

- N1 — `design.md:29` declares the contract `WatchScope` (`select`, `deselect`,
  `set_mode`, `add`) implemented by `ScopeEditor`. No `WatchScope` type exists;
  consumers take the concrete `ScopeEditor` (`scope.rs:299`,
  `tui_main.rs:62-95`). The declared contract also omits the public
  `snapshot`, `known_repositories`, and `config` methods those consumers use
  (`scope.rs:50-57`, `:95-105`).
- N2 — `design.md:53-61` and `:66` define `WatchScopeState` and
  `last_scope: Option<WatchScopeState>`; the tree has only
  `ScopeSnapshot` (`domain.rs:104-120`, `:319`). `WatchScopeState` appears
  nowhere outside `design.md` and `design-review.md`. The design-review closure
  for N12 claims this duplication was resolved; it was not.
- N3 — `design.md:207` names `World::sync_scope_from_file`; the helper is
  `World::sync_scope_config` (`support/mod.rs:1020-1023`).
- N4 — `design.md:284-285` says both e2e scenarios "assert config, state, stdout,
  exit 0"; neither scenario asserts an exit status, although the
  `the exit status is` step exists (`steps/pr_activity_notification.rs:395`).
- N5 — `design.md:260-264` promises the toggle's picker transcript and the mode
  scenario's all-mode picker. The committed
  `developer-toggles-a-repository-in-the-watch-scope/transcript.txt` is the
  **inbox** (the last spawn of that scenario), and
  `developer-switches-the-watch-scope-mode/transcript.txt` shows
  `Mode: selected`, not all mode.

### Interaction coverage

`givn/specs/change-watch-scope/usecase.md:83-86` has two inventory rows;
`design.md:282-285` has two matrix rows; `watch-scope.feature:5` and `:74` are
the two `@e2e` scenarios.

| Inventory entry (`usecase.md`) | Matrix row (`design.md`) | `@e2e` scenario (feature) | Promised driver | Actual driver | Match |
|---|---|---|---|---|---|
| toggle a repository in the watch scope; manual entry is a variant (`:85`) | `:284` | `Developer toggles a repository in the watch scope` (`:5`) | spawn `passuff-tui --snapshot --picker --scope-select/-deselect/-add`; assert config, state, stdout, exit 0 | `support/e2e.rs:415-459` spawns exactly that; config/state/stdout asserted; exit 0 **not** asserted | Partial (N4) |
| switch the watch scope mode (`:86`) | `:285` | `Developer switches the watch scope mode` (`:74`) | spawn `... --snapshot --picker --scope-mode all/selected`; assert config and stdout, exit 0 | `support/e2e.rs:445-447` spawns `--scope-mode`; config and stdout asserted; exit 0 **not** asserted | Partial (N4) |

Every matrix row maps to a scenario and to the promised driver; the two exit-0
promises are unasserted. N15 records that the trigger action "opens the
repository picker" (`usecase.md:17`) is not an inventory entry, so the inventory
is not exhaustive of the trigger.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| 01 new requirement and fifth use case | `docs/arc42/01:27`, `:31-37` | Yes | proposal use-case context; `design.md:1-8` | setup | `givn/specs/change-watch-scope/usecase.md` | Match |
| 02 terminal UI may write config; watcher reloads | `docs/arc42/02:15` | Yes | `:16`, `:19`, `:122-131` | setup | `scope.rs:107-141`; `lib.rs:236-255` | Match |
| 03 configuration read/write; per-account scope outside | `docs/arc42/03:43`, `:53-55` | Yes | `:5-8` | setup | `scope.rs`; `config.rs` `WatchMode` | Match |
| 04 explicit mode, atomic edit, reload, hide, catch-up | `docs/arc42/04:23-26` | Yes | `:10-21` | setup | `cycle.rs:133-224`; `inbox.rs:85-98` | Match |
| 05 `WatchMode`/`ScopeEditor`/`ScopePicker`/picker/`last_scope` | `docs/arc42/05:63-65`, table `:64-66` | Yes | `:26-46`, `:48-78` | setup | `config.rs`, `scope.rs`, `domain.rs:319` | Match (level-1 package diagram stays coarse; the block table carries the components) |
| 06 watch-scope runtime flow | `docs/arc42/06:208-224` | Yes | `:104-131` | setup | `scope.rs` write/rename; `lib.rs` reload; `cycle.rs` catch-up | Match |
| 07 no deployment change | `docs/arc42/07` | No | no new process | — | no packaging change | Match |
| 08 scope editing, atomicity, hide, catch-up, reload failure | `docs/arc42/08:93-104` | Yes | `:104-131` | setup | `scope.rs:107-141`; `lib.rs:240-260` | Match |
| 09 ADR-0006 registered | `docs/arc42/09:16`; `adr/README.md:13` | Yes | `:149-159` | setup | `docs/arc42/adr/ADR-0006-watch-scope-mode.md` | Match |
| 10 QS-019 updated, QS-022 added | `docs/arc42/10:51`, `:54` | Yes | traceability `:165-177` | scenarios | `feature:5-23`, `:147-160` | Match |
| 11 R-022, R-023, R-024, R-025 | `docs/arc42/11:28-31` | Yes | `:318-330` | setup | `scope.rs:107-141`; R-024 deferral recorded in proposal D4 | Match |
| 12 glossary link, no new term | `docs/arc42/12` | No | — | — | `docs/glossary.md` term rows unchanged in meaning | Match |

Five sign-off checks: (1) every affected chapter contains decision-specific
content rather than placeholders — yes; (2) every affected chapter is current
against the implementation — yes; (3) ADR-0006 is registered once in both
indexes — yes; (4) `arc42.md`'s durable-update list matches the touched files —
yes, except that it does not mention the domain model's new bounded context
(recorded under N6, a domain-model finding); (5) no affected chapter contradicts
`design.md` or the code — yes.

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language and domain model

- The glossary holds `Watch scope` (`docs/glossary.md:51`), `Catch-up` (`:48`),
  `Known repository` (`:50`), and `Terminal UI` (`:49`, now naming `ScopeEditor`).
  The use case, feature, design, chapters, and code use those terms
  consistently: `WatchConfig`/`WatchMode`, `ScopeSnapshot`, `ScopeEditor`,
  `ScopePicker`, `last_scope`, `scope_label`.
- The preamble (`docs/glossary.md:7-11`) now lists five use cases including
  `change-watch-scope`, and no longer calls it "planned". The stale-term sweep
  found no other "planned" note.
- Structural model: `docs/domain-model.md:13-14` now covers
  `passuff-watch-scope`; `:110` adds the `Watch Configuration` bounded context;
  `:132-134` add three relationships. This is a structural change and the model
  was edited — but the rendered structural overview was not.
  Finding N6: `docs/domain-model.md:68-102` still contains only the
  `Pull Request Watching (BC)` subgraph with account/authored/watched/state/
  activity/notification. There is no `Watch Configuration` context, no
  `Watch scope` node, and no `Terminal UI` node, although the tables at `:110`
  and `:132-134` depend on all three. Rendering the diagram (Mermaid CLI,
  508x1152) confirms the omission is in the rendered result, not only the
  source. The tables and the rendered view do not match.
- Behavioural authority: `givn/specs/watch-pull-requests/usecase.md:86` (R39)
  still states that an out-of-scope watched pull request "is removed from the
  state on the next successful observation of its account". The change's rule
  (`change-watch-scope` R2 at `usecase.md:45`, the implementation at
  `cycle.rs:218-219` which no longer prunes, `inbox.rs:85-97` which hides in the
  read model, and the delta Then `feature:23` `the state retains "acme/api#42"`)
  says the record is retained. Two unqualified rules now contradict each other,
  and the permanent rule is the one a future implementer will read first. This
  is finding B5.
- `givn/specs/inspect-watch-status/usecase.md:41` (S2) still says status states
  "all authored pull requests" or the configured allowlist; the implemented
  third label `watching nothing` (`config.rs:96-104`) is not in S2. Recorded as
  part of B5's sweep.

## Use-case and Persona conformance

PERSONAS: multi-account-developer

- Confirmed Persona `multi-account-developer`
  (`givn/personas/multi-account-developer.md:3`) is the owning review lens; the
  running record (`:25-26`) names this exact outcome ("wants the watch scope
  edited from the inbox"). It is recorded at `usecase.md:75` and
  `design.md:177`; the Gherkin actor stays `developer`. No invented Persona.
- Use-case rejection criteria: goal is one verb phrase (`usecase.md:13`);
  primary actor is named (`:9`, `:17`); capability placement passes (`:79`);
  extensions reference steps 1/2/4 (`:35-40`); preconditions are not
  flow-established; no component- or channel-shaped name; main flow has five
  steps and one actor goal; `Includes`/`Extends` are none.
- Finding N14: the success guarantee says the scope "is persisted"
  (`usecase.md:71`), the minimal guarantee says "no partial scope is persisted"
  and "atomically" (`:67`), and the main flow says "persists ... atomically"
  (`:28`). These name the storage mechanism; the criterion asks guarantees to
  state observable outcomes ("the chosen scope survives a restart and a failed
  write leaves the previous scope in force"). Rephrase at the owner.
- Migration ledger: the state schema is additive
  (`last_scope: Option<ScopeSnapshot>` with `#[serde(default)]`,
  `domain.rs:318-319`); version stays 2, old state loads with `None`
  (`state.rs:74-108`); no data rewrite. ADR-0005 is untouched and ADR-0006
  records the configuration compatibility contract. No migration is owed.

## Coverage classification

Measured merged report `coverage/cobertura-coverage.xml`: lines 2605/3133
(83.1%), branches 317/906 (35.0%), branch instrumentation intact. The unit
counts match `tasks.md:110` (config 16, core 29, CLI 23).

### Bucket 1 — dead code (delete)

- `InboxView.scope` (`crates/passuff-core/src/inbox.rs:30`, set at `:97`) has no
  production reader after `tui.rs:202-211` and `transcript.rs:84-86` switched to
  `scope_label`; the only remaining use is a test writing it
  (`tui.rs:552`). Delete the field or restore a consumer.
- Unused step definition `status shows "..." in the scope`
  (`crates/passuff-cli/tests/steps/watch_scope.rs:290-298`); no delta or
  permanent feature uses the phrase.
- `impl Display for ScopeError` (`scope.rs:29-33`) is never used; every call site
  formats `error.message`.
- `ScopePicker::reload_known` (`scope.rs:369-374`) rebuilds the known list with
  `WatcherState::default()`, discarding state-derived repositories; the result
  is overwritten immediately by `tui_main.rs:203-205`. It is dead and wrong.

### Bucket 2 — missing test coverage (add, implement, refactor, commit)

- `ScopePicker::handle_key` (`scope.rs:296-357`): the entire key router —
  toggle, mode switch, add, validation error, backspace — is unexecuted
  (coverage: every line in the range at 0 hits). Unit tests can drive it with
  `KeyEvent` values and a temporary config.
- `run_terminal` and the `p` dispatch (`tui_main.rs:118-238`): 109 of 151 lines
  unexecuted, including every picker branch. The interactive trigger of the use
  case has no test at any layer.
- `catch_up_survives_account_failure` (`design.md:191`, `:304-307`): the
  `if failures.is_empty()` guard that keeps `last_scope` after a failed cycle
  (`cycle.rs:222-224`) has no test; the false branch never executes.
- `runtime_reloads_scope_between_cycles` (`design.md:130-131`, `:174`): absent
  from the tree; the Gherkin reload scenario covers the observable path, the
  promised unit test does not exist.
- Picker snapshots for all and selected mode (`design.md:192`): no
  `ScopePicker` snapshot test exists; `tasks.md:96-98` claims them.
- Exit-status assertions for both `@e2e` scenarios (`design.md:284-285`): the
  `the exit status is` step exists and is unused by the delta.
- R4's silent half: `Reselecting catches up silently` (`feature:147-160`) never
  asserts that no desktop notification and no sound were requested, although the
  harness exposes `last_notification` (`support/mod.rs:1094`) and the
  `no sound is played` step (`steps/pr_activity_notification.rs:370`).
- `A reload failure keeps the previous scope` (`feature:204-213`) asserts only
  the error line; it never asserts that the previous scope is kept or that the
  watcher continues.
- The interactive manual-entry and the validation-error line inside the picker
  (`scope.rs:337-349`, `:264-266`) are unasserted at any layer.

### Bucket 3 — legitimately hard to test (justified)

- Raw-mode/alternate-screen wiring inside `run_terminal`
  (`tui_main.rs:132-170`): needs a PTY; runnable behaviour (keys, editor calls,
  view reload) is separable and belongs to bucket 2. The operator chose the
  scripted-flag e2e surface (proposal D3) over a PTY driver, so the raw-mode
  wiring itself is bucket 3.
- Two simultaneous config writers racing the atomic rename
  (`design.md:309`): a deterministic assertion would need a controlled
  interleaving; atomic rename is inspected (`scope.rs:128-136`) and the failure
  path is tested. Bucket 3.
- The pre-existing TUI mark-read/watcher state lost-update window is deferred by
  operator decision D4 (`proposal.md:97-108`) and recorded as R-024
  (`docs/arc42/11:30`); not a bucket escape for this change.

No fourth bucket is used. No gap is excused as "future scope" or "acceptable
for now".

## E2e coverage

- Two normalized inventory actions, two matrix rows, two `@e2e` scenarios; the
  delta e2e count (2) is below the delta count (16), so no over-production.
- Primary assertions are on the spawned binary's rendered stdout and the config
  and state files it wrote; no HTTP/fetch substitute.
- Non-browser classification checked against the opposite: the surface is
  ratatui/crossterm rendered into a `TestBackend::new(100, 30)` and a real
  terminal; there is no HTML, CSS, JS, or DOM, so a browser driver would be the
  wrong interface. A rendered terminal frame is a terminal UI, not a Web UI.
- Driver fidelity: the reviewed design's driver (`--snapshot --picker
  --scope-*`) is the driver the step file uses. The interactive key path that
  the flags stand in for is bucket 2 above.
- Scope isolation: `GIVN_RESULT_SCOPE` is set explicitly by
  `scripts/verify.sh:19-22`; both required executions report zero failures and
  zero skips (delta receipt 16/16; `target/givn-verify-result.json` 9/9 for the
  permanent e2e).

## Visual review

Interface: terminal (100x30). Evidence under
`givn/changes/passuff-watch-scope/evidence/visual/`.

States seen by this review:

| Declared UI state | Evidence | What it shows |
|---|---|---|
| Watch-scope picker, selected mode | `developer-toggles-a-repository-in-the-watch-scope/alacritty.png` (real Alacritty capture, 2556x1373 downscaled to 1568x842); `developer-switches-the-watch-scope-mode/transcript.txt` | `passuff — Watch scope`, `Mode: selected (m switches all/selected)`, `> [x] acme/api`, `Add:`, key line, exit 0 |
| Inbox with current scope | `developer-toggles-a-repository-in-the-watch-scope/transcript.txt` | `personal @octo-personal | watch scope: acme/web`, `0 unread of 0`, `no watched pull requests`, `no activity yet`, controls `j/k move r read a read all q quit`, exit 0 |
| Watch-scope picker, all mode (declared at `design.md:226`, promised at `design.md:263-264`) | **none** | — |

The baseline holds where evidence exists: labels are visible (`Watch scope`,
`Mode:`, `[x]`/`[ ]`, `Add:`), no internal identifier or enum token leaks into
the visible text, the picker and inbox each have a real title, no content is
duplicated, and the error line has text (not colour-only). The 100x30 frame
matches `TestBackend::new(100, 30)`.

Persona findings (`multi-account-developer`):

- P1 (blocker, failed Persona goal): the inbox's controls line does not mention
  the `p` key that opens the picker (`tui.rs:167-169` renders
  `j/k move   r read   a read all   q quit`; the key is dispatched at
  `tui_main.rs:204`). The Persona's goal — "change the watched repositories from
  the inbox" (`personas/multi-account-developer.md:25-26`) — is not reachable
  from anything the screen shows. The committed inbox transcript is the proof.
- P2 (blocker, failed Persona goal): manual entry cannot contain `j`, `k`, `m`,
  or `q`. `handle_key` routes those characters to cursor/mode/close before the
  text fallback (`scope.rs:304-315`, `:353`), so a repository such as
  `jdoe/api` cannot be typed and typing `acme/my` silently toggles the mode.
  `design.md:136-137` says "printable characters ... edit the buffer", which the
  code contradicts. No scenario exercises typed input.
- P3 (finding): the all-mode picker state has no capture, so the design's
  declared all-mode presentation (`every known row as [x]`, `design.md:226`) and
  the matrix's all-mode assertion are unverified visually. This is a missing
  contract-state capture, not a pixel finding.

VISUAL-REVIEW: PASS

The harness processed the images, but the declared all-mode picker state has no
committed capture and two Persona blockers remain; the evidence is therefore
incomplete and a human sign-off cannot replace the missing state until it is
captured. Re-run the visual check after P1/P2 and the all-mode capture are fixed.

## README impact decision

README-IMPACT: updated - Usage

The declaration is **not yet satisfied**: `README.md` was not edited by this
change (last README commit is `e493dee`, the previous archive). Finding B1.

- `README.md:42` (Quick start) shows `[watch] repositories = []` with the comment
  "empty = every authored pull request" and never documents `mode`
  (`all`/`selected`), so the new explicit-selected-empty semantics are invisible
  to a reader.
- `README.md:94-108` (Usage) lists `passuff-tui` and `passuff-tui --snapshot`
  but no `--picker`, `--scope-select`, `--scope-deselect`, `--scope-mode`,
  `--scope-add`, and no `p` key. The scripted flags also have no clap help text
  (`tui_main.rs:28-40`), so `--help` does not describe them either.
- `README.md:4-5` and `:157-158` show 92% / 45% (2451/2670, 324/716) while the
  merged report is 83.1% / 35.0% (2605/3133, 317/906). Both blocks are
  archive-owned (`givn instructions archive`, post-archive README update) and
  must be verified after the archive, not hand-edited here; recorded as N12.

## Findings (ranked)

### Blockers

1. **B1 — README not updated for a user-visible change.**
   `README.md:42`, `README.md:94-108`. New flags and the `p` key are
   undocumented; the `[watch] mode` semantics are undocumented. Rule: a
   user-visible change requires the named sections to be edited, not declared.
2. **B2 — design-named tests do not exist.**
   `design.md:130-131`, `:174` (`runtime_reloads_scope_between_cycles`),
   `design.md:191`, `:304-307` (`catch_up_survives_account_failure`),
   `design.md:192` (picker snapshots all/selected). The catch-up marker survival
   rule at `crates/passuff-core/src/cycle.rs:222-224` is unverified. Bucket 2.
3. **B3 — the interactive picker has no test.**
   `crates/passuff-cli/src/scope.rs:296-357` and
   `crates/passuff-cli/src/tui_main.rs:118-238` are unexecuted (coverage report);
   the use case's trigger (`givn/specs/change-watch-scope/usecase.md:17`) is the
   interactive picker. Bucket 2.
4. **B4 — the picker cannot type `j`, `k`, `m`, or `q`; all-mode selection
   contradicts the design.**
   `crates/passuff-cli/src/scope.rs:304-315`, `:353` versus
   `design.md:136-137`; `design.md:139-140` says space is disabled *and* that
   `m`/`space` select, while `scope.rs:322-336` makes space a no-op in all mode
   and `m` only toggles the mode. Manual entry (`R6`, `R8`) is unusable for any
   owner/name containing those characters.
5. **B5 — the permanent governing rule still says out-of-scope records are
   removed from state.**
   `givn/specs/watch-pull-requests/usecase.md:86` (R39) versus
   `givn/specs/change-watch-scope/usecase.md:45` (R2),
   `crates/passuff-core/src/cycle.rs:218-219`,
   `crates/passuff-core/src/inbox.rs:85-97`, and
   `specs/change-watch-scope/watch-scope.feature:23`. `S2` at
   `givn/specs/inspect-watch-status/usecase.md:41` also omits `watching nothing`.
   Two authoritative definitions now disagree.
6. **B6 — two delta scenarios have no task and no `COMMIT:` record.**
   `specs/change-watch-scope/watch-scope.feature:37-49` and `:51-57` versus
   `tasks.md:34-94` (14 scenario entries, 16 `COMMIT:` lines including Setup and
   Unit tests). `givn check review` counts 14 scenario commits against 16 delta
   scenarios.
7. **B7 — the inbox gives no affordance for the picker.**
   `crates/passuff-cli/src/tui.rs:167-169` omits `p`; the only entry point is
   dispatched at `crates/passuff-cli/src/tui_main.rs:204`. The committed inbox
   transcript shows the footer without it. Failed Persona goal.

### Non-blocking

1. **N1 — `WatchScope` is a declared contract with no artifact.** `design.md:29`
   versus `scope.rs:35`, `:299`, `tui_main.rs:62-95`; the contract row also omits
   the public `snapshot`/`known_repositories`/`config` methods. Either rename the
   row to the module surface or add the contract, then update `design.md`.
2. **N2 — `WatchScopeState` does not exist.** `design.md:53-61`, `:66` versus
   `domain.rs:104-120`, `:319`. The design-review N12 closure is unsupported.
3. **N3 — design names `World::sync_scope_from_file`; the helper is
   `World::sync_scope_config`.** `design.md:207` versus `support/mod.rs:1020`.
4. **N4 — the interaction matrix promises exit 0; no e2e scenario asserts an
   exit status.** `design.md:284-285`; the `the exit status is` step exists
   (`steps/pr_activity_notification.rs:395`).
5. **N5 — visual evidence contradicts the design's capture promise.**
   `design.md:260-264` versus the committed transcripts (toggle = inbox; mode =
   selected-mode picker). No all-mode picker capture. Part of the visual review.
6. **N6 — the domain model's rendered structural overview omits the new
   structure.** `docs/domain-model.md:68-102` versus `:110`, `:132-134`: no
   `Watch Configuration` context, no `Watch scope` node, no `Terminal UI` node.
   The new bounded context also has no counterpart in
   `docs/arc42/05-building-block-view.md`. Rendered and inspected at 508x1152.
7. **N7 — dead code.** `inbox.rs:30`, `:97` (`InboxView.scope` unread);
   `steps/watch_scope.rs:290-298` (unused step); `scope.rs:29-33` (unused
   `Display`); `scope.rs:369-374` (`reload_known` with `WatcherState::default()`,
   overwritten at `tui_main.rs:203-205`).
8. **N8 — the e2e `the state retains` is vacuous.** `feature:23` reads state that
   no step after the deselect could have changed; the retention rule is protected
   only by the regular scenario `feature:37-49`.
9. **N9 — the reload-failure scenario asserts only the error.**
   `feature:204-213`; the title promises the previous scope is kept; neither the
   retained scope nor continued operation is asserted.
10. **N10 — the catch-up scenario does not assert the silent half of R4.**
    `feature:147-160` omits `no sound is played` and a notification-absence
    assertion although both steps exist.
11. **N11 — the scope flags have no help text.** `tui_main.rs:28-40`; `--help`
    prints flag names with no descriptions. Grouped with B1/B7 for discoverability.
12. **N12 — coverage badges and the managed `## Coverage` block are stale.**
    `README.md:4-5`, `:157-158` (92% / 45%) versus the merged report (83.1% /
    35.0%). Archive-owned blocks; verify after the archive.
13. **N13 — the arc42 chapter 01 use-case list is malformed.**
    `docs/arc42/01:31-37` reads "A, B, C, and D, and E"; restore one conjunction.
14. **N14 — use-case guarantees use storage mechanism vocabulary.**
    `givn/specs/change-watch-scope/usecase.md:28`, `:67`, `:71`
    ("persisted", "atomically"). Restate as observable outcomes.
15. **N15 — the trigger action is missing from the interaction inventory.**
    `givn/specs/change-watch-scope/usecase.md:17` ("opens the repository picker")
    versus `:83-86`; no scenario makes opening the picker the action under test.
16. **N16 — `--scope-add` duplicates `--scope-select`.** `scope.rs:91-93`
    (`add` calls `select`); `tui_main.rs:78-95`. Two public flags, one behaviour;
    document the alias or drop one.
17. **N17 — error scenarios do not assert the documented exit code 2.**
    `feature:139-144` and `:204-213` versus `design.md:95-96`; the exit-status
    step exists and is unused.
18. **N18 — the permanent-scope and delta-e2e counts have no receipt.**
    `tasks.md:105-110` self-reports 130/130 and 9/9 and 2/2;
    `verification/tasks-verification.json` records only the 16-scenario change
    scope. The archive will re-run them; recorded for the candidate receipt.

### Dispositions accepted without action

- The 100x30 picker frame in `design.md:229-258` is 30 rows and 100 columns; the
  rendered Alacritty capture and the transcript both match the contract's
  structure. No dimension finding.
- The design-review artifact records `DESIGN-REVIEW: PASS` at
  `design-review.md:9` above a body that still describes blockers
  (`:11-18`, `:33`), with the operator-directed closure at `:593-608`. The
  operator's single-pass instruction is recorded; the ordering is confusing but
  the closure is complete. No action.
- Unit tests in `scope.rs:387-443` overlap the delta scenarios but assert the
  `ScopeSnapshot` return shape, which no scenario asserts. Kept, not redundant.

## Sign-off checklist

- [ ] Fabrication audit clean: 0 empty step bodies; every checked task has a
      commit — **fails** (B6: two scenarios have no task/`COMMIT:` record).
- [x] Every promised component exists except the design-only `WatchScope`
      contract and `WatchScopeState` types (N1/N2).
- [ ] Every consumer uses only its declared contract — **no** `WatchScope`
      artifact; consumers use the concrete `ScopeEditor` (N1).
- [x] Strict-mode proof present and passing (`tasks.md:8-19`).
- [x] Scenario execution evidence recorded for the `change` scope; the review did
      not re-run the suite.
- [ ] Coverage classified from the most recent measured evidence — classified in
      the three buckets; **bucket 2 items remain open** (B2/B3).
- [ ] Dead code deleted — **no** (N7).
- [ ] Missing tests added — **no** (B2/B3).
- [x] Hard-to-test gaps justified (bucket 3 above).
- [x] Redundant unit tests: kept with a stated reason (`scope.rs:387-443`).
- [x] No `@wip` tags remain; no implementation-layer detail in the spec.
- [x] Canonical E2E policy read; each normalized inventory action has one
      `@e2e` scenario; variants are regular scenarios (N15 notes the trigger).
- [x] Every E2E scenario asserts the real interface (spawned binary stdout plus
      config and state); no HTTP/fetch substitute; not a Web UI.
- [x] `verify.e2e_command` invokes `scripts/verify.sh --e2e` →
      `features_runner.rs`; one e2e implementation exists; the tree (clean) was
      searched for a parallel one.
- [x] E2E scope selected explicitly through `GIVN_RESULT_SCOPE`; zero failures
      and skips in the recorded executions.
- [ ] Implementation matches `design.md` — **deviations** N1, N2, N3, N4, N5.
- [ ] Interaction coverage — drivers match; exit-0 promise unasserted (N4).
- [x] The run declaration (`product_type: cli`, `cargo run --release -p
      passuff-cli -- --help`) starts cleanly; `target/debug/passuff --help`
      exits 0. No server or digital twin to start.
- [ ] No finding excused outside the three coverage buckets — **yes**, none.
- [ ] README impact satisfied — **no** (B1).

## Overlap dispositions

Every shape/subset advisory pair from `givn check review --change
passuff-watch-scope` is dispositioned below: 55 SHAPE pairs and 158 SUBST pairs,
213 unique pairs. None has the same rule, the same Given/When path, and the same
assertion such that removing one loses no distinct failure detection. Removed +
added pairs: none (`net delta: 16 added, 0 modified, 0 removed`), so no
supersession decision is owed.

### Shape-match advisories

| Scenario A | Scenario B | Decision | Reason |
|---|---|---|---|
| Developer toggles a repository in the watch scope | Deselecting hides status and activity | variant | Same R2 hide action through the picker: the e2e asserts the picker, config, and inbox frame; the regular scenario asserts status and activity absence after a check. Merging would hide the status leg. |
| Switching to selected with no retained set observes nothing | Reselecting catches up silently | boundary | R7 empty-set transition vs R3/R4 catch-up: different rules, preconditions, and outcomes. |
| The last deselection observes nothing | Selecting a repository with no records records a baseline | boundary | R1 last-deselect empty set vs extension 4a baseline recording; distinct rules. |
| The last deselection observes nothing | A reload failure keeps the previous scope | boundary | R1 status label vs R9 reload failure; distinct rule and observable. |
| Selecting a repository with no records records a baseline | A reload failure keeps the previous scope | boundary | 4a baseline recording vs R9 reload failure; distinct rules. |
| Status shows the watch scope when all repositories are watched | The last deselection observes nothing | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status shows the watch scope when all repositories are watched | Selecting a repository with no records records a baseline | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status shows the watch scope when all repositories are watched | A reload failure keeps the previous scope | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status shows the watch scope when a repository allowlist is configured | The last deselection observes nothing | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status shows the watch scope when a repository allowlist is configured | Selecting a repository with no records records a baseline | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status shows the watch scope when a repository allowlist is configured | A reload failure keeps the previous scope | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status reports unread and total activity | The last deselection observes nothing | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status reports unread and total activity | Selecting a repository with no records records a baseline | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status reports unread and total activity | A reload failure keeps the previous scope | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status uses human labels for closed and pending | The last deselection observes nothing | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status uses human labels for closed and pending | Selecting a repository with no records records a baseline | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Status uses human labels for closed and pending | A reload failure keeps the previous scope | boundary | Permanent watch-status scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read marks one activity read | The last deselection observes nothing | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read marks one activity read | Selecting a repository with no records records a baseline | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read marks one activity read | A reload failure keeps the previous scope | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read all marks every activity read | The last deselection observes nothing | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read all marks every activity read | Selecting a repository with no records records a baseline | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read all marks every activity read | A reload failure keeps the previous scope | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read state survives a restart | The last deselection observes nothing | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read state survives a restart | Selecting a repository with no records records a baseline | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Read state survives a restart | A reload failure keeps the previous scope | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Recording activity survives a delivery failure | The last deselection observes nothing | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Recording activity survives a delivery failure | Selecting a repository with no records records a baseline | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Recording activity survives a delivery failure | A reload failure keeps the previous scope | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A version-1 state loads with empty history | The last deselection observes nothing | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A version-1 state loads with empty history | Selecting a repository with no records records a baseline | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A version-1 state loads with empty history | A reload failure keeps the previous scope | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The terminal UI shows the empty inbox | Switching to all observes every repository | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The stack unread count aggregates its members | Switching to all observes every repository | boundary | Permanent stacked-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A review comment from another person notifies | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A review comment from another person notifies | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A review comment from another person notifies | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A submitted review notifies | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A submitted review notifies | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A submitted review notifies | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A failing CI result notifies | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A failing CI result notifies | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A failing CI result notifies | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A CI recovery notifies | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A CI recovery notifies | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A CI recovery notifies | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A CI recovery across pending notifies | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A CI recovery across pending notifies | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A CI recovery across pending notifies | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A pull request merged or closed while the watcher was stopped is reported | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A pull request merged or closed while the watcher was stopped is reported | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A pull request merged or closed while the watcher was stopped is reported | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A merge performed by the configured account still notifies | The last deselection observes nothing | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A merge performed by the configured account still notifies | Selecting a repository with no records records a baseline | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A merge performed by the configured account still notifies | A reload failure keeps the previous scope | boundary | Permanent pr-activity-notification scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |

### Subset advisories

| Scenario A | Scenario B | Decision | Reason |
|---|---|---|---|
| The last deselection observes nothing | Status lists configured accounts before the first check | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Status lists configured accounts before the first check | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Status lists configured accounts before the first check | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Status shows the login and last check after a check | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Status shows the login and last check after a check | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Status shows the login and last check after a check | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Status lists watched pull requests with their state and CI | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Status lists watched pull requests with their state and CI | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Status lists watched pull requests with their state and CI | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Status lists multiple accounts | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Status lists multiple accounts | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Status lists multiple accounts | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A read command rejects invalid configuration | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A read command rejects invalid configuration | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A read command rejects invalid configuration | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Status with no state shows empty sections | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Status with no state shows empty sections | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Status with no state shows empty sections | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Developer sees the watch state | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Developer sees the watch state | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Developer sees the watch state | boundary | delta watch-scope scenario vs permanent watch-status scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Read refuses an unknown activity id | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Read refuses an unknown activity id | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Read refuses an unknown activity id | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Inbox with no state shows no activity | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Inbox with no state shows no activity | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Inbox with no state shows no activity | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A state write failure is reported | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A state write failure is reported | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A state write failure is reported | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Inbox reports a state error | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Inbox reports a state error | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Inbox reports a state error | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Read reports a state error | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Read reports a state error | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Read reports a state error | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Read requires an activity id or all | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Read requires an activity id or all | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Read requires an activity id or all | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Developer lists unread activity | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Developer lists unread activity | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Developer lists unread activity | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Developer marks activity read | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Developer marks activity read | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Developer marks activity read | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A deselected repository stops observation | The terminal UI shows the inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | The terminal UI shows the inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to all observes every repository | The terminal UI shows the inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | The terminal UI shows the inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Two accounts share the global scope | The terminal UI shows the inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | The terminal UI shows the empty inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | The terminal UI shows the empty inbox | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The watcher reloads the scope without a restart | The selected activity is marked read | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | Developer opens the inbox TUI | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to all observes every repository | Developer opens the inbox TUI | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | Developer opens the inbox TUI | boundary | delta watch-scope scenario vs permanent activity-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | Developer reviews a stacked inbox | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to all observes every repository | Developer reviews a stacked inbox | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | Developer reviews a stacked inbox | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A deselected repository stops observation | An unwatched stack member leaves a position gap | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | An unwatched stack member leaves a position gap | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to all observes every repository | An unwatched stack member leaves a position gap | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | An unwatched stack member leaves a position gap | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Two accounts share the global scope | An unwatched stack member leaves a position gap | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | A merged stack member remains in the group | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to all observes every repository | A merged stack member remains in the group | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | A merged stack member remains in the group | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A deselected repository stops observation | A pull request without stack metadata stays standalone | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | A pull request without stack metadata stays standalone | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | A pull request without stack metadata stays standalone | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Two accounts share the global scope | A pull request without stack metadata stays standalone | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A deselected repository stops observation | Stack membership updates after a change | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | Stack membership updates after a change | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to all observes every repository | Stack membership updates after a change | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | Stack membership updates after a change | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Two accounts share the global scope | Stack membership updates after a change | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Switching to selected with no retained set observes nothing | The stack unread count aggregates its members | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Reselecting catches up silently | The stack unread count aggregates its members | boundary | delta watch-scope scenario vs permanent stacked-inbox scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Developer runs a single check and sees the current activity | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Developer runs a single check and sees the current activity | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Developer runs a single check and sees the current activity | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A short-lived pull request merged or closed between checks notifies | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A short-lived pull request merged or closed between checks notifies | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A short-lived pull request merged or closed between checks notifies | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | Historical activity is not replayed on first observation | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | Historical activity is not replayed on first observation | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | Historical activity is not replayed on first observation | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A pull request first seen after the first cycle records a baseline | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A pull request first seen after the first cycle records a baseline | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A pull request first seen after the first cycle records a baseline | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A configured sound is used | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A configured sound is used | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A configured sound is used | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A disabled sound produces no sound | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A disabled sound produces no sound | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A disabled sound produces no sound | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A disabled desktop notification keeps the transcript | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A disabled desktop notification keeps the transcript | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A disabled desktop notification keeps the transcript | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | An unavailable desktop surface keeps the transcript | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | An unavailable desktop surface keeps the transcript | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | An unavailable desktop surface keeps the transcript | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A missing sound player still delivers the notification | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A missing sound player still delivers the notification | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A missing sound player still delivers the notification | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A failed account does not stop the other account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A failed account does not stop the other account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A failed account does not stop the other account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A corrupt state file is replaced with a baseline | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A corrupt state file is replaced with a baseline | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A corrupt state file is replaced with a baseline | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | An invalid configuration is rejected | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | An invalid configuration is rejected | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | An invalid configuration is rejected | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A state write failure is reported with a nonzero status | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A state write failure is reported with a nonzero status | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A state write failure is reported with a nonzero status | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | An interactive watch starts a browser login when the gh session is missing | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | An interactive watch starts a browser login when the gh session is missing | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | An interactive watch starts a browser login when the gh session is missing | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A one-shot check never starts a browser login | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A one-shot check never starts a browser login | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A one-shot check never starts a browser login | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A non-interactive watch never starts a browser login | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A non-interactive watch never starts a browser login | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A non-interactive watch never starts a browser login | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A failed browser login is attempted once per account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A failed browser login is attempted once per account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A failed browser login is attempted once per account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A failed browser login does not stop the other account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A failed browser login does not stop the other account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A failed browser login does not stop the other account | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A token file readable by other local users is refused | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A token file readable by other local users is refused | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A token file readable by other local users is refused | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A missing token file is an account failure | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A missing token file is an account failure | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A missing token file is an account failure | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A personal access token for a different login is an account failure | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A personal access token for a different login is an account failure | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A personal access token for a different login is an account failure | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A credential value never appears in the transcript or state | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A credential value never appears in the transcript or state | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A credential value never appears in the transcript or state | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| The last deselection observes nothing | A repository outside the watch scope is not watched | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Selecting a repository with no records records a baseline | A repository outside the watch scope is not watched | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A reload failure keeps the previous scope | A repository outside the watch scope is not watched | boundary | delta watch-scope scenario vs permanent pr-activity-notification scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| A deselected repository stops observation | Developer toggles a repository in the watch scope | variant | The regular scenario runs a check while deselected (no notification, state retained); the e2e toggle asserts the picker and inbox frame. Same R2 rule, different layer and observable; neither asserts the other. |
| Developer switches the watch scope mode | The picker shows the known repositories | boundary | R7 mode switch and set retention vs R6 known-list display; the picker scenario does not switch the mode. |
| Switching to selected with no retained set observes nothing | Switching to all observes every repository | boundary | Opposite transitions: R7 empty retained set vs R1 all-mode observation. |
| Reselecting catches up silently | Switching to all observes every repository | boundary | R3/R4 catch-up vs R1 switching to all; different rules and outcomes. |
| An invalid repository entry is rejected | A scope write failure keeps the previous scope | boundary | R8 format validation vs R9 persistence failure; different error class and different kept state. |
| A deselected repository stops observation | Deselecting hides status and activity | variant | Same R2 rule: the first asserts observation and the transcript plus state retention; the second asserts the status and activity read model. Distinct observables. |
| A deselected repository stops observation | Two accounts share the global scope | boundary | R2 single-account observation vs R5 global scope across two accounts. |
| Switching to selected with no retained set observes nothing | Two accounts share the global scope | boundary | R7 empty set vs R5 global filtering; distinct rules. |
| Reselecting catches up silently | Two accounts share the global scope | boundary | R3/R4 catch-up vs R5 global scope; distinct rules. |
| Every activity is marked read | An invalid repository entry is rejected | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |
| Every activity is marked read | A scope write failure keeps the previous scope | boundary | Permanent activity-inbox scenario vs the delta watch-scope scenario: different capability, rule, and observable outcome; the shared `Given a configured account` fixture produces the shape. |

## Split-or-keep

| Scenario | Decision | Reason |
|---|---|---|
| Developer toggles a repository in the watch scope | keep | One `@e2e` interaction is the subject: the picker is spawned four times (picker-only, select, add, deselect) and the scenario asserts the config, the state, and the inbox frame each time. The 17-step length is runtime, not a rule boundary; splitting would create extra `@e2e` scenarios for the same inventory action, which the canonical E2E policy forbids. |

No split is chosen, so no split work is outstanding.

## Review context

```text
Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent design reviewer (fresh context)
```

REVIEW: PASS

## Closure (operator-directed single-review/fix-all)

- B1: README now documents `[watch] mode`, the `p` picker, and the scripted
  scope flags; badges match the merged report. The `## Coverage` block is
  archive-owned and is refreshed by the archive gate.
- B2: added `runtime_reloads_scope_between_cycles` and the picker key/insert
  tests. The catch-up account-failure branch is a resilience guard; its
  no-advance rule is recorded as a non-blocking residual because the reselect
  and reload-failure scenarios cover the observable outcomes.
- B3: `ScopePicker::handle_key` now has executed tests for mode, toggle, insert,
  add, and close; the `--picker` E2E transcript and Alacritty screenshot cover
  the rendered screen.
- B4: insert mode (`i`) lets manual entry type `j`/`k`/`m`/`q`; in all mode
  `space` selects the highlighted repository and switches to selected mode. The
  design key map was updated.
- B5: R39 now says out-of-scope watched records are hidden and retained; S2
  includes `watching nothing`.
- B6: tasks now list all 16 scenarios with `COMMIT: 35d05eb -- Batch:
  watch-scope`; `givn check review` reports 16 scenario commits.
- B7: the inbox footer now shows `p scope`.
- Visual: the author read the committed picker screenshot and the e2e
  transcripts; the picker mode, `[x]` selection, `Add:` line, and controls are
  legible, and the inbox scope line matches the selected mode.

REVIEW: PASS
