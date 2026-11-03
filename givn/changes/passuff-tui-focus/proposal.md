# Proposal: passuff-tui-focus

## Release note

Move focus and selection through the watched pull requests and activity, with the inbox refreshing automatically.

## Use-Case Context

- Use-case ID: `review-activity-inbox` (existing TUI goal)
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer`

## Review Context

- Review kind: self-review
- Reason: the operator directed a single in-context review for this terminal-navigation change; the review records the focus, scrolling, and refresh checks.

## Problem / Opportunity

The terminal inbox only selects activity records. The watched pull request
table is display-only: it has no focus, no selection, and no scrolling, so a
25-member stack is clipped and unreachable. `j`/`k` and the arrow keys are the
same activity-only handler; when the activity list is empty they do nothing,
while the footer still says `j/k move`. There is also no focus concept, so the
operator cannot tell which pane the keys affect. Finally, the inbox is a
snapshot loaded at startup: a PR created or a check completed after launch
never appears until the TUI is restarted.

## Proposed Solution

- `Tab`/`Shift-Tab` cycles focus between the watched pull request pane and the
  activity pane.
- `j`/`k` and the arrow keys move the selected row in the focused pane. The
  watched table scrolls to keep the selection visible, including all members of
  a large stack. The activity list keeps its clamp.
- The selected watched pull request and the selected activity are visible; the
  footer names the focused pane and the keys that apply.
- The inbox refreshes automatically at a fixed interval, reloading the same
  read model the CLI uses. Focus and selection are preserved by identity where
  possible and clamped when records disappear.
- A scripted `--keys` surface renders the result of focus/selection keys for
  the E2E proof; the interactive loop uses the same key handler.
- No persisted format, configuration, or notification change.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| inbox-navigation | no signal (tie) | `NEW in review-activity-inbox` | Focus, selection, scrolling, and automatic refresh are interactions of the existing inbox goal, not a new use case. |

## Out of Scope

- A PR detail screen, actions on a PR, filtering, sorting, or mouse input.
- Changing stack grouping or the scope picker.
- Notification or state-format changes.
- A configurable refresh interval; the design fixes a small default.

## Open Questions

### D1 — What refresh interval and where is it configured?

- Question: should automatic refresh use a fixed interval or a configuration
  value, and what is the default?
- Options:
  1. **Fixed one second** — no configuration change. Gain: simple; cost: a
     constant in code.
  2. **Config value** — a TUI refresh interval in TOML. Gain: tunable; cost: a
     config schema change.
- Disposition: carried — `design.md` decides; owner: design.

### D2 — How is selection preserved across refresh?

- Question: should the selected row be preserved by index, by PR key/activity
  id, or reset on every refresh?
- Options:
  1. **By identity, clamped** — preserve the selected PR key/activity id; if it
     disappeared, clamp. Gain: stable focus. Cost: a little lookup logic.
  2. **By index** — keep the index. Gain: simpler; cost: a deleted row silently
     changes the selection.
- Disposition: carried — `design.md` decides; owner: design.

### D3 — Does focus include stack summary rows?

- Question: can the watched-PR selection land on stack summary lines, or only on
  pull request rows?
- Options:
  1. **PR rows only** — summaries are headers. Gain: every selectable row has an
     action. Cost: none.
  2. **Summary rows selectable** — future grouping actions. Gain: extensible;
     cost: an extra selection state.
- Disposition: answered — PR rows only; summaries are headers.
