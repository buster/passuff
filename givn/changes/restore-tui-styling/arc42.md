# arc42 Assessment: restore-tui-styling

## Chapter assessment

| # | Chapter | Affected | Reason or change |
|---|---|---|---|
| 1 | Introduction and Goals | Yes | The goals are unchanged, but chapter 1 now carries a refinement note: the visual contract supports the existing terminal-interface goal and adds no new goal. |
| 2 | Architecture Constraints | No | Python, the terminal interface, the Gherkin corpus, and the watn palette are existing constraints. The palette values are now recorded in the design system, not invented. |
| 3 | Context and Scope | No | No external system, interface, actor, or scope boundary changes. The same terminal surfaces are restyled. |
| 4 | Solution Strategy | Yes | The terminal UI strategy changes from a pure text frame to a role-tagged styled frame with a plain, ANSI, and role-record projection; the palette and composition are owned by `docs/design/design-system.md`. |
| 5 | Building Block View | Yes | `TuiApp` now applies the role-tagged palette and attributes; `TerminalPresentation` is added as the frame model and projection component; the save-prompt state is part of the TUI adapter. |
| 6 | Runtime View | Yes | The quit flow gains the save prompt: `q` opens `Save sort and filter settings?`, `y` persists and exits, `n` discards and exits, and `Esc` cancels. The observation and redraw flow is unchanged. |
| 7 | Deployment View | No | No new dependency, service, port, or deployment boundary. The TUI remains a stdlib `curses` binary in the `passuff` package. |
| 8 | Cross-cutting Concepts | Yes | The palette roles, the titled-frame composition, the styled/ANSI/role snapshot, accessibility (no color alone), and the visual evidence path are cross-cutting concepts. |
| 9 | Architecture Decisions | No | No new ADR. The qualification is below. |
| 10 | Quality Requirements | Yes | Added QS-034 (visual fidelity) and updated the terminal UI descriptions; the visual contract becomes executable through the role record and ANSI transcript. |
| 11 | Risks and Technical Debt | Yes | Added R-037 for palette/composition drift and terminals without 256-color support, and R-038 for the save-prompt quit behavior. |
| 12 | Glossary | Yes | Added `Watn palette`, `Palette role`, `Styled frame`, and `Role record` to the domain glossary, and clarified the distinction from the observation snapshot. |

## ADR qualification

No candidate qualifies.

- **Architectural commitment:** the change adds the `TerminalPresentation`
  presentation component inside the existing terminal adapter, and no
  ownership boundary, dependency direction, deployment or data boundary, or
  compatibility boundary. The palette is a quality constraint owned by the
  durable design system.
- **Real alternatives:** a styled frame with ANSI and role projections, an
  ANSI-only frame, and a screenshot-only artifact are materially different as
  evidence mechanisms, but they are design and evidence choices, not
  architectural boundaries.
- **Durable consequence:** reversing the styled frame would change the
  presentation layer and its visual tests, not a system boundary, persisted
  data, or deployment contract. The visual contract itself is durable in
  `docs/design/design-system.md`.
- **Existing decisions:** ADR-0003 owns the core/adapter boundary and ADR-0002
  owns local desktop notifications. Both are unchanged. No ADR is added,
  amended, or superseded.

## Status

STATUS: DONE
