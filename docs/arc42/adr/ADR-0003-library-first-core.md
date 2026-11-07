---
status: accepted
date: 2026-11-02
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Make the core a UI-agnostic library and the CLI its first adapter

## Context and Problem Statement

passuff needs a command-line interface today and a terminal user interface in a
future change. The shared configuration schema and the observation,
classification, and state logic must be shared by both. A monolithic binary or
a library surface shaped around CLI needs would force the future TUI to
duplicate domain logic or depend on terminal and desktop-delivery adapters it
does not want.

## Decision Drivers

- Reuse of the configuration schema and the observation, classification, and
  state logic by every UI.
- A dependency direction that keeps terminal and delivery libraries out of the
  core.
- Testability of the core without spawning a CLI process or a desktop daemon.
- A clear surface for a future TUI and any other adapter.
- A small, reviewable first implementation.

## Considered Options

- One binary with domain, delivery, and CLI concerns mixed together.
- One package with a library module and a CLI entry point, sharing one dependency graph.
- A workspace with a config library, a core library, and
  `passuff` as the first binary adapter; a future `passuff-tui` member.

## Decision Outcome

Chosen option: "A workspace with a config library, a core library, and
`passuff` as the first binary adapter; a future `passuff-tui` member", because it
gives the core a UI-neutral contract while keeping the CLI's terminal and
notification dependencies out of the libraries.

### Consequences

- Good, because the config and core libraries can be consumed by the CLI,
  the future TUI, and the Gherkin harness without a process boundary.
- Good, because argument parsing, notification, terminal rendering, and terminal
  input stay in the UI adapter packages; the shared config library has no UI dependency.
- Good, because the core refresh returns a `RefreshReport`, so a UI decides
  how to present and deliver notifications.
- Bad, because the workspace adds package boundaries, a public core API, and
  runner and coverage commands that target the CLI package.
- Bad, because the core API and config schema become contracts that must be
  maintained as the inbox and TUI arrive.

### Confirmation

The core library's dependency graph contains no argument parsing, notification,
terminal rendering, or terminal input library; it may depend on the config library
for the shared `CoreConfig` type.
The regular Gherkin scenarios call the CLI runtime's check entry point with injected ports
and assert the returned `RefreshReport` and exit code. The CLI E2E scenarios
spawn the `passuff` binary. The follow-up TUI and inbox changes will validate
the contracts when they are designed.

## Pros and Cons of the Options

### One binary with mixed concerns

- Good, because it has the fewest packages and the least setup.
- Bad, because the future TUI cannot reuse the core without moving modules.
- Bad, because domain tests can accidentally depend on CLI or delivery code.
- Bad, because the dependency graph includes terminal and notification libraries
  for every consumer.

### One package with a library module and a CLI entry point

- Good, because it provides a library entry point with little setup.
- Good, because the CLI can call the library directly.
- Bad, because the package's dependency graph still includes CLI and delivery
  libraries for any library consumer.
- Bad, because a future TUI would either join the same package or need a
  dependency feature split.

### Workspace with a config library, a core library, and `passuff`

- Good, because the dependency direction is explicit and enforced by the package layout.
- Good, because the config schema and core can be tested and consumed without a
  terminal or desktop surface.
- Good, because the future TUI is a natural sibling member.
- Neutral, because the runner and coverage commands need package selectors.
- Bad, because the core and config APIs are now contracts with more than one
  consumer.

## More Information

- [Use case: watch-pull-requests](../../../givn/specs/watch-pull-requests/usecase.md)
- [Building block view](../05-building-block-view.md)
- [ADR register](../09-architecture-decisions.md)

## Refinement (2026-11-02)

The TUI ships as a second binary (`passuff-tui`) in the `passuff` package
rather than a sibling workspace member. The decision outcome is unchanged: the
core stays UI-agnostic and the TUI consumes the config and core libraries
and `InboxService`. The package choice shares the Gherkin runner and the
CLI binary test seam; a future split is a packaging change.
