---
status: accepted
date: 2026-11-03
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Represent the watch scope with an explicit all/selected mode

## Context and Problem Statement

The watch scope is an optional repository allowlist; an empty list means all
repositories. The terminal picker needs to select and deselect repositories,
including deselecting the last one. With an empty list meaning all, the last
deselection would silently broaden observation to every repository. A legacy
non-empty allowlist must keep its current meaning.

## Decision Drivers

- Deselecting a repository must never broaden observation.
- Existing configuration files must keep their meaning.
- The terminal picker must show the current mode and the selected set.
- An empty selected set is a legitimate observable state (watch nothing).

## Considered Options

- Keep an empty list meaning all and forbid the last deselection.
- Keep an empty list meaning all; the last deselection switches to all.
- Add an explicit all/selected mode; an empty selected set means none.

## Decision Outcome

Chosen option: "Add an explicit all/selected mode; an empty selected set means
none", because it is the only option where every picker action matches its
label while preserving every legacy file. A missing mode resolves to selected
for a non-empty list and all for an empty list; an explicit selected mode with
an empty list observes nothing.

### Consequences

- Good, because deselecting never broadens observation and "watch nothing" is
  representable.
- Good, because legacy files load without an explicit migration.
- Bad, because the configuration schema has a new field and the picker has a
  mode control.
- Bad, because a configuration writer must preserve the field, not merely the
  repository list.

### Confirmation

Configuration unit tests cover the three legacy shapes. The Gherkin scenarios
`The last deselection observes nothing`, `Developer switches the watch scope
mode`, and `Switching to selected with no retained set observes nothing` assert
the observable outcomes.

## Pros and Cons of the Options

### Empty list means all; forbid the last deselection

- Good, because no schema change is needed.
- Bad, because "watch nothing" is not representable and the disabled toggle has
  no visible reason.

### Empty list means all; last deselection switches to all

- Good, because current semantics survive.
- Bad, because the last action does the opposite of its label.

### Explicit all/selected mode

- Good, because every picker action is truthful and legacy files keep their
  meaning.
- Bad, because the schema gains a mode and the writer must preserve it.

## More Information

- Design: `givn/changes/passuff-watch-scope/design.md`.
- Rules R1, R7, R10: `givn/specs/change-watch-scope/usecase.md`.
