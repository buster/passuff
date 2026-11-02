# Review: passuff-auth-modes

Review kind: independent
PERSONAS: multi-account-developer
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

## Implementation evidence check

### @e2e tag integrity

| Feature file | @e2e before | @e2e after | Change |
|---|---|---|---|
| `givn/changes/passuff-auth-modes/specs/watch-pull-requests/pr-activity-notification.feature` | 0 | 0 | no delta scenario carries the tag; the browser-login flow is a regular variant of "start continuous watching" |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | 4 | unchanged; the four permanent `@e2e` titles remain |

No `@e2e` tag was removed. `givn lint --change passuff-auth-modes` exits 0
and ends with `givn lint: 1 file(s) checked — clean`; the only output is
advisory SHAPE/SUBST findings. `tasks.md:3` now says "Regular scenarios first,
then end-to-end" and contains no `@e2e` token. B3 is resolved.

### Empty step bodies

**0 empty/trivial step bodies found across 2 files**: 75 functions in
`crates/passuff-cli/tests/steps/pr_activity_notification.rs` and 6 in
`crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`. No
`unimplemented!()` or `todo!()` exists under `crates/`. Every step body drives
the `World`, the runtime, or an assertion.

### Scenario commit evidence

| Checked scenarios | `COMMIT:` entries | Batch | Evidence |
|---|---|---|---|
| 15 non-e2e scenario tasks | 15 | `auth-config`, `credential-resolution`, `gh-session`, `identity-and-watch` | each names its batch and `7fa8bed` |
| E2E smoke task | shared `7fa8bed` evidence | `e2e` | the configured `--e2e` command runs the permanent four |

`git show --stat 7fa8bed` touches production code
(`crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/{credential,cycle,domain}.rs`,
`crates/passuff-cli/src/{lib,main,gh_login}.rs`), the shared harness, both step
files, the feature, and the docs. The commit message names all four batches:
`Batches: auth-config, credential-resolution, gh-session, identity-and-watch`.
The change is not marked `CHARACTERIZATION`.

Preflight evidence verified from artifacts:

- delta regular 27/27: `verification/tasks-result.json` and
  `verification/tasks-verification.json` (`runner_exit: 0`, head `f6433cb`);
- permanent regular 52/52 and e2e 4/4: `tasks.md:50`; the e2e artifact
  `target/givn-verify-result.json` is `{"failed":0,"passed":4,"skipped":0,"total":4}`;
- unit 32: confirmed with `cargo test --workspace --lib` (`32 passed, 3 suites`);
- strict-mode probe: `tasks.md:13-21` records `FAILED: Strict mode probe :: Undefined step must fail`, `undefined Given`, `total 1, passed 0, failed 1`, cargo exit 1.

### Components and contracts

| Contract (design `Contracts and Components`) | Promised component | Present |
|---|---|---|
| `CredentialSource` | credential resolver over four source adapters | yes: `SystemCredentialSource` (`credential.rs:219`), `EnvironmentCredential` (`:51`), `CommandCredential` (`:68`), `FileCredential` (`:102`), `GhOAuthCredential` (`:144`) |
| `GhLogin` | CLI composition-root adapter plus a recording fake | yes: `SystemGhLogin` (`crates/passuff-cli/src/gh_login.rs:9`) and `RecordingGhLogin` (tests) |
| `WatchControl` | `passuff-cli` runtime control | yes: `Interrupt` and `StopAfter` (`cli/src/lib.rs:49-104`); `wait` is a superset of the design trait |

Consumers use the declared surfaces: `CliPorts` holds `Arc<dyn CredentialSource>`,
`Arc<dyn GitHubGateway>`, `Arc<dyn SnapshotStore>`, `Arc<dyn NotificationDelivery>`
(`cli/src/lib.rs:32-37`); the composition root is `system_credentials`
(`cli/src/lib.rs:21-29`), which constructs `SystemCredentialSource` and
`SystemGhLogin`; `main.rs` calls the helper rather than the implementation. No
consumer imports a core implementation type outside the composition root and
tests.

