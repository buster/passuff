# Review: passuff-auth-modes

Review kind: independent
PERSONAS: multi-account-developer
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

## Implementation evidence check

### @e2e tag integrity

| Feature file | @e2e before | @e2e after | Change |
|---|---|---|---|
| `givn/changes/passuff-auth-modes/specs/watch-pull-requests/pr-activity-notification.feature` | 0 | 0 | no delta scenario carries the tag; the login flow is a regular variant |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | 4 | unchanged; the four permanent `@e2e` titles remain |

No `@e2e` tag was removed. The delta reclassifies the browser-login flow as a
regular variant of "start continuous watching", so no delta `@e2e` is owed.
`givn lint` still reports the tasks.md cross-reference warning (blocker B3):
`tasks.md:3` contains the literal `@e2e` while the delta specs contain none.

### Empty step bodies

**0 empty/trivial step bodies found across 2 files** (75 functions in
`crates/passuff-cli/tests/steps/pr_activity_notification.rs`, 6 in
`crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`). No
`unimplemented!()` or `todo!()` remains under `crates/`. Every step body drives
the `World`, the runtime, or an assertion.

### Scenario commit evidence

| Checked scenarios | `COMMIT:` entries | Batch | Evidence |
|---|---|---|---|
| 15 non-e2e scenario tasks | 15 | `auth-config`, `credential-resolution`, `gh-session`, `identity-and-watch` | each names its batch and `7fa8bed` |
| 1 E2E smoke task | 1 | `e2e` | names `7fa8bed` |

`git show --stat 7fa8bed` touches production code
(`crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/{credential,cycle,domain}.rs`,
`crates/passuff-cli/src/{lib,main,gh_login}.rs`), the shared harness, both step
files, and the spec/docs. Every `COMMIT:` line names `Batch: <name>`. The change
is not marked `CHARACTERIZATION`.

### Components and contracts

| Contract (design `Contracts and Components`) | Promised component | Present |
|---|---|---|
| `CredentialSource` | credential resolver over four source adapters | yes: `SystemCredentialSource` (`credential.rs:219`), `EnvironmentCredential` (`:51`), `CommandCredential` (`:68`), `FileCredential` (`:102`), `GhOAuthCredential` (`:144`) |
| `GhLogin` | CLI composition-root adapter plus a recording fake | yes: `SystemGhLogin` (`crates/passuff-cli/src/gh_login.rs:9`) and `RecordingGhLogin` (tests) |
| `WatchControl` | `passuff-cli` runtime control | yes: `Interrupt` and `StopAfter` (`cli/src/lib.rs:42-104`); `wait` is a superset of the design trait |

Consumers use the declared surfaces: `CliPorts` holds `Arc<dyn CredentialSource>`,
`Arc<dyn GitHubGateway>`, `Arc<dyn SnapshotStore>`, `Arc<dyn NotificationDelivery>`;
the composition root is `system_credentials` (`crates/passuff-cli/src/lib.rs:21-29`),
which constructs `SystemCredentialSource` and `SystemGhLogin`; `main.rs` calls the
helper rather than the implementation. No consumer imports a core implementation
type outside the composition root and tests. The `WatchControl` implementation
adds `wait` beyond `design:245-248` (superset, non-blocking).

### Strict-mode proof

`tasks.md:13-21` records the undefined-step probe: `FAILED: Strict mode probe ::
Undefined step must fail`, `undefined Given`, `total 1, passed 0, failed 1`, cargo
exit 1. The runner enforces it with `.fail_on_skipped()`
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
`tests/support/e2e.rs:365,380,396`).

### `verify.e2e_command` binding and parallel implementations

- `givn/commands.yaml` binds `verify.e2e_command: "./scripts/verify.sh --e2e"`.
- `scripts/verify.sh:19-42` sets `GIVN_RESULT_SCOPE=e2e` for `--e2e`, defaults the
  corpus to `$root/givn/specs`, and falls back to `$root/givn/specs` when a given
  features path contains no `@e2e`.
