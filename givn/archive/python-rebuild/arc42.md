# arc42 Assessment: python-rebuild

## Chapter assessment

| # | Chapter | Affected | Reason or change |
|---|---|---|---|
| 1 | Introduction and Goals | Yes | Corrected the authoritative use-case list from five to six and added `set-up-passuff`; the goals themselves are unchanged. |
| 2 | Architecture Constraints | No | The Python target, platform scope, Gherkin corpus, and coverage gate were already recorded; the rebuild changes none of them. |
| 3 | Context and Scope | No | No external system, interface, actor, or scope boundary changes. |
| 4 | Solution Strategy | Yes | Replaced the custom Gherkin runner with `behave`, the terminal UI library with stdlib `curses` plus a pure frame renderer, and the test row with the local twins and the separate E2E step tree; clarified TOML preservation and rejection. |
| 5 | Building Block View | Yes | Corrected the stale harness and input-thread statements, moved the recording double out of production, removed the unused `StackGrouper`, and fixed the read-model routing; the component map is unchanged. |
| 6 | Runtime View | Yes | Replaced the Rust-era `TuiAction::OpenUrl` reference with a language-neutral open intent and `toml_edit` with `tomlkit`. |
| 7 | Deployment View | Yes | Marked the `Passuff.app` bundle as deferred (2026-11-08) in chapter 7 and added TD-007 in chapter 11; the CLI/TUI deployment is unchanged. |
| 8 | Cross-cutting Concepts | Yes | Updated typed configuration and unknown-key rejection, the TUI threading and snapshot description with the activity body bounded at 4 KiB, the 80-character list excerpt, `tomlkit` scope writes, and the behave runner, separate E2E steps, and twin model. |
| 9 | Architecture Decisions | No | No new ADR. The rebuild reuses ADR-0003's core boundary and ADR-0001, ADR-0002, ADR-0004, ADR-0005, ADR-0006, and ADR-0007; the verification union is process guidance, not an architectural boundary. |
| 10 | Quality Requirements | No | No quality attribute or scenario changes; the rebuild restores the existing quality goals. |
| 11 | Risks and Technical Debt | Yes | Updated R-019, R-020, and R-027; recorded TD-005 as resolved and added TD-007 for the deferred bundle. |
| 12 | Glossary | Yes | No new domain terms; refreshed the `CredentialSpec`, `Inbox`, and `Terminal UI` symbols, added `inbox-navigation` and `initial-setup` to the capability list, and corrected the activity-detail body. |

## ADR qualification

No candidate qualifies.

- The verification union (V1) is process guidance owned by chapter 8 and
  `scripts/verify.sh`; it fixes no component, contract, storage, or deployment
  boundary.
- The runner (`behave`), TUI rendering (`curses` plus a frame renderer), TOML
  writer (`tomlkit`), and test twins are implementation choices owned by
  `design.md`. Each is reversible within this change and none defines a new
  port/adapter boundary.
- ADR-0003 already owns the core/adapter boundary and ADR-0005 owns the state
  migration. No existing ADR is amended or superseded.

## Status

STATUS: DONE
