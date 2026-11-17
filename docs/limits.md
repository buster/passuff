# passuff limits

The recorded limits a rebuild must keep. `givn/specification.yaml` declares this
file under `limits`; `crates/passuff-e2e/features/boundary.feature` enforces
the parts a scenario can observe.

## Terminal budgets

The product is qualified at three sizes. The minimum is a product rule; the two
budgets are the sizes the frozen suite drives.

| Size | Kind | What the frame must show |
|---|---|---|
| 40x12 | minimum | At exactly 40x12, the two panes and the footer render. Below it, the frame shows `Terminal too small. Resize to at least 40x12.` and the process still exits 0 on quit. |
| 80x24 | narrow budget | Header (3 rows), watched pane (9 rows), activity pane (9 rows), footer (3 rows). |
| 120x40 | wide budget | Header (3 rows), watched pane (17 rows), activity pane (17 rows), footer (3 rows). |

### What fits at 80x24

- The header names the accounts, the watch scope, and the unread count with the
  sort order.
- The watched pane shows its column header and up to six rows. One row is one
  pull request: position marker, key, stack position, state, CI result, involved
  people, and title. The title clips at the pane edge: `Add retry loop` renders
  as `Add`, and `Cover the pagination window` renders as `Cover the pagina`.
- The activity pane shows the count line and up to six rows. One row is one
  activity record: id, time, pull request key, and display text.
- The footer shows the focus, the key hints, and the status line.

### What fits at 120x40

- The same four regions with eight more rows in each pane: the watched pane
  shows up to fourteen rows and the activity pane up to fourteen records.
- Titles that clip at 80x24 render in full.
- The activity and pull request detail overlays are centered, at most 100
  columns by 30 rows, at least 20 columns by 6 rows, and scroll inside that
  frame. The screen behind the overlay is not required to stay visible.

## Clipping and paging

| Rule | Behavior |
|---|---|
| One row, one line | A watched row and an activity row never wrap. Long text clips at the pane edge. |
| Pane clip | A pane renders at most the rows its inner height allows; the rest is not drawn. |
| Activity window | The activity pane keeps the selected record visible: the first visible record is `selection + 1 - visible_rows`, clamped at zero. |
| Watched window | The watched pane keeps the selected pull request visible by the same rule. |
| Overlay scroll | A detail overlay scrolls its lines; it never grows past the clamped height. |

## Product exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | One or more accounts failed |
| 2 | Configuration, usage, or argument error |
| 3 | State failure |

| Surface | Codes |
|---|---|
| `passuff --help`, `--version` | 0 |
| `passuff check`, successful pass | 0 |
| `passuff check`, one or more accounts failed | 1 |
| `passuff check`, invalid configuration | 2 |
| `passuff check`, state unreadable or unwritable | 3 |
| `passuff status`, `passuff inbox` | 0, or 2 for configuration and 3 for state |
| `passuff read <id>`, unknown id | 2, with `passuff: no activity with id <id>` |
| `passuff read`, invalid argument combination | 2, with `passuff: read requires exactly one of <id> or --all` |
| `passuff setup` | 2 for validation or cancellation, 3 for an I/O failure |
| `passuff notify-test` | 1 when an enabled surface fails, 2 when both are disabled |
| `passuff-tui` | 0 on quit, 3 without a terminal or on a state load failure |
| `passuff` watch loop | 0 when SIGINT arrives between passes |

## Runner result file

`givn` runs a scope with `GIVN_RESULT_FILE` and `GIVN_RESULT_SCOPE` set. The
command's exit code and the file are both part of the result: exit 0 without a
valid file, or a valid file with a non-zero exit, is not a passing scope.

### File contract

The runner removes any previous file, then writes JSON with four counts:

```json
{"total":29,"passed":29,"failed":0,"skipped":0}
```

| Field | Rule |
|---|---|
| `total` | Greater than zero. Zero scenarios is a failure. |
| `passed` | `passed + failed + skipped` equals `total`. |
| `failed` | Zero. |
| `skipped` | Zero. |
| `scope` | Optional. When present it must equal `change` or `e2e`, matching the invocation. |

A missing file, malformed JSON, a zero total, a non-zero failed or skipped
count, or counts that do not reconcile fails the scope.

### Scopes

| Scope | Command | Behavior |
|---|---|---|
| `change` | `./scripts/verify.sh` | Honours `GIVN_FEATURES`; runs the component corpus in process with fakes. |
| `e2e` | `./scripts/verify.sh --e2e` | Ignores `GIVN_FEATURES`; builds the product and runs the frozen `crates/passuff-e2e` package against the binaries. |

The frozen runner exits 2 when `PASSUFF_BIN_DIR` does not hold both binaries,
and 1 when a scenario fails, a step is undefined or skipped, a parse error
occurs, or no scenario ran. It writes the four counts without `scope`.

## What this document does not limit

Platform, dependency, and legal constraints stay in
[the architecture constraints](arc42/02-architecture-constraints.md), which
`givn/specification.yaml` also declares. Product behavior stays in
`givn/specs/`.

## Sources

| Fact | Source |
|---|---|
| Commands, exit codes, keymap | `docs/reference.md` |
| Frame composition at each size | `crates/passuff-cli/src/tui/mod.rs`, `crates/passuff-cli/src/tui/render.rs` |
| Asserted strings and sizes | `crates/passuff-e2e/features/boundary.feature` |
| Runner counts and exit code | `crates/passuff-e2e/tests/e2e_runner.rs` |
| Scope selection | `scripts/verify.sh`, `givn/commands.yaml` |
| Result-file acceptance | `givn` 0.9.20, `src/proof.rs` (`RunnerResult::read`) |