- Executed `./scripts/verify.sh --e2e` (no `GIVN_FEATURES`): `total 4, passed 4,
  failed 0, skipped 0`, exit 0. Executed with
  `GIVN_FEATURES=givn/changes/passuff-auth-modes/specs`: same 4/4, exit 0 (fallback
  proven). Blocker B1 is resolved.
- Only two step files exist (`pr_activity_notification.rs`,
  `pr_activity_notification_e2e.rs`), plus the single driver in
  `tests/support/e2e.rs`; no untracked or alternative e2e implementation was found.

### Design deviation check

| `design.md` statement | As built | Disposition |
|---|---|---|
| Four named source adapters (`design:63`) | all four types exist (`credential.rs:51,68,102,144`) | match |
| `verify.e2e_command` with `--e2e` and explicit scope (`design:286-288`) | scope and corpus are explicit; the configured command runs 4/4 | match |
| "the core's credential sources start token processes" (`design:57`) | `passuff-core` spawns `gh auth token` and `/bin/sh -c`; the login adapter is CLI-side | match |
| gh-plus-PAT message (`design:274-275`) | code emits `must not set a personal access token source when auth is gh_oauth` (`config:249`) | match |
| Step-definition file layout (`design:298-301`) | two files as named | match |
| Single-scenario `GIVN_SCENARIO` filter (`design:292-294`) | implemented (`features_runner.rs:29`) | match |
| Login retry "once per account per process" (`design:31`) | attempt set keyed by login (`credential.rs:147,203`) | **mismatch** (B9 residual) |

### Interaction cross-reference

| Inventory entry (`usecase.md` `## Interactions`) | Design matrix row | `@e2e` scenario | Step definitions | Promised driving mechanism |
|---|---|---|---|---|
| start continuous watching | yes (`design:362`) | `Developer starts watching and is notified of a new comment` | `e2e_steps::starts_watching` -> `World::e2e_start_watching` | real CLI subprocess, stdout transcript + desktop sink |
| run one observation pass | yes (`design:363`) | `Developer runs a single check and sees the current activity` | `e2e_steps::runs_one_check` -> `World::e2e_run_check` | real CLI subprocess, stdout transcript |
| show usage | yes (`design:364`) | `Developer asks for usage` | `e2e_steps::developer_runs_with` | real CLI subprocess, stdout |
| show version | yes (`design:365`) | `Developer asks for the version` | `e2e_steps::developer_runs_with` | real CLI subprocess, stdout |

All four inventory entries have a matrix row, a permanent `@e2e` scenario with
the same interaction, and step definitions that spawn the real binary. There is
no unmapped inventory entry and no extra `@e2e` scenario.

