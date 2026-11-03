# Arc42 Impact Assessment: passuff-activity-detail

The terminal inbox gains an activity detail overlay and a browser launch for the
selected activity's pull request.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Reviewing an activity and reaching its pull request. |
| 2 | 02-architecture-constraints | No | No new platform or dependency constraint; the platform opener is a child process. |
| 3 | 03-context-and-scope | Yes | The terminal inbox launches the system browser; a new outbound interaction. |
| 4 | 04-solution-strategy | Yes | Detail overlay plus an opener adapter; side effect kept out of `TuiApp`. |
| 5 | 05-building-block-view | Yes | `SystemBrowserOpener`, `ActivityDetail`, `TuiAction::OpenUrl`. |
| 6 | 06-runtime-view | Yes | Open-detail and open-in-browser flow. |
| 7 | 07-deployment-view | No | Same binaries and packaging. |
| 8 | 08-crosscutting-concepts | Yes | Popup rendering, key precedence, browser failure handling. |
| 9 | 09-architecture-decisions | No | No new ADR; the opener is a small external adapter under ADR-0003's boundary. |
| 10 | 10-quality-requirements | Yes | QS-027 reaching the pull request from an activity. |
| 11 | 11-risks-and-technical-debt | Yes | R-030 missing or failing browser opener. |
| 12 | 12-glossary | Yes | Activity detail term. |

## Structural assessment

No new aggregate. `PullRequestView` gains the pull request URL; the popup reads
the existing activity record and pull request view. The domain model's
relationships are unchanged.

## Status

STATUS: DONE
