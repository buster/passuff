---
status: accepted
date: 2026-11-02
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Delegate GitHub authentication to the gh CLI instead of owning an OAuth application

## Context and Problem Statement

passuff observes pull requests for several GitHub accounts and needs one
credential per account. The operator may already have authenticated those
accounts with the GitHub CLI, which stores OAuth tokens and can open a browser
consent flow. passuff must choose whether it acquires OAuth tokens through gh,
registers and operates its own OAuth application, or supports only personal
access tokens.

## Decision Drivers

- No passuff-operated OAuth application, client ID lifecycle, or consent screen.
- Reuse the operator's existing gh sessions and multi-account storage.
- Keep personal access tokens available for headless and scripted use.
- Never store a token in the configuration file or transcript.
- A future TUI must be able to replace the interactive surface without changing
  the core.

## Considered Options

- Delegate OAuth acquisition and browser login to the gh CLI.
- Register a passuff OAuth application and implement the device flow.
- Support personal access tokens only.

## Decision Outcome

Chosen option: "Delegate OAuth acquisition and browser login to the gh CLI",
because it reuses an existing, operator-controlled consent and token store
without making passuff responsible for an OAuth application, while personal
access tokens remain the headless alternative.

### Consequences

- Good, because the operator can add an account with `gh auth login` and passuff
  reads it with `gh auth token --user`, so no new client secret or consent
  screen is shipped.
- Good, because personal access tokens stay a first-class mode for headless
  runs and do not depend on gh.
- Bad, because gh becomes a runtime dependency for the OAuth mode, and its
  login surface decides which account the browser authorizes; passuff can only
  validate the expected login after the fact.
- Bad, because the future TUI cannot inherit a terminal for gh's prompts and
  will need its own device flow with a registered application.
- Bad, because gh login can change the host's git protocol preference; passuff
  does not pass `--git-protocol` and leaves that choice to the operator.

### Confirmation

The credential boundary is covered by integration tests that run a fake `gh`
executable through `PATH`: token resolution, browser-login gating, one attempt
per gh login per process, and identity mismatch. Configuration validation tests cover the
credential matrix. The archive's regular and E2E suites must stay green.

## Pros and Cons of the Options

### Delegate to the gh CLI

- Good, because it reuses the operator's existing sessions and consent flow.
- Good, because passuff stores no OAuth client credentials.
- Neutral, because gh must be installed for this mode only.
- Bad, because account selection happens in the browser, not in passuff.

### Register a passuff OAuth application

- Good, because passuff controls the device flow and can target an account
  before consent.
- Bad, because it requires a registered application, a client ID, and an
  ongoing consent surface.
- Bad, because the first TUI is not part of this change, so the benefit is not
  realized now.

### Personal access tokens only

- Good, because it removes the gh dependency entirely.
- Bad, because it cannot reuse an existing gh session or open a browser when a
  token is missing.
- Bad, because the operator must create and rotate a PAT per account.

## More Information

- Design: `givn/archive/passuff-auth-modes/design.md` after archive.
- Rules R32–R38: `docs/arc42/12-glossary.md` links the glossary; the owning use
  case is `givn/specs/watch-pull-requests/usecase.md`.
- Supersedes nothing; ADR-0001 covers observation, ADR-0003 covers the core/UI
  boundary, and neither covers credential acquisition.
