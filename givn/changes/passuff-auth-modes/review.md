# Review: passuff-auth-modes

Review kind: independent
PERSONAS: multi-account-developer
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

## Implementation evidence check

### @e2e tag integrity

| Feature file | @e2e before | @e2e after | Change |
|---|---|---|---|
| `givn/changes/passuff-auth-modes/specs/watch-pull-requests/pr-activity-notification.feature` | 0 | 0 | no scenario was tagged or untagged by this change |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | 4 | unmodified by this change; the four titles are `Developer starts watching and is notified of a new comment`, `Developer runs a single check and sees the current activity`, `Developer asks for usage`, `Developer asks for the version` (`feature:4-38`) |

No `@e2e` tag was removed from any delta scenario, because the delta never
contained one: the plan (design `Interaction coverage`, `Testing strategy`)
reclassifies the browser-login case as a regular variant of an existing action.
`givn lint` still fires its tasks.md cross-reference warning (blocker B3): the
E2E Smoke Tests section of `tasks.md` names `@e2e` while no delta spec carries
the tag.

### Empty step bodies

Brace-counted scan of every `fn` in both step files: **0 empty/trivial bodies
found across 2 files** (71 functions in
`crates/passuff-cli/tests/steps/pr_activity_notification.rs`, 6 in
`crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs`). No
`unimplemented!()` or `todo!()` remains anywhere under `crates/`. Every step
body calls a `World` operation that resolves a credential, drives the runtime,
or asserts on transcript/report/attempt state.

### Scenario commit evidence

| Checked scenarios | `COMMIT:` entries | Batch | Evidence |
|---|---|---|---|
| 15 non-e2e scenario tasks | 15 | `auth-config`, `credential-resolution`, `gh-session`, `identity-and-watch` | each names its batch and `7fa8bed` |
| 1 E2E smoke task | 1 | `e2e` | names `7fa8bed` |

`git show --stat 7fa8bed` touches production code
(`crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/{credential,cycle,domain}.rs`,
`crates/passuff-cli/src/{lib,main,gh_login}.rs`), the shared harness, both step
files, and the spec/docs. The shared batch revision is valid: every `COMMIT:`
line names `Batch: <name>`. The change is not marked `CHARACTERIZATION`.

### Components and contracts

| Contract (design `Contracts and Components`) | Promised component | Present |
|---|---|---|
| `CredentialSource` | credential resolver in `passuff-core` | yes: `SystemCredentialSource` (`credential.rs:42`) |
| `CredentialSource` source adapters | `EnvironmentCredential`, `CommandCredential`, `FileCredential`, `GhOAuthCredential` | **no** — the four sources are inline `match` arms in `SystemCredentialSource::resolve` (`credential.rs:154-188`); no such types exist in the tree (blocker B4) |
| `GhLogin` | CLI composition-root adapter plus a recording fake | yes: `SystemGhLogin` (`crates/passuff-cli/src/gh_login.rs:10`) and `RecordingGhLogin` (tests) |
| `WatchControl` | `passuff-cli` runtime control | yes: `Interrupt` and `StopAfter` (`cli/src/lib.rs:42-104`) |

Consumers use the declared surfaces: `CliPorts` holds `Arc<dyn CredentialSource>`,
`Arc<dyn GitHubGateway>`, `Arc<dyn SnapshotStore>`, `Arc<dyn NotificationDelivery>`;
`gh_login.rs` imports the `passuff_core::{CredentialError, GhLogin}` contract;
no consumer imports a core implementation type. The `WatchControl` implementation
adds a `wait` method not named in `design:245-248` (superset, non-blocking).

### Strict-mode proof

`tasks.md:13-21` records the undefined-step probe: `FAILED: Strict mode probe ::
Undefined step must fail`, `undefined Given`, `total 1, passed 0, failed 1`, cargo
exit 1. The runner enforces it with `.fail_on_skipped()` and a non-zero exit for
`total == 0` (`crates/passuff-cli/tests/features_runner.rs:49,92-96`).

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
- `scripts/verify.sh` sets `GIVN_RESULT_SCOPE=e2e` for `--e2e`, then defaults
  `GIVN_FEATURES` to `$root/givn/changes` whenever a change spec directory
  exists (`verify.sh:20-33`). The runner only filters by scope; it does not
  swap corpora.
