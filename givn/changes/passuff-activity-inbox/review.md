# Review: passuff-activity-inbox

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context), re-review after the second FAIL
PERSONAS: multi-account-developer

The context string repeats `design-review.md:4` verbatim because the review gate
requires one recorded reviewer context for the independent review; this pass ran
in its own fresh context with no author history and changed no code, test, spec,
design, task, ADR, glossary, coverage, or README file.

Result: **FAIL**. Five blockers remain (B1-B5) plus six non-blocking findings
(N1-N6). `givn check review` exits 1 on the proposal's capability-routing row
before any judgement of the body.

## Implementation evidence check

### @e2e tag integrity

| Feature file | Planned `@e2e` | As built | Change |
|---|---|---|---|
| `givn/changes/passuff-activity-inbox/specs/inspect-watch-status/watch-status.feature` | 1 (status) | 1 (`Developer sees the watch state`) | added, matches the interaction inventory |
| `givn/changes/passuff-activity-inbox/specs/review-activity-inbox/activity-inbox.feature` | 2 (inbox, read) | 2 (`Developer lists unread activity`, `Developer marks activity read`) | added, matches the inventory |
| `givn/changes/passuff-activity-inbox/specs/watch-pull-requests/pr-activity-notification.feature` | 0 | 0 | the watch scope adds no consumer action; `start continuous watching` keeps its permanent `@e2e` |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | 4 | unchanged |

No scenario lost an `@e2e` tag. Delta `@e2e` count 3 < delta scenario count 24,
so the `--e2e` gate is not saturated. No `@wip` tag remains in the delta or the
permanent specs.

### Empty step bodies

99 annotated step definitions across 4 files
(`activity_inbox.rs` 11, `watch_status.rs` 11, `pr_activity_notification.rs` 71,
`pr_activity_notification_e2e.rs` 6). **One empty body found**:

- `crates/passuff-cli/tests/steps/activity_inbox.rs:44-45` —
  `async fn reopens(_world: &mut World) {}`. The scenario
  `Read state survives a restart` (`activity-inbox.feature:84-94`) names a
  restart that the step never performs. The scenario's assertion still holds
  against the file-backed `JsonStateStore` because `run_read` and `run_status`
  each build a fresh `Inbox` over the file, but the step itself is fabricated
  (step 0.1). Finding B2.

### Scenario commit evidence

| Checked scenarios | `COMMIT:` entries | Batches named | Evidence |
|---|---|---|---|
| 21 non-e2e tasks | 21 | `state-history` (3), `watch-scope` (1), `inbox-service` (9), `cli-read` (8) | every line names `1f9a3c8` and its batch |
| 3 `@e2e` proofs | 3 | `cli-read` (1), `inbox-service` (2) | same revision, batch named |

`git show --stat 1f9a3c8` touches production code
(`crates/passuff-core/src/{domain,state,cycle,inbox}.rs`,
`crates/passuff-config/src/lib.rs`, `crates/passuff-cli/src/{cli,lib,main,inbox_cli,transcript}.rs`),
both new step files, the shared harness, the docs, and the ADR. The commit body
names all four batches (`Batches: state-history, watch-scope, inbox-service, cli-read`).
No scenario commit is comment-, tag-, task-, feature-, or transcript-only. The
change is not marked `CHARACTERIZATION`.

Preflight evidence, read from artifacts (this review does not re-run the suite):

- delta 24/24 and permanent 80/80 — `tasks.md:40-44`;
- change receipt `verification/tasks-verification.json` (`status: passed`,
  `head 27c6887`, `runner_exit: 0`, `total 24, passed 24`), reused for the
  documentation-only revision `be7c5d4`; `verification/tasks-result.json` is
  `{"failed":0,"passed":24,"skipped":0,"total":24}`;
- delta `@e2e` 3/3 and permanent `@e2e` 4/4 (`tasks.md:40-44`);
- unit 44 = 5 + 16 + 23 (passuff-cli, passuff-core, passuff-config);
- strict-mode probe: `tasks.md:11-18` records `undefined Given`,
  `givn test-result: total 1, passed 0, failed 1, skipped 0`, `cargo exit status: 1`.

### Promised components and contracts

| Promised in `design.md` | Present | Notes |
|---|---|---|
| `SnapshotStore` / `JsonStateStore` v2 | `crates/passuff-core/src/state.rs:12-180` | `version`, `activity`, `next_activity_id`; v1 rewrite at `:119-129` |
| `InboxService` (`view`, `mark_read`, `mark_all_read`) | `crates/passuff-core/src/inbox.rs:41-45` | exact signature |
| `Inbox` (`passuff-core`) | `inbox.rs:47-67`, exported at `crates/passuff-core/src/lib.rs:21` | account merge, scope copy, newest-first, unread count |
| `InboxView`, `AccountView`, `PullRequestView`, `ReadOutcome` | `inbox.rs:10-37` | `ReadOutcome { found, changed, unread }` as designed |
| `InboxCli` (`passuff-cli`) | `crates/passuff-cli/src/inbox_cli.rs:10-108` | status, inbox, read; exit 0/2/3 |
| `TranscriptRenderer` status/inbox/read | `crates/passuff-cli/src/transcript.rs:75-159` | text rules as specified |
| `WatchConfig` (`[watch] repositories`) | `crates/passuff-config/src/lib.rs:33-51`, `:333-358` | validation, case-insensitive match, empty = all |
| `ActivityRecord` / `ActivityEvent` serde | `crates/passuff-core/src/domain.rs:274-300` | `read` flag, `a<n>` id |

