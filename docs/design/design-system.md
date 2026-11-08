# Design System

This is the durable visual contract for the terminal interface. Rule T25 of the
`review-activity-inbox` use case points here; this file records the exact
palette values and composition so the visual contract does not depend on the
original Rust source or its animated recording.

## Palette

The palette is watn's indexed palette. The `Role` names are the contract; the
indexed value is the rendering.

| Role | Indexed color | Attributes | Use |
|---|---|---|---|
| `accent` | 81 (cyan-blue) | normal | labels, the watch-scope login, the controls title |
| `accent_bold` | 81 | bold | key hints, table headers |
| `bright` | 15 / terminal white | normal | primary text, account labels, stack positions, the total count |
| `bright_bold` | 15 / terminal white | bold | the `passuff` brand and the frame or picker title brand |
| `body` | 252 (bright gray) | normal | description and activity body text |
| `gold` | 214 (amber) | normal | attention text, status |
| `gold_bold` | 214 | bold | selected row text, the `▶` marker, and the unread count |
| `green` | 114 (green) | normal | enabled settings values, the focused pane border |
| `green_bold` | 114 | bold | the focused pane title, Markdown headings |
| `rule` | 240 (gray) | normal | overlay separators |
| `dim` | default foreground | dim | borders, unfocused titles, metadata values, secondary labels, prompt labels |

The default terminal background is used; no foreground color is set on text
that is not in the table.

When a terminal reports fewer than 256 colors, the curses fallback maps
`accent` 81 to cyan, `bright` 15 to white, `body` 252 to white, `gold` 214 to
yellow, `green` 114 to green, and `rule` 240 to the default foreground; `dim`
and bold are unchanged. The role record and ANSI snapshot are independent of
the terminal.

## Composition

The main screen is one titled outer frame and four stacked regions:

1. **Outer frame** — dim border, title `passuff — Inbox` (`passuff` bright
   bold, `— Inbox` dim).
2. **Header line** — account labels bright, logins accent, separators dim;
   `watch scope: ` dim and its value bright.
3. **Counts and focus line** — unread count gold bold, ` unread of ` dim, total
   bright; an optional filtered note dim; `Focus: ` dim and the focused pane
   name accent bold. Live fetching, countdown, and API-budget indicators are
   not part of this change.
4. **Watched table** — bordered block. The title is
   `Watched pull requests · <all|filtered> · sort: <field> <arrow>`; the focused
   border and title are green, the unfocused border is dim and the title dim.
   Table headers are `Pull request`, `State`, `CI`, `☰` when a stack is
   visible, `☺`, and `Title`; headers are accent bold and centered. Rows show
   the key, state, CI, stack position/size, people, and title. The selected row
   is gold bold with the `▶` marker; ignored rows are dim and their title is
   prefixed `⊘`.
5. **Activity table** — bordered block. The title is `Activity` plus the active
   filter or unread state. Headers `When`, `Event`, `Stack`, `Read` are accent
   bold and centered. The selected row is gold bold with the `▶` marker; the
   event excerpt and the body excerpt, when present, are body text.
6. **Controls bar** — bordered block with the `Controls` title in accent.
   Keys are accent bold, labels dim, separators ` · ` dim; a status message is
   gold.

Modal overlays use a dim bordered block and a clear rectangle behind them.
Activity detail, pull request detail, settings, and help titles are accent bold;
the save prompt title is accent (normal):

- **Activity detail** — title `Activity <id>`, accent labels, dim values,
  rule color separators, bright-gray body, and a hint whose keys are accent
  bold and labels dim.
- **Pull request detail** — same frame; Markdown headings are green bold,
  metadata labels accent, values dim, body bright gray.
- **Help** — accent-bold pane heading and key/label rows.
- **Settings** — `Settings` accent-bold title; the selected row marker and label
  gold bold, enabled values green, disabled values dim; hint keys accent bold.
- **Watch-scope picker** — title `passuff — Watch scope` (`passuff` bright
  bold, the rest dim); `Mode: ` dim and the mode accent bold; `[x]`/`[ ]` and
  focus marks green/dim; the selected repository is gold; footer keys accent bold.
- **Save prompt** — accent (normal) title `Leave`, bright question, accent-bold keys,
  dim labels; the current sort, direction, filters, ignored view, and
  notification settings persist only on `y`, `n` discards, and `Esc` cancels
  the quit.

## Viewports

- Minimum 80x24.
- Recommended 120x40.
- `--snapshot` renders 100x30 by default and honors `COLUMNS` and `LINES`.

## Accessibility

- Keyboard-only operation for every action.
- Never color alone: every role is paired with text, a symbol (`▶`, `☰`, `☺`,
  `⊘`), or position.
- Focus is visible through the green border and title and the `Focus:` label.
- The palette is the original watn palette; the terminal's own contrast
  handling applies.

## Capture and evidence

- `--snapshot` prints the plain frame; `--snapshot --ansi` prints the same
  frame with the palette and attributes; `--snapshot --roles` prints a
  structured per-line span record with role names.
- Every `@e2e` scenario commits `transcript.txt` (plain),
  `transcript.ansi` (styled), and `roles.json` (the role record) under
  `givn/changes/<id>/evidence/visual/<scenario-slug>/`.
- The visual review reads the styled artifact and the role record, not only
  the plain transcript.
- Reference views from the original recording are under
  `givn/changes/restore-tui-styling/evidence/reference/`:
  `main-inbox.png`, `activity-selection.png`, and `settings-overlay.png`.

## Motion

No motion is part of this change. The original fetching spinner is out of
scope; the counts line is static.
