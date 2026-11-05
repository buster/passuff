# Proposal: passuff-activity-time

## Release note

The inbox now shows when each activity happened instead of when passuff fetched it.

## Use-Case Context

- Use-case ID: `review-activity-inbox`
- Ideation topic: none — this change answers a direct report about the inbox
  time, not an ideation seed.
- Seed path: none
- Confirmed Personas: `multi-account-developer` — confirmed 2026-11-02 by the
  operator's brief and the instruction to proceed.

## Review Context

- Review kind: self-review
- Reason: single-operator project; no independent reviewer context exists. Not a
  major change: the stored record gains one optional field with a
  backward-compatible default, and no interface or permission is removed.

## Problem / Opportunity

The inbox reports when passuff fetched an activity, not when the activity
happened. A comment written at 14:00 and fetched by the 14:05 check is shown as
14:05, and every event discovered in the same check shares one timestamp, so the
shown order is the discovery order rather than the order of events. The developer
cannot tell how old an alert is or line it up with the conversation on GitHub.

GitHub already reports the real times. passuff fetches them, stores them on the
comment, review, and review comment, and then discards them when it records the
activity. State changes are the exception: the merge and close times are not
fetched at all today.

## Proposed Solution

- Each recorded activity keeps two times: when passuff fetched it, and, when
  GitHub reports one, when it happened. A comment uses its creation time, a
  review comment its creation time, a review its submission time, an opened pull
  request its creation time, and a merge or close the reported merge or close
  time.
- The activity table's `When` column and the `passuff inbox` line show when the
  activity happened.
- A CI failure or a CI recovery has no reported time; it shows the fetch time
  and is marked approximate with a leading `~`, which fits the existing column.
- The activity list is ordered newest first by the time it shows.
- The activity detail popup shows both times when they differ, so the fetch
  delay stays visible.
- Activity recorded before this change, and state files written by an earlier
  version, keep working: a record without a reported time shows its fetch time
  as approximate.

## Capability Routing

Route reported no signal (the top candidates tie at 12.00).

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| activity-inbox | no signal | `EXTEND activity-inbox` | The record's occurrence time and the `passuff inbox` line belong to the existing activity-review goal. |
| inbox-navigation | no signal | `EXTEND inbox-navigation` | The `When` column and the detail popup are part of the existing terminal table and popup, not a new goal. |

## Out of Scope

- Notification grouping, delivery, and their content; notifications carry no
  time today and this change does not add one.
- Backfilling occurrence times for activity recorded before the change.
- Relative times ("5 minutes ago"), a configurable time zone, or a clock
  display preference.
- Ordering the activity by the recorded id or by pull request.
- The watched pull request table; its times are unchanged.

## Open Questions

None. The report settles the behaviour: show when the activity happened, and
keep the fetch time visible where it is not the same.
