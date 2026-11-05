# Arc42 Impact Assessment: passuff-activity-time

Activity records keep when an event happened next to when it was fetched, and
the inbox surfaces show the occurrence time.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | No | No new goal or stakeholder; the existing review goal gains accuracy, recorded as a quality scenario in chapter 10. |
| 2 | 02-architecture-constraints | No | No new platform, dependency, or legal constraint; `chrono` is already a dependency. |
| 3 | 03-context-and-scope | No | GitHub remains the only external system; the existing node query gains two fields, which is not a new interface. |
| 4 | 04-solution-strategy | No | The classification-then-record strategy is unchanged; the timestamp is captured where the diff is already classified. |
| 5 | 05-building-block-view | Yes | The Inbox building block's state description now names the fetch time and the occurrence time. |
| 6 | 06-runtime-view | Yes | The record-activity scenario now appends both times and the read model orders by the effective time. |
| 7 | 07-deployment-view | No | Same binaries, packaging, and deployment targets. |
| 8 | 08-crosscutting-concepts | Yes | New "Activity time" concept: the two times, the `~` marker, the ordering rule, and state compatibility. |
| 9 | 09-architecture-decisions | No | No new ADR. The record shape and the marker are feature behavior owned by the capability spec and `design.md`; no component, authority, storage, or compatibility boundary moves. |
| 10 | 10-quality-requirements | Yes | QS-032 records the occurrence-time accuracy scenario. |
| 11 | 11-risks-and-technical-debt | Yes | R-035 records the mixing of reported and fetch times in one ordered list. |
| 12 | 12-glossary | Yes | `docs/glossary.md` defines occurrence time and fetch time and updates the activity record and detail terms. |

## Structural assessment

No new aggregate, component, or contract. `ActivityRecord` gains one optional
attribute with a serde default; `AuthoredPullRequest` gains the merge and close
times that the state-change event already needed. `ActivityEvent` and
`Notification` are unchanged, so grouping and delivery keep their existing
shape. The read-model helpers `effective_time` and `is_approximate` live with
the existing record accessors.

## ADR qualification

- Architectural commitment: no component, ownership, dependency-direction, or
  storage boundary moves; the record keeps the same owner and version.
- Real alternatives: showing the fetch time only versus recording the source
  time were considered, but the report settles the behavior and the choice is
  reversible within this change.
- Durable consequence: reversal would be a display change; the added field is
  optional and ignored by older readers through the serde default.
- Existing decisions: ADR-0003 (core/UI boundary) and the state compatibility
  contract are unaffected.

Route: no new ADR; `design.md`, the capability spec, and chapters 05/06/08 own
the decision.

## Status

STATUS: DONE