Contract discipline: `InboxCli` holds `Arc<dyn InboxService>`
(`inbox_cli.rs:11`), `Inbox` holds `Arc<dyn SnapshotStore>`
(`inbox.rs:48`). Only the composition root (`crates/passuff-cli/src/main.rs:13,47`)
and the tests construct the implementations; no consumer imports `Inbox` or
`JsonStateStore` through the rendering path. `passuff-core` still exposes no
CLI/terminal dependency.

### Strict proof, e2e assertions, command files

- Strict proof present with output (quoted above). `.fail_on_skipped()` is set
  at `crates/passuff-cli/tests/features_runner.rs:54`.
- `@e2e` `Then` steps assert CLI stdout, never repository state:
  `watch-status.feature:75-83` (account line, `acme/api#42 open CI passing`,
  exit 0), `activity-inbox.feature:146-156` (`New comment from @alice`,
  `acme/api#42`, exit 0), `:158-169` (`1 activity record read`, then
  `no unread activity`, exit 0). No downgraded e2e scenario.
- `givn/commands.yaml` binds `verify.command: ./scripts/verify.sh` and
  `verify.e2e_command: ./scripts/verify.sh --e2e`, exactly as `design.md:205-231`
  names them. `scripts/verify.sh:19-22,53` maps `--e2e` to
  `GIVN_RESULT_SCOPE=e2e`; `features_runner.rs:31,58-64` applies the scope
  filter. The e2e scope is therefore selected explicitly.
- `verify.e2e_command` invokes `crates/passuff-cli/tests/features_runner.rs`
  only. No second `@e2e` implementation exists for status, inbox, or read:
  the tree has four step modules (`features_runner.rs:3-10`), and
  `git status` reports no untracked test or step file. `e2e_run_subcommand`
  spawns `env!("CARGO_BIN_EXE_passuff")` with `--config` and `--state`
  (`crates/passuff-cli/tests/support/e2e.rs:392-412`), which is the strongest
  available interface, so no weaker parallel implementation is in review.

### Interaction coverage

| Use case | Capability | Inventory entry | Design matrix row | `@e2e` scenario | Driving mechanism |
|---|---|---|---|---|---|
| `inspect-watch-status` | watch-status | show the watch state | yes (`design.md:237`) | `Developer sees the watch state` | `e2e_run_status` → real binary |
| `review-activity-inbox` | activity-inbox | list unread activity | yes (`design.md:238`) | `Developer lists unread activity` | `e2e_run_inbox(false)` → real binary |
| `review-activity-inbox` | activity-inbox | mark activity read | yes (`design.md:239`) | `Developer marks activity read` | `e2e_run_read_all` → real binary |
| `watch-pull-requests` | pr-activity-notification | start continuous watching | yes (`design.md:240`, "unchanged") | permanent `Developer starts watching and is notified of a new comment` | `e2e_start_watching` → real binary |
| `watch-pull-requests` | pr-activity-notification | run one observation pass | **missing row** | permanent `Developer runs a single check and sees the current activity` | `e2e_run_with` → real binary |
| `watch-pull-requests` | pr-activity-notification | show usage | **missing row** | permanent `Developer asks for usage` | real binary `--help` |
| `watch-pull-requests` | pr-activity-notification | show version | **missing row** | permanent `Developer asks for version` | real binary `--version` |

The three new/changed interactions map cleanly to matrix rows, feature
scenarios, and the promised driver. The three unchanged watch actions have E2E
coverage in the permanent spec (`givn lint` reports the capability-aware
inventory clean) but no matrix row. Finding N2.

### Design conformance (as-built vs `design.md`)

