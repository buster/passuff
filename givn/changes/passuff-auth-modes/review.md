# Review: passuff-auth-modes

Review kind: independent
PERSONAS: multi-account-developer
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

This pass re-checked all three prior blockers (N1, N2, N3) and re-ran the full
review on the current tree. N1 and N2 were resolved in the earlier pass; N3 (the
unused `impl Default for SystemGhLogin`) is resolved by `0dd4214`, a single-file
deletion of six lines. No blocker remains.

## Implementation evidence check

### @e2e tag integrity

| Feature file | @e2e before | @e2e after | Change |
|---|---|---|---|
| `givn/changes/passuff-auth-modes/specs/watch-pull-requests/pr-activity-notification.feature` | 0 | 0 | no delta scenario carries the tag; the browser-login flow is a regular variant of "start continuous watching" |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | 4 | unchanged; the four permanent `@e2e` titles remain |

No `@e2e` tag was removed. `givn lint --change passuff-auth-modes` reports
`1 file(s) checked — clean`; the only output is advisory SUBST findings.
`tasks.md:3` says "Regular scenarios first, then end-to-end" and contains no
`@e2e` token. B3 stays resolved.

### Empty step bodies

**0 empty/trivial step bodies found across 2 files**: 70 step attributes with 71
functions in `crates/passuff-cli/tests/steps/pr_activity_notification.rs` and 6
in `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs` (76 step
definitions total). No `unimplemented!()` or `todo!()` exists under `crates/`.
Every step body drives the `World`, the runtime, or an assertion.

### Scenario commit evidence

| Checked scenarios | `COMMIT:` entries | Batch | Evidence |
|---|---|---|---|
| 15 non-e2e scenario tasks | 15 | `auth-config`, `credential-resolution`, `gh-session`, `identity-and-watch` | each names its batch and `7fa8bed` |
| E2E smoke task | shared `7fa8bed` evidence | `e2e` | the configured `--e2e` command runs the permanent four |

`git show --stat 7fa8bed` touches production code
(`crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/{credential,cycle,domain}.rs`,
`crates/passuff-cli/src/{lib,main,gh_login}.rs`), the shared harness, both step
files, and the docs. The commit message names all four batches:
`Batches: auth-config, credential-resolution, gh-session, identity-and-watch`.
The change is not marked `CHARACTERIZATION`. The N2 remediation commit
`e6293c5` also touches production code (`gh_login.rs`, `cli/src/lib.rs`).

Preflight evidence verified from artifacts and re-run in this context:

- delta regular 27/27: `verification/tasks-result.json` (`{"total":27,"passed":27,"failed":0,"skipped":0}`)
  and `verification/tasks-verification.json` (`runner_exit: 0`, head `b9d74ba`,
  `command: ./scripts/verify.sh`, content digest `6563fcc`);
- permanent regular 52/52 and e2e 4/4: `tasks.md:50`; the e2e artifact
  `target/givn-verify-result.json` is `{"failed":0,"passed":4,"skipped":0,"total":4}`;
- unit 32: `cargo test --workspace --lib` executed five times across the two
  review contexts (three in the first, two after `0dd4214`), each run
  `4 + 15 + 13 = 32 passed; 0 failed`, exit 0;
- strict-mode probe: `tasks.md:13-21` records `FAILED: Strict mode probe :: Undefined step must fail`, `undefined Given`, `total 1, passed 0, failed 1`, cargo exit 1.

### Test-determinism finding (prior blocker N2) — resolved

`crates/passuff-cli/src/gh_login.rs` now stores an injected program and the
tests no longer touch `PATH`:

- `SystemGhLogin { program: PathBuf }` (`gh_login.rs:8-10`), `new()` returns
  the production `"gh"` (`:13-17`), `with_program()` injects any path (`:19-23`);
  the login command spawns `&self.program` (`:31`), not a `PATH` lookup.
- `login_runs_the_web_flow_for_the_host` (`:79-87`) and
  `login_reports_a_gh_failure` (`:99-105`) build the fake script through
  `tempfile::tempdir()` and pass `dir.path().join("gh")` — an absolute path.
  `login_reports_a_missing_gh` (`:89-96`) injects
  `dir.path().join("missing")`. No test prepends a directory to `PATH` and no
  restore step can race.