**Why no new `@e2e` scenario is valid.** The gh browser login is a recovery
precondition of "start continuous watching", not a new consumer action: it adds
no command, flag, endpoint, or screen, and the consumer observes the same
transcript/watch outcome. A variant of an action is a regular scenario. The login
variant is covered by regular scenarios (`feature:16-22`, `:25-32`, `:35-41`).
The four permanent `@e2e` scenarios remain exactly one per action.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Credential privacy constraint | `01`, `02` (`:16`) | affected | scope/constraints | R35 batch | `config:200-366`, `credential:106-131` | yes |
| gh CLI external system and credential interface | `03` (`:33-46`) | affected | tech decision | gh-session batch | `credential:159-215`, `gh_login.rs` | yes |
| Delegate OAuth to gh, PAT for headless | `04` (`:19-22`), ADR-0004 | affected | OAuth ownership | gh-session | `credential:190-216` | yes |
| `PassuffCore::with_ports` and `refresh` | `05` (`:54`) | affected | runtime flow | setup | `cli/src/lib.rs:126-142` | yes |
| Credential acquisition building block | `05` (`:122-136`) | affected | Component view | credential batches | `credential.rs` | yes |
| `CredentialSource` / `CredentialResolver` | `05` (`:19,58`) | affected | `CredentialSource` only | — | no `CredentialResolver` symbol exists | yes |
| "once per gh login per process" | `05` (`:135`), `08` (`:15-18`), ADR-0004:61 | affected | `design:31` says "once per account per process" | R38 batch | attempt set keyed by **login** (`credential.rs:147,203`) | **no** — design wording (B9) |
| Interactive recovery sequence | `06` (`:132-165`) | affected | runtime flow | gh-session | `credential:196-215`, `main:43-49` | **no** — `06:161` says "attempted for the account" (B9) |
| Deployment topology unchanged | `07` | not affected | — | — | no deployment change | yes |
| Configuration and credential rules | `08` (`:3-18`) | affected | data model | auth-config | `config:195-380` | yes |
| No token in transcript/state/evidence | `08` (`:79-89`), `10` QS-014 | affected | visual contract | token-absence batch | `feature:155-160`, step `:738-747` | yes |
| ADR-0004 in the register | `09` (`:14`) | affected | ADR impact | docs | ADR file + index | yes |
| QS-013, QS-014, QS-015 | `10` (`:45-47`) | affected | use-case traceability | gh-session, auth-config, identity | scenarios `feature:16,100,130,155` | yes |
| R-012..R-015 | `11` (`:18-21`) | affected | risks | all | code matches R-015 re-check (`credential:106-131`) | yes |
| ADR-0004 confirmation: validation matrix | ADR-0004 Confirmation | — | validation matrix | auth-config | `config:585-613` parses valid `gh_oauth`, host derivation, missing login | yes |
| Configured login set excludes a mismatched login | glossary `Configured login set`, `usecase.md` R1 | chapter 12 row | identity validation | identity batch | `cycle:79-113` excludes the login | yes |
| Identity-mismatch observation wording | `usecase.md` extension 1c | — | `design:206-221` | identity batch | `cycle:65-96` reports and skips | yes |

ARC42 CONFORMANCE: NOT CLEAN — B9: `docs/arc42/06-runtime-view.md:161` and
`design.md:31` still describe the login attempt as per account, contradicting
R38, `05:135`, `08:15-18`, ADR-0004, and the per-login implementation.

## Ubiquitous language conformance

| Glossary term | Definition owner | Spec / design wording | Code symbol | Verdict |
|---|---|---|---|---|
| Credential mode | `docs/glossary.md:14` | "credential mode" in usecase R32 and `design:6` | `AuthMode`, `CredentialSpec` | consistent |
| Expected login | `docs/glossary.md:15` | "expected login" in R36 and the feature steps | `AccountConfig.expected_login` | consistent |
| gh session | `docs/glossary.md:16` | "gh session" in R33/R34 and the feature steps | `CredentialSpec::GhOAuth` | consistent |
| Personal access token | `docs/glossary.md:17` | "personal access token" in R32/R35/R37 and the feature steps | `CredentialSpec::{Env,Command,File}` | consistent |
| Token file | `docs/glossary.md:18` | "token file" in R35 and the feature steps | `CredentialSpec::File` | consistent |
| Configured login set | `docs/glossary.md:27`, usecase R1:48 | R36 excludes a mismatched login from the set | `LoginSet` built from accepted observations + persisted logins (`cycle:96-113`) | consistent |
| Login retry | `design:31` | "once per account per process" | `GhOAuthCredential.attempted: HashSet<String>` keyed by login | **inconsistent** — contradicts R38 "once per gh login per process" (B9) |

Structural model: `docs/domain-model.md` records credential mode and expected
login as configuration/validation attributes of the Account, not new entities.
The gh CLI is an infrastructure external system recorded in arc42 chapter 03, not
a domain relationship; no aggregate, entity, or boundary changed, so no diagram
or relationship-table change is required. No structural finding.

