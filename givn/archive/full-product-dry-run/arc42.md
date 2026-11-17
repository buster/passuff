# arc42 Documentation Update: full-product-dry-run

## Impact assessment

| # | Chapter | Affected? | Reason | Summary of change (if Yes) |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | No | The change records a trial; no goal, stakeholder, or quality attribute changes. | |
| 2 | 02 architecture-constraints | No | `docs/limits.md` and the declared constraints stay; the trial keeps both and changes neither. The declaration's coverage is recorded as a finding, not edited. | |
| 3 | 03 context-and-scope | No | No external system, interface, or user-facing surface is added or changed. | |
| 4 | 04 solution-strategy | No | The solution strategy, the three production crates, and the frozen package stay. | |
| 5 | 05 building-block-view | No | No building block is added, removed, or re-owned; the trial rebuilds the existing ones in a copy. | |
| 6 | 06 runtime-view | No | No runtime flow changes; the rebuilt product runs the flows the corpus already states. | |
| 7 | 07 deployment-view | No | No deployment or packaging change; the copy is trial-only and is not deployed. | |
| 8 | 08 crosscutting-concepts | No | The test strategy and the two test scopes stay; the trial runs the existing suites. | |
| 9 | 09 architecture-decisions | No | No lasting component, contract, storage, or compatibility boundary is created. The change strategy lives in the change's design ledger, not a project ADR. | |
| 10 | 10 quality-requirements | No | No new quality scenario; the existing quality requirements and the frozen suite stay. | |
| 11 | 11 risks-and-technical-debt | No | The trial accepts no new risk and records no product debt. A discovered `specification defect` is a recorded trial finding routed to a later change, not a debt entry in the product architecture. | |
| 12 | 12 glossary | No | No product domain term is added; "spec copy", "manifest", "gap table", and "neighbour" are process vocabulary defined by `givn instructions ise`. | |

## Notes

The assessment checked `docs/arc42/adr/README.md` and chapter 09 for an
existing decision about the rebuild boundary. No ADR owns it; the trial is
governed by `givn/specification.yaml` and the change's design ledger.

ADR qualification:

```text
Architectural commitment: none, a whole-product trial in a copy
Real alternatives: promote the rebuilt product or keep the checkout product (D6 chooses keep)
Durable consequence: none, the trial copy is outside the repository and is not deployed
Existing-decision search: docs/arc42/adr/README.md and 09-architecture-decisions.md; no matching decision
ADR action: NONE
ADR target: none, the change design and the trial evidence own the record
```

## Status

STATUS: DONE