- `grep -rn 'set_var|remove_var|"PATH"' crates/passuff-cli/src/` finds no match:
  the CLI lib tests never mutate the process environment. The only `PATH`
  mutation left in the workspace is inside integration-test support
  (`crates/passuff-cli/tests/support/mod.rs:464-467`), which runs under the
  runner's `max_concurrent_scenarios(Some(1))` (`features_runner.rs:53`), so
  scenarios are serialized and the fake `gh` directory is an absolute
  `TempDir` path.
- `SystemGhLogin::new()` is constructed only in `system_credentials`
  (`crates/passuff-cli/src/lib.rs:21-29`); no unit test resolves a gha spec
  through it. `system_credentials_resolves_in_both_modes`
  (`lib.rs:311-333`) resolves a file token only, so no real `gh` can be
  invoked by the lib suite.
- Three consecutive `cargo test --workspace --lib` runs passed in ~0.1s with no
  hang, no device-code output, and no `gh` invocation.
- Residual note (not a demonstrated defect): the `gh_source` test
  (`crates/passuff-core/src/credential.rs:292-325`) still swaps `PATH` for the
  duration of one test. It is the only `PATH` consumer in that test binary
  (`CommandCredential` spawns `/bin/sh` by absolute path), and the suite passed
  three times; no third-party resolution can interleave with it today. The
  `HOME` swap in `crates/passuff-config/src/lib.rs:640-648` is likewise the only
  `~` consumer in its crate.

### Dead-code removal (prior blocker N3) — resolved

`git show 0dd4214` is a one-file, six-line deletion: it removes
`impl Default for SystemGhLogin` from `crates/passuff-cli/src/gh_login.rs` and
touches nothing else (`1 file changed, 6 deletions(-)`; no test, doc, spec, or
config change in the commit).

- `grep -rn 'impl Default for SystemGhLogin'` over the tree has no match outside
  this review file.
- No unused constructor remains: `new()` is called at `cli/src/lib.rs:24` and
  `with_program()` at `gh_login.rs:81,92,101`; `cargo build --workspace
  --all-targets` emits no dead-code warning and `cargo clippy --workspace
  --all-targets` reports no issues.
- `cargo test --workspace --lib` passed twice in this context after the deletion
  (`32 passed`, 3 suites, exit 0 in both runs).

### Components and contracts

| Contract (design `Contracts and Components`) | Promised component | Present |
|---|---|---|
| `CredentialSource` | credential resolver over four source adapters | yes: `SystemCredentialSource` (`credential.rs:219`), `EnvironmentCredential` (`:52`), `CommandCredential` (`:69`), `FileCredential` (`:103`), `GhOAuthCredential` (`:144`) |
| `GhLogin` | CLI composition-root adapter plus a recording fake | yes: `SystemGhLogin` (`crates/passuff-cli/src/gh_login.rs:8`) and `RecordingGhLogin` (`tests/support/mod.rs:758`) |
| `WatchControl` | `passuff-cli` runtime control | yes: `Interrupt` (`cli/src/lib.rs:54`) and `StopAfter` (`:87`); `wait` is a superset of the design trait |

Consumers use the declared surfaces: `CliPorts` holds `Arc<dyn CredentialSource>`,
`Arc<dyn GitHubGateway>`, `Arc<dyn SnapshotStore>`, `Arc<dyn NotificationDelivery>`
(`cli/src/lib.rs:32-37`); the composition root is `system_credentials`
(`cli/src/lib.rs:21-29`), which constructs `SystemCredentialSource` and
`SystemGhLogin`; `main.rs:43` calls the helper rather than the implementation.
No consumer imports a core implementation type outside the composition root and
tests.

### Strict-mode proof

`tasks.md:13-21` records the undefined-step probe and non-zero exit. The runner
enforces it with `.fail_on_skipped()` (`crates/passuff-cli/tests/features_runner.rs:51`)
and a non-zero exit for `total == 0` (`:93-97`).

