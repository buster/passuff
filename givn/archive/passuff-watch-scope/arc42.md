# Arc42 Impact Assessment: passuff-watch-scope

Adds an explicit all/selected watch scope, a terminal repository picker, hidden
out-of-scope records, silent catch-up, and configuration reload for a running
watcher.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Requirement to change the watch scope from the terminal inbox. |
| 2 | 02-architecture-constraints | Yes | The terminal UI may write the configuration atomically; the watcher reloads each cycle. |
| 3 | 03-context-and-scope | Yes | Configuration becomes read/write; per-account scope and a full catalog stay outside. |
| 4 | 04-solution-strategy | Yes | Explicit watch mode, atomic config edit, reload, hide, and catch-up. |
| 5 | 05-building-block-view | Yes | `WatchMode`, `ScopeEditor`, `ScopePicker`, `TuiApp` picker, `last_scope`. |
| 6 | 06-runtime-view | Yes | Picker edit, atomic write, reload, filtering, and catch-up flow. |
| 7 | 07-deployment-view | No | No new process, service, or packaging change. |
| 8 | 08-crosscutting-concepts | Yes | Watch-scope editing, atomic writes, hide, catch-up, reload failure. |
| 9 | 09-architecture-decisions | Yes | ADR-0006 records the explicit all/selected scope, legacy resolution, and empty-selected meaning. |
| 10 | 10-quality-requirements | Yes | QS-022 for the picker, hide, and catch-up. |
| 11 | 11-risks-and-technical-debt | Yes | R-022 concurrent/failed config writes; R-023 picker clipping; R-024 pre-existing state save race. |
| 12 | 12-glossary | No | The Watch scope definition already covers all/selected/empty; no new term. |

## Durable updates made

- Chapter 01 requirement list.
- Chapter 02 configuration-write/reload constraint.
- Chapter 03 read/write configuration and scope boundaries.
- Chapter 04 explicit watch-scope strategy.
- Chapter 05 `WatchMode`, `ScopeEditor`, `ScopePicker`, and `TuiApp` responsibilities.
- Chapter 06 watch-scope edit runtime flow.
- Chapter 08 watch-scope editing, atomic writes, reload, and catch-up.
- Chapter 10 QS-022.
- Chapter 11 R-022/R-023/R-024.
- ADR-0006 and the chapter 09 register/index.
- Domain model: no new aggregate; the watch scope stays configuration and the
  Terminal UI gains a configuration-edit relationship.
- Glossary: Terminal UI now names `ScopeEditor`.

## ADR qualification (ADR-0006)

- Commitment: the configuration schema gains a mode; every configuration writer
  must preserve it and every reader must resolve legacy files by list presence.
- Alternatives: keep empty = all and forbid the last deselection; keep empty =
  all and switch to all on the last deselection; explicit all/selected.
- Durable consequence: every future config writer and migration must honor the
  mode and the legacy resolution rule, not just the repository list.
- Existing-decision search: ADR-0005 covers the state migration, not the
  configuration semantics; no earlier ADR owns this decision.
- Cheaper home: `design.md` and chapter 08 describe the mechanism, but a durable
  compatibility contract that future writers must honor belongs in an ADR.
- Result: NEW_ADR (ADR-0006).

## Status

STATUS: DONE
