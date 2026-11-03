# Arc42 Impact Assessment: passuff-stacked-inbox

Adds GitHub-reported stack membership to the observation and persisted watch
state, and a read-only stacked projection to the terminal inbox. No notification,
configuration, or delivery contract changes.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | The requirement to review stacked pull requests as one unit is added. |
| 2 | 02-architecture-constraints | Yes | Stack metadata is preview-gated and read-only; a rejected stack query retries once, adding at most one retry of the account observation per account per cycle. |
| 3 | 03-context-and-scope | Yes | GitHub stack metadata becomes a read-only, preview-gated input; stack mutation stays outside. |
| 4 | 04-solution-strategy | Yes | Read-only stack enrichment and a terminal projection over the existing read model. |
| 5 | 05-building-block-view | Yes | `PullRequestStack` value object; the GitHub adapter and `TuiApp` gain stack responsibilities. |
| 6 | 06-runtime-view | Yes | New stacked-inbox runtime flow: observation, state, grouping, activity labels, degradation. |
| 7 | 07-deployment-view | No | No process, service, or packaging change. |
| 8 | 08-crosscutting-concepts | Yes | Stack metadata persistence and rejection degradation. |
| 9 | 09-architecture-decisions | No | No new ADR; read-only observation stays within ADR-0001 and additive state within ADR-0005. |
| 10 | 10-quality-requirements | Yes | QS-021 covers the stacked snapshot and standalone degradation. |
| 11 | 11-risks-and-technical-debt | Yes | R-021 records preview availability and the retry path. |
| 12 | 12-glossary | Yes | Pull request stack, stack position, and stack base terms with `PullRequestStack` symbols. |

## Durable updates made

- Chapter 01 requirement list.
- Chapter 02 preview-gated stack metadata constraint.
- Chapter 03 technical context and scope boundaries.
- Chapter 04 solution strategy.
- Chapter 05 building blocks.
- Chapter 06 stacked-inbox scenario.
- Chapter 08 stack metadata cross-cutting rule.
- Chapter 10 QS-021.
- Chapter 11 R-021.
- Glossary code symbols for the three stack terms.

## Status

STATUS: DONE