### E2E Then assertions

| Permanent `@e2e` scenario | Primary assertion |
|---|---|
| Developer starts watching and is notified of a new comment | terminal transcript line for the pull request, desktop notification body, one sound, exit 0 |
| Developer runs a single check and sees the current activity | terminal transcript, desktop notification body, one sound, exit 0 |
| Developer asks for usage | usage text on stdout, exit 0 |
| Developer asks for the version | version line on stdout, exit 0 |

None asserts only repository/database state. All four read the stdout of a real
subprocess (`Command::new(env!("CARGO_BIN_EXE_passuff"))`,
`crates/passuff-cli/tests/support/e2e.rs:365,380,396`), asserted by
`pr_activity_notification_e2e.rs:21-43` (`Commands:`/`check` in usage,
`^passuff \d+\.\d+\.\d+$` for version) and the notification wait.

### `verify.e2e_command` binding and parallel implementations

- `givn/commands.yaml` binds `verify.e2e_command: "./scripts/verify.sh --e2e"`.
- `scripts/verify.sh:19-42` sets `GIVN_RESULT_SCOPE=e2e` for `--e2e`, defaults
  the corpus to `$root/givn/specs`, and falls back to `$root/givn/specs` when a
  given features path contains no `@e2e`.
- Only two step files exist (`pr_activity_notification.rs`,
  `pr_activity_notification_e2e.rs`), plus the single driver in
  `tests/support/e2e.rs`; no untracked or alternative e2e implementation was
  found.

### Design deviation check

| `design.md` statement | As built | Disposition |
|---|---|---|
| Four named source adapters (`design:63`) | all four types exist (`credential.rs:52,69,103,144`) | match |
| `verify.e2e_command` with `--e2e` and explicit scope (`design:286-288`) | scope and corpus are explicit; the configured command runs 4/4 | match |
| "the core's credential sources start token processes" (`design:57`) | `passuff-core` spawns `gh auth token` (`credential.rs:159-187`) and `/bin/sh -c` (`:77-82`); the login adapter is CLI-side | match |
| gh-plus-PAT message (`design:274-275`) | code emits `must not set a personal access token source when auth is gh_oauth` (`config:249`) and the feature asserts it (`:193`) | match |
| Two step-definition files (`design:298-301`) | `steps/pr_activity_notification.rs` + `steps/pr_activity_notification_e2e.rs` | match |
| Single-scenario `GIVN_SCENARIO` filter (`design:292-294`) | implemented (`features_runner.rs:29,67-69`) | match |
| Login retry "once per gh login per process" (`design:31`) | attempt set keyed by login (`credential.rs:150-156,203-215`); `docs/arc42/05:135`, `08:18`, ADR-0004 agree | match |
| Arc42 runtime wording (`docs/arc42/06-runtime-view.md:161`) | "records that a login was attempted for the gh login" | match |

### Interaction cross-reference

| Inventory entry (`usecase.md:118-121`) | Design matrix row (`design:355-362`) | `@e2e` scenario | Step definitions | Promised driving mechanism |
|---|---|---|---|---|
| start continuous watching | yes | `Developer starts watching and is notified of a new comment` | `e2e_steps::starts_watching` -> `World::e2e_start_watching` | real CLI subprocess, stdout transcript + desktop sink |
| run one observation pass | yes | `Developer runs a single check and sees the current activity` | `e2e_steps::runs_one_check` -> `World::e2e_run_check` | real CLI subprocess, stdout transcript |
| show usage | yes | `Developer asks for usage` | `e2e_steps::developer_runs_with` | real CLI subprocess, stdout |
| show version | yes | `Developer asks for the version` | `e2e_steps::developer_runs_with` | real CLI subprocess, stdout |

