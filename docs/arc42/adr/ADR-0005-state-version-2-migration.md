---
status: accepted
date: 2026-11-02
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Migrate the persisted watch state to version 2 with an additive rewrite

## Context and Problem Statement

The activity inbox adds a bounded history and read state to the watch state,
which previously carried only account and pull-request observations. An existing
version-1 state file must keep its watched pull requests and seen IDs; the
operator must not lose the no-replay guarantees when upgrading.

## Decision Drivers

- Existing state must load without manual deletion or a replay of activity.
- The format must remain a single inspectable JSON file.
- Future schema changes need an explicit version and quarantine rule.
- A failed migration must be visible, not silently treated as an empty state.

## Considered Options

- Keep version 1 and add defaulted fields without a version bump.
- Bump to version 2 and rewrite a parsed version-1 state on first load.
- Move the history to a second file with its own schema.

## Decision Outcome

Chosen option: "Bump to version 2 and rewrite a parsed version-1 state on first
load", because it preserves the existing observations, gives the new fields an
explicit compatibility boundary, and keeps one atomic state file.

### Consequences

- Good, because a version-1 file loads with its watched pull requests, seen IDs,
  and terminal-state normalization intact, and gains an empty history.
- Good, because unknown versions still quarantine instead of being read as a
  partial state.
- Bad, because a read command can write the state during migration; a read-only
  state filesystem becomes a state error.
- Bad, because `next_activity_id` must be recomputed from existing records to
  avoid collisions; the load path owns that repair.

### Confirmation

State unit tests cover a version-1 file loading with empty history, the version-2
rewrite, preservation of a watched pull request, and unknown-version quarantine.
The Gherkin scenario `A version-1 state loads with empty history` asserts the
same through the CLI.

## Pros and Cons of the Options

### Keep version 1 with defaulted fields

- Good, because no rewrite is needed.
- Bad, because readers cannot distinguish an old file from a new one and future
  migrations lose an explicit boundary.

### Version 2 with an additive rewrite

- Good, because compatibility is explicit and the existing file is preserved.
- Bad, because load performs a write and can fail on a read-only filesystem.

### A second history file

- Good, because the watch state stays small.
- Bad, because two files must be written atomically together, and a crash can
  split history from observations.

## More Information

- Design: `givn/changes/passuff-activity-inbox/design.md`.
- Rules R5-R7: `givn/specs/review-activity-inbox/usecase.md`.
