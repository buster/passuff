# Proposal: passuff-setup

## Release note

Set up passuff interactively by creating a validated account configuration when none exists.

## Use-Case Context

- Use-case ID: `set-up-passuff` (new)
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer`

## Review Context

- Review kind: self-review
- Reason: the operator directed a single in-context review for this first-run setup change; the review records the credential and persistence checks in `review.md`.

## Problem / Opportunity

A fresh machine has no `~/.config/passuff/config.toml`. Both binaries load the
file at startup and exit 2 when it is missing, so the terminal inbox cannot
start. The README tells the operator to write TOML by hand, including the
credential mode, expected login, and watch scope. That is error-prone,
especially for the first account, and it blocks the inbox on the exact screen
the operator is trying to open.

## Proposed Solution

- When the configuration file is missing, `passuff-tui` opens a setup screen
  instead of exiting 2. The screen collects one or more accounts and the watch
  scope, writes the configuration atomically, and then continues to the inbox.
- `passuff setup` runs the same setup flow from the command line, for a
  non-TUI first run.
- Setup prompts for the account label, the credential mode (gh OAuth, token
  environment variable, token command, or token file), the expected GitHub
  login, and the watch scope (all repositories or a selected set).
- Setup validates the configuration before writing it and never writes a token
  into the configuration or state. A token file is referenced by path; the
  file's permissions are checked.
- Setup creates the configuration directory when needed, writes through a
  temporary file and rename, and does not overwrite an existing configuration
  without an explicit confirmation.
- A configuration created by setup is immediately usable by `passuff`,
  `passuff check`, and `passuff-tui`.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| initial-setup | no signal (tie) | `NEW in set-up-passuff` | Setup is a new first-run goal, separate from watch, status, inbox, stacks, and watch-scope editing. |

## Out of Scope

- Editing accounts or credentials after setup; a later change may reuse the
  same service.
- Migrating an existing configuration to the new setup screen.
- Hosted or browser-based onboarding.
- Verifying every account against GitHub during setup; a credential that
  cannot be resolved is reported without blocking the configuration write.

## Open Questions

### D1 — Which setup surfaces ship?

- Question: is setup available from the terminal inbox, from `passuff setup`,
  or both?
- Options:
  1. **Both** — the TUI opens setup when the config is missing, and
     `passuff setup` runs the same flow. Gain: the reported failure is fixed
     and a scriptable/non-TUI path exists. Cost: two entry points over one
     service.
  2. **TUI only** — setup starts from `passuff-tui`. Gain: smaller surface.
     Cost: `passuff check` still exits 2 on a fresh machine.
  3. **CLI only** — `passuff setup`; the TUI tells the operator to run it.
     Gain: simplest testable surface. Cost: the inbox still does not open
     directly on a fresh machine.
- Disposition: carried — `design.md` decides; owner: design.

### D2 — How do tests drive the prompts?

- Question: should the setup service take an injected reader/writer so the
  Gherkin runner can script answers, or should the binary expose
  non-interactive setup flags?
- Options:
  1. **Injected session** — the service reads a scripted input stream and
     writes a transcript; the binary uses stdin/stdout, the tests use a
     vector. Gain: the real service is tested without a PTY. Cost: a small
     IO abstraction.
  2. **Scripted flags** — `--account`, `--auth`, `--login`, `--watch` flags.
     Gain: simple E2E. Cost: a second input surface to maintain.
- Disposition: carried — `design.md` decides; owner: design.

### D3 — Does setup resolve credentials and check the login?

- Question: during setup, should passuff run the credential source and verify
  the resolved GitHub login, or only validate the configuration shape?
- Options:
  1. **Resolve and verify** — fail the account step when the token/login is
     wrong. Gain: early feedback. Cost: network/gh dependency during setup.
  2. **Shape only** — write the configuration and report unresolved
     credentials later. Gain: setup works offline. Cost: a bad credential is
     discovered on the first check.
- Disposition: carried — `design.md` decides; owner: design.

### D4 — What happens when a configuration already exists?

- Question: should setup refuse, overwrite, or ask before replacing an
  existing configuration?
- Options:
  1. **Ask, default no** — a confirmation before replacing. Gain: safe.
     Cost: one more prompt.
  2. **Refuse** — setup only for a missing file. Gain: simplest. Cost: no
     re-run without deleting the file.
- Disposition: carried — `design.md` decides; owner: design.
