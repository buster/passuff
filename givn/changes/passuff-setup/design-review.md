# Design Review: passuff-setup

Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: self-review (operator-directed single in-context review)

## Findings and closure

| Finding | Closure |
|---|---|
| D1 surface choice was open | Both surfaces: `passuff setup` and the first-run TUI path share `SetupSession`; design records it. |
| D2 prompt driving was open | Injected `BufRead`/`Write`; the Gherkin step scripts the answers; the e2e spawns the real `passuff setup`. |
| D3 credential check was open | Shape and token-file permission only; no GitHub call; R-026 records the first-check failure. |
| D4 existing config was open | Ask, default no; the preserve/replace scenarios cover both. |
| Prompt order differed between design and implementation | Use case corrected: add-another is asked after each account, then the watch scope. |
| Invalid repository did not print an error | `validate_repository` errors now go through `fail`, which writes the message. |
| Escaped-quote assertions were brittle | The specs use dedicated `contains auth`, `contains token env`, and `contains login` steps. |
| A duplicate TUI step was ambiguous | The setup step file reuses the existing TUI step. |
| The e2e setup had no config directory | The `no configuration file exists` step creates the e2e config/state paths before removing the file. |
| Token file permissions | `check_token_file` checks `0o077` and reports `insecure`. |
| Write failure | Temp file plus rename; a failure reports `cannot write configuration` and leaves no file. |

## Verification

- Delta: 8/8; delta e2e: 1/1; permanent regular: 146/146; permanent e2e: 11/11.
- Unit: `passuff-cli` 32, including seven `SetupSession` tests.
- The shared session is exercised by the CLI e2e, the regular scenarios, and
  the unit tests. The interactive TUI first-run branch is the one path not
  driven by a PTY; the non-terminal hint and the shared session are covered.
- Visual: setup is line-based; the setup transcript and the post-setup inbox
  are the evidence; the inbox layout is the existing approved TUI contract.
- Contracts: `SetupSession` is the only new contract; it consumes
  `ConfigLoader` and writes the configuration file.
- Use case: one goal, one actor, one capability, one inventory entry, one e2e.

Residual: the interactive TUI prompt fallback is not driven through a PTY,
matching the project's recorded R-019 posture. This is a non-blocking test
residual, not a behavioral gap.

DESIGN-REVIEW: PASS
