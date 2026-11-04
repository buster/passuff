# Arc42 assessment: passuff-observation-targets

## Chapter selection

| Chapter | Affected | Reason |
|---|---|---|
| 01 Introduction and goals | No | The product goal, actors, and top quality goals are unchanged. |
| 02 Constraints | No | No new platform, dependency, or deployment constraint. |
| 03 Context and scope | No | GitHub remains the only external system; no new boundary. |
| 04 Solution strategy | Yes | The observation strategy changes from a fixed full page to probe-selected targets plus a reconciliation pass. |
| 05 Building block view | Yes | `PassuffCore`, `GitHubApiAdapter`, and `ObservationSession` change responsibilities and contracts. |
| 06 Runtime view | Yes | The check cycle and the live inbox observation flows change. |
| 07 Deployment view | No | No deployment or distribution change. |
| 08 Crosscutting concepts | Yes | Observation targeting, terminal retirement, and reconciliation replace the fixed full-observation concept. |
| 09 Architecture decisions | Yes | ADR-0007 is amended: the probe now selects targets and the inbox reconciles. |
| 10 Quality requirements | Yes | The thrift scenario changes; two new scenarios cover terminal retirement and reconciliation freshness. |
| 11 Risks and technical debt | Yes | TD-003 (full-cycle cost) closes; residual risks R-031/R-032 are added. |
| 12 Glossary | Yes | New terms: changed pull request, observation target, reconciliation pass, reconciliation interval, terminal retirement. |

## ADR qualification

- **Candidate:** probe-selected observation targets with a configurable reconciliation pass.
- **Architectural commitment:** fixes the observation strategy and the shape of the change probe and observation contracts across the core, the live inbox, and the headless watcher.
- **Real alternatives:** a smaller fixed observation set (open plus known watched) versus the current full page; a webhook or push channel is already rejected by ADR-0001/ADR-0007 and the local-only constraint.
- **Durable consequence:** the probe fingerprint and the reconciliation interval bound notification detection; reversing requires restoring the full-page observation and reworking the target-selection tests.
- **Existing decisions:** ADR-0007 "Probe before observing and pace the inbox against the account allowance" owns the observation strategy. This change refines it, so the route is `AMEND_ADR` for ADR-0007. No `NEW_ADR`: the cheaper home is ADR-0007 plus the Gherkin specifications and `design.md`.
- **Chapter 11 consequence:** the amended ADR's bad consequences (terminal retirement, fingerprint blind spots, pass-bounded involved freshness) are recorded as risks and as the changed thrift scenario.

## Notes

- The glossary and domain model were updated before review: `docs/glossary.md` gains changed pull request, observation target, reconciliation pass, reconciliation interval, and terminal retirement; `docs/domain-model.md` records the probe → targets → observation relationship.
- No new dependency, deployment unit, or external boundary is introduced.

## Status

STATUS: DONE
