# Arc42 Impact Assessment: passuff-auth-modes

This change adds explicit credential modes, a gh-delegated OAuth path with an
interactive browser login, a permission-checked token file, and identity
validation. The core/CLI boundary is unchanged; the credential adapter grows
inside the existing contract.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Goals are unchanged; the quality goals gain interactive recovery, credential privacy, and identity correctness. Chapter 10 records the new quality scenarios. |
| 2 | 02-architecture-constraints | Yes | The credential constraint now covers the gh session, helper commands, and permission-checked token files; secrets still must not enter configuration or state. |
| 3 | 03-context-and-scope | Yes | The gh CLI becomes an external system for token reads and interactive login; the credential interface row now covers the session, environment, command, and token file. |
| 4 | 04-solution-strategy | Yes | The strategy adds delegation of OAuth to gh while keeping PATs for headless use; ADR-0004 records the choice. |
| 5 | 05-building-block-view | Yes | A credential-acquisition section describes environment, command, token file, gh OAuth, and identity validation. |
| 6 | 06-runtime-view | Yes | A new sequence covers interactive credential recovery and identity validation. |
| 7 | 07-deployment-view | No | The workspace and deployment topology are unchanged; gh is already an operator tool on the same workstation. |
| 8 | 08-crosscutting-concepts | Yes | Configuration and credential rules, security, and account-failure handling are updated; the stale `PassuffCore::open` reference is corrected to `with_ports`. |
| 9 | 09-architecture-decisions | Yes | ADR-0004 is added to the register and the ADR index. |
| 10 | 10-quality-requirements | Yes | QS-013 (interactive recovery), QS-014 (token-file privacy), and QS-015 (identity mismatch) are added. |
| 11 | 11-risks-and-technical-debt | Yes | R-012 (gh version), R-013 (browser-selected account), R-014 (git protocol side effect), and R-015 (permission change between load and use) are added. |
| 12 | 12-glossary | Yes | Credential mode, expected login, gh session, personal access token, and token file are added to `docs/glossary.md`; the domain model records that they are configuration and validation attributes. |

## ADR qualification

### Candidate: delegate GitHub authentication to the gh CLI

- Architectural commitment: yes. It fixes the external authentication
  dependency, the credential acquisition boundary, and the consent surface for
  the OAuth mode.
- Real alternatives: yes. A passuff-owned OAuth application with a device flow
  and a PAT-only product are materially different credential boundaries.
- Durable consequence: yes. Reversing the choice would replace the token
  acquisition path, add an OAuth application registration, and change the TUI
  login plan; the gh runtime dependency is durable for this mode.
- Existing decisions: none. ADR-0001 covers observation, ADR-0002 covers
  delivery, and ADR-0003 covers the core/UI boundary; no ADR covers credential
  acquisition.
- Route: `NEW_ADR` at
  `docs/arc42/adr/ADR-0004-gh-delegated-authentication.md`.

No other candidate qualifies: the token-file permission rule and the identity
check are configuration and validation behavior owned by the use case and
design, not new architectural boundaries.

## Status

STATUS: DONE
