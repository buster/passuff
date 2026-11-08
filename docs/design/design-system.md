# Design System

This is the durable index for the terminal interface's style contract. The
normative palette and screen rules live in T25 of the `review-activity-inbox`
use case (`givn/specs/review-activity-inbox/usecase.md`); that use case is the
single source of truth. This file records token names and primitives and does
not copy the palette values.

## Tokens

- Borders: dim.
- Labels, key hints, table headers: cyan-blue; key hints bold.
- Selected rows and attention text: gold.
- Focus borders and Markdown headings: green.
- Metadata values: muted.
- Description and activity body: bright gray.
- Stack, people, and ignored markers: `☰`, `☺`, `⊘`.

## Primitives

- Pane: a bordered region with a title and a focus state.
- Table row: key, state, CI result, stack position, people, and selection
  marker.
- Overlay: a centered bordered box with metadata, a rule, and a scrollable
  body.
- Footer: key hints for the focused pane.
- Status line: a transient failure or save message.

## Accessibility and capture

Keyboard-only operation; text plus symbols, never color alone. `--snapshot`
renders the same frame the curses loop paints. E2E transcripts are recorded
under `givn/changes/<id>/evidence/visual/<scenario-slug>/transcript.txt`.
