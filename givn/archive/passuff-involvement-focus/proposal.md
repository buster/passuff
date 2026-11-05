# Proposal: passuff-involvement-focus

## Release note

The inbox can focus on the involvement relations you choose, and notifications follow that focus without changing what passuff fetches.

## Use-Case Context

- Use-case IDs: `change-watch-scope` (`watch-scope`), `review-activity-inbox`
  (`activity-inbox`, `inbox-navigation`), `watch-pull-requests`
  (`pr-activity-notification`).
- Ideation topic: none — the report answers a direct request about inbox noise.
- Seed path: none
- Confirmed Personas: `multi-account-developer` — confirmed 2026-11-02 by the
  operator's brief and the instruction to proceed.

## Review Context

- Review kind: self-review
- Reason: single-operator project with the established review kind for its
  inbox changes. Not a major change: the config addition is optional and
  backward-compatible, and the removed manual-entry flag could not change what
  passuff observes — a repository entered by hand was never fetched unless the
  developer authored or was involved in a pull request there. The touched
  capabilities share one operator-facing surface and no stored format migration,
  security boundary, or concurrency behavior changes.

## Problem / Opportunity

The inbox now watches three kinds of pull request: the developer's authored
pull requests, pull requests assigned to them, and pull requests whose review
was requested from them. GitHub's `review-requested` search also matches
requests made to the developer's teams, so a busy team's review queue floods the
inbox and the notifications with work the developer has not personally taken on.
The developer wants to keep fetching everything so the history is complete, but
narrow what is shown and announced to the relations they are working on, and
change that narrowing without waiting for a new fetch.

The watch-scope screen also offers a manual `owner/name` entry that cannot
produce a watched pull request: observation fetches only authored and involved
pull requests, so a repository entered by hand never contributes one. The
screen advertises typing while typing does nothing until a separate insert mode
is entered.

## Proposed Solution

- Every watched pull request records why it is watched: authored by the
  developer, assigned to them, a review requested from them directly, or a
  review requested from a team they belong to. A pull request can have more than
  one reason.
- The watch scope gains a focus: a set of those relations, all enabled by
  default. The terminal watch-scope screen shows the four relations as toggles
  beside the repository list and persists the choice in the configuration.
- Fetching is unchanged: every relation stays observed and recorded. Changing
  the focus never triggers a fetch and never loses history; widening it shows
  the retained activity immediately.
- The watched table, the activity list, and the unread count show only pull
  requests whose reasons intersect the focus. A pull request outside the focus
  produces no notification, because a notification for a hidden pull request is
  confusing; its activity is still recorded and appears when the focus widens.
- A configuration written before this change has unknown reasons for its
  watched pull requests, so those stay visible until the next observation
  records their reasons.
- The manual `owner/name` entry is removed from the watch-scope screen and the
  scripted flags, together with its insert mode and its documentation. The
  repository list and the scope mode remain.

## Capability Routing

Route reported no signal (the top candidates tie at 12.00).

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| watch-scope | no signal | `EXTEND watch-scope` | The focus is part of the persisted watch scope and is edited on the same screen. |
| activity-inbox | no signal | `EXTEND activity-inbox` | The read model and the `passuff inbox` line show only focused pull requests. |
| inbox-navigation | no signal | `EXTEND inbox-navigation` | The watched table and the unread count follow the focus. |
| pr-activity-notification | no signal | `EXTEND pr-activity-notification` | Notifications are produced only for focused pull requests. |

## Out of Scope

- Changing which pull requests are observed: authored, assigned, and
  review-requested (direct or team) stay.
- Filtering by repository, label, author, draft state, or CI state; the
  repository scope and the activity filters remain separate.
- Per-account focus; the focus is one global set, like the repository scope.
- Reclassifying existing activity: a record's pull request is looked up when the
  inbox is built.
- Notifying about a pull request outside the focus, or recalling a notification
  already delivered.
- Any change to the notification content or grouping.

## Open Questions

None. The operator settled the behavior: fetch as today, filter after the fetch,
and gate notifications on the same filter.
