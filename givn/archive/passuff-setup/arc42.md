# Arc42 Impact Assessment: passuff-setup

Adds a line-based first-run setup flow shared by `passuff setup` and the
terminal UI.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | First-run setup requirement. |
| 2 | 02-architecture-constraints | No | No new platform or dependency constraint; local prompts and files only. |
| 3 | 03-context-and-scope | Yes | Setup writes configuration on a fresh machine. |
| 4 | 04-solution-strategy | Yes | Shared SetupSession, validated atomic write. |
| 5 | 05-building-block-view | Yes | `SetupSession`. |
| 6 | 06-runtime-view | Yes | First-run setup flow. |
| 7 | 07-deployment-view | No | No service or packaging change. |
| 8 | 08-crosscutting-concepts | Yes | Setup secrets, validation, atomic write, non-terminal hint. |
| 9 | 09-architecture-decisions | No | No new ADR; setup is a local adapter over the existing config schema. |
| 10 | 10-quality-requirements | Yes | QS-023 onboarding. |
| 11 | 11-risks-and-technical-debt | Yes | R-026 unverified credential. |
| 12 | 12-glossary | Yes | Initial setup term. |

## Structural assessment

No new aggregate; setup is a configuration-writing adapter. The domain model's
configuration relationship is unchanged.

## Status

STATUS: DONE
