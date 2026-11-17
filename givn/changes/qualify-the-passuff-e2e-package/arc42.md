# arc42 Documentation Update: qualify-the-passuff-e2e-package

## Impact assessment

| # | Chapter | Affected? | Reason | Summary of change (if Yes) |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | No | No goal, stakeholder, or quality attribute changes; the change records existing limits and qualifies an existing test asset. | |
| 2 | 02 architecture-constraints | Yes | The terminal budgets and the exit-code and runner-result contract gain a declared home; the Specification must name it. | New `Declared limits` subsection naming `docs/limits.md` and the two declared `limits` entries. |
| 3 | 03 context-and-scope | No | No external system, interface, or user-facing surface is added or changed. | |
| 4 | 04 solution-strategy | No | No technical strategy or approach changes; the package and runner stay as delivered. | |
| 5 | 05 building-block-view | No | No module, component, or dependency changes. | |
| 6 | 06 runtime-view | No | No runtime flow changes; the suite and the product run as before. | |
| 7 | 07 deployment-view | No | No deployment or packaging change. | |
| 8 | 08 crosscutting-concepts | Yes | The testing subsection described the in-process CLI E2E and the D-Bus fake; the frozen package now drives the built binaries and the qualification script is the sensitivity check. | The `Testing and coverage` subsection names both scopes, the binary-and-pty acceptance package, the notification-log seam, and the twenty seeded defects. |
| 9 | 09 architecture-decisions | No | Searched ADR-0001 to ADR-0007 and the register; no lasting component, contract, storage, or compatibility boundary is created. A declaration entry and a test-asset qualification are documentation and tests, not a new ADR. | |
| 10 | 10 quality-requirements | No | No new quality scenario; the existing testability rows (QS-009, QS-012) still describe the suite. | |
| 11 | 11 risks-and-technical-debt | Yes | R-019's mitigation claimed no PTY coverage; the frozen package now drives the TUI through a real pty at both budgets. The 29-scenario acceptance limit is unrecorded debt. | R-019 mitigation updated; TD-007 records the suite-versus-product limit and its plan. |
| 12 | 12 glossary | No | No product domain term is added; "seeded defect", "qualification", and "budget" belong to the test tooling, and the glossary owns product vocabulary. | |

## Notes

The assessment checked `docs/arc42/adr/README.md` and chapter 09 for an existing
decision about the acceptance boundary. ADR-0003 records the UI-agnostic core;
none records the test boundary, and this change does not establish one.

ADR qualification:

```text
Architectural commitment: none, a declared limit document and a test-asset qualification
Real alternatives: replace the arc42 limits entry, or declare both documents (D1)
Durable consequence: reversing the declaration means re-declaring limits; no code, contract, or state changes
Existing-decision search: docs/arc42/adr/README.md and 09-architecture-decisions.md, ADR-0001 to ADR-0007; no matching decision
ADR action: NONE
ADR target: none, docs/limits.md and givn/specification.yaml own the decision
```

## Status

STATUS: DONE
