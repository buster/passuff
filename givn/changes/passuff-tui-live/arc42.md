# Arc42 Impact Assessment: passuff-tui-live

The open terminal inbox becomes a watcher: a cheap change probe is paced against
the account allowance and a full check cycle runs only on a change.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Freshness goal for the terminal inbox. |
| 2 | 02-architecture-constraints | No | No new platform, dependency, or deployment constraint. |
| 3 | 03-context-and-scope | Yes | The open inbox now contacts GitHub; the row was split from the read-only commands. |
| 4 | 04-solution-strategy | Yes | Probe-before-observing replaces the poll-based refresh strategy. |
| 5 | 05-building-block-view | Yes | `ObservationSession`, `PassuffCore::probe`, and the TUI's observation channel. |
| 6 | 06-runtime-view | Yes | Live observation scenario; the navigation scenario loses its refresh tick. |
| 7 | 07-deployment-view | No | Same binaries and packaging; no service or file layout change. |
| 8 | 08-crosscutting-concepts | Yes | Allowance pacing, input thread, failure backoff, snapshot surfaces. |
| 9 | 09-architecture-decisions | Yes | ADR-0007 probe before observing and pace against the allowance. |
| 10 | 10-quality-requirements | Yes | QS-025 timeliness and QS-026 thrift. |
| 11 | 11-risks-and-technical-debt | Yes | R-002 updated; R-028/R-029 and TD-003/TD-004 added. |
| 12 | 12-glossary | Yes | Change probe; watcher and terminal UI widened. |

## Structural assessment

No new aggregate. The change adds one read-side query and one scheduler
component; `WatcherState` is unchanged. The domain model's relationships are
unchanged; the glossary gains the probe term.

## Status

STATUS: DONE
