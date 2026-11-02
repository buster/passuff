Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context), re-review after the second FAIL

# Design Review: passuff-activity-inbox (third review)

Scope: `proposal.md`, `design.md`, `arc42.md`, the three delta features, the
three use cases, the ADR and affected arc42 chapters, `docs/glossary.md`, and
`docs/domain-model.md`. Closed operator decisions (split, 500 cap, per-event
read, global scope, transcript layout) are not re-opened.

## Blocker resolution

| # | Prior blocker | Status | Evidence |
|---|---|---|---|
| 1 | Watch use-case out-of-scope contradicted R39 | **Resolved** | `givn/specs/watch-pull-requests/usecase.md:132-143` now excludes only per-account scopes and separate history/status use cases; R39 is at `:86`. |
| 2 | Per-event read untested | **Resolved** | `activity-inbox.feature:58-71` records two events, marks the newest read, asserts `1 unread of 2`, CI read and comment unread. QS-017 matches at `docs/arc42/10-quality-requirements.md:49`. |
| 3 | Text rules unasserted | **Resolved** | `activity-inbox.feature:16-19` asserts activity id/time positively and `Open`, `Success`, comment body negatively; `watch-status.feature:42` asserts the PR title. |
| 4 | Write failure and no-state write untested | **Resolved** | `activity-inbox.feature:128-135` asserts `state error` and exit 3; `design.md:224-227` documents `SeededFailingStore`; `activity-inbox.feature:119-125` asserts no state file was created. |
| 5 | Version-2 id normalization off by one | **Resolved** | `design.md:106-108` recomputes from `max(parsed counter, maximum record number)` and appends `+1`; v1 migration at `:102-105`. |
| 6 | Glossary/domain drift | **Resolved** | `docs/glossary.md:7-8` names all three governing use cases; `:39-42` defines activity history, the account/scope Inbox, and the `tracked` anti-term. `docs/domain-model.md:114` and `:127-129` add the Activity Record aggregate and relationships. |
| 7 | Chapter 01 and arc42 row 1 stale | **Resolved** | `docs/arc42/01-introduction-and-goals.md:22-32` adds status/history requirements and all three use cases; `arc42.md:9` marks row 1 Yes. |

## New-defect check

- **Use-case placement/rejection:** capabilities are placed in the correct use
  cases; no new placement failure found.
- **Scenario RED-ness:** every delta scenario asserts a concrete transcript or
  state value; no vacuous scenario found.
- **Testability wiring:** strict mode, the `unimplemented!()` stub, the
  single-scenario `GIVN_SCENARIO` command, the temp `JsonStateStore` harness,
  and real-binary E2E wiring are named and match the repository.
- **Contracts:** `InboxService` remains the read contract, `Inbox` the single
  implementation, and `InboxCli` the CLI adapter.
- **State-v2 boundaries:** migration, missing-file no-write, unknown-version
  quarantine, observation time, and cap behavior are bounded; the id rule no
  longer collides.
- **E2E consumer actions:** exactly three new `@e2e` scenarios, one each for
  status, inbox, and read, matching the interaction inventory.

Remaining blockers: none.

DESIGN-REVIEW: PASS
