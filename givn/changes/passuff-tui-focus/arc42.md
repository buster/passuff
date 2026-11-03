# Arc42 Impact Assessment: passuff-tui-focus

Adds pane focus, watched-PR selection/scroll, and automatic read-model refresh
to the terminal inbox.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Inbox navigation and live refresh requirement. |
| 2 | 02-architecture-constraints | No | No new platform or dependency. |
| 3 | 03-context-and-scope | No | Same local read model and terminal surface. |
| 4 | 04-solution-strategy | Yes | Focus-aware TUI with poll-based refresh. |
| 5 | 05-building-block-view | Yes | `TuiApp` focus/selection and the `--keys` surface. |
| 6 | 06-runtime-view | Yes | Focus keys, scrolling, refresh tick. |
| 7 | 07-deployment-view | No | No packaging change. |
| 8 | 08-crosscutting-concepts | Yes | Automatic read-model refresh and selection preservation. |
| 9 | 09-architecture-decisions | No | No new ADR; a local UI behavior. |
| 10 | 10-quality-requirements | Yes | QS-024 navigation/refresh. |
| 11 | 11-risks-and-technical-debt | Yes | R-027 table scroll/timer. |
| 12 | 12-glossary | No | No new domain term; focus is a UI state. |

## Status

STATUS: DONE