All four inventory entries have a matrix row, a permanent `@e2e` scenario with
the same interaction, and step definitions that spawn the real binary. There is
no unmapped inventory entry and no extra `@e2e` scenario. The gh browser login
is a recovery precondition of "start continuous watching", not a new consumer
action, so the regular variant is correct.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Credential privacy constraint | `01`, `02` (`02:16`) | affected | scope/constraints | R35 batch | `config:315-345`, `credential:106-131` | yes |
| gh CLI external system and credential interface | `03` (`03:33,45-46`) | affected | tech decision | gh-session batch | `credential:159-215`, `gh_login.rs` | yes |
| Delegate OAuth to gh, PAT for headless | `04` (`04:19-22`), ADR-0004 | affected | OAuth ownership | gh-session | `credential:191-216` | yes |
| `PassuffCore::with_ports` and `refresh` | `05` (`05:54`) | affected | runtime flow | setup | `cli/src/lib.rs:126-142` | yes |
| Credential acquisition building block | `05` (`05:122-136`) | affected | Component view | credential batches | `credential.rs` | yes |
| `CredentialSource` only (no `CredentialResolver`) | `05` (`05:19,58`) | affected | `CredentialSource` | — | no `CredentialResolver` symbol exists | yes |
| "once per gh login per process" | `05` (`05:135`), `08` (`08:15-18`), ADR-0004:60-61 | affected | `design:31` | R38 batch | attempt set keyed by login (`credential.rs:150-156,203`) | yes |
| Interactive recovery sequence | `06` (`06:132-165`) | affected | runtime flow | gh-session | `credential:196-215`, `main:43-49`; `06:161` says "attempted for the gh login" | yes |
| Deployment topology unchanged | `07` | not affected | — | — | no deployment change | yes |
| Configuration and credential rules | `08` (`08:3-18,83-89`) | affected | data model | auth-config | `config:195-380` | yes |
| No token in transcript/state/evidence | `08` (`08:79-89`), `10` QS-014 | affected | visual contract | token-absence batch | `feature:155-160`, step `:738-747` | yes |
| ADR-0004 in the register | `09` (`09:14`) | affected | ADR impact | docs | `docs/arc42/adr/ADR-0004-gh-delegated-authentication.md` + register row | yes |
| QS-013, QS-014, QS-015 | `10` (`10:45-47`) | affected | use-case traceability | gh-session, auth-config, identity | scenarios `feature:16,100,130,155` | yes |
| R-012..R-015 | `11` (`11:18-21`) | affected | risks | all | code matches R-015 re-check (`credential:106-131`) | yes |
| Glossary delegation | `12` (links `docs/glossary.md`) | affected | dictionary table | docs | glossary carries credential mode, expected login, gh session, personal access token, token file | yes |
| ADR-0004 confirmation: validation matrix | ADR-0004 Confirmation | — | validation matrix | auth-config | `config:585-613` parses valid `gh_oauth`, host derivation, missing login | yes |
| Configured login set excludes a mismatched login | glossary `Configured login set`, `usecase.md` R1 | chapter 12 row | identity validation | identity batch | `cycle:79-113` excludes the login; `LoginSet` at `cycle:113` | yes |
| Identity-mismatch observation wording | `usecase.md` extension 1c | — | `design:206-221` | identity batch | `cycle:65-96` reports and skips | yes |

Sign-off checks:

- [x] Every affected durable chapter (01, 02, 03, 04, 05, 06, 08, 09, 10, 11, 12) contains current, decision-specific content.
- [x] Every change-level `arc42.md` claim matches the durable chapter, `design.md`, `tasks.md`, and the implementation.
- [x] The ADR-0004 qualification and register row are present and current.
- [x] Glossary and domain model reflect the change without inventing structure.
- [x] No stale contradiction remains; the prior B9 per-account wording is gone from `design.md:31` and `docs/arc42/06:161`.

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