Commands, scope selection, step-file layout, runner, and driver match what
`design.md` named. One deviation: `design.md:225-226` names a
`SeededFailingStore` fixture for the state-write-failure scenario; as built the
scenario sets `fail_state_write` and drives a real read-only state directory
through `with_state_write_guard` (`crates/passuff-cli/tests/support/mod.rs:691-711`,
step `watch_status.rs:31-33`). The behaviour is equivalent and the real
filesystem failure is the stronger proof, but `design.md` was not updated.
Finding N1.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Requirements overview names the three use cases and the status/history goals | `01-introduction-and-goals.md:20-34` | ch. 01 affected | Scope | `cli-read`, `inbox-service` | `givn/specs/{watch-pull-requests,inspect-watch-status,review-activity-inbox}/usecase.md` | yes |
| Status/inbox/read are a terminal interface over config+state, no GitHub call | `03-context-and-scope.md:53-60` | ch. 03 affected | Scope, `InboxService` | `cli-read` | `inbox_cli.rs` has no gateway; `inbox.rs` uses only `SnapshotStore` | yes |
| Bounded history read model as the solution strategy, ADR-0005 linked | `04-solution-strategy.md:20-24` | ch. 04 affected | Technology decisions | setup | ADR-0005 exists and is linked | yes |
| `Inbox` building block with `InboxService`, state v2, watch scope | `05-building-block-view.md:138-149` | ch. 05 affected | Contracts and Components | setup, `inbox-service` | `crates/passuff-core/src/inbox.rs` | yes |
| `InboxCli` as a CLI building block | `05-building-block-view.md:50-63` and `:151-160` | ch. 05 affected | Contracts and Components (`InboxCli`) | `cli-read` | `crates/passuff-cli/src/inbox_cli.rs` exists but no level-1 node, table row, or CLI-delivery element names it | **no** |
| `JsonStateStore` load/quarantine rule | `05-building-block-view.md:104,107` | ch. 05 affected | Data model, migration | `state-history` | `state.rs:94-95` migrates v1 and rewrites; only an unsupported version is quarantined | **no** |
| Recording, status, and read runtime flows | `06-runtime-view.md:171-200` | ch. 06 affected | Runtime flow | all batches | `cycle.rs:196-221`, `inbox.rs:66-130` | yes |
| Deployment unchanged | `07-deployment-view.md` | not affected | — | — | same binary, same paths | yes |
| History retention, migration, read persistence, scope filtering | `08-crosscutting-concepts.md:20-31` | ch. 08 affected | Data model | all batches | `state.rs`, `cycle.rs:145-152,210-213` | yes |
| ADR-0005 register row and body | `09-architecture-decisions.md:19` | ch. 09 affected | ADR route | docs commit | `docs/arc42/adr/ADR-0005-state-version-2-migration.md` | yes |
| QS-016…QS-019 | `10-quality-requirements.md:48-51` | ch. 10 affected | Use-case traceability | all batches | delta scenarios `watch-status.feature:34-44`, `activity-inbox.feature:97-144` | yes |
| R-016…R-018, TD-004 closed | `11-risks-and-technical-debt.md:22-24,33` | ch. 11 affected | Risks | docs commit | `state.rs` migration error path; `state.rs:54-63` id recompute; `InboxService` | yes |
| Glossary delegation, activity record / inbox / read state / watch scope | `12-glossary.md:9-12`, `docs/glossary.md:39-42` | ch. 12 affected | Data model | docs commit | terms match the code symbols | yes |
| `next_activity_id` recomputed from the maximum record number | `design.md:106-108` | ch. 05/11 affected | Data model | `state-history` | `state.rs:54-63`, unit test `next_activity_id_is_recomputed_from_existing_records` | yes |

Sign-off checks:

- [x] Every affected durable chapter contains current, decision-specific
      content (01, 03, 04, 06, 08, 09, 10, 11, 12).
- [ ] Every change-level `arc42.md` claim matches the durable chapter,
      `design.md`, `tasks.md`, and the implementation — two rows above say no
      (`05` CLI building block; `05` quarantine wording).
- [x] The ADR-0005 qualification and the register row are present and current.
- [x] Glossary and domain model carry the new terms and structure without
      inventing entities.
- [ ] No stale contradiction remains — `05-building-block-view.md:104` still
      says a wrong-version file is quarantined while `08` and ADR-0005 migrate
      version 1.

ARC42 CONFORMANCE: FINDING

## Ubiquitous language conformance

Terms used by the delta, the design, and the code resolve to the durable
glossary: account (`Account`), activity record (`ActivityRecord`), activity
event (`ActivityEvent`), activity history (`WatcherState.activity`), inbox
(`Inbox`), read state (`ActivityRecord.read`), watch scope (`WatchConfig`),
watched pull request (`WatchedPullRequest`), authored pull request,
notification. Anti-terms hold: the specifications never call the inbox a
"list" or a "feed", and never call a record a "notification".

