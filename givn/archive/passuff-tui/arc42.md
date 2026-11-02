# Arc42 Impact Assessment: passuff-tui

Adds a second terminal binary over the existing inbox read model. No persisted
state, core contract, or configuration change.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | The operator can browse the inbox in a terminal UI; chapter 01 gains the requirement. |
| 2 | 02-architecture-constraints | No | No new platform or deployment constraint; ratatui/crossterm are cached dependencies. |
| 3 | 03-context-and-scope | Yes | A terminal UI surface is added; it reads config/state and never contacts GitHub. |
| 4 | 04-solution-strategy | Yes | Second adapter over `InboxService`, shipped in the CLI package for shared test infrastructure. |
| 5 | 05-building-block-view | Yes | `TuiApp`, `TuiAction`, and the `passuff-tui` binary. |
| 6 | 06-runtime-view | Yes | TUI load, render, key dispatch, read/reload, quit. |
| 7 | 07-deployment-view | Yes | A second binary in the same workstation install; no server. |
| 8 | 08-crosscutting-concepts | Yes | Terminal rendering, key handling, snapshot mode, error exits. |
| 9 | 09-architecture-decisions | No | No new ADR; ADR-0003 covers the adapter boundary. |
| 10 | 10-quality-requirements | Yes | QS-020 for snapshot/interactive verification and read actions. |
| 11 | 11-risks-and-technical-debt | Yes | No PTY test, terminal requirement, packaging deviation. |
| 12 | 12-glossary | Yes | Terminal UI term. |

## Status

STATUS: DONE