### Strict-mode proof

`tasks.md:13-21` records the undefined-step probe and non-zero exit. The runner
enforces it with `.fail_on_skipped()`
(`crates/passuff-cli/tests/features_runner.rs:50`) and a non-zero exit for
`total == 0` (`:90-96`).

### E2E Then assertions

| Permanent `@e2e` scenario | Primary assertion |
|---|---|
| Developer starts watching and is notified of a new comment | terminal transcript line for the pull request, desktop notification body, one sound, exit 0 |
| Developer runs a single check and sees the current activity | terminal transcript, desktop notification body, one sound, exit 0 |
| Developer asks for usage | usage text on stdout, exit 0 |
| Developer asks for the version | version line on stdout, exit 0 |

None asserts only repository/database state. All four read the stdout of a real
subprocess (`Command::new(env!("CARGO_BIN_EXE_passuff"))`,
`crates/passuff-cli/tests/support/e2e.rs:365,380,396`).

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
| Four named source adapters (`design:63`) | all four types exist (`credential.rs:51,68,102,144`) | match |
| `verify.e2e_command` with `--e2e` and explicit scope (`design:286-288`) | scope and corpus are explicit; the configured command runs 4/4 | match |
| "the core's credential sources start token processes" (`design:57`) | `passuff-core` spawns `gh auth token` and `/bin/sh -c`; the login adapter is CLI-side | match |
| gh-plus-PAT message (`design:274-275`) | code emits `must not set a personal access token source when auth is gh_oauth` (`config:249`) | match |
| Step-definition file layout (`design:298-301`) | two files as named | match |
| Single-scenario `GIVN_SCENARIO` filter (`design:292-294`) | implemented (`features_runner.rs:29`) | match |
| Login retry "once per gh login per process" (`design:31`) | `design:31` now says exactly that; attempt set keyed by login (`credential.rs:147,203`) | match |
| Arc42 runtime wording `docs/arc42/06-runtime-view.md:161` | now "records that a login was attempted for the gh login" | match |

### Interaction cross-reference

| Inventory entry (`usecase.md:118-121`) | Design matrix row (`design:357-362`) | `@e2e` scenario | Step definitions | Promised driving mechanism |
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

### Test-determinism finding (blocker N2)

