# Proposal: restore-tui-styling

## Release note

Passuff's terminal inbox renders the original watn styling: the palette,
titled bordered frame, watched and activity tables, controls bar, modal
overlays, and the save-on-quit prompt.

## Use-Case Context

- Use-case ID: `review-activity-inbox`
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` (confirmed 2026-11-02)

## Review Context

- Review kind: independent
- Reason: the change makes a user-facing visual contract across four
  capabilities executable and rewrites the terminal presentation; the visual
  evidence needs an independent review.

## Problem / Opportunity

The rebuilt terminal inbox is functionally complete but visually flat. It
prints a monochrome, vertical list separated by a 60-character rule. The
original passuff main branch and its recorded `passuff.gif` show a different
composition: an outer titled frame, an account and watch-scope header, a
watched-pull-request table, an activity table, a controls bar, and modal
overlays, all in watn's indexed palette.

The durable spec names the palette roles in rule T25 (terminal-interface rule
25 of `review-activity-inbox`) and the primitives in
`docs/design/design-system.md`, but it does not record the indexed values, and
no scenario or transcript can distinguish the intended palette and layout from
the monochrome list. The Ise report records the gap as finding A14 and fix item
B16. A user opening the inbox sees the right words and the wrong interface.

The operator approved the original main-branch composition and indexed palette
as the target on 2026-11-08, and chose to include the save-on-quit prompt in
this change.

## Proposed Solution

The terminal inbox presents the original composition and styling:

- An outer frame titled `passuff — Inbox`; an account and watch-scope header;
  an unread-count and focus line; a watched pull-request table with the headers
  `Pull request`, `State`, `CI`, `Title`, and the `☰`/`☺` columns when a stack or
  people are present; an activity table with the headers `When`, `Event`,
  `Stack`, `Read`; a controls bar with the current pane's keys; and modal
  overlays for activity detail, pull-request detail, help, settings, the save
  prompt, and the watch-scope picker.
- The palette roles are the original ones: dim rules, borders, and unfocused
  titles; cyan-blue labels, key hints, and table headers; bold key hints; gold
  selected rows and attention text; green focused pane borders and Markdown
  headings; muted metadata; bright-gray description and activity body text.
  The durable design system records the exact indexed values.
- Focus is visible: the focused pane's border and title are green, the
  unfocused pane is dim, the selected row carries a gold marker and gold text,
  and an ignored row is dimmed and prefixed `⊘`.
- On quit, the inbox asks `Save sort and filter settings?` with `y`, `n`, and
  `Esc`; `y` persists the sort, direction, filters, ignored view, and
  notification settings; `n` discards the current changes; `Esc` cancels the
  quit. This restores terminal-interface rule T22 (the settings-persistence
  rule) while keeping every other key unchanged.
- Apart from the save-on-quit prompt, the keyboard behavior, data, CLI
  transcript, notifications, sound, and browser behavior do not change.

The snapshot and visual evidence preserve color and attributes, so a scenario
and the visual review can distinguish the intended palette and layout from a
monochrome or unstyled frame.

## Capability Routing

`givn spec route` reported no signal. The decisions and rationale are recorded
directly.

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| `activity-inbox` | no signal | `EXTEND activity-inbox` | The main inbox screen, activity detail, and pull-request open affordance are this capability's surface; terminal-interface rule 25 is its rule. |
| `inbox-navigation` | no signal | `EXTEND inbox-navigation` | Pane focus, filters, help and settings overlays, the save prompt, and the controls bar are navigation behavior. |
| `stacked-inbox` | no signal | `EXTEND stacked-inbox` | The `☰` stack column and stack positions are this capability's presentation. |
| `watch-scope` | no signal | `EXTEND watch-scope` | The scope picker overlay is this capability's presentation. |

## Out of Scope

- New keys or behavior beyond the save-on-quit prompt; the data model, CLI
  text output, notifications, sound, browser, setup flow, and network behavior.
- New screens or capabilities beyond the save prompt; the existing screens and
  overlays are styled, and the save prompt restores the original quit contract.
- Live fetching, countdown, and API-budget indicators; the counts line keeps the
  unread count and focus for this change.
- Font selection, terminal-emulator configuration, and the macOS bundle.
- Replacing the watn palette or adding a theme system.

## Open Questions

### D1 — How is the visual contract made executable?

- Question: Should the terminal snapshot expose color and attributes as an
  ANSI transcript, a structured frame record, or both, and which form should
  the E2E and visual review assert?
- Options:
  1. **ANSI transcript** — the existing snapshot prints escape sequences with
     the text; gain: the captured artifact is what the terminal displays;
     cost: assertions must parse escape sequences; in practice: the snapshot
     text contains `\x1b[38;5;214m` around a selected row.
  2. **Structured frame record** — the snapshot emits text plus role names per
     span; gain: scenarios assert roles without terminal parsing; cost: the
     screen geometry is a second representation; in practice: a JSON line says
     `{"text": "read", "role": "muted"}`.
  3. **Both** — ANSI for the human-viewable artifact and the structured record
     for assertions; gain: one artifact serves both consumers; cost: two
     outputs to keep consistent; in practice: the E2E scenario reads the roles
     while the committed transcript preserves the escape sequences.
- Disposition: carried — `design.md` decides the evidence mechanism; the
  proposal requires only that the snapshot can prove the palette and layout
  roles.
