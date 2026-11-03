# Proposal: passuff-stacked-inbox

## Release note

Show stacked pull requests together in the inbox with their stack position, size, base, and grouped activity.

## Use-Case Context

- Use-case ID: `review-pull-request-stacks` (new; extends `review-activity-inbox`)
- Ideation topic: `inbox-scope-and-stacks`
- Seed path: `givn/ideation/inbox-scope-and-stacks/changes/passuff-stacked-inbox.md`
- Confirmed Personas: `multi-account-developer`

## Review Context

- Review kind: independent
- Reason: the change adds persisted stack membership to watcher state and a user-visible terminal inbox grouping, so the design and review must not share the author's context.

## Problem / Opportunity

The developer uses stacked pull requests heavily. passuff records every watched
pull request as an unrelated row, so a stack arrives as several independent
entries and the developer reconstructs the order, the shared base, and which
activity belongs together by hand. GitHub now reports stack membership for pull
requests in repositories where the stacked-PRs preview is enabled. The inbox
can use that fact to present the stack as the unit of work the developer
actually reviews.

## Proposed Solution

- When GitHub reports stack membership for a watched pull request, passuff
  records the stack number, size, position, and ultimate base with the
  observation.
- The inbox groups watched pull requests by repository and stack number and
  orders each group by stack position from the bottom.
- The group shows the stack base and size once and shows each watched member
  with its position.
- A stack member that no configured account authored is not shown; the group
  keeps the full size, so positions may have gaps.
- Activity records group under the stack of their pull request, and the group
  shows one aggregated unread count.
- A pull request without stack metadata, or whose stack metadata cannot be
  read, appears as a standalone row; the account observation still succeeds.
- Stack grouping is read-only. passuff never creates, extends, or dissolves a
  stack.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| stacked-inbox | no signal (tie) | `NEW in review-pull-request-stacks` | The handoff maps the new review goal to its own permanent use case; the existing `activity-inbox` keeps flat review and read behaviour. |

## Out of Scope

- Ref-chain inference of stack membership; only GitHub-reported metadata is used.
- Showing stack members that no configured account authored.
- Creating, extending, or dissolving stacks.
- Changing notification delivery for stack members.
- Editing the watch scope; a separate change owns it.
- A webhook or streaming transport; polling stays.

## Open Questions

### D1 — How should observation degrade when the preview-gated stack field is unavailable?

- Question: when a repository does not have the stacked-PRs preview, or GitHub
  rejects the stack field, how does observation continue without losing the
  account's other pull request data?
- Options:
  1. **Probe and retry** — request stack metadata, and on a rejection retry the
     account observation without it. Gain: stack data where available. Cost: an
     extra request on the first rejection and a remembered capability.
  2. **Configuration flag** — query stack metadata only when the operator
     enables it. Gain: no rejected request. Cost: another setting the developer
     must know.
  3. **Standalone on error** — treat the rejection as "no stacks" for that
     cycle. Gain: simple. Cost: loses the account's observations for that cycle.
- Disposition: carried — `design.md` decides the mechanism; owner: design.

### D2 — Which state schema version carries stack membership?

- Question: are stack number, size, position, and base added to the existing
  version-2 state or carried by a new state version?
- Options:
  1. **Version 2 with optional fields** — add fields with defaults. Gain: no
     migration. Cost: old readers see unknown fields and a version that does
     not explain the shape.
  2. **Version 3** — a new version with migration from version 2. Gain: explicit
     shape. Cost: migration code and fixtures.
- Disposition: carried — `design.md` decides the state version and migration;
  owner: design.

### D3 — Does the delta extend the existing inbox capability or add a new one?

- Question: should stack grouping be a capability under the existing
  `review-activity-inbox` goal, or a new capability under the new
  `review-pull-request-stacks` goal?
- Options:
  1. **New capability under the new use case** — `stacked-inbox` owns grouping.
     Gain: the handoff mapping holds; the new goal owns its rules.
     Cost: a second inbox capability to keep aligned.
  2. **Extend `activity-inbox`** — grouping becomes a variant of flat review.
     Gain: fewer capabilities. Cost: contradicts the confirmed handoff mapping.
- Disposition: answered — the handoff and the capability routing table choose
  `NEW in review-pull-request-stacks`; the existing `activity-inbox` keeps flat
  review and read behaviour.