The three new `SystemGhLogin` unit tests at
`crates/passuff-cli/src/gh_login.rs:61-90` mutate the process-global `PATH`
without serialization. `login_runs_the_web_flow_for_the_host` and
`login_reports_a_gh_failure` prepend a temporary fake `gh`, while
`login_reports_a_missing_gh` sets `PATH=/nonexistent` and later restores the
original value. When they interleave, the fake `gh` is not found and the real
`gh` is run. Repeated `cargo test -p passuff-cli --lib` reproduced a hang: the
`login_reports_a_gh_failure` test invoked real `gh auth login --web` and printed
a GitHub one-time device code (`https://github.com/login/device`). The 300-second
login timeout then blocks the unit preflight. `--test-threads=1` passes
consistently, which confirms the environment race rather than a production
defect. The recorded "32 passed" run is not a deterministic result.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Credential privacy constraint | `01`, `02` (`:16`) | affected | scope/constraints | R35 batch | `config:200-366`, `credential:106-131` | yes |
| gh CLI external system and credential interface | `03` (`:33-46`) | affected | tech decision | gh-session batch | `credential:159-215`, `gh_login.rs` | yes |
| Delegate OAuth to gh, PAT for headless | `04` (`:19-22`), ADR-0004 | affected | OAuth ownership | gh-session | `credential:190-216` | yes |
| `PassuffCore::with_ports` and `refresh` | `05` (`:54`) | affected | runtime flow | setup | `cli/src/lib.rs:126-142` | yes |
| Credential acquisition building block | `05` (`:122-136`) | affected | Component view | credential batches | `credential.rs` | yes |
| `CredentialSource` / `CredentialResolver` | `05` (`:19,58`) | affected | `CredentialSource` only | — | no `CredentialResolver` symbol exists | yes |
| "once per gh login per process" | `05` (`:135`), `08` (`:15-18`), ADR-0004:60-61 | affected | `design:31` | R38 batch | attempt set keyed by **login** (`credential.rs:147,203`) | yes |
| Interactive recovery sequence | `06` (`:132-165`) | affected | runtime flow | gh-session | `credential:196-215`, `main:43-49`; `06:161` says "for the gh login" | yes |
| Deployment topology unchanged | `07` | not affected | — | — | no deployment change | yes |
| Configuration and credential rules | `08` (`:3-18`) | affected | data model | auth-config | `config:195-380` | yes |
| No token in transcript/state/evidence | `08` (`:79-89`), `10` QS-014 | affected | visual contract | token-absence batch | `feature:155-160`, step `:738-747` | yes |
| ADR-0004 in the register | `09` (`:11`) | affected | ADR impact | docs | ADR file + index | yes |
| QS-013, QS-014, QS-015 | `10` (`:45-47`) | affected | use-case traceability | gh-session, auth-config, identity | scenarios `feature:16,100,130,155` | yes |
| R-012..R-015 | `11` (`:18-21`) | affected | risks | all | code matches R-015 re-check (`credential:106-131`) | yes |
| ADR-0004 confirmation: validation matrix | ADR-0004 Confirmation | — | validation matrix | auth-config | `config:585-613` parses valid `gh_oauth`, host derivation, missing login | yes |
| Configured login set excludes a mismatched login | glossary `Configured login set`, `usecase.md` R1 | chapter 12 row | identity validation | identity batch | `cycle:79-113` excludes the login | yes |
| Identity-mismatch observation wording | `usecase.md` extension 1c | — | `design:206-221` | identity batch | `cycle:65-96` reports and skips | yes |

Sign-off checks:

- [x] Every affected durable chapter (01, 02, 03, 04, 05, 06, 08, 09, 10, 11, 12) contains current, decision-specific content.
- [x] Every change-level `arc42.md` claim matches the durable chapter, `design.md`, `tasks.md`, and the implementation.
- [x] The ADR-0004 qualification and register row are present and current.
- [x] Glossary and domain model reflect the change without inventing structure.
- [x] No stale contradiction remains; the prior B9 per-account wording is gone from `design.md:31` and `docs/arc42/06:161`, and the R38 rule is consistent across chapter 05, chapter 08, ADR-0004, and the code.

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
| Login retry | `usecase.md:85` R38, `design:31` | "once per gh login per process" | `GhOAuthCredential.attempted: HashSet<String>` keyed by login | consistent in the governing rule |

One naming follow-up remains, not a rule contradiction: the delta scenario
title at `feature:44` still says "A failed browser login is attempted once per
account", while R38 and the code are per gh login. The scenario uses two
distinct logins, so its assertions are correct; the title should be aligned in a
follow-up naming batch. It does not change the governing rule or the behavior.

Structural model: `docs/domain-model.md` records credential mode and expected
login as configuration/validation attributes of the Account, not new entities.
The gh CLI is an infrastructure external system recorded in arc42 chapter 03, not
a domain relationship; no aggregate, entity, or boundary changed, so no diagram
or relationship-table change is required. No structural finding.

## Coverage

Measured artifacts (merged report `coverage/cobertura-coverage.xml`):

| Scope | Lines | Branches |
|---|---|---|
| non-e2e `coverage/non-e2e-cobertura.xml` | 1142/1428 (79.97%) | 191/614 (31.11%) |
| e2e `coverage/e2e-cobertura.xml` | 547/1428 (38.31%) | 86/614 (14.01%) |
| merged `coverage/cobertura-coverage.xml` | 1221/1428 (85.50%) | 166/462 (35.93%) |

