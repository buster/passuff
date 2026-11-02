# Tasks: passuff-auth-modes

Behavior batches. Non-@e2e first, then @e2e. One commit may serve several scenarios in a batch.

Legend: an unchecked box means pending; a checked box is done only when the batch commit is filled in. Check boxes immediately after each batch.

---

## Strict-mode proof (do first)

Empty/no-op steps are indistinguishable from PASS. Prove the runner rejects an unimplemented step before any scenario work.

- [x] **Proof-of-strictness**: run `./scripts/verify.sh` against an undefined/`unimplemented!()` step in the new delta. Confirm a non-zero exit and `failed > 0`.

      ```
./scripts/verify.sh with GIVN_FEATURES=/tmp/auth-strict-probe.feature
FAILED: Strict mode probe :: Undefined step must fail
  undefined Given  -> no auth step definition binds this phrase
givn test-result: total 1, passed 0, failed 1, skipped 0
cargo exit status: 1
      ```

---

## Domain Constraints

| Rule | Constraint | Persona impact |
|---|---|---|
| R32 | Exactly one credential source per account; gh OAuth needs an expected login; literal tokens are rejected. | Configuration mistakes fail before any observation. |
| R33 | gh OAuth obtains its token from the operator's gh session for the declared login and host. | Existing gh setup is reused. |
| R34 | Interactive watch may start gh's browser login; `check` and non-interactive runs never do. | No browser popups in scripts. |
| R35 | PAT sources: environment, helper command, or private token file (`0600`/`0400`); missing file is an account failure, insecure file is a configuration error. | Credentials stay out of config and cannot leak to other users. |
| R36 | A credential for a different login (case-insensitive) is an account failure naming both logins; its pull requests are not processed and its login does not join the configured login set. | A wrong token cannot silently watch the wrong account or suppress another account's activity. |
| R37 | A legacy environment/command configuration without `auth` remains a PAT account. | Existing configurations keep working. |
| R38 | A failed browser login is attempted once per account per process and does not stop other accounts. | No browser loops; partial outages still notify. |

---

## Setup

- [x] Set up the credential test harness: fake `gh` on `PATH` (token per login, login marker, invocation log), `RecordingGhLogin` for per-account attempt recording, token-file modes `600`/`400`/`640`/`604`/`660`, `HOME` override for `~`, credential-rotation helper, and the negative notification assertion (`no notification for "<key>"`). Implement `GIVN_SCENARIO` exact-title filtering in `features_runner.rs`. Keep `verify.command: ./scripts/verify.sh` and `verify.e2e_command: ./scripts/verify.sh --e2e`.

      Fake gh on PATH (token per login), RecordingGhLogin per-account attempts, token modes 600/400/640/604/660, HOME override, credential rotation, no-notification step, GIVN_SCENARIO filter.
      COMMIT 7fa8bed.
      `./scripts/verify.sh` delta: 23/23; permanent: 52/52.

- [x] Prove strict mode with the new delta, then return to a passing baseline. Record the e2e scenario-count proof (e2e 4 < full count) and the exact new count.

      Strict mode proven for the delta runner with the undefined-step probe above (failed 1, exit 1).
      Delta regular: 23/23. Permanent regular: 52/52. E2E: 4/4.
      Scenario-count proof: e2e 4 < full corpus 75 (52 permanent + 23 delta).

---

## Non-E2E Tasks

### Batch: `auth-config`

- [x] Scenario: `A token file readable by other local users is refused`
  - [x] **Design constraints:** R35. `ConfigLoader::load` checks `mode & 0o077 == 0` and owner read; modes `640`, `604`, `660` fail with `token_file is not private: <path>`; exit 2; no observation.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A token file readable by other local users is refused" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `auth-config`

- [x] Scenario: `An invalid credential configuration is rejected`
  - [x] **Design constraints:** R32. Matrix: no source, unknown `auth`, two PAT sources, gh OAuth without `github_user`, gh OAuth plus a PAT source, literal `token`. Exact messages: `must select exactly one credential source`, `has unknown auth mode`, `requires github_user`, `must not set a personal access token source`, `must not contain a literal token`; exit 2.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "An invalid credential configuration is rejected" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `auth-config`

### Batch: `credential-resolution`

- [x] Scenario: `A token file readable only by the operator is used`
  - [x] **Design constraints:** R35. `CredentialSpec::File` reads the expanded path, requires a regular file, re-checks `0o077`, trims, rejects empty; modes `600` and `400` accepted; notification after a baseline and a new comment.
        Files: `crates/passuff-core/src/credential.rs`, `crates/passuff-core/src/cycle.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A token file readable only by the operator is used" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `credential-resolution`

- [x] Scenario: `A missing token file is an account failure`
  - [x] **Design constraints:** R35. A missing file resolves as a credential failure with `token file is missing: <path>`; `check` exits 1; the failure is an account failure, not a configuration error.
        Files: `crates/passuff-core/src/credential.rs`, `crates/passuff-core/src/cycle.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A missing token file is an account failure" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `credential-resolution`

- [x] Scenario: `A token file path starting with ~ is expanded`
  - [x] **Design constraints:** R35. `~`/`~/...` expand from `HOME`; a relative path is a configuration error; `HOME` unset is a credential failure. The test sets `HOME` to a temp directory.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/credential.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A token file path starting with ~ is expanded" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `credential-resolution`