| Glossary term | Definition owner | Spec / design wording | Code symbol | Verdict |
|---|---|---|---|---|
| Credential mode | `docs/glossary.md:14` | "credential mode" in usecase R32 and `design:6` | `AuthMode`, `CredentialSpec` | consistent |
| Expected login | `docs/glossary.md:15` | "expected login" in R36 and the feature steps | `AccountConfig.expected_login` | consistent |
| gh session | `docs/glossary.md:16` | "gh session" in R33/R34 and the feature steps | `CredentialSpec::GhOAuth` | consistent |
| Personal access token | `docs/glossary.md:17` | "personal access token" in R32/R35/R37 and the feature steps | `CredentialSpec::{Env,Command,File}` | consistent |
| Token file | `docs/glossary.md:18` | "token file" in R35 and the feature steps | `CredentialSpec::File` | consistent |
| Configured login set | `docs/glossary.md:27`, usecase R1:48 | R36 excludes a mismatched login from the set | `LoginSet` built from accepted observations + persisted logins (`cycle:96-113`) | consistent |
| Login retry | `usecase.md:85` R38, `design:31` | "once per gh login per process" | `GhOAuthCredential.attempted: Mutex<HashSet<String>>` keyed by login (`credential:150-156,203`) | consistent in the governing rule |

One naming follow-up remains, unchanged from the prior review and not a rule
contradiction: the delta scenario title at `feature:44` says "A failed browser
login is attempted once per account", while R34/R38 say "once per gh login".
The scenario configures one distinct gh login per account, so its assertions
hold under either reading; only the noun needs aligning in a follow-up naming
batch. The governing rule is single-sourced in `usecase.md`, `design.md`,
ADR-0004, the arc42 chapters, and the code.

Structural model: `docs/domain-model.md` records credential mode and expected
login as configuration/validation attributes of the Account, not new entities.
The gh CLI is an infrastructure external system recorded in arc42 chapter 03.
No aggregate, entity, or boundary changed, so no diagram or relationship-table
change is required. No structural finding.

## Coverage

Measured artifacts (root attributes of the three reports):

| Scope | Lines | Branches |
|---|---|---|
| non-e2e `coverage/non-e2e-cobertura.xml` | 1143/1432 (79.82%) | 191/614 (31.11%) |
| e2e `coverage/e2e-cobertura.xml` | 547/1432 (38.20%) | 86/614 (14.01%) |
| merged `coverage/cobertura-coverage.xml` | 1222/1432 (85.34%) | 166/462 (35.93%) |

The README badge matches the merged report (`README.md:3-6`: 85.3% / 35.9%).
The managed `## Coverage` block at `README.md:140-152` still carries the
archive-owned 89% text; the archive gate rewrites the badge and block.

The three reports were written at 21:40, before `0dd4214` (21:46). The deleted
lines were 0-hit dead code, so a regenerated merged report drops them from the
denominator and the ratio can only rise; the badge is not re-cut in this review.

### Residual gaps and classification

Bucket 1 — dead code (delete):

| Region | Evidence | Status |
|---|---|---|
| `crates/passuff-cli/src/gh_login.rs` — former `impl Default for SystemGhLogin` (was `:26-30`) | added in `e6293c5`; the merged report recorded lines 27-29 at 0 hits in both scopes; no call site existed (`SystemGhLogin::new()` at `cli/src/lib.rs:24` is the only construction); deleted in `0dd4214`, verified above | **resolved — deleted; bucket 1 is now empty** |

Bucket 2 — missing tests (all change-introduced gaps resolved; the remaining
rows are unchanged pre-existing residuals):