The README badge matches the merged report (`README.md:3-6`). The managed
`## Coverage` block at `README.md:137-151` still contains the archive-owned 89%
text; the archive gate owns that block, as stated in the archive instructions.

### Residual gaps and classification

Bucket 1 — dead code (delete): **none**. `AccountConfig::auth_mode()` and the
`Default for SystemCredentialSource` impl are gone; no unused `CredentialResolver`
remains.

Bucket 2 — missing tests:

| Region | Rule | Status |
|---|---|---|
| `config:213` | R32 `github_user` whitespace rejection | resolved: `rejects_a_github_user_with_whitespace` (`config:652-662`) |
| `config:319-321` | R35 HOME unset | resolved: `rejects_a_tilde_token_file_without_home` (`config:640-648`) |
| `credential:170-171` | R33 gh missing binary | resolved: `credential:291-325` |
| `credential:175-183` | R33 non-zero, empty, non-UTF8 gh output | resolved: `credential:291-325` |
| `gh_login:23-29` | R34 gh missing binary | resolved: `login_reports_a_missing_gh` (`gh_login.rs:73-80`) |
| `feature:104` | R35 missing token file message | resolved: the Then asserts `token file is missing: ` (`feature:104`); `tasks.md:114` now calls HOME unset a configuration error |
| `cli/src/lib.rs:148,153,178-181,196,209,219-221,273-298` | R31 | pre-existing uncovered lines in unchanged CLI rendering/delivery code, carried from the archived review |
| `main.rs:20,27-39,71-72` | R19 | pre-existing default-path and recording-delivery lines, unchanged |
| `cycle:71-72,118-119,150` | R17/R36 | pre-existing gateway-error, login-change reset, and baseline branches, unchanged |
| `config:162-164,177,365-375` | R19 | pre-existing configuration read, empty-accounts, and graphql-URL validation branches, unchanged |
| `classifier.rs`, `github.rs`, `state.rs`, `sound.rs`, `transcript.rs`, `delivery.rs` | various | pre-existing permanent-suite residuals in unchanged files |

Bucket 3 — legitimately hard to test:

| Region | Technical justification |
|---|---|
| `credential:78`, `credential:167`, `gh_login:13` | The 10-second command and gh-token timeouts and the 300-second login timeout fire only after a real wall-clock wait. The workspace does not enable tokio's paused-clock test facility for these crates; a deterministic test would either wait 10/300 seconds in the preflight or leave a live child process after the timeout. Concrete justification: wall-clock wait, no injectable clock in the current harness. |
| `credential:173`, `gh_login:27` | Non-`NotFound` spawn faults (EACCES/E2BIG/OS resource errors) are OS-level exec failures; the fake-gh harness models exit codes and token output, not exec faults. The product contract names missing binary, non-zero exit, empty output, and timeout, not arbitrary exec faults. |
| `config:337-341`, `credential:111,123` | The "cannot be inspected"/`FileUnreadable` branches require a filesystem change between the load check and the read (TOCTOU) or a non-`NotFound` metadata fault; not deterministically drivable without privilege or fault injection. |
| `config:390-392,420-422` | Serde `Visitor::expecting` diagnostics are reached only on a type-mismatch deserialization error; required trait methods, not a product rule (pre-existing). |
| `SystemGhLogin` real browser-consent success | A real `gh auth login --web` needs a browser, network, and interactive authorization; the port is faked and the process-status branches are tested. |

No classification outside the three buckets is used. The coverage percentage is
recorded, not used as a gate threshold.

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
| watch start and notification | `developer-starts-watching...`: `passuff: Fresh baseline recorded for acme/api#42`, `passuff: watching`, `passuff: acme/api#42`, `New comment from @alice` | baseline holds: labelled, no internal identifiers, no duplicate content, no token |
| one-shot check | `developer-runs-a-single-check...`: `passuff: acme/api#42` / `New comment from @alice`, exit 0 | baseline holds |
| usage and version | real title line, command list, `passuff 0.1.0`, exit 0 | baseline holds; no internal state names |
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