- [x] Scenario: `A legacy environment credential remains a personal access token account`
  - [x] **Design constraints:** R37. No `auth` and `token_env` parses as PAT; existing resolution and notification behavior; no migration required.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/credential.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A legacy environment credential remains a personal access token account" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `credential-resolution`

- [x] Scenario: `A legacy command credential remains a personal access token account`
  - [x] **Design constraints:** R37. No `auth` and `token_command` parses as PAT; `/bin/sh -c`, 10-second timeout, trimmed output.
        Files: `crates/passuff-config/src/lib.rs`, `crates/passuff-core/src/credential.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A legacy command credential remains a personal access token account" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `credential-resolution`

### Batch: `gh-session`

- [x] Scenario: `A gh OAuth account uses the existing gh session`
  - [x] **Design constraints:** R33. `CredentialSpec::GhOAuth` runs `gh auth token --user <login> --hostname <host>`; host `github.com` for `api.github.com`, otherwise the GraphQL host; output trimmed; notification after a baseline.
        Files: `crates/passuff-core/src/credential.rs`, `crates/passuff-core/src/cycle.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A gh OAuth account uses the existing gh session" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `gh-session`

- [x] Scenario: `An interactive watch starts a browser login when the gh session is missing`
  - [x] **Design constraints:** R33/R34. `GhLogin` port with inherited stdio and `gh auth login --web --hostname <host>`; only when watch + stdin/stdout TTY; retry `gh auth token` once; the watch reaches `passuff: watching`.
        Files: `crates/passuff-core/src/credential.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/src/main.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "An interactive watch starts a browser login when the gh session is missing" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `gh-session`

- [x] Scenario: `A one-shot check never starts a browser login`
  - [x] **Design constraints:** R34. `check` builds the credential source with `interactive = false`; a missing gh session is an account failure, exit 1; no login call.
        Files: `crates/passuff-cli/src/main.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A one-shot check never starts a browser login" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `gh-session`

- [x] Scenario: `A non-interactive watch never starts a browser login`
  - [x] **Design constraints:** R34. Watch with `stdin`/`stdout` not a TTY builds `interactive = false`; one observation cycle reports the account failure and no login call.
        Files: `crates/passuff-cli/src/main.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A non-interactive watch never starts a browser login" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `gh-session`

- [x] Scenario: `A failed browser login is attempted once per account`
  - [x] **Design constraints:** R38. Attempt set keyed by login in the credential source; two accounts, two cycles, one login attempt per account; both account failures are reported.
        Files: `crates/passuff-core/src/credential.rs`, `crates/passuff-cli/src/lib.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A failed browser login is attempted once per account" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `gh-session`

- [x] Scenario: `A failed browser login does not stop the other account`
  - [x] **Design constraints:** R38. The failing account sorts first (`alpha` before `personal`); its login failure is reported and the healthy account still notifies in the same cycle.
        Files: `crates/passuff-core/src/credential.rs`, `crates/passuff-core/src/cycle.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A failed browser login does not stop the other account" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `gh-session`

### Batch: `identity-and-watch`

- [x] Scenario: `A personal access token for a different login is an account failure`
  - [x] **Design constraints:** R36. Baseline both accounts; rotate the work credential to `octo-other` and the authored owner of `acme/work#7`; later comments on both pull requests; assert `credential belongs to @octo-other, expected @octo-work`, personal notification from the `octo-other` comment (login-set exclusion), no notification for `acme/work#7` (suppression), exit 1. Comparison is case-insensitive.
        Files: `crates/passuff-core/src/cycle.rs`, `crates/passuff-core/src/domain.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`, `crates/passuff-cli/tests/support/mod.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A personal access token for a different login is an account failure" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `identity-and-watch`

- [x] Scenario: `A personal access token whose login differs only in case is accepted`
  - [x] **Design constraints:** R36. ASCII case-insensitive comparison; observation login `octo-WORK`, expected `octo-work`, authored filter matches; notification after a baseline.
        Files: `crates/passuff-core/src/cycle.rs`, `crates/passuff-core/src/domain.rs`, `crates/passuff-cli/tests/steps/pr_activity_notification.rs`
  - [x] Implement the scenario and its tests. Run the relevant scenarios. Must pass.
        ```
./scripts/verify.sh (delta) -> 23/23; permanent -> 52/52; scenario "A personal access token whose login differs only in case is accepted" green. COMMIT 7fa8bed.
        ```
  - [x] REFACTOR: clean up, no behaviour change. Re-run, must still pass.
        ```
cargo fmt --all (no behavior change); delta 23/23, permanent 52/52. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `identity-and-watch`

---

## E2E Smoke Tests

- [x] Keep the existing four `@e2e` scenarios green; no new `@e2e` scenario is added because the consumer actions are unchanged and the login variant is a regular scenario. Record the e2e count (4) strictly less than the full count, and capture a fresh run of `./scripts/verify.sh --e2e`.
  - [x] Run `./scripts/verify.sh --e2e`. Must pass.
        ```
./scripts/verify.sh --e2e -> total 4, passed 4, failed 0, skipped 0. COMMIT 7fa8bed.
        ```
  - [x] COMMIT: `7fa8bed` -- Batch: `e2e`

---

**Done when:** every box checked with a batch commit, `./scripts/verify.sh` and `./scripts/verify.sh --e2e` exit 0 on the full suite, and the e2e setup proof records an e2e count strictly less than the full count.