| Region | Rule | Status |
|---|---|---|
| `config:213` | R32 `github_user` whitespace rejection | resolved: `rejects_a_github_user_with_whitespace` (`config:652-662`) |
| `config:319-321` | R35 HOME unset | resolved: `rejects_a_tilde_token_file_without_home` (`config:640-648`) |
| `credential:170-171` | R33 gh missing binary | resolved: `credential:292-325` |
| `credential:175-183` | R33 non-zero, empty, non-UTF8 gh output | resolved: `credential:292-325` |
| `gh_login:23-29` (pre-fix lines) | R34 gh missing binary | resolved: `login_reports_a_missing_gh` (`gh_login.rs:89-96`) |
| `feature:104` | R35 missing token file message | resolved: the Then asserts `token file is missing: ` (`feature:104`); `credential:26` renders the path |
| `cli/src/lib.rs:148,153,178-181,196,209,219-221,273-298` | R31 | pre-existing uncovered lines in unchanged CLI rendering/delivery code (`git blame`: `aff41de`, pre-change), carried from the archived review |
| `main.rs:20,27-39,71-72` | R19 | pre-existing default-path and recording-delivery lines (`aff41de`/`aab11e8`), unchanged |
| `cycle:71-72,118-119,150` | R17/R36 | pre-existing gateway-error, login-change reset, and baseline branches (`aff41de`), unchanged |
| `config:162-164,177,365-375` | R19 | pre-existing configuration read, empty-accounts, and graphql-URL validation branches (`aff41de`/`aab11e8`), unchanged |
| `config:390-392,420-422` | — | pre-existing serde `Visitor::expecting` diagnostics (`aff41de`), unchanged |
| `domain.rs:44-46,72-73,77,166-168` | — | pre-existing helper impls (`PrState::is_terminal`, `CiResult::default`, `WatchedPullRequest::key`) from `aff41de`, unchanged |
| `classifier.rs`, `github.rs`, `state.rs`, `sound.rs`, `transcript.rs`, `delivery.rs` | various | pre-existing permanent-suite residuals in files untouched by this change |

Bucket 3 — legitimately hard to test:

| Region | Technical justification |
|---|---|
| `credential:77-85,160-167`, `gh_login:29-39` | The 10-second command and gh-token timeouts and the 300-second login timeout fire only after a real wall-clock wait. The workspace enables no tokio paused clock for these crates; a deterministic test would either burn 10/300 seconds in the preflight or leave a live child after the timeout. |
| `credential:173`, `gh_login:40-46` | Non-`NotFound` spawn faults (EACCES/E2BIG/OS resource errors) are OS exec failures; the fake-gh harness models exit codes and output, not exec faults. The product contract names missing binary, non-zero exit, empty output, and timeout, not arbitrary exec faults. |
| `config:337-341`, `credential:111,123` | The "cannot be inspected"/`FileUnreadable` branches require a filesystem change between the load check and the read (TOCTOU) or a non-`NotFound` metadata fault; not drivable without privilege or fault injection. |
| `SystemGhLogin` real browser-consent success | A real `gh auth login --web` needs a browser, network, and interactive authorization; the port is faked and the process-status branches are tested. |

Region-attribution artifacts, not gaps: `config:326` (closing brace of the
`~`-expansion `if`; the push at `:325` is covered), `credential:73,99`
(async-trait wrapper line and closing brace; body `74-98` is covered),
`gh_login:52-53` (closing braces of `login` and of the `GhLogin` impl), and
`config:646` (closing brace of a
test-only conditional restore).

Bucket 1 is empty: the only entry was the `Default` impl, deleted in `0dd4214`
matching the B8 disposition for `impl Default for SystemCredentialSource`. No
uncovered, uncalled code remains in the change's regions.

## E2e coverage

- Inventory mapping: four entries, four permanent `@e2e` scenarios, one per
  normalized action (cross-reference table above).
- No delta `@e2e` scenario; valid because the login is a variant of an existing
  action. No under- or over-coverage.
- Real-interface primary assertion: stdout of the spawned `passuff` binary, plus
  the desktop sink and sound log; no browser route, no fetch, no direct API call.
- **Non-browser classification.** The opposite classification is "Web UI: a
  rendered page". It does not hold: passuff renders no HTML, CSS, or JavaScript,
  serves no HTTP page, and has no browser client. The consumer boundary is a
  terminal process invocation; a browser driver has no surface to attach to, and
  an HTTP client would be a lower-level substitute for the CLI. The terminal
  transcript is therefore the correct visual artifact (terminal-interface rule);
  the API-only exemption does not apply because the CLI transcript is the
  interface.

## Visual review

Interface: terminal. Evidence: the four permanent `@e2e` transcripts in
`target/visual/`:
`developer-asks-for-the-version/transcript.txt`,
`developer-asks-for-usage/transcript.txt`,
`developer-runs-a-single-check-and-sees-the-current-activity/transcript.txt`,
`developer-starts-watching-and-is-notified-of-a-new-comment/transcript.txt`.
No delta `@e2e` scenario exists by design (`design:378-386`), so no delta
transcript is owed. The new auth terminal lines are assessed from the renderer
(`transcript.rs:37-44`) and the regular-scenario assertions.