Code symbol refinement without second vocabulary: `InboxService` (contract),
`InboxView`/`AccountView`/`PullRequestView` (the read model's view types),
`ReadOutcome` (command result). The glossary's `Inbox` entry covers the read
model; the view types are its shape, not new domain nouns.

Structural model: `docs/domain-model.md` gains the Activity Record aggregate
(`:114`), the Activity Review boundary (`:60`), and the three new relationship
rows (`:127-129,132`) plus the diagram node `activity`. The structure matches
the implementation. One stale sentence: `docs/domain-model.md:13-14` still says
the model covers "`passuff-pr-watch` … and the `passuff-auth-modes` change".
Finding N3.

No second definition of watch scope exists: `WatchConfig::is_all`/`allows_repository`
are the only scope predicates, and the use case, design, and glossary all
describe the same empty-means-all rule.

## Coverage

Measured artifacts (root attributes, `measure-coverage.sh` regular + e2e scopes):

| Scope | Lines | Branches |
|---|---|---|
| non-e2e `coverage/non-e2e-cobertura.xml` | 1607/2000 (80.35%) | 217/752 (28.86%) |
| e2e `coverage/e2e-cobertura.xml` | 719/2000 (35.95%) | 99/752 (13.16%) |
| merged `coverage/cobertura-coverage.xml` | 1727/2000 (86.35%) | 193/582 (33.16%) |

The README badge matches the merged report (`README.md:4-5`). The managed
`## Coverage` block (`README.md:145-159`) still carries the previous archive's
numbers; the archive gate rewrites that block, as recorded in the archived
`passuff-auth-modes` review, so it is not a finding. Branch instrumentation is
present in both scopes; no line-only report.

Change-introduced regions, per file (merged report):

| File | Lines | Notes |
|---|---|---|
| `crates/passuff-core/src/inbox.rs` | 120/120 | full, branch complete |
| `crates/passuff-core/src/state.rs` | 161/181 | uncovered: pre-existing save I/O arms and the quarantine error arm |
| `crates/passuff-core/src/cycle.rs` | 97/128 | uncovered: pre-existing identity-mismatch, login-reset, and log lines |
| `crates/passuff-cli/src/inbox_cli.rs` | 42/58 | uncovered: new config/state/usage error arms (below) |
| `crates/passuff-cli/src/transcript.rs` | 94/118 | uncovered: `closed`, `CI pending`, `CI not started`, unused `render_activity_event` |
| `crates/passuff-config/src/lib.rs` | 285/311 | uncovered: pre-existing config read/token-path branches; `validate_watch_scope` is covered |

Bucket 1 — dead code (delete):

| Region | Evidence | Status |
|---|---|---|
| `crates/passuff-cli/src/transcript.rs:157-159` `pub fn render_activity_event` | added by `1f9a3c8`; `grep -rn render_activity_event` over the workspace returns only the definition; it only forwards to the private `render_event` | **unresolved — finding B3** |

Bucket 2 — missing tests:

| Region | Rule | Status |
|---|---|---|
| `inbox_cli.rs:23-31,56,67,80-84` | Use-case extensions 1a: status/inbox/read with an invalid configuration print the configuration error and exit 2; design CLI contract column `2 config` | **unresolved — finding B4** |
| `inbox_cli.rs:71,94,98` | Use-case extension 1b: inbox/read with an unreadable, unmigratable, or unwritable state print the state error and exit 3 | **unresolved — finding B4** |
| `inbox_cli.rs:100-105` | `design.md:202-203` "read requires exactly one of an id or `--all`; neither/both is a usage error (exit 2)" | **unresolved — finding B4** |
| `transcript.rs:166,174,175` | S3 human labels: `closed`, `CI pending`, `CI not started` | **unresolved — finding B4** |
| `state.rs:120-123,137-161` | pre-existing save failure arms (serialize/fsync/permissions/rename); unchanged by this change, carried from the archived review; the write-failure contract is covered by `save_reports_a_write_failure` and the read-only-directory scenario | carried, unchanged |
| `cycle.rs:47-53,82-84,100-116,137-138,223-226` | pre-existing credential-failure, observation-failure, identity-mismatch, login-reset, and log lines; permanent-suite paths outside the regular scope's feature set | carried, unchanged |
| `main.rs:24-41,79-80`, `config:185-187,200,254,272,375-379,389,393,409-444,472-474,698`, `domain.rs:166-168`, `github.rs`, `classifier.rs`, `sound.rs`, `delivery.rs`, `credential.rs`, `gh_login.rs:45` | pre-existing residuals in files or regions untouched by this change | carried, unchanged |

Bucket 3 — legitimately hard to test (new in this change): none.

Redundant unit tests: `design.md:229-231` plans unit tests for v1 migration, id
collision, missing-file read semantics, scope validation, recording order, and
delivery failure. `state.rs` and `config` keep that coverage, and
`next_activity_id_is_recomputed_from_existing_records` (`state.rs:290-306`)
protects an invariant no scenario asserts. Of the three `inbox.rs` tests,
`missing_state_is_empty_and_mark_all_does_not_create_it` and
`mark_read_is_per_event_and_persists` protect cases the scenarios do not assert
exactly (`mark_all_read` against a missing state; a repeated `mark_read`), but
`view_merges_accounts_and_orders_activity` re-asserts ordering, unread count, and
the watched-PR view that `Inbox lists unread activity newest first` and the
status scenarios already prove through the CLI. Finding N6.

## E2e coverage

- Inventory mapping: 3 delta actions (status, inbox, read), 3 delta `@e2e`
  scenarios, one each; 4 unchanged permanent `@e2e` scenarios. No under- or
  over-coverage; `givn lint` compares the capability-aware inventory and reports
  clean.
- Real-interface primary assertion: stdout of the spawned `passuff` binary with
  `--config` and `--state` (`support/e2e.rs:392-412`). The assertions read the
  transcript (`personal @octo-personal`, `acme/api#42 open CI passing`,
  `New comment from @alice`, `no unread activity`) and the exit status. No
  scenario asserts only on the state file.
- Driver: a real subprocess. The matrix names `real passuff status/inbox/read`,
  and the step bodies spawn the binary; no in-process service call substitutes
  for the CLI.
- **Non-browser classification.** The opposite classification is "Web UI: a
  rendered page is the interface". It does not hold: passuff renders no HTML,
  CSS, or JavaScript, serves no page, and has no browser client; the consumer
  boundary is a terminal process invocation. The API-only exemption also does
  not apply, because the CLI transcript — not an HTTP response — is the
  interface, so terminal transcripts are the correct evidence kind.
- Scope isolation: `verify.e2e_command` maps to `GIVN_RESULT_SCOPE=e2e`
  (`scripts/verify.sh:19-22`), and the runner filters on it
  (`features_runner.rs:58-64`); the regular command is `change` scope. The two
  executions are honestly isolated, and both report zero failures and skips.

## Visual review

Interface: terminal (`design.md:244-252`). Evidence committed under
`givn/changes/passuff-activity-inbox/evidence/visual/`, one final-state
transcript per delta `@e2e` scenario:

| Scenario | Transcript | State seen |
|---|---|---|
| `Developer sees the watch state` | `developer-sees-the-watch-state/transcript.txt` | status: account with login and last check, `watch scope: all authored pull requests`, `acme/api#42 open CI passing Pull request acme/api#42`, `activity: 0 unread of 0` |
| `Developer lists unread activity` | `developer-lists-unread-activity/transcript.txt` | inbox unread: `a1 <time> acme/api#42 unread New comment from @alice` |
| `Developer marks activity read` | `developer-marks-activity-read/transcript.txt` | empty state after `read --all`: `passuff: no unread activity`, exit 0 |

Persona `multi-account-developer` goal reachability:

- Inspect the watch: the accounts line names the label, the resolved login, and
  the last check time; the scope line states the allowlist or
  "all authored pull requests"; the watched-PR line carries key, human state,
  human CI label, and title. The Persona can tell what is watched and what the
  CI result is.
- Review activity: each record carries the addressable id, observation time,
  pull-request key, read state, and a human event summary. The Persona can pick
  the id to pass to `read`.
- Acknowledge activity: after `read --all` the inbox reports no unread activity,
  so the Persona sees the goal reached.

Baseline against the text rules (`design.md:198-203`):

- no enum name, state token, raw comment body, or command text appears in any
  transcript (`Open`, `Success`, `Can you split this migration?` are asserted
  absent by `watch-status.feature:43-44` and `activity-inbox.feature:17-19`);
- `passuff:` headings are present and stable; no duplicated content; the
  activity id (`a1`) is the documented CLI argument, not an internal identifier;
- no control, no error state, and no motion exist in this interface, so the
  control, error-recovery, and reduced-motion rules are not applicable.

The `read` confirmation line (`passuff: 1 activity record read`) is asserted in
the scenario but not retained in the committed transcript, because the evidence
capture is the scenario's final state and the final command is the follow-up
`inbox`. That matches the terminal evidence rule (one final-state transcript per
scenario); it is recorded here as an evidence note, not a contract finding.

No Persona finding. No contract rule is violated.

VISUAL-REVIEW: PASS

## Overlap dispositions

All 107 advisory shape pairs reported for this change are dispositioned below.
No pair is a `duplicate`: removing either scenario would lose a distinct failure
detection. No scenario was removed, so no removed+added supersession decision
applies. The 433 `[SUBST]` subset advisories are pairings of a permanent watch
rule (notification, credential, configuration) with a delta status/inbox rule;
they are distinct rules, not subsets, and are covered by the same decisions.

| Scenario A | Scenario B | Finding | Decision | Reason |
|---|---|---|---|---|
| Status lists configured accounts before the first check | Status lists multiple accounts | shape | variant | S1 with different data: one account before its first check against two resolved accounts |
| Status shows the login and last check after a check | A repository outside the watch scope is not watched | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when all repositories are watched | Status shows the watch scope when a repository allowlist is configured | shape | variant | one rule S2 with different configuration data; merging loses the empty-allowlist wording |
| Status shows the watch scope when all repositories are watched | Status reports unread and total activity | shape | variant | S2 against S4: different section of the same status transcript |
| Status shows the watch scope when all repositories are watched | Read marks one activity read | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when all repositories are watched | Read all marks every activity read | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when all repositories are watched | Read state survives a restart | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when all repositories are watched | Recording activity survives a delivery failure | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when all repositories are watched | A version-1 state loads with empty history | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when a repository allowlist is configured | Status reports unread and total activity | shape | variant | S2 against S4: different section of the same status transcript |
| Status shows the watch scope when a repository allowlist is configured | Read marks one activity read | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when a repository allowlist is configured | Read all marks every activity read | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when a repository allowlist is configured | Read state survives a restart | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when a repository allowlist is configured | Recording activity survives a delivery failure | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status shows the watch scope when a repository allowlist is configured | A version-1 state loads with empty history | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status reports unread and total activity | Read marks one activity read | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status reports unread and total activity | Read all marks every activity read | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status reports unread and total activity | Read state survives a restart | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status reports unread and total activity | Recording activity survives a delivery failure | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status reports unread and total activity | A version-1 state loads with empty history | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Status with no state shows empty sections | Developer sees the watch state | shape | boundary | empty-state boundary of the same status action at a different layer |
| Status with no state shows empty sections | Developer lists unread activity | shape | variant | different layer (spawned binary against the in-process harness) and a distinct invariant |
| Developer sees the watch state | Developer lists unread activity | shape | variant | different capability and observable outcome; only the Given vocabulary overlaps |
| Read marks one activity read | Read all marks every activity read | shape | variant | R3 (one record) against R4 (every record): different rule and count |
| Read marks one activity read | Read state survives a restart | shape | variant | R3 against R6: acknowledgment against persistence |
| Read marks one activity read | Recording activity survives a delivery failure | shape | variant | R3 against R5/R6: delivery independence |
| Read marks one activity read | A version-1 state loads with empty history | shape | variant | R3 against R7: migration path |
| Read all marks every activity read | Read state survives a restart | shape | variant | R4 against R6: bulk acknowledgment against persistence |
| Read all marks every activity read | Recording activity survives a delivery failure | shape | variant | R4 against R5/R6: delivery independence |
| Read all marks every activity read | A version-1 state loads with empty history | shape | variant | R4 against R7: migration path |
| Read refuses an unknown activity id | A state write failure is reported | shape | variant | two distinct refusals: unknown id against state write failure |
| Read refuses an unknown activity id | Developer marks activity read | shape | boundary | refusal edge of the mark-read action |
| Read state survives a restart | Recording activity survives a delivery failure | shape | variant | R6 against R5: persistence against recording order |
| Read state survives a restart | A version-1 state loads with empty history | shape | variant | R6 against R7: restart persistence against migration |
| Recording activity survives a delivery failure | A version-1 state loads with empty history | shape | variant | R5 against R7: record-then-deliver against migration |
| A state write failure is reported | Developer marks activity read | shape | boundary | state-failure edge of the mark-read action |
| A review comment from another person notifies | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A review comment from another person notifies | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A submitted review notifies | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A failing CI result notifies | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery notifies | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A CI recovery across pending notifies | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A pull request merged or closed while the watcher was stopped is reported | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Status shows the watch scope when all repositories are watched | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Status shows the watch scope when a repository allowlist is configured | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Status reports unread and total activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Read marks one activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Read all marks every activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Read state survives a restart | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | Recording activity survives a delivery failure | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A merge performed by the configured account still notifies | A version-1 state loads with empty history | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| An unavailable desktop surface keeps the transcript | Status with no state shows empty sections | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| An unavailable desktop surface keeps the transcript | Developer sees the watch state | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| An unavailable desktop surface keeps the transcript | Developer lists unread activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| An invalid configuration is rejected | Read refuses an unknown activity id | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| An invalid configuration is rejected | A state write failure is reported | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| An invalid configuration is rejected | Developer marks activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A state write failure is reported with a nonzero status | Read refuses an unknown activity id | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A state write failure is reported with a nonzero status | A state write failure is reported | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A state write failure is reported with a nonzero status | Developer marks activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A token file readable by other local users is refused | Read refuses an unknown activity id | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A token file readable by other local users is refused | A state write failure is reported | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A token file readable by other local users is refused | Developer marks activity read | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A missing token file is an account failure | Status with no state shows empty sections | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A missing token file is an account failure | Developer sees the watch state | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |
| A missing token file is an account failure | Developer lists unread activity | shape | variant | permanent watch rule; the delta scenario asserts a distinct status or inbox invariant with shared setup vocabulary |

## Split-or-keep

No scenario exceeds the gate's split threshold; `givn check review` and
`givn lint` reported no long-scenario finding. No split-or-keep rows are
required. For the record, the four longest delta scenarios (`Read marks only the
named activity` 12 steps, `Read state survives a restart` 10, `Inbox lists unread
activity newest first` 10, `A version-1 state loads with empty history` 10) are
keep decisions: each holds one rule set and one transcript outcome, and splitting
would duplicate the check-run setup.

## Findings

### Blockers

**B1 — `givn check review` fails on the proposal's capability-routing row.**
`givn/changes/passuff-activity-inbox/proposal.md:37` labels the row
`pr-activity-notification (watch scope)` in the *Proposed capability* column.
The gate compares that cell with the built delta's capability tag and reports:

```
capability routing decisions do not match the built delta:
  delta capability 'pr-activity-notification' is not declared in the Capability
  Routing table — the proposal records no 'NEW in <usecase-id>' or
  'EXTEND <capability>' decision for it
```

Verified cause: replacing the cell with the bare `pr-activity-notification`
(the form the archived `passuff-auth-modes` proposal uses) removes the error and
the gate then validates routing. The decision column already reads
`EXTEND pr-activity-notification`; only the first column is wrong. This is a
proposal/spec defect, not a `review.md` marker, so it cannot be closed here.

**B2 — fabricated step (step 0.1).**
`crates/passuff-cli/tests/steps/activity_inbox.rs:45` is
`async fn reopens(_world: &mut World) {}`. `Read state survives a restart`
(`activity-inbox.feature:92`) names a restart the step never performs. The
scenario still proves file persistence because `run_read` and `run_status` each
build a fresh `Inbox` over `JsonStateStore`, but the step promises an action it
does not take. Fix: drop the step, or have it reload the store explicitly.

**B3 — dead code (bucket 1).**
`crates/passuff-cli/src/transcript.rs:157-159` adds
`pub fn render_activity_event`, which has no caller anywhere in the workspace
(including tests) and only forwards to the private `render_event`. It was added
by `1f9a3c8`. Delete it.

**B4 — change-introduced behaviour with no scenario (bucket 2).**
Four specified behaviours are implemented and uncovered:

1. status/inbox/read with an invalid configuration print the configuration error
   and exit 2 (`inspect-watch-status/usecase.md` extension 1a,
   `review-activity-inbox/usecase.md` extension 1a, `design.md:170-178`);
   `inbox_cli.rs:23-31,56,67,80-84` is never executed.
2. inbox/read with a state that cannot be read, migrated, or written exit 3
   (`review-activity-inbox/usecase.md` extension 1b); only the `status` arm is
   covered by `A state write failure is reported`. `inbox_cli.rs:71,94,98` is
   never executed.
3. `read` with neither an id nor `--all` is a usage error with exit 2
   (`design.md:202-203`); `inbox_cli.rs:100-105` is never executed.
4. status human labels `closed`, `CI pending`, and `CI not started` (S3,
   `design.md:200-201`); `transcript.rs:166,174,175` are never executed. Only
   `open`/`merged` and `CI passing`/`CI failing` are asserted.

Each is missing test coverage, not a hard-to-test gap: the existing
`World::build_config` / `fail_state_write` / `read` harness already supports all
four as regular scenarios (or one `Scenario Outline` for the labels).

**B5 — Arc42 building-block view is incomplete and contradictory for an
affected chapter.**
`docs/arc42/05-building-block-view.md` was marked affected because it carries
the new `Inbox` service, the state version, and the CLI renderer, but only the
level-2 `### Inbox` section (`:138-149`) was added:

- the level-1 diagram (`:5-48`), the building-block table (`:50-63`), and the
  `### CLI delivery` subsection (`:151-160`) never name `Inbox`, `InboxService`,
  or `InboxCli`, so the new CLI adapter has no place in the view;
- `:104` still reads "corrupt or wrong-version file is quarantined", which
  contradicts `08-crosscutting-concepts.md:20-31`, ADR-0005, and
  `state.rs:94-95`: a version-1 file is migrated and rewritten, and only an
  unsupported version is quarantined.

Both are documentation fixes; no code changes follow from them.

### Non-blocking findings

| ID | Finding | Disposition |
|---|---|---|
| N1 | `design.md:225-226` names a `SeededFailingStore` for the write-failure scenario; as built it uses a real read-only state directory (`support/mod.rs:691-711`) with the seeded version-1 file, and `FailingStore` (`support/mod.rs:995-1014`) serves the pre-existing check-path scenarios. | Correct the `design.md` fixture paragraph; behaviour is equivalent and the real filesystem failure is the stronger proof. |
| N2 | The design's interaction matrix (`design.md:233-240`) has no row for three inventory entries of `watch-pull-requests` (`run one observation pass`, `show usage`, `show version`); its one row says only "unchanged". | Add the three titles (permanent `@e2e` coverage exists and `givn lint` reports the inventory clean). |
| N3 | `docs/domain-model.md:13-14` still says the model covers "`passuff-pr-watch` … and the `passuff-auth-modes` change". | Update the scope sentence to name `passuff-activity-inbox`; the structure itself is current. |
| N4 | `README.md:111` labels exit `2` as "configuration error", while `passuff read <unknown id>` and the `read` usage error also exit 2 (`design.md:202-203`). | Extend the Usage exit-code line (for example "2 configuration, usage, or unknown activity id"). |
| N5 | `git status` shows `evidence/visual/developer-lists-unread-activity/transcript.txt` and `evidence/visual/developer-sees-the-watch-state/transcript.txt` modified (timestamps only). | Commit the regenerated evidence with the closure batch so the tree is clean. |
| N6 | `crates/passuff-core/src/inbox.rs:243-255` `view_merges_accounts_and_orders_activity` duplicates the ordering, unread count, and view assertions of `Inbox lists unread activity newest first` and the status scenarios; the `.feature` file is the source of truth for behaviour already proved through `InboxCli`. | Delete the view unit test; keep the two that assert cases the scenarios do not (`mark_all_read` on a missing state, repeated `mark_read`). |

No finding in this report is excused outside the three coverage buckets. The
`read`-result transcript omission and the `JsonStateStore` unit tests are
recorded above as evidence notes with an explicit keep decision, not as
exceptions to a required check.

## README impact

README-IMPACT: updated - Quick start, Usage

- Quick start: `README.md:42-43` adds the `[watch] repositories` block to the
  configuration example, including the "empty = every authored pull request"
  rule.
- Usage: `README.md:99-103` adds `status`, `inbox`, `inbox --all`, `read <id>`,
  and `read --all`.
- Badge: `108b1f0` re-cuts the merged coverage badge to 86.4% / 33.2%, matching
  `coverage/cobertura-coverage.xml`.
- Prerequisites (`README.md:82-92`) is deliberately *not* claimed: the section
  was not edited by this change and the change adds no new prerequisite (no new
  dependency, tool, or platform requirement). Naming it would be a false record.
  If the operator wants it named, edit that section first.
- The managed `## Coverage` block (`README.md:145-159`) is archive-owned and is
  rewritten by the archive gate; it is not part of this decision.
- No placeholder text is open in the README.

## Sign-off checklist

- [x] Fabrication audit: 1 empty step body found and recorded (B2), not
      remediated in this pass; the remaining 98 step definitions drive the
      harness, a real subprocess, or an assertion.
- [x] Every checked task has a commit touching production code (`1f9a3c8`, four
      batches; 24 scenarios).
- [ ] Every promised component exists — yes — and the building-block view names
      them — no (B5).
- [x] Every consumer uses only its declared contract (`InboxService`,
      `SnapshotStore`; composition root excepted).
- [x] Strict-mode proof present and passing.
- [x] Scenario execution evidence recorded per scope (delta 24/24, permanent
      80/80, delta e2e 3/3, permanent e2e 4/4) from the receipt and `tasks.md`;
      the suite was not re-run in this review.
- [x] Coverage classified from the merged report; branch instrumentation intact.
- [ ] Every coverage gap classified and resolved — bucket 2 has four unresolved
      change-introduced rows (B4).
- [ ] Dead code deleted — `render_activity_event` remains (B3).
- [ ] Redundant unit tests removed — one view test duplicates the scenario
      coverage (N6).
- [x] No `@wip` tags; no implementation-layer detail in the spec beyond the
      persisted-state facts the use case makes observable.
- [x] Canonical E2E policy read from the resolved specs instruction; every
      normalized inventory action has one `@e2e` scenario; variants are regular
      scenarios.
- [x] Every E2E scenario has a real-interface primary assertion (CLI stdout) and
      uses the reviewed driver (spawned binary); no browser UI is involved.
- [x] The run declaration passes; the CLI starts cleanly (`passuff --help`
      transcript in `evidence/visual/developer-asks-for-usage`).
- [x] The exact file `verify.e2e_command` invokes was identified
      (`features_runner.rs` via `scripts/verify.sh --e2e`) and the tree,
      including untracked files, was searched for a second implementation —
      none exists.
- [x] The E2E scope is selected explicitly (`GIVN_RESULT_SCOPE=e2e`); both
      required executions report zero failures and zero skips.
- [x] Implementation commands, file layout, framework, and driver match
      `design.md`; the one fixture-naming deviation is recorded (N1) and does
      not change the interface under test.
- [ ] Interaction coverage verified — all rows map for the three new/changed
      interactions; three unchanged inventory entries have no matrix row (N2).
- [ ] Arc42 conformance clean — two `05` rows mismatch (B5).
- [ ] Ubiquitous language and structural model current — one stale scope
      sentence (N3).
- [x] No finding excused with a classification outside the three buckets.

## Verdict

Four blockers and one conformance mismatch remain: the capability-routing gate
failure (B1), a fabricated restart step (B2), dead code (B3), four specified
behaviours without a scenario (B4), and the stale/incomplete chapter 05 (B5).
The change is not signable. `givn check review --change passuff-activity-inbox`
exits 1.

REVIEW: FAIL
