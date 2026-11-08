# Screen evidence

Real captures of the rebuilt terminal inbox and side-by-side comparisons with
the original passuff recording.

## Reference (original `passuff.gif`)

- `../reference/main-inbox.png` — the main inbox.
- `../reference/activity-selection.png` — the selected activity row.
- `../reference/settings-overlay.png` — the settings overlay.

## Rebuilt (Alacritty, implemented TUI)

Captured from `passuff-tui` in Alacritty 110 columns by 52 lines at 1500x1080,
DejaVu Sans Mono, with a representative two-account inbox.

- `real/main-inbox.png` — the main inbox.
- `real/main-selected.png` — the watched pane focused on the selected PR.
- `real/activity-detail.png` — the activity detail overlay.
- `real/pr-detail.png` — the pull request detail overlay.
- `real/settings-overlay.png` — the settings overlay.
- `real/save-prompt.png` — the save-on-quit prompt.
- `real/help-overlay.png` — the help overlay.
- `real/scope-picker.png` — the watch-scope picker.

## Comparisons

- `comparisons/main-inbox-comparison.png` — original main inbox left, rebuilt
  main inbox right.
- `comparisons/activity-selection-comparison.png` — original activity
  selection left, rebuilt selected-PR view right.
- `comparisons/settings-overlay-comparison.png` — original settings overlay
  left, rebuilt settings overlay right.

The original and rebuilt captures use different sample data. The comparison is
for composition, palette, focus, selection, table headers, and the overlay
frames; it is not a pixel diff.

The rebuilt main capture is also the durable design reference at
`docs/design/passuff-tui-alacritty.png`.
