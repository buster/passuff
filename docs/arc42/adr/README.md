# Architecture Decision Records

One architecturally significant decision gets exactly one ADR. The chapter-09
register is [`../09-architecture-decisions.md`](../09-architecture-decisions.md).

| ADR | Title | Status | Date | File |
|---|---|---|---|---|
| ADR-0001 | Poll authored pull requests through the GitHub GraphQL API | accepted | 2026-11-02 | [ADR-0001](ADR-0001-poll-authored-pull-requests.md) |
| ADR-0002 | Deliver through local desktop notifications and a bundled sound | accepted | 2026-11-02 | [ADR-0002](ADR-0002-local-desktop-notifications.md) |
| ADR-0003 | Make the core a UI-agnostic library and the CLI its first adapter | accepted | 2026-11-02 | [ADR-0003](ADR-0003-library-first-core.md) |
| ADR-0004 | Delegate GitHub authentication to the gh CLI instead of owning an OAuth application | accepted | 2026-11-02 | [ADR-0004](ADR-0004-gh-delegated-authentication.md) |
| ADR-0005 | Migrate the persisted watch state to version 2 with an additive rewrite | accepted | 2026-11-02 | [ADR-0005](ADR-0005-state-version-2-migration.md) |
| ADR-0006 | Represent the watch scope with an explicit all/selected mode | accepted | 2026-11-03 | [ADR-0006](ADR-0006-watch-scope-mode.md) |
| ADR-0007 | Probe before observing, target changed pull requests, and reconcile the open set | accepted | 2026-11-03 | [ADR-0007](ADR-0007-probe-before-observing.md) |

Historical ADRs move to `docs/arc42/adr/archive/` with their ID and filename
preserved; both indexes keep the link. The template is
[`adr-template.md`](adr-template.md).