- Executed configured command: `./scripts/verify.sh --e2e` -> `total 0` and
  exit 1. With the same script and `GIVN_FEATURES=givn/specs` -> `total 4,
  passed 4, failed 0, skipped 0`, exit 0. Blocker B1: the configured E2E command
  cannot execute the four permanent `@e2e` scenarios during a change.
- Parallel implementation search: only two step files exist
  (`pr_activity_notification.rs`, `pr_activity_notification_e2e.rs`), plus the
  single driver in `tests/support/e2e.rs`; no untracked or alternative e2e
  implementation was found.

### Design deviation check

| `design.md` statement | As built | Disposition |
|---|---|---|
| Four named source adapters (`design:63`) | inline match arms | blocker B4 |
| `verify.e2e_command` with `--e2e` and explicit scope (`design:286-288`) | scope is explicit but the corpus is not; command yields 0 scenarios | blocker B1 |
| "the core never starts a process" (`design:57`) | `passuff-core` spawns `gh auth token` and `/bin/sh -c` (`credential.rs:68,164`) | blocker B9 |
| gh-plus-PAT message (`design:274-275`) | code emits `must not set a personal access token source when auth is gh_oauth` (`config:258`) | blocker B9 |
| Step-definition file layout (`design:298-301`) | two files as named | match |
| Single-scenario `GIVN_SCENARIO` filter (`design:292-294`) | implemented (`features_runner.rs:28`) | match |

### Interaction cross-reference

| Inventory entry (`usecase.md` `## Interactions`) | Design matrix row | `@e2e` scenario | Step definitions | Promised driving mechanism |
|---|---|---|---|---|
| start continuous watching | yes (`design:359`) | `Developer starts watching and is notified of a new comment` | `e2e_steps::starts_watching` -> `World::e2e_start_watching` | real CLI subprocess, stdout transcript + desktop sink |
| run one observation pass | yes (`design:360`) | `Developer runs a single check and sees the current activity` | `e2e_steps::runs_one_check` -> `World::e2e_run_check` | real CLI subprocess, stdout transcript |
| show usage | yes (`design:361`) | `Developer asks for usage` | `e2e_steps::developer_runs_with` | real CLI subprocess, stdout |
| show version | yes (`design:362`) | `Developer asks for the version` | `e2e_steps::developer_runs_with` | real CLI subprocess, stdout |

All four inventory entries have a matrix row, a permanent `@e2e` scenario with
the same interaction, and step definitions that spawn the real binary. There is
no unmapped inventory entry and no extra `@e2e` scenario.