## Coverage

Measured artifacts (merged report `coverage/cobertura-coverage.xml`): non-e2e
1084/1380 lines, e2e 547/1380 lines, merged 1163/1380 (84.3%) and 158/460
branches (34.3%). The README badge matches the merged report; the managed
`## Coverage` block is owned by the archive gate (archive instructions), so its
stale 89% text is not a finding.

Execution evidence verified from receipts/artifacts: delta regular 27/27
(`verification/tasks-result.json`), permanent regular 52/52, e2e 4/4 (re-run
through the configured command), unit 28 passed; strict-mode probe failed 1 with
exit 1.

### Residual gaps and classification

Bucket 1 — dead code (delete): **none**. `AccountConfig::auth_mode()` and the
`Default for SystemCredentialSource` impl are gone; no unused `CredentialResolver`
remains.

Bucket 2 — missing tests:

| Region | Rule | Finding |
|---|---|---|
| `config:213` | R32 | `github_user must be a login` (whitespace) is untested |
| `config:319-321` | R35 | HOME unset is untested; `tasks.md:114` calls it a "credential failure" while the code returns a configuration error (exit 2) |
| `credential:167` | R33 | gh token timeout is untested |
| `credential:170-173` | R33 | gh missing binary and other process errors are untested |
| `credential:179` | R33 | non-UTF8 gh output is untested |
| `credential:183` | R33 | empty gh output is untested |
| `gh_login:23-29` | R34 | login timeout and gh-missing branches are untested |
| `feature:100-105` | R35 | the missing-token-file Then asserts the error class but not the promised `token file is missing: <path>` detail |
| `cli/src/lib.rs:148,153,178-181,196,209,219-221,273-298` | R31 | state-error and delivery-error rendering (pre-existing, carried from the archived review) |
| `main.rs:20,27-39,71-72` | R19 | default config/state paths and the recording-delivery hook (pre-existing) |
| `cycle:71-72,118-119,150` | R17/R36 | gateway-error failure, login-change reset, and baseline-recorded branches (pre-existing) |
| `config:162-164,177,365-375` | R19 | configuration read error, no accounts, graphql-URL validation (pre-existing) |
| `classifier.rs`, `github.rs`, `state.rs`, `sound.rs`, `transcript.rs`, `delivery.rs` | various | pre-existing permanent-suite residuals (unchanged files) |

Bucket 3 — legitimately hard to test:

| Region | Technical justification |
|---|---|
| `config:337-341`, `credential:111,123` | The "cannot be inspected"/`FileUnreadable` branches require a filesystem change between the load check and the read (TOCTOU) or a non-`NotFound` metadata fault; not deterministically drivable without privilege or fault injection. |
| `config:390-392,420-422` | Serde `Visitor::expecting` diagnostics are reached only on a type-mismatch deserialization error; required trait methods, not a product rule (pre-existing). |
| `SystemGhLogin` real browser-consent success | A real `gh auth login --web` needs a browser, network, and interactive authorization; the port is faked and the process-status branches stay bucket 2. |

No classification outside the three buckets is used.

## E2e coverage

- Inventory mapping: four entries, four permanent `@e2e` scenarios, one per
  normalized action (cross-reference table above).
- No delta `@e2e` scenario; valid because the login is a variant of an existing
  action (see `Why no new @e2e scenario is valid`). No under- or over-coverage.
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
| missing token file names the path | code renders `token file is missing: <path>`; the scenario asserts the class only (coverage finding, not a visual defect) | text rule holds visually; path assertion missing (B6) |
| configuration errors name the account/path | code wraps `passuff: configuration error: <message>` with `account '<label>'`; `feature:86-97,181-194` asserts the message class | text rule holds |
| failed gh login names gh | `feature:56` asserts `gh auth login failed`; `GhLoginFailed` displays `gh auth login failed` (`credential.rs:24-25`) | contract finding B7 fixed |
| no token or token-file content | all four transcripts contain no token; `feature:155-160` asserts the credential value is in neither transcript nor state | holds and is asserted (B10) |

