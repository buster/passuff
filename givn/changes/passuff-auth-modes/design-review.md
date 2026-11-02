Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

# Design review: passuff-auth-modes (final re-check)

Verdict: **PASS**. The last blocker from the prior review is resolved and no
regression or new blocker was found. Reviewed as a plan against `proposal.md`,
the delta feature, `usecase.md` R1-R38, `design.md`, `arc42.md`,
`docs/arc42/`, ADR-0004, and the existing test harness as context only.

## Prior blocker

| ID | Prior blocker | Status | Evidence |
|---|---|---|---|
| B6 | The legacy environment scenario had no prior baseline and no new activity, so its notification Then could not go GREEN under R14/R26. | Resolved | `feature:141-147`: environment credential (`:142`), authored `acme/api#42` (`:143`), `the watcher has already observed "acme/api#42"` (`:144`), `alice` comment (`:145`), check (`:146`), notification (`:147`). The baseline step sets `pending_baseline` (`steps/pr_activity_notification.rs:38-44`) and the comment step resolves that baseline before applying the comment (`:164`), so the comment is new activity, not baseline content. |

The legacy command scenario retains the same shape (`feature:150-156`: baseline
`:153`, comment `:154`, check `:155`, notification `:156`) and remains
implementable. Prior resolutions still hold: B1a/B1b (`feature:116-129`,
`design:217-238`), B2 (accepted rows now baseline+new activity; invalid rows
remain RED), B3 (`design:242-258`), B4 (`usecase.md:79,84`), B5
(`usecase.md:21,38`).

## Regression check

All eight notification-expecting check scenarios in the delta declare a prior
baseline: `feature:10, 65, 75, 110, 121, 135, 144, 153`. Every other
`passuff runs one check` scenario expects no notification or an error:
`:29-32` (browser login refused), `:88-90` and `:161-163` (configuration
errors), `:101-103` (missing token file). No acceptance scenario's first check
asserts a notification without a baseline. The permanent feature is unmodified
(`git diff` empty) and keeps its four `@e2e` scenarios
(`givn/specs/watch-pull-requests/pr-activity-notification.feature:4,16,28,34`).

Runtime evidence: `GIVN_FEATURES=givn/changes/passuff-auth-modes/specs
GIVN_RESULT_SCOPE=regular` reports 23 scenarios, 0 passed, 23 failed, all on
undefined steps. The legacy environment scenario fails on its first undefined
Given (`a configured account "work" with an environment credential`), not on
its notification Then, so the Then is reachable once the step exists. `givn
lint --change passuff-auth-modes` exits 2 with the 15 expected `@wip` findings
plus advisory shape/subset notices; the feature parses.

## Required branches