**Why no new `@e2e` scenario is valid.** The gh browser login is a recovery
precondition of "start continuous watching", not a new consumer action: it adds
no command, flag, endpoint, or screen, and the consumer observes the same
transcript/watch outcome. Under the canonical E2E policy a variant of an
action is a regular scenario, and a second `@e2e` for the same action is
over-coverage. The login variant is proven by regular scenarios
(`feature:16-22` interactive login; `feature:24-41` check and non-interactive
refusal). The four permanent `@e2e` scenarios remain exactly one per action.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Credential privacy constraint | `01`, `02` (`:16`) | affected | scope/constraints | R35 batch | `config:340-366`, `credential:126-142` | yes |
| gh CLI external system and credential interface | `03` (`:33-46`) | affected | tech decision | gh-session batch | `credential:65-93`, `gh_login.rs` | yes |
| Delegate OAuth to gh, PAT for headless | `04` (`:19-22`), ADR-0004 | affected | OAuth ownership | gh-session | `credential:95-116` | yes |
| `PassuffCore::with_ports` and `refresh` | `05` (`:54`) | affected | runtime flow | setup | `cli/src/lib.rs:126-142` | yes |
| Credential acquisition building block | `05` (`:122-136`) | affected | Component view | credential batches | `credential.rs` | yes |
| `CredentialSource` / `CredentialResolver` | `05` (`:19,58`) | affected | `CredentialSource` only | — | no `CredentialResolver` symbol exists | **no** — stale name (B9) |
| "once per account per process" | `05` (`:135`), `08` (`:15-17`), ADR-0004 Confirmation | affected | login retry `design:31` | R38 batch | attempt set keyed by **login** (`credential.rs:45,100-108`) | **no** (B9) |
| Interactive recovery sequence | `06` (`:132-165`) | affected | runtime flow | gh-session | `credential:95-116`, `main:43-49` | yes (wording "for the account" vs login-keyed attempt: B9) |
| Deployment topology unchanged | `07` | not affected | — | — | no deployment change | yes |
| Configuration and credential rules | `08` (`:3-18`) | affected | data model | auth-config | `config:200-370` | yes |
| No token in transcript/state/evidence | `08` (`:79-89`), `10` QS-014 | affected | visual contract | — | code prints no token; no test asserts absence | yes on code; untested (B10) |
| ADR-0004 in the register | `09` (`:14`) | affected | ADR impact | docs | ADR file + index | yes |
| QS-013, QS-014, QS-015 | `10` (`:45-47`) | affected | use-case traceability | gh-session, auth-config, identity | scenarios `feature:16,86,115` | yes |
| R-012..R-015 | `11` (`:18-21`) | affected | risks | all | code matches R-015 re-check (`credential:126-132`), untested (B6) | yes on code |
| ADR-0004 confirmation: "one attempt per process" | ADR-0004 Confirmation | — | login retry | R38 | per-login attempt set | **no** (B9) |
| ADR-0004 confirmation: validation tests cover the matrix | ADR-0004 Confirmation | — | validation matrix | auth-config | invalid matrix covered; valid `gh_oauth` parse path unexecuted (`config:261-267`) | **partial** (B5) |
| Configured login set excludes a mismatched login | glossary `Configured login set`, `usecase.md` R1 | chapter 12 row | identity validation | identity batch | `cycle:79-113` excludes the login | **no** — R1/glossary still say "all configured accounts" (B9) |
| Identity-mismatch observation wording | `usecase.md` extension 1c | — | `design:206-221` | identity batch | `cycle:65-75` observes, then `:79-96` rejects | **no** — 1c says "does not observe" (B9) |

ARC42 CONFORMANCE: NOT CLEAN — B4, B9 are unresolved.

## Ubiquitous language conformance

| Glossary term | Definition owner | Spec / design wording | Code symbol | Verdict |
|---|---|---|---|---|
| Credential mode | `docs/glossary.md` | "credential mode" in usecase R32 and `design` (scope says "authentication mode" at `design:5`) | `AuthMode`, `CredentialSpec` | minor drift: "authentication mode" is not a glossary term (B9) |
| Expected login | glossary | "expected login" in R36 and the feature steps | `AccountConfig.expected_login` | consistent |
| gh session | glossary | "gh session" in R33/R34 and the feature steps | `CredentialSpec::GhOAuth` | consistent |
| Personal access token | glossary | "personal access token" in R32/R35/R37 and the feature steps (design also uses the abbreviation PAT) | `CredentialSpec::{Env,Command,File}` | consistent |
| Token file | glossary | "token file" in R35 and the feature steps | `CredentialSpec::File` | consistent |
| Configured login set | glossary, usecase R1 | R36 excludes a mismatched login from the set | `LoginSet` built from accepted observations + persisted logins (`cycle:96-113`) | **inconsistent**: R1 and the glossary still define the set as "all configured accounts" (B9) |