| Declared state / text rule | Evidence | Verdict |
|---|---|---|
| watch start and notification | `passuff: Fresh baseline recorded for acme/api#42`, `passuff: watching`, `passuff: acme/api#42`, `New comment from @alice`, exit 0 | baseline holds: labelled, no internal identifiers, no duplicate content, no token |
| one-shot check | `passuff: acme/api#42` / `New comment from @alice`, exit 0 | baseline holds |
| usage and version | real title line, `Commands:` list, `passuff 0.1.0`, exit 0 | baseline holds; no internal state names |
| identity mismatch names both logins | `feature:140` asserts `credential belongs to @octo-other, expected @octo-work`; rendered as `passuff: account work failed: ...` (`transcript.rs:37-44`) | text rule holds |
| missing token file names the path | code renders `token file is missing: <path>` (`credential.rs:26`); the scenario asserts the `token file is missing: ` prefix (`feature:104`) | text rule holds; exact path value is not separately asserted |
| configuration errors name the account/path | code wraps `passuff: configuration error: <message>` with `account '<label>'`; `feature:86-97,181-194` asserts the message class | text rule holds |
| failed gh login names gh | `feature:56` asserts `gh auth login failed`; `GhLoginFailed` displays `gh auth login failed` (`credential.rs:24-25`) | text rule holds |
| no token or token-file content | all four transcripts contain no token; `feature:155-160` asserts the credential value is in neither transcript nor state | holds and is asserted |

Persona `multi-account-developer`: reaches the goal for the four permanent
actions and sees labelled, actionable auth failures with no token. No visual
defect found.

VISUAL-REVIEW: PASS

## Overlap dispositions

The current shape findings are dispositioned below. The subset findings are
advisory pairings of a permanent notification rule with a delta auth rule; they
are distinct rules and are not shape duplicates. Each shape pair is a variant
asserting a different credential mode or validation rule, or a boundary
isolating a distinct auth rule that the other scenario does not assert.

| Scenario A | Scenario B | Finding | Decision | Reason |
|---|---|---|---|---|
| A gh OAuth account uses the existing gh session | A token file readable only by the operator is used | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A gh OAuth account uses the existing gh session | A token file path starting with ~ is expanded | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A gh OAuth account uses the existing gh session | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A gh OAuth account uses the existing gh session | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A gh OAuth account uses the existing gh session | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file readable only by the operator is used | A token file path starting with ~ is expanded | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file readable only by the operator is used | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file readable only by the operator is used | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file readable only by the operator is used | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file path starting with ~ is expanded | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file path starting with ~ is expanded | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A token file path starting with ~ is expanded | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A personal access token whose login differs only in case is accepted | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A personal access token whose login differs only in case is accepted | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| A legacy environment credential remains a personal access token account | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes (R32-R38); merging hides which mode regressed |
| An unavailable desktop surface keeps the transcript | A missing token file is an account failure | shape | boundary | R35 missing token file account failure (exit 1) is a distinct rule from the delivery fallback; merging loses the discriminating outcome |
| An invalid configuration is rejected | A token file readable by other local users is refused | shape | variant | same configuration-rejection action with a different validation rule (R19 vs R35); merging folds a privacy rule into the generic scenario |
| A state write failure is reported with a nonzero status | A token file readable by other local users is refused | shape | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |

## Split-or-keep

No scenario exceeds the gate's split threshold; no `[LONG]` finding was
reported by `givn lint` or `givn check review`. No split-or-keep rows are
required.

## README impact decision

README-IMPACT: updated - Quick start, Prerequisites

`README.md:40-67` replaces the environment-only account example with
`gh_oauth`/`github_pat`, token-file permissions, the interactive-login rule, and
the identity rule. `README.md:80-89` adds the `gh` 2.40+ prerequisite. The
coverage badge is refreshed to the merged 85.3%/35.9% (`README.md:3-6`); the
managed `## Coverage` block is owned by the archive gate.

## Blockers