| Branch | Finding |
|---|---|
| Scope | Signed off. Explicit `auth` modes, PAT via env/command/permission-checked file, `~` expansion, literal-token rejection, identity mismatch, legacy env and command, gh session login, and the closed operator decisions (no literal token; gh delegation; no browser in check/non-interactive) are all specified and traced. The GHES `--hostname` derivation (`design:199-202`) still has no scenario, and R38's "abandoned" case is only covered as "always fails"; both are carried as non-blocking. |
| Tech choices | Signed off. `tokio::process`, `IsTerminal`, `0o077 == 0` plus owner read, `StopAfter`, and no new dependency fit the constraints; `tokio` `process` is already enabled in `passuff-core`. Non-blocking gaps unchanged: `design:57` falsely says the core never starts a process; `resolve(spec)` (`design:168`) has no account key for the per-account attempt rule; `GhLogin` (`design:64`) has no method signature. |
| Missing scenarios | Signed off with carried findings: GHES `--hostname`, R38 timeout and success-without-token, token-file empty/relative/HOME-unset, gh token missing-binary/non-zero/empty/timeout, and a direct assertion that the gh session supplied the token. R32-R38 each have at least one discriminating scenario; the two legacy scenarios now discriminate R37. |
| Testability | Signed off. Strict mode is `.fail_on_skipped()` (`features_runner.rs:49`); the run above proves every new scenario is RED on undefined steps; the stub pattern is `unimplemented!()` (`design:290-291`). The single-scenario command at `design:292-294` is still unsupported: the runner reads `GIVN_FEATURES`, `GIVN_RESULT_SCOPE`, and `GIVN_RESULT_FILE` (`features_runner.rs:24-28`) but not `GIVN_SCENARIO`; implement the filter or drop the claim before tasks. The invalid-config TOML harness also still needs describing (`support/mod.rs:277-281` returns a generic error). |
| E2E fidelity and interaction inventory | Signed off. Four permanent `@e2e` scenarios remain, no tag removed, no new `@e2e` added; the auth change adds no consumer action and the unchanged driver still covers the four inventory rows (`usecase.md:116-121`; `design:357-362`). The browser login is an external gh surface, not a passuff screen. |
| Risk | Signed off with carried findings: the 300-second `gh auth login` inside the sequential account loop is a head-of-line delay not recorded in `design:405-421`; the fake-gh/watch-harness risk and HOME-unset token-file behaviour are also unrecorded. Token-file TOCTOU, gh login side effects, and version gaps are recorded. |
| Contracts and components | Signed off. One `CredentialSource` contract with four named adapters; `GhLogin` and `WatchControl` each have one implementing component. Carried: false "core never starts a process"; per-account attempt keying; `GhLogin` signature; `expected_login` can diverge from `GhOAuth.login`; the CLI-contract gh-plus-PAT message (`design:274-275`) differs from the validation matrix (`design:96-97`). |
| Decisions ledger | Signed off. D1 resolves at `design:148-150`; D2 at `:151-156`; the closed operator decisions match the design and the proposal records no seed. |
| Version freshness | Signed off. Installed gh is 2.103.0; `gh auth token --user` is available from 2.40+; cucumber 0.23.0 is already in `Cargo.lock`; `tokio` `process` is already enabled in `passuff-core`; no new crate. |

## Other branches rechecked

- Persona, deferrals, decision timing, interface classification, domain model,
  and the visual contract: no change from the prior review; no issue.
- Architecture documentation: prior findings persist (chapter 05 `:35,58`,
  chapter 08 `:43`, glossary `AuthMode` at `:14`, ADR-0004 "one attempt per
  process" at `:60`). Documentation fixes, not design blockers.
- Use-case wording: extension 1c (`usecase.md:39`) and main-flow step 1
  (`usecase.md:26`) still describe the mismatched account as unobserved, while
  `design:206-221` observes it for identity only. Fix before tasks.

## Non-blocking findings carried to tasks

These were open in the prior review and remain open; none blocks the design
gate, but all must be resolved while tasks are written or implemented:

- Implement the `GIVN_SCENARIO` filter or remove the single-scenario claim.
- Describe the invalid-configuration TOML harness; the current `World` path
  cannot produce the six specific messages.
- Specify that the fake gh login-success path writes the per-login token file,
  otherwise `passuff: watching` cannot be asserted after the retry.
- Key the attempt rule by account, or restate R34/R38 per login with the
  equivalence recorded.
- Give `GhLogin` a method signature and enforce `expected_login == GhOAuth.login`.
- Align the CLI-contract gh-plus-PAT message with the validation matrix and the
  feature's expected text.
- Fix use-case extension 1c and main-flow step 1 wording.
- Update arc42 chapters 02, 05, 07, 08; ADR-0004's "one attempt per process";
  and the glossary's `AuthMode`.
- Correct the false "core never starts a process" sentence (`design:57`).
- Strengthen the missing-token-file Then (`feature:102`) to name the file and
  path.
- Add a no-new-baseline or retained-baseline assertion to the identity scenario.
- Record the login head-of-line delay and the fake-gh/watch-harness risk.

## Sign-off

The prior blocker is resolved, both legacy scenarios now have a prior baseline
and new activity, and no regression was introduced. No blocker remains. The
branches above are signed off with the listed non-blocking findings carried to
tasks.

DESIGN-REVIEW: PASS
