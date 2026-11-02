# Design Review: passuff-tui

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context)

Re-review after FAIL, scoped to B3 and to regressions in the files touched by the
fix. B1, B2, B4-B9 were closed in the previous round and are not re-litigated.

## Verdict

DESIGN-REVIEW: PASS

## Blocker resolution

B3 closed: `specs/review-activity-inbox/activity-inbox.feature:21` now asserts
`And the TUI does not show "token-TOKEN_PERSONAL"`, the live value of the
personal account's token in that scenario, so the assertion can fail.

## Evidence

- Fixture: the installed account token is `token-{TOKEN_<LABEL>}`
  (`crates/passuff-cli/tests/support/mod.rs:236-266`, value at `:245`). T1
  configures account `personal`, so the rendered surface carries
  `token-TOKEN_PERSONAL`; `ghp_` appears only in the invalid-config fixture
  (`:440`), which never reaches a screen.
- The other three negatives (`Open`, `Success`, comment body) are unchanged and
  still meaningful: each names a string an implementation could plausibly
  render (enum variant, status label, comment text).
- No regression in the touched artifacts. `design.md:96` still forbids token,
  body, command text, and enum-name output; `design.md:162` now lists chapters
  01, 03, 04, 05, 06, 07, 08, 10, 11, 12, matching the modified chapters;
  `proposal.md:25` includes Escape; interaction inventory
  (`givn/specs/review-activity-inbox/usecase.md:80-84`) and the design matrix
  (`design.md:119-123`) list the same three rows; `givn lint --change
  passuff-tui` reports only `@wip` and advisory findings.

## Residual advisories (non-blocking)

- Selection is asserted through the rendered marker; `design.md` shows `>` in the
  layout but does not name it as the assertion seam.
- Count observations use two step families (`the TUI shows` in T1, `the TUI
  reports` in the read scenarios); both are implementable but one verb is
  simpler.
- Register `steps/activity_inbox_tui.rs` in `tests/features_runner.rs` and prove
  one scenario GREEN before writing the rest.

DESIGN-REVIEW: PASS