No open blockers. N3 was the last one; it is resolved by `0dd4214` and the
classification in `### Residual gaps and classification` above now has an empty
bucket 1.

Prior blockers verified resolved:

- N1 — `proposal.md:38` Capability Routing first column is
  `pr-activity-notification`, matching the delta capability at
  `specs/watch-pull-requests/pr-activity-notification.feature`; the row's
  Decision cell is `EXTEND pr-activity-notification`.
- N2 — see the determinism section: injected program, absolute fake-script
  paths, no `PATH` mutation in the CLI lib tests, three consecutive green
  `cargo test --workspace --lib` runs (32 passed each).

Earlier blockers re-verified:

- B1: `givn/commands.yaml` binds `./scripts/verify.sh --e2e`; `verify.sh:19-42`
  selects the permanent corpus for the e2e scope; `target/givn-verify-result.json`
  is 4/4, exit 0.
- B2: `givn check review` reports the receipt reused across the
  documentation-only revision `b82512e` (tested at `b9d74ba`, 27 scenarios).
- B3: `givn lint --change passuff-auth-modes` is clean (exit 0) and `tasks.md`
  contains no `@e2e` token.
- B4: the four adapters exist (`credential.rs:52,69,103,144`) and the
  composition root wires them (`credential.rs:219-250`).
- B5: valid `gh_oauth` parsing is tested (`config:589,604,621,656`) and the host
  derivation is exercised (`config:585-613`).
- B6: `config:640-648` tests HOME unset; `feature:104` asserts
  `token file is missing: `; `tasks.md:114` calls HOME unset a configuration
  error.
- B7: `credential:292-325` covers missing binary, non-zero, empty, and non-UTF8;
  `gh_login.rs:89-105` covers missing gh and failure; the timeout branches are
  bucket 3.
- B8: `AccountConfig::auth_mode()`, `impl Default for SystemCredentialSource`,
  and `impl Default for SystemGhLogin` (`0dd4214`) are gone; no
  `CredentialResolver` remains.
- B9: `design.md:31`, `docs/arc42/05:135`, `08:18`, `06:161`, and ADR-0004 use
  the per-gh-login wording; `design-review.md:102-113` records the remediation
  re-check and `DESIGN-REVIEW: PASS`.
- B10: `feature:155-160` asserts the credential value appears in neither the
  transcript nor the state file.
- C1: `config:652-662` covers the `github_user` whitespace rejection.

## Sign-off checklist

- Fabrication audit: clean — 0 empty step bodies, 0 `unimplemented!()`, strict proof present.
- Every checked scenario has a batch commit touching production code: yes (`7fa8bed`; all four batch names in the message).
- Every promised component exists: yes (four adapters, `GhLogin`, `WatchControl`).
- Every consumer uses only its declared contract: yes (composition root + tests excepted).
- Strict-mode proof present and passing: yes (`tasks.md:13-21`).
- Scenario execution evidence per scope: delta 27/27 receipt, permanent 52/52, e2e 4/4, unit 32 passed in every run (two after `0dd4214`).
- Coverage classified from the merged report: yes — bucket 1 empty, change-introduced bucket 2 rows resolved, bucket 3 justified.
- Dead code deleted: yes — `impl Default for SystemGhLogin` removed in `0dd4214`.
- No `@wip`; no implementation detail in the spec: yes.
- Canonical E2E policy: every inventory action has one `@e2e`; variants are regular: yes.
- Real-interface assertions and reviewed driver: yes (spawned CLI binary).
- Local run command: `cargo run --release -p passuff-cli -- --help`; the e2e child starts the same binary and exits cleanly for usage and version.
- Exact e2e file(s) identified and tree searched for a parallel implementation: yes; no parallel implementation.
- Explicit scope selection: `GIVN_RESULT_SCOPE=e2e`; the configured command runs 4/4.
- Implementation matches design (commands, layout, driver): yes.
- Interaction coverage verified: yes.
- Ubiquitous language and structural model: consistent; one non-blocking scenario-title naming follow-up.
- No finding excused outside the three buckets: yes.

REVIEW: PASS
