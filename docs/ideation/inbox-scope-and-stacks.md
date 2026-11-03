# Ideation: inbox scope and stacks

Status: handoff confirmed 2026-11-03. Two change seeds are ready; no change has
been proposed or implemented.

## Why this topic exists

The inbox is not organized around the work the developer actually does:

- Changing which repositories are watched requires editing TOML and restarting
  the watcher.
- A stack of pull requests appears as unrelated rows, so the developer rebuilds
  the stack by hand.
- The developer also asked whether GitHub offers a more efficient streaming
  channel. It does not offer a local push stream; polling stays. Webhooks would
  add an inbound surface and are a separate future topic, not part of this one.

The topic turns those ideas into two change-ready use cases.

## Decisions

| ID | Question | Chosen | Rejected and why |
|---|---|---|---|
| Q1 | What happens to stored history when a repository is deselected? | Hide: stop observing, keep records, exclude them from the inbox and status. | Retain visible: the inbox would contradict the scope line. Prune: destroys reviewable history. |
| Q2 | Where does stack membership come from? | GitHub stack metadata only; absent means standalone. | Ref-chain fallback: incomplete and ambiguous across merges. Metadata with fallback: two sources disagree and double the test surface. |
| Q3 | Is the watch scope global or per account? | Global, one repository set for all accounts. | Per account: changes the configuration shape outside the adopted frame. |
| Q4 | On reselect, does observation resume from the old baseline or start fresh? | Catch up silently: record gap activity without a notification or sound. | Resume: delayed notification burst. Fresh baseline: history hole. |
| Q5 | Where does the repository picker get its candidates? | Known repositories plus manual owner/name entry. | Known only: cannot preselect a new repository. Full catalog: pagination, extra API calls, large list. |
| Q6 | What does an empty watch scope mean? | Explicit all/selected mode; an empty selected set observes nothing. | Forbid the last deselection: cannot express none. Switch to all: the action reverses its label. |

Full records with options, gains, costs, and examples are in
[`questions.md`](../../givn/ideation/inbox-scope-and-stacks/questions.md).

## Terms

Confirmed 2026-11-03 and promoted to the durable glossary:

| Term | Meaning |
|---|---|
| Repository | A GitHub owner/name resource that contains pull requests. "Project" is not the term. |
| Pull request stack | An ordered chain of pull requests in one repository, identified by a repository-scoped number and a total size, where each member has a 1-based position and the chain shares one ultimate base. |
| Stack position | The 1-based ordinal of a pull request within its stack, from the bottom. |
| Stack base | The branch the whole stack ultimately targets, which only the bottom pull request directly targets. |
| Known repository | A repository seen in an authored pull request or present in the watch scope. |
| Catch-up | Recording activity that happened while a repository was outside the watch scope, delivered to the inbox without a notification. |
| Watch scope | The set of repositories observed: all repositories, or selected repositories, where an empty selected set observes none. |

Rejected terms: "project" (collides with GitHub Projects boards), "stack of
commits" (members are pull requests), and "notification backlog" (catch-up is
silent).

## Domain model

The chain has two paths that share one observation cycle.

**Watch scope.** The developer selects, deselects, or adds a repository, or
switches the scope mode. The watch scope configuration emits the selection or
deselection. Deselection stops observation and hides the repository's records;
the records stay in watcher state. Selection restores the records and marks
catch-up pending. On the next observation, catch-up records gap activity as
unread records without producing a notification. The mode switch establishes
all-repositories or selected-repositories observation; an empty selected set
means none.

**Stacks.** GitHub reports authored pull requests and, where the preview is
enabled, stack metadata. The observation records stack number, position, size,
and base on the watched pull request. The inbox groups watched pull requests by
repository and stack number, orders by position, shows the base and size once,
and groups activity under the stack. An absent stack renders standalone. The
stack is an external fact recorded on the watch aggregate, not a new aggregate:
passuff issues no stack commands.

The aggregates stay `Watched Pull Request` (baseline, seen identities, CI, state,
stack membership; at most one notification per event) and `Activity Record`
(stable id, observation time, read flag; bounded history). The watch scope is
configuration, the known repository index and catch-up pending are records, and
the notification is a report. The rendered canvas is in
[`eventstorming.md`](../../givn/ideation/inbox-scope-and-stacks/eventstorming.md).

## Use cases

### Change which repositories passuff observes

`change-watch-scope`, level `!`, primary actor Developer.

- The picker shows the scope mode, known repositories with selected state, and a
  manual-entry field.
- Selecting or deselecting updates the scope atomically and persists it.
- Deselecting hides records; selecting restores them. Selecting a repository
  with retained records marks catch-up pending.
- The change applies to the next check cycle of a running watcher.
- Switching from all to selected restores the retained set or starts with all
  known repositories selected; the selected set is retained while the mode is
  all.
- Minimal guarantee: a failed write or invalid entry leaves the previous mode,
  set, and visibility. Success: the scope is persisted and the inbox and status
  show exactly the in-scope records.

Full contract:
[`use-cases/change-watch-scope.md`](../../givn/ideation/inbox-scope-and-stacks/use-cases/change-watch-scope.md).

### Review a stack of pull requests as one unit

`review-pull-request-stacks`, level `!`, primary actor Developer; extends
`review-activity-inbox` when stack metadata is present.

- Groups watched pull requests by repository and stack number, ordered by
  position.
- Shows the stack base and size once; omits members no configured account
  watched; positions may have gaps.
- Groups activity under the stack and aggregates the unread count.
- Absent or rejected stack metadata degrades to standalone rows; the account
  observation does not fail.
- Minimal guarantee: missing or malformed metadata leaves rows standalone and
  the existing inbox behavior correct. Success: each stack is one group with
  base, size, ordered members, and activity.

Full contract:
[`use-cases/review-pull-request-stacks.md`](../../givn/ideation/inbox-scope-and-stacks/use-cases/review-pull-request-stacks.md).

## Change seeds

| Seed | Use case | Start | Dependencies |
|---|---|---|---|
| [`passuff-stacked-inbox`](../../givn/ideation/inbox-scope-and-stacks/changes/passuff-stacked-inbox.md) | `review-pull-request-stacks` | 1 | None. Adds stack metadata to observation and state, and the stacked inbox projection. |
| [`passuff-watch-scope`](../../givn/ideation/inbox-scope-and-stacks/changes/passuff-watch-scope.md) | `change-watch-scope` | 2 | No functional dependency on the first; run sequentially because both touch the terminal UI and inbox read model. |

Each seed carries its accepted decisions and the open questions left to its
proposal: preview-error handling and state version for stacks; configuration
mode and migration, watcher reload, and picker mechanics for watch scope.

## Permanent use-case mapping

| Use case | Permanent document |
|---|---|
| `change-watch-scope` | `givn/specs/change-watch-scope/usecase.md` (new) |
| `review-pull-request-stacks` | `givn/specs/review-pull-request-stacks/usecase.md` (new, extends `review-activity-inbox`) |

Capabilities are routed by each proposal. No capability is claimed here.

## Next

Run `/givn-propose` for `passuff-stacked-inbox` first, then for
`passuff-watch-scope`. Handoff does not start either change.
