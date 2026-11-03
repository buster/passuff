# Use Case Definitions: inbox-scope-and-stacks

Confirmed: 2026-11-03 by the operator (Adopt). The two-goal cut and its
boundaries are the basis for elaboration; this does not authorize
implementation or change creation.

## Sized list

| Use-case ID | Level | Primary actor | Goal (one active-verb phrase) | Changes or observes | Persona |
|---|---|---|---|---|---|
| `change-watch-scope` | ! | Developer | Change which repositories passuff observes. | Changing: watch scope configuration and record visibility. | `multi-account-developer` |
| `review-pull-request-stacks` | ! | Developer | Review a stack of pull requests as one unit. | Observing: the inbox read model. | `multi-account-developer` |

Both are leaf use cases. Neither has child use cases.

## `change-watch-scope`

- **Boundary:** the global watch scope and the visibility of already recorded
  watched pull requests and activity. It does not change GitHub, does not
  create notifications, and does not edit per-account scope.
- **Value:** the developer changes observation without editing TOML or
  restarting the watcher.
- **Primary actor:** developer. **Supporting:** none.
- **Interactions:** select repository, deselect repository, add a repository by
  owner/name (a variant of select), switch the scope mode.
- **Rules carried:** Q1 Hide, Q3 global scope, Q4 catch-up silently, Q5 known
  plus manual entry.
- **Depends on:** a writable configuration path, a known repository index, the
  inbox and status read models, and the catch-up policy in the check cycle.

## `review-pull-request-stacks`

- **Boundary:** the inbox grouping, ordering, and activity presentation for
  watched pull requests that belong to a stack. It does not create, extend, or
  dissolve stacks, and it does not observe GitHub directly.
- **Value:** the developer reviews a stack as one unit instead of reconstructing
  it from unrelated rows.
- **Primary actor:** developer. **Supporting:** GitHub supplies stack metadata
  during observation; the watcher records it.
- **Interactions:** open the inbox, review a stack, mark activity read.
- **Rules carried:** Q2 metadata only; group by repository and stack number;
  order by stack position; show stack base and size; missing metadata means
  standalone.
- **Depends on:** stack metadata in the observation and state schemas, and the
  inbox read model.
- **Relationship:** extends `review-activity-inbox` when watched pull requests
  carry stack metadata.

## Not cut as separate use cases

| Candidate | Disposition | Reason |
|---|---|---|
| Catch up on reselected repository | Rule under `change-watch-scope` | It is a system policy triggered by the developer's selection, not a separate developer goal. The activity lands in the existing inbox review goal. |
| Observe stack membership | Rule under `watch-pull-requests` | Observation is the existing watcher goal; the developer does not command GitHub stack reads. |
| Choose a repository from a picker | Interaction under `change-watch-scope` | It is the screen for the scope-change goal, not a goal of its own. |
| Prune deselected history | Rejected by Q1 | The selected rule hides records and retains them. |
| Per-account scope | Rejected by Q3 | The adopted Frame excludes it. |

## Sizing checks

- **One goal, one primary actor:** each Goal is one verb phrase; the developer
  can stop after either and the system is consistent.
- **Goal-shaped:** no use-case name names a component, mechanism, or channel.
- **Level discipline:** both are `!`; no summary use case is needed.
- **Mutation containment:** `change-watch-scope` declares a change;
  `review-pull-request-stacks` declares observation. The catch-up activity
  writing belongs to the watcher's existing changing goal, referenced by a rule.
- **Extension discipline:** stack absence, unknown repository, persistence
  failure, and hidden records are extensions under these two goals, not new
  actor goals.
- **Preconditions:** the developer has configured at least one account and the
  watcher has a readable configuration and state; none of these is established
  by the main flow.

## Interaction inventory draft

| Use case | Interaction | Kind |
|---|---|---|
| `change-watch-scope` | Select a repository for observation | changing |
| `change-watch-scope` | Deselect a repository from observation | changing |
| `change-watch-scope` | Add a repository by owner/name | changing variant of select |
| `change-watch-scope` | Switch the scope mode between all and selected | changing |
| `review-pull-request-stacks` | Open the inbox | observing |
| `review-pull-request-stacks` | Review a stack and its activity | observing |
| `review-pull-request-stacks` | Mark activity read | changing, owned by the existing inbox goal |

The E2E cut will be decided at Handoff; the existing TUI e2e already drives
`passuff-tui --snapshot`.

## Elaboration review

- Rules complete: scope modes, hide, catch-up, global scope, known plus manual
  entry, format-only validation, atomic persistence, legacy configuration
  semantics, stack metadata source, grouping, omitted members, degradation.
- Examples concrete: named repositories, stack numbers, positions, sizes, and
  observable inbox outcomes.
- Contract unambiguous: every extension branches from a main-flow step and
  protects a named guarantee; every changing use case names the state it
  changes; the observing use case names the mutation it excludes.
- Relationship: `review-pull-request-stacks` extends `review-activity-inbox`
  only when watched pull requests carry stack metadata.
- Corrections made during review: the all-to-selected switch now restores a
  retained set instead of contradicting its retention rule; the duplicate
  review interaction row was removed.
- Open questions: none. Q1-Q6 are closed.

## Handoff

Confirmed 2026-11-03 by the operator (Confirm).

| Use case | Permanent leaf | Relationship |
|---|---|---|
| `change-watch-scope` | `givn/specs/change-watch-scope/usecase.md` (new) | standalone |
| `review-pull-request-stacks` | `givn/specs/review-pull-request-stacks/usecase.md` (new) | extends `review-activity-inbox` when stack metadata is present |

Seeds: `passuff-stacked-inbox` (start 1), `passuff-watch-scope` (start 2).
Published topic: `docs/ideation/inbox-scope-and-stacks.md`.
