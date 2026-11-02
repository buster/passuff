# Arc42 Impact Assessment: passuff-pr-watch

This is the first completed change. It establishes the real twelve-chapter
baseline proportionate to a single-operator Rust CLI.

Revision 2026-11-02: the design was hardened before sign-off. The core is a
UI-agnostic library (`passuff-core`), configuration is a shared library
(`passuff-config`), the CLI is the first adapter (`passuff-cli`), and the Linux
E2E scope uses a private D-Bus fake. ADR-0003 records the core boundary. A
second hardening pass added the CLI library entry point, the `ci_failed` recovery
flag, 30-day reopen tracking, explicit D-Bus readiness/isolation, credential
redaction, and unit-test coverage. The activity inbox and TUI remain follow-up
changes.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | New project baseline: requirements, quality goals including reusability, stakeholders, and the permanent specification link. |
| 2 | 02-architecture-constraints | Yes | Rust, macOS/Linux, no inbound surface, GraphQL, local state, tokens outside config, UI-agnostic core, GPL-3.0-or-later. |
| 3 | 03-context-and-scope | Yes | GitHub, desktop notification surface, audio player, config/state files. |
| 4 | 04-solution-strategy | Yes | Poll authored PRs, pure classifier, core persists before return, CLI delivers, three-crate library-first workspace, Linux D-Bus E2E. |
| 5 | 05-building-block-view | Yes | `passuff-config`, `passuff-core` (core, classifier, GitHub adapter, state, credential resolver), and `passuff-cli` (transcript, delivery, sound); future `passuff-tui`. |
| 6 | 06-runtime-view | Yes | Check cycle with core/CLI split, continuous watch, one-account failure. |
| 7 | 07-deployment-view | Yes | Workstation CLI, macOS app bundle, no server. |
| 8 | 08-crosscutting-concepts | Yes | Credentials, persistence, dedupe, error isolation, UI adapter boundary, Linux D-Bus fake, macOS recording delivery, rate limits. |
| 9 | 09-architecture-decisions | Yes | Three ADRs: polling, local desktop delivery, library-first core. |
| 10 | 10-quality-requirements | Yes | Accuracy, replay, reliability, timeliness, security, portability, reusability, Linux desktop testability. |
| 11 | 11-risks-and-technical-debt | Yes | macOS identity, polling latency, rate limits, daemon/player absence, at-most-once loss, manual macOS check, core API churn, missing inbox query API, `dbus-daemon` dependency. |
| 12 | 12-glossary | Yes | Notification redefined as a grouped report; Observation and Account failure added; code symbols reconciled with the design. |

All twelve chapter files exist, contain decision-specific content, and use
Mermaid only.

## ADR qualification

### Candidate 1: Poll authored pull requests instead of webhooks or the notifications feed

- Architectural commitment: yes. It fixes the deployment boundary (no inbound
  endpoint), the read-only GitHub boundary, and the durable observation model.
- Real alternatives: yes. Webhooks and the notifications feed are materially
  different mechanisms with different deployment and filtering behaviour.
- Durable consequence: yes. Reversing the choice would add a public endpoint,
  webhook secrets, delivery retries, and a different filtering model.
- Existing decisions: none. No ADR register entry exists before this change.
- Cheaper home: `design.md` plus `docs/arc42/03-context-and-scope.md` could
  document the observation mechanism. Rejected because the choice fixes a
  deployment and security boundary with lasting operational consequences.
- Route: `NEW_ADR` at `docs/arc42/adr/ADR-0001-poll-authored-pull-requests.md`.

### Candidate 2: Local desktop notifications and a bundled sound instead of a hosted push channel

- Architectural commitment: yes. It fixes the delivery boundary (no third-party
  account or hosted relay), the platform adapter boundary, and the packaging
  approach for macOS.
- Real alternatives: yes. Hosted push and email are materially different
  delivery mechanisms with different credential and availability models.
- Durable consequence: yes. Reversing the choice would add a third-party
  account, a second credential store, and a different failure domain; the macOS
  bundle identity is also a lasting packaging commitment.
- Existing decisions: none at the time of the decision. ADR-0001 covers
  observation, not delivery.
- Cheaper home: the use-case contract and `design.md` could document the
  delivery choice. Rejected because the choice fixes the delivery boundary and
  macOS packaging, both durable across future changes.
- Route: `NEW_ADR` at `docs/arc42/adr/ADR-0002-local-desktop-notifications.md`.

### Candidate 3: Library-first core with UI adapters

- Architectural commitment: yes. It fixes the component ownership and dependency
  direction between `passuff-config`, `passuff-core`, `passuff-cli`, and a
  future `passuff-tui`.
- Real alternatives: yes. A single mixed binary and a single package with
  `lib.rs` plus `main.rs` are materially different dependency boundaries.
- Durable consequence: yes. The core API is a contract with more than one
  consumer; reversing the split later requires moving modules, changing runner
  and coverage paths, and updating every consumer.
- Existing decisions: none. ADR-0001 covers observation; ADR-0002 covers
  delivery; neither covers the core/UI boundary.
- Cheaper home: `design.md` could document the workspace layout. Rejected
  because the dependency direction is a lasting component boundary with a
  second planned consumer.
- Route: `NEW_ADR` at `docs/arc42/adr/ADR-0003-library-first-core.md`.

### Candidate 4: Versioned JSON state instead of SQLite

- Architectural commitment: no. The state is local and has no external
  consumer; the format can change within a focused change.
- Real alternatives: yes, but the choice is reversible and does not fix a
  component or deployment boundary.
- Durable consequence: no migration or cross-component contract is required.
- Existing decisions: none.
- Route: no ADR; the decision belongs in `design.md` (`JsonStateStore`).

### Candidate 5: Rust + Tokio + clap stack

- Architectural commitment: no. The stack is a project technology choice, not a
  durable component or ownership boundary.
- Real alternatives: yes, but the choice is recorded in the design's technology
  table and can be revisited without an architectural migration boundary.
- Durable consequence: no.
- Existing decisions: none.
- Route: no ADR; the decision belongs in `design.md`.

## Status

STATUS: DONE
