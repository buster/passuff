# Arc42 Impact Assessment: passuff-involvement-focus

Watched pull requests record their involvement relations, and a configured focus
narrows the inbox and notifications without changing observation.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | No | No new goal or stakeholder; the existing review goal gains a focus quality scenario in chapter 10. |
| 2 | 02-architecture-constraints | No | No new platform, dependency, or legal constraint; the existing GraphQL client is reused. |
| 3 | 03-context-and-scope | No | GitHub remains the only external system; three documented search qualifiers replace two, which is not a new interface. |
| 4 | 04-solution-strategy | No | The classify-then-record strategy is unchanged; the focus is applied after recording. |
| 5 | 05-building-block-view | Yes | The Inbox state description now includes each watched pull request's involvement relations. |
| 6 | 06-runtime-view | Yes | New focus flow: the notification gate and the filtered read model. |
| 7 | 07-deployment-view | No | Same binaries, packaging, and deployment targets. |
| 8 | 08-crosscutting-concepts | Yes | New involvement-and-focus concept: discovery tags, refresh rules, the gate, and compatibility. |
| 9 | 09-architecture-decisions | No | No ADR. The focus is product behavior owned by the capability specs and `design.md`; no component, authority, storage, or compatibility boundary moves. |
| 10 | 10-quality-requirements | Yes | QS-033 records the focus scenario. |
| 11 | 11-risks-and-technical-debt | Yes | R-036 records the availability of the team-review search qualifier. |
| 12 | 12-glossary | Yes | `docs/glossary.md` defines involvement relation and focus. |

## Structural assessment

No new aggregate or component. `WatchedPullRequest` gains one list attribute
with a serde default; `PullRequestView` mirrors it; `AccountObservation` carries
the relation with each discovered node instead of a bare node id. The cycle and
the read model gain one gate each. `ScopeEditor` gains `set_focus` and loses
`add`, both within its existing watch-scope responsibility.

## ADR qualification

- Architectural commitment: no component, ownership, dependency-direction, or
  storage boundary moves; the state stays version 2 with an optional field.
- Real alternatives: filtering only the view, or skipping discovery of excluded
  relations, were considered; the operator settled post-fetch filtering, and the
  choice is reversible within this change.
- Durable consequence: reversal is a config and filter change; old state loads
  with empty relations and stays visible.
- Existing decisions: ADR-0003 (core/UI boundary) and the version-2 state
  contract are unaffected.

Route: no new ADR; `design.md`, the capability specs, and chapters 05/06/08 own
the decision.

## Status

STATUS: DONE
