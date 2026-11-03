# 9. Architecture Decisions

This chapter is the register/index for architecture decisions. Each decision
has exactly one ADR body under `docs/arc42/adr/`. The register records the ID,
title, status, date, and link; it does not contain ADR bodies.

## Register

| ADR | Title | Status | Date | File |
|---|---|---|---|---|
| ADR-0001 | Poll authored pull requests through the GitHub GraphQL API | accepted | 2026-11-02 | [ADR-0001](adr/ADR-0001-poll-authored-pull-requests.md) |
| ADR-0002 | Deliver through local desktop notifications and a bundled sound | accepted | 2026-11-02 | [ADR-0002](adr/ADR-0002-local-desktop-notifications.md) |
| ADR-0003 | Make the core a UI-agnostic library and the CLI its first adapter | accepted | 2026-11-02 | [ADR-0003](adr/ADR-0003-library-first-core.md) |
| ADR-0004 | Delegate GitHub authentication to the gh CLI instead of owning an OAuth application | accepted | 2026-11-02 | [ADR-0004](adr/ADR-0004-gh-delegated-authentication.md) |
| ADR-0005 | Migrate the persisted watch state to version 2 with an additive rewrite | accepted | 2026-11-02 | [ADR-0005](adr/ADR-0005-state-version-2-migration.md) |
| ADR-0006 | Represent the watch scope with an explicit all/selected mode | accepted | 2026-11-03 | [ADR-0006](adr/ADR-0006-watch-scope-mode.md) |
| ADR-0007 | Probe before observing and pace the inbox against the account allowance | accepted | 2026-11-03 | [ADR-0007](adr/ADR-0007-probe-before-observing.md) |

## Contract

- The ADR directory index is [`adr/README.md`](adr/README.md).
- The qualification for each candidate was recorded in the change's arc42
  assessment and is archived with the change; the durable rationale lives in
  each ADR.
- A refinement amends the existing ADR; a replacement supersedes it and links
  to the replacement. Historical ADRs move to `adr/archive/` with their ID and
  filename preserved.
- The permitted statuses are `proposed`, `rejected`, `accepted`, `deprecated`,
  and `superseded by ADR-NNNN`.