Persona `multi-account-developer`: reaches the goal for the four permanent
actions and sees labelled, actionable auth failures with no token. The
missing-token-file path assertion and the HOME-unset branch are dispositioned as
coverage findings B6, not visual defects.

VISUAL-REVIEW: PASS

## Overlap dispositions

Advisory shape/subset findings are dispositioned below. Shape pairs are variants
asserting different credential modes or rules; subset pairs are boundaries
isolating a distinct auth rule that the other scenario does not assert.

| Scenario A | Scenario B | Finding | Decision | Reason |
|---|---|---|---|---|
| A gh OAuth account uses the existing gh session | A token file readable only by the operator is used | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A token file path starting with ~ is expanded | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A token file path starting with ~ is expanded | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file path starting with ~ is expanded | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file path starting with ~ is expanded | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file path starting with ~ is expanded | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A personal access token whose login differs only in case is accepted | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A personal access token whose login differs only in case is accepted | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A legacy environment credential remains a personal access token account | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| An unavailable desktop surface keeps the transcript | A missing token file is an account failure | shape | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| An invalid configuration is rejected | A token file readable by other local users is refused | shape | variant | same configuration-rejection action with a different validation rule (R19 vs R35); merging would fold a new privacy rule into the generic scenario |
| A state write failure is reported with a nonzero status | A token file readable by other local users is refused | shape | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |
| A token file readable by other local users is refused | Developer runs a single check and sees the current activity | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A gh OAuth account uses the existing gh session | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable by other local users is refused | A disabled desktop notification keeps the transcript | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | An unavailable desktop surface keeps the transcript | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | A missing sound player still delivers the notification | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | A missing sound player still delivers the notification | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A gh OAuth account uses the existing gh session | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failed browser login does not stop the other account | A failed account does not stop the other account | subset | boundary | distinct rule boundary: R38 failed-account isolation vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable only by the operator is used | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable by other local users is refused | A failed account does not stop the other account | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file path starting with ~ is expanded | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A non-interactive watch never starts a browser login | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable by other local users is refused | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A gh OAuth account uses the existing gh session | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable by other local users is refused | A missing token file is an account failure | subset | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A failed browser login does not stop the other account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: R38 failed-account isolation vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable only by the operator is used | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable by other local users is refused | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file path starting with ~ is expanded | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| An invalid configuration is rejected | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A state write failure is reported with a nonzero status | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| An invalid configuration is rejected | A missing token file is an account failure | subset | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |
| A state write failure is reported with a nonzero status | A missing token file is an account failure | subset | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |
| Developer starts watching and is notified of a new comment | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A failed account does not stop the other account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| An invalid configuration is rejected | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A state write failure is reported with a nonzero status | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| A submitted review notifies | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| A failing CI result notifies | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| A CI recovery notifies | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| A CI recovery across pending notifies | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| A pull request merged or closed while the watcher was stopped is reported | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| A merge performed by the configured account still notifies | A credential value never appears in the transcript or state | subset | boundary | distinct rule boundary: the scenario asserts a different rule from the paired scenario, so merging would lose the discriminating outcome |
| Developer starts watching and is notified of a new comment | An invalid credential configuration is rejected | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R32 credential-mode validation (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |

135 findings dispositioned (117 subset, 18 shape).

## Split-or-keep

No scenario exceeds the gate's split threshold; no `[LONG]` finding was reported.
No split-or-keep rows are required.

## README impact decision

README-IMPACT: updated - Quick start, Prerequisites

`README.md:15-66` replaces the environment-only account example with
`gh_oauth`/`github_pat`, token-file permissions, the interactive-login rule, and
the identity rule. `README.md:81-89` adds the `gh` 2.40+ prerequisite. The
coverage badge is refreshed to the merged 84.3%/34.3%
(`README.md:3-6`); the managed `## Coverage` block is owned by the archive gate.

## Blockers

| ID | Blocker | Evidence | Required fix |
|---|---|---|---|
| B3 | `givn lint --change` and `givn check review` fail on the tasks.md `@e2e` cross-reference | `givn lint --change passuff-auth-modes` exits 2: `[WARN] @e2e scenarios referenced in tasks.md but none found in specs/`; `tasks.md:3` contains the literal `@e2e` | reword `tasks.md:3` so it does not contain the `@e2e` token (the delta reclassification itself is correct; do not add a delta `@e2e`) |
| B6 | HOME unset is untested and misclassified in tasks.md; the missing-token-file path detail is not asserted | `config:319-321` returns a configuration error for `~` with no `HOME`; `tasks.md:114` calls it a "credential failure"; `feature:104` asserts only `token file is missing`, not `token file is missing: <path>` promised by `design:274-275` | add a scenario/unit test for HOME unset (expect configuration error, exit 2), correct `tasks.md:114`, and assert the actual missing path |
| B7 | The gh external-boundary cases promised by `design:336-343` remain untested | `credential:167-183` (timeout, missing binary, non-UTF8, empty output) and `gh_login:23-29` (timeout, gh missing) have no scenario or unit test; the fake-gh harness can drive them | add the fake-gh boundary tests; keep the SystemGhLogin success/failure tests already added |
| B9 | Per-account wording contradicts the per-login rule; design-review.md is stale | `docs/arc42/06-runtime-view.md:161` says the login attempt is recorded "for the account"; `design.md:31` says "once per account per process"; `credential.rs:147,203` keys attempts by login; `design-review.md` was last written at `7fa8bed`, before the `design.md` edits in `8fd4ae9` | change both statements to "once per gh login per process"; update `design-review.md` (or record the same independent review context as covering the design changes) |
| C1 | `github_user` whitespace validation is untested | `config:213` rejects a login with whitespace; the invalid matrix (`feature:181-194`) does not include it | add the whitespace case to the validation scenario or a unit test |

B1, B2, B4, B5, B8, B10 are resolved and verified above.

## Sign-off checklist

- Fabrication audit: clean — 0 empty step bodies, 0 `unimplemented!()`, strict proof present.
- Every checked scenario has a batch commit touching production code: yes (`7fa8bed`).
- Every promised component exists: yes (four adapters, `GhLogin`, `WatchControl`).
- Every consumer uses only its declared contract: yes (composition root + tests excepted).
- Strict-mode proof present and passing: yes (`tasks.md:13-21`).
- Scenario execution evidence per scope: delta 27/27 receipt, permanent 52/52, e2e 4/4 via the configured command, unit 28.
- Coverage classified from the merged report: yes; gaps resolved: **no** (B6, B7, C1).
- Dead code deleted: yes (B8).
- No `@wip`; no implementation detail in the spec: yes.
- Canonical E2E policy: every inventory action has one `@e2e`; variants are regular: yes.
- Real-interface assertions and reviewed driver: yes (spawned CLI binary).
- Local run command: the declared `run.command` is a CLI `--help` invocation; the e2e child starts the same binary and exits cleanly for usage and version.
- Exact e2e file(s) identified and tree searched for a parallel implementation: yes; no parallel implementation.
- Explicit scope selection: `GIVN_RESULT_SCOPE=e2e`; the configured command now runs 4/4 (B1 fixed).
- Implementation matches design (commands, layout, driver): **no** — B9 wording.
- Interaction coverage verified: yes.
- Ubiquitous language and structural model: **no** — design:31 per-account wording (B9).
- No finding excused outside the three buckets: yes — every gap is bucket 1, 2, or 3.

REVIEW: FAIL