Structural model: `docs/domain-model.md` was updated (paragraph on credential
mode/expected login as attributes; external `gh CLI` is not yet drawn — the
model's external systems remain GitHub, desktop surface, audio). The change
adds no entity or aggregate, so no diagram change is required; the paragraph is
consistent with the implemented structure. No structural finding.

## Coverage

Measured artifacts (merged report `coverage/cobertura-coverage.xml`):
non-e2e 949/1273 lines, e2e 535/1273 lines, merged 1032/1273 (81.07%) and
136/466 branches (29.18%). The README badge matches the merged report; the
`## Coverage` managed block is updated by the archive gate (archive
instructions, post-archive README update), so its stale text is not a finding.

Execution evidence verified during this review: delta regular 23/23
(`verification/tasks-result.json`; re-run 23/23), permanent regular 52/52
(re-run), permanent e2e 4/4 (with the permanent corpus), unit 20 passed.

### Residual gaps and classification

Bucket 1 — dead code (delete):

| Region | Finding |
|---|---|
| `crates/passuff-config/src/lib.rs:46-51` | `AccountConfig::auth_mode()` has no caller anywhere in the workspace |
| `crates/passuff-core/src/credential.rs:146-150` | `Default for SystemCredentialSource` has no constructor call; `headless()`/`with_login()` are used directly |

Bucket 2 — missing tests (blockers unless stated otherwise):

| Region | Rule | Finding |
|---|---|---|
| `config:261-267` | R32/R33 | no scenario parses a **valid** `auth = "gh_oauth"` + `github_user` configuration; the gh scenarios build `AppConfig` in memory (`support/mod.rs:486-510`), so this construction and its `expected_login` are unexecuted (B5) |
| `config:317-322` | R33 | `gh_host` (api.github.com -> github.com, GHES host) is never executed (B5) |
| `config:220-225` | R32 | `github_user must be a login` (whitespace) has no scenario |
| `config:327-331`, `340-343`, `351-356` | R35 | HOME-unset `~`, relative-path rejection, and non-regular-file rejection are promised in the design boundary table but have no scenario (B6) |
| `credential:126-132` | R35 | the resolve-time permission re-check (load-to-use window) has no scenario (B6) |
| `credential:139` | R35 | an empty token file has no scenario (B6) |
| `credential:73,76-77,79,85,88-89` | R33 | gh timeout, missing binary, non-zero exit, non-UTF8 and empty-output branches have no scenario (B7) |
| `credential:110-115` | R38 | a login that runs and then fails, and the retry after a failed login, have no scenario; the observed failure text is `credential command failed` and does not name gh, contrary to `design:414` (B7) |
| `gh_login.rs:11,23-29,35` | R34 | the new `SystemGhLogin` adapter has zero covering lines; its exit-status/NotFound branches are drivable with the existing fake-gh-on-PATH harness (B7) |
| `main.rs:43-46` | R34 | the real TTY -> `SystemCredentialSource::interactive(SystemGhLogin)` composition root is never executed; e2e child processes are non-TTY (B7) |
| feature `:99-103` | R35 | the missing-token-file Then asserts only `account work failed`, not the promised `token file is missing: <path>` detail; a generic credential failure passes |
| no scenario | R35 / QS-014 | no assertion that the resolved token value never appears in transcript or state (B10) |
| `cli/src/lib.rs:136,141,166-169,184,197,207-209,261-286` | R31 | state-error and watch-error rendering paths are unexecuted (pre-existing, testable with the failing store) |
| `cycle:71-72,118-119,150` | R17/R36 | gateway-error failure, login-change reset, and baseline-recorded branches are unexecuted (pre-existing; testable with the fake gateway) |
| `config:171-173,186,374-377,384-386` | R19 | configuration read error, no-accounts, and graphql-URL validation branches are unexecuted (pre-existing) |
| `classifier.rs`, `github.rs`, `state.rs`, `sound.rs`, `transcript.rs`, `delivery.rs` | various | pre-existing permanent-suite residuals (132 uncovered lines in these six unchanged files; 241 total across the merged report); carried from the archived review, not introduced by this change |

Bucket 3 — legitimately hard to test:

| Region | Technical justification |
|---|---|
| `gh_login.rs` real browser consent success | a real `gh auth login --web` requires a browser, a network round trip, and an interactive GitHub authorization; it cannot be deterministic in the suite. The `GhLogin` port is faked at the credential boundary; the process-status and timeout branches remain bucket 2 |
| `config:399-401,429-431` | serde `Visitor::expecting` diagnostics are reached only on a type-mismatch deserialization error; required trait methods, not a product rule (pre-existing) |

## E2e coverage

- Inventory mapping: four entries, four permanent `@e2e` scenarios, one per
  action (cross-reference table above).
- No delta `@e2e` scenario; valid because the login is a variant of an existing
  action (see `Why no new @e2e scenario is valid`). No under- or over-coverage.
- Real-interface primary assertion: stdout of the spawned `passuff` binary, plus
  the desktop sink and sound log; no browser route, no fetch, no direct API call.
- **Non-browser classification.** The opposite classification is "Web UI: a
  rendered page". It does not hold: passuff renders no HTML, CSS, or JavaScript,
  serves no HTTP page, and has no browser client. The consumer boundary is a
  terminal process invocation; a browser driver has no surface to attach to, and
  an HTTP client would be a lower-level substitute for the CLI. The terminal
  transcript is therefore the correct visual artifact (terminal-interface rule),
  and API-only exemption does not apply because the CLI transcript is the
  interface.

## Visual review

Interface: terminal. Evidence: the four permanent `@e2e` transcripts produced by
this review run in `target/visual/`:
`developer-asks-for-the-version/transcript.txt`,
`developer-asks-for-usage/transcript.txt`,
`developer-runs-a-single-check-and-sees-the-current-activity/transcript.txt`,
`developer-starts-watching-and-is-notified-of-a-new-comment/transcript.txt`.
No delta `@e2e` scenario exists by design (`design:378-386`), so no delta
screenshot/transcript is owed.

| Declared state / text rule | Evidence | Verdict |
|---|---|---|
| watch start and notification | `developer-starts-watching...`: `passuff: Fresh baseline recorded for acme/api#42`, `passuff: watching`, `passuff: acme/api#42`, `New comment from @alice` | baseline holds: labelled, no internal identifiers, no duplicate content, no token |
| one-shot check | `developer-runs-a-single-check...`: `passuff: acme/api#42` / `New comment from @alice`, exit 0 | baseline holds |
| usage and version | real title line, command list, `passuff 0.1.0`, exit 0 | baseline holds; no internal state names |
| identity mismatch names both logins | asserted by `feature:125` (`credential belongs to @octo-other, expected @octo-work`); composed at `cycle:84-90` | text rule holds |
| configuration errors name the account/path | asserted by `feature:89`, `161-162`, `166-171`; rendered as `passuff: configuration error: <message>` | text rule holds |
| no token or token-file content | all four transcripts contain no token; code paths carry only paths and messages | holds visually; not asserted (B10) |
| failed gh login names gh | transcript produced by the run: `passuff: account alpha failed: credential command failed` | **contract finding**: `design:414` promises the account failure names gh (B7) |

Persona `multi-account-developer`: reaches the goal for the four permanent
actions (starts watching, runs a check, reads usage and version) with what is
shown. The failed-login diagnostic is dispositioned as blocker B7 (fix the
credential-error text so a gh failure names gh).

VISUAL-REVIEW: PASS

## Overlap dispositions

Advisory shape/subset findings are dispositioned below. Shape pairs are variants asserting different credential modes or rules; subset pairs are boundaries isolating a distinct auth rule that the other scenario does not assert.

| Scenario A | Scenario B | Finding | Decision | Reason |
|---|---|---|---|---|
| A CI recovery across pending notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery across pending notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A CI recovery notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A failed account does not stop the other account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A failed browser login does not stop the other account | A failed account does not stop the other account | subset | boundary | distinct rule boundary: R38 failed-account isolation vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A failed browser login does not stop the other account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: R38 failed-account isolation vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A failing CI result notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A failing CI result notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy command credential remains a personal access token account | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A legacy environment credential remains a personal access token account | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A merge performed by the configured account still notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A missing token file is an account failure | A disabled desktop notification keeps the transcript | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | A failed account does not stop the other account | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | A missing sound player still delivers the notification | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | An unavailable desktop surface keeps the transcript | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A missing token file is an account failure | Developer runs a single check and sees the current activity | subset | boundary | distinct rule boundary: R35 missing token file account failure (exit 1) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| An invalid configuration is rejected | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| An invalid configuration is rejected | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A non-interactive watch never starts a browser login | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A personal access token whose login differs only in case is accepted | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A pull request merged or closed while the watcher was stopped is reported | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A review comment from another person notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A state write failure is reported with a nonzero status | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A state write failure is reported with a nonzero status | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A failed browser login is attempted once per account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 one attempt per login in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | An interactive watch starts a browser login when the gh session is missing | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A non-interactive watch never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A submitted review notifies | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file path starting with ~ is expanded | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable by other local users is refused | A disabled desktop notification keeps the transcript | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | A failed account does not stop the other account | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | A missing sound player still delivers the notification | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | An unavailable desktop surface keeps the transcript | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable by other local users is refused | Developer runs a single check and sees the current activity | subset | boundary | distinct rule boundary: R35 token-file permission configuration error (exit 2) vs the rule asserted by scenario B; merging would lose the discriminating outcome |
| A token file readable only by the operator is used | A failed account does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | A failed browser login does not stop the other account | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R38 failed-account isolation in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | A short-lived pull request merged or closed between checks notifies | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| A token file readable only by the operator is used | Each configured account is observed | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A missing token file is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 missing token file account failure (exit 1) in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | An invalid credential configuration is rejected | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R32 credential-mode validation (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A one-shot check never starts a browser login | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs the auth rule in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A personal access token for a different login is an account failure | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R36 identity failure and login-set exclusion in B; the step-prefix match is a shape artifact, not a shared rule |
| Developer starts watching and is notified of a new comment | A token file readable by other local users is refused | subset | boundary | distinct rule boundary: the permanent rule asserted by A vs R35 token-file permission configuration error (exit 2) in B; the step-prefix match is a shape artifact, not a shared rule |
| A gh OAuth account uses the existing gh session | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A token file path starting with ~ is expanded | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A gh OAuth account uses the existing gh session | A token file readable only by the operator is used | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A legacy environment credential remains a personal access token account | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| An invalid configuration is rejected | A missing token file is an account failure | shape | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |
| An invalid configuration is rejected | A token file readable by other local users is refused | shape | variant | same configuration-rejection action with a different validation rule (R19 vs R35); merging would fold a new privacy rule into the generic scenario |
| A personal access token whose login differs only in case is accepted | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A personal access token whose login differs only in case is accepted | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A state write failure is reported with a nonzero status | A missing token file is an account failure | shape | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |
| A state write failure is reported with a nonzero status | A token file readable by other local users is refused | shape | boundary | different failure class (state/configuration vs credential or identity); the shared step skeleton does not assert the same rule |
| A token file path starting with ~ is expanded | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file path starting with ~ is expanded | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file path starting with ~ is expanded | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable by other local users is refused | A missing token file is an account failure | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A legacy command credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A legacy environment credential remains a personal access token account | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A personal access token whose login differs only in case is accepted | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |
| A token file readable only by the operator is used | A token file path starting with ~ is expanded | shape | variant | same step skeleton across different credential modes/rules (R32-R38); merging hides which mode or rule regressed |

113 subset findings and 20 shape findings dispositioned (133 rows).

## Split-or-keep

No scenario exceeds the gate's split threshold; no split-or-keep rows are required.
(No `[LONG]` finding was reported by `givn lint` or `givn check review`.)

## README impact decision

README-IMPACT: updated - Quick start, Prerequisites

`README.md:40-56` replaces the environment-only account example with
`gh_oauth`/`github_pat`, token-file permissions, the interactive-login rule, and
the identity rule (Quick start). `README.md:81-84` adds the `gh` 2.40+
prerequisite. The coverage badge is refreshed to the merged 81.1%/29.2%; the
managed `## Coverage` block is owned by the archive gate.

## Blockers

| ID | Blocker | Evidence | Required fix |
|---|---|---|---|
| B1 | The configured E2E command does not execute the e2e suite | `./scripts/verify.sh --e2e` -> `total 0`, exit 1; 4/4 only with `GIVN_FEATURES=givn/specs` (`commands.yaml`, `verify.sh:20-33`) | select the permanent corpus for the e2e scope in `verify.sh`/`commands.yaml`, then re-run the configured command and record the receipt |
| B2 | Stale change-verification receipt; the mechanical preflight fails | `givn check review`: tested content `1f4225...` != current `657d15...`; receipt head `0023de1`, later commits `ad93cff`, `076c42d` | re-run `givn check tasks --change passuff-auth-modes` on the current HEAD |
| B3 | `givn lint` exits 2 on the tasks.md `@e2e` cross-reference | `[WARN] @e2e scenarios referenced in tasks.md but none found in specs/` | resolve the cross-reference in `tasks.md` (the reclassification itself is correct; do not add a delta `@e2e`) |
| B4 | Four promised source adapters are missing | `design:63` names `EnvironmentCredential`, `CommandCredential`, `FileCredential`, `GhOAuthCredential`; only `SystemCredentialSource` exists | implement the named adapters or update `design.md` and re-run design-review |
| B5 | The valid `gh_oauth` configuration path is untested | `config:261-267`, `317-322` unexecuted; no scenario parses `auth = "gh_oauth"` | add a scenario that configures a gh account through TOML and observes the gh session token; cover the host derivation |
| B6 | R35 token-file boundaries are untested and one Then is too weak | `config:327-331,340-343,351-356`; `credential:126-132,139`; `feature:102` asserts only the generic failure | add the boundary scenarios and assert `token file is missing: <path>` |
| B7 | gh process/login boundaries and the interactive composition root are untested; failed login does not name gh | `credential:73-89,110-115`; `gh_login.rs` zero coverage; `main:43-46` unexecuted; transcript `credential command failed` | cover the fake-gh status branches, wire a TTY/watch test to the real composition root, and make the login failure name gh |
| B8 | Dead code | `config:46-51` `auth_mode()`; `credential:146-150` `Default` impl | delete both |
| B9 | Documentation and contract conformance mismatches | per-account vs per-login attempt (`05:135`, `08:15-17`, ADR-0004); "core never starts a process" (`design:57`); gh+PAT message (`design:274-275` vs `config:258`); stale `CredentialResolver` (`05:19,58`); R1/glossary login-set definition vs R36; extension 1c "does not observe"; "authentication mode" vs "credential mode" | correct the affected docs/spec/design; if `design.md` changes, re-run design-review |
| B10 | No test asserts that tokens never reach the transcript or state | QS-014 and `design:373-376`; no negative token assertion in any scenario | add a transcript/state assertion that the resolved token does not appear |

## Sign-off checklist

- Fabrication audit: clean — 0 empty step bodies, 0 `unimplemented!()`, strict proof present.
- Every checked scenario has a batch commit touching production code: yes (`7fa8bed`).
- Every promised component exists: **no** — B4.
- Every consumer uses only its declared contract: yes.
- Strict-mode proof present and passing: yes (`tasks.md:13-21`).
- Scenario execution evidence per scope: delta regular receipt + re-run; permanent regular and e2e re-run; the configured e2e command is broken (B1).
- Coverage classified from the merged report: yes; gaps resolved: **no** (B5-B7, B10).
- Dead code deleted: **no** (B8).
- No `@wip`; no implementation detail in the spec: yes.
- Canonical E2E policy: every inventory action has one `@e2e`; variants are regular: yes.
- Real-interface assertions and reviewed driver: yes (spawned CLI binary).
- Local run command: the declared `run.command` is `cargo run --release -p passuff-cli -- --help`; the product is a CLI with no stack to start, and the e2e child starts the same binary (debug profile) and exits cleanly for usage and version.
- Exact e2e file(s) identified and tree searched for a parallel implementation: yes; command binding fails (B1).
- Explicit scope selection: `GIVN_RESULT_SCOPE=e2e` set by `verify.sh`; the corpus selection is missing (B1).
- Implementation matches design (commands, layout, driver): **no** — B1, B4, B9.
- Interaction coverage verified: yes.
- No finding excused outside the three buckets: yes — every gap is dead code, missing test, or a justified hard-to-test case.

REVIEW: FAIL