The 18 current shape findings are dispositioned below. The 117 subset findings
are advisory pairings of a permanent notification rule with a delta auth rule;
they are distinct rules and are not shape duplicates. Each shape pair is a
variant asserting a different credential mode or validation rule, or a boundary
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
reported. No split-or-keep rows are required.

## README impact decision

README-IMPACT: updated - Quick start, Prerequisites

`README.md:15-66` replaces the environment-only account example with
`gh_oauth`/`github_pat`, token-file permissions, the interactive-login rule, and
the identity rule. `README.md:81-89` adds the `gh` 2.40+ prerequisite. The
coverage badge is refreshed to the merged 85.5%/35.9% (`README.md:3-6`); the
managed `## Coverage` block is owned by the archive gate.

## Blockers

| ID | Blocker | Evidence | Required fix |
|---|---|---|---|
| N1 | Capability routing does not name the built delta capability | `givn check review --change passuff-auth-modes` reports: `delta capability 'pr-activity-notification' is not declared in the Capability Routing table`. `proposal.md:38` has `Proposed capability = "Account authentication and credential sources"` while the Decision cell is `EXTEND pr-activity-notification`; the gate matches the row by the Proposed capability column. | Change the row's Proposed capability to `pr-activity-notification` (or add a row for it) in `proposal.md`, then re-run the gate. The review cannot edit the proposal. |
| N2 | The new `SystemGhLogin` unit tests race on the process-global `PATH` and can invoke real `gh` | `crates/passuff-cli/src/gh_login.rs:61-90`: three tests mutate `PATH` concurrently. Repeated `cargo test -p passuff-cli --lib` hung at `login_reports_a_gh_failure` after real `gh auth login --web` printed a GitHub device code; `--test-threads=1` passes. The 300-second login timeout can block the unit preflight and the recorded 32-passed result is not deterministic. | Serialize the three tests with a shared test mutex or run them on a single test thread; ensure the fake `gh` is the only `gh` resolved. Re-run the unit preflight. |

The five prior blockers are verified resolved:

- B3: `givn lint --change passuff-auth-modes` exits 0 and is clean; `tasks.md`
  has no `@e2e` token.
- B6: `config:640-648` tests HOME unset; `feature:104` asserts
  `token file is missing: `; `tasks.md:114` says configuration error.
- B7: `credential:291-325` covers missing binary, non-zero, empty, and non-UTF8;
  `gh_login.rs:73-80` covers missing gh; the timeout branches are bucket 3.
- B9: `design.md:31` and `docs/arc42/06:161` use the per-gh-login wording;
  `design-review.md:106-113` records the remediation re-check and
  `DESIGN-REVIEW: PASS`.
- C1: `config:652-662` covers the `github_user` whitespace rejection.

## Sign-off checklist

- Fabrication audit: clean — 0 empty step bodies, 0 `unimplemented!()`, strict proof present.
- Every checked scenario has a batch commit touching production code: yes (`7fa8bed`; all four batch names in the message).
- Every promised component exists: yes (four adapters, `GhLogin`, `WatchControl`).
- Every consumer uses only its declared contract: yes (composition root + tests excepted).
- Strict-mode proof present and passing: yes (`tasks.md:13-21`).
- Scenario execution evidence per scope: delta 27/27 receipt, permanent 52/52, e2e 4/4, unit 32 passed once; the unit suite is nondeterministic (N2).
- Coverage classified from the merged report: yes; gaps introduced by this change are resolved; unchanged pre-existing bucket 2 residuals are recorded above.
- Dead code deleted: yes.
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

REVIEW: FAIL
