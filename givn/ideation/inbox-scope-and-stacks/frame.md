# Frame: inbox-scope-and-stacks

Confirmed: 2026-11-03 by the operator (Adopt). This confirmation covers the two
goals, the boundaries above, and the stakeholder list; it does not authorize
implementation, change creation, or delivery order.

## Problem in one sentence

The inbox is not organized around the work the developer actually does: changing
which repositories are watched requires editing TOML and restarting, and a stack
of pull requests appears as unrelated rows, so the developer reconstructs the
stack by hand.

If the named solutions are the answer, the questions are:

- "How does the developer change the watch scope without leaving the inbox?"
- "How does the inbox present a stack of pull requests as one unit of work?"

## Goal

Two outcomes, one topic, separate future changes:

1. The developer can change the watch scope from the terminal UI, without editing
   the configuration file or restarting the watcher.
2. The inbox shows stacked pull requests together, with stack position and shared
   activity, so the developer reviews the stack as one unit.

## Scope

Inside:

- Watch-scope editing from the terminal UI: seeing candidate repositories,
  selecting and deselecting them, and persisting the result.
- Stack membership from GitHub's stack metadata where the repository has the
  preview enabled, and a fallback when it does not.
- Stack-aware inbox read model: grouping watched pull requests and their activity
  by stack, with position and the stack's ultimate base.
- Behaviour for standalone pull requests in the same inbox.
- Multi-account and repository-allowlist interactions.

Outside:

- GitHub transport and latency: webhooks, relays, and streaming. Answered during
  the preceding discussion; polling stays. A future transport topic owns it.
- Creating, extending, or dissolving stacks; passuff stays read-only toward
  GitHub.
- GitHub Projects boards; "project" in the request means a repository in the
  watch scope.
- Non-authored pull requests, review of code diffs, and desktop delivery
  changes.
- Per-account watch scope; the current scope is one global repository allowlist.

## Stakeholders

| Stakeholder | Interest |
|---|---|
| Developer (operator) | Changes the watched repositories without editing TOML; reviews a stack as one unit instead of reconstructing it from rows. |
| Maintainer (same person) | A small, testable Rust codebase; no hosted infrastructure; a UI-agnostic core; a state schema that survives upgrades; preview-absence degrades cleanly. |
| GitHub | Receives read-only queries; stack metadata is available only where the stacked-PRs preview is enabled; no mutation requests. |

## Research protocol

Commands and sources read before framing:

- `givn status`: four archived changes, no active change.
- `givn spec tree`: three use cases — `watch-pull-requests` (56 scenarios),
  `inspect-watch-status` (11), `review-activity-inbox` (27).
- `docs/glossary.md`, `docs/domain-model.md`, `docs/arc42/01-introduction-and-goals.md`,
  `docs/arc42/03-context-and-scope.md`, `README.md`.
- `givn/personas/multi-account-developer.md`.
- Code: `crates/passuff-config/src/lib.rs` (`WatchConfig.repositories`),
  `crates/passuff-core/src/inbox.rs` (`InboxService::view`),
  `crates/passuff-core/src/github.rs` (GraphQL `MAIN_QUERY` fetches
  `baseRefName`; no `headRefName`, no `stack`),
  `crates/passuff-core/src/domain.rs` (`WatchedPullRequest` stores no refs),
  `crates/passuff-cli/src/tui.rs` (read-only rendering).
- GitHub documentation: stacked pull requests APIs and webhooks; GraphQL
  read-only `stack` and `stackEntry`; REST `stack` object; `stack` in
  `pull_request` webhook payloads; private preview availability.

## What the project already owns

| Knowledge | Owner |
|---|---|
| Eligibility, classification, and notification rules | `watch-pull-requests` use case and `pr-activity-notification` capability |
| Watch scope display and watched pull request status | `inspect-watch-status` use case and `watch-status` capability |
| Inbox read model, read state, terminal UI | `review-activity-inbox` use case and `activity-inbox` capability |
| Watch scope term and configuration boundary | `docs/glossary.md`, `docs/domain-model.md` |
| Read-only, local-only posture, no hosted infrastructure | `docs/arc42/01`, `docs/arc42/03` |
| Multi-account developer as review lens | `givn/personas/multi-account-developer.md` |

## Gaps

1. No configuration mutation path. Configuration is loaded once; the TUI
   explicitly excluded editing configuration or state.
2. No repository candidate inventory. The inbox lists watched pull requests, not
   repositories eligible for selection.
3. No stack term, structure, or persisted fields. The observation reaches
   `baseRefName` but stores neither base nor head ref, and never asks GitHub for
   stack membership.
4. No rule for deselecting a repository: keep its history, hide it, or prune it.
5. Stack membership is preview-gated and repo-scoped; the fallback and the
   multi-account duplicate rule are undefined.
