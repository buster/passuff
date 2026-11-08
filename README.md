# passuff

<!-- givn:begin:coverage-badge -->
[![Line Coverage: 88%](https://img.shields.io/badge/line%20coverage-88%25-brightgreen)](coverage/cobertura-coverage.xml)
[![Branch Coverage: 81%](https://img.shields.io/badge/branch%20coverage-81%25-brightgreen)](coverage/cobertura-coverage.xml)
<!-- givn:end:coverage-badge -->

```text
╭────────────╮
│ Pass uff!  │
╰────┬───────╯
  (☞ﾟヮﾟ)☞
```

Multi-account GitHub pull request inbox and desktop notifier for your terminal.

Instead of checking every account and pull request tab, passuff watches the pull
requests you authored, are assigned to, or must review, and notifies you only
when someone else acts: comments, reviews, CI failures and recoveries, and
merged or closed pull requests. It runs on Linux and macOS, prints a transcript,
raises one desktop notification per check pass, and plays a sound.

![passuff inbox demo](passuff.gif)

## Install

passuff requires Python 3.11 or newer. Install it from a checkout with
`pip install .`; the package provides the `passuff` and `passuff-tui` entry
points. The GitHub release carries the changelog.

## Quick start

```sh
passuff setup
passuff check
```

`setup` asks for an account and watch scope and writes
`~/.config/passuff/config.toml`. `check` runs one observation pass; `passuff`
watches continuously.

> **Example:** after a review request and a failed CI run, `check` prints the
> activity and sends one digest. The demo above is the same inbox in
> `passuff-tui`.

Open the full-screen inbox:

```sh
passuff-tui
```

## Prerequisites

| Requirement | Needed for |
|---|---|
| Python 3.11 or newer | Run and develop |
| GitHub token with `read:user` and repository read access | `github_pat` accounts |
| `gh` 2.40 or newer | `gh_oauth` accounts |
| D-Bus and `paplay`, `pw-play`, `aplay`, or `canberra-gtk-play` | Sound on Linux |
| Notification Center and `afplay` | Notifications and sound on macOS |

## Commands

| Command | Behavior |
|---|---|
| `passuff` | Watch until interrupted |
| `passuff check` | Run one observation pass and print notifications |
| `passuff status` | Accounts, watch scope, watched pull requests, unread count |
| `passuff inbox` | List unread activity; `--all` includes read activity |
| `passuff read <id>` | Mark one activity record read; `--all` marks every record |
| `passuff setup` | Create the configuration interactively |
| `passuff notify-test` | Send a test notification and play the configured sound |
| `passuff-tui` | Open the full-screen inbox |

Flags, exit codes, and the full TUI keymap: [docs/reference.md](docs/reference.md).

## Terminal inbox

`passuff-tui` observes the configured accounts itself: it asks GitHub for a
cheap change signal, observes only the open pull requests that changed, and
reconciles every watched open pull request at `reconciliation_interval_secs`
(default one hour). Running `passuff` at the same time is redundant: both
observe and write the same state.

| Key | Action |
|---|---|
| `Tab` | Switch focus between watched pull requests and activity |
| `↑` `↓`, `j` `k`, `PgUp` `PgDn`, `Home` `End` | Move in the focused pane |
| `Enter` | Open the selected pull request or activity detail |
| `x` | Ignore or unignore the selected pull request (watched pane) |
| `r` `a` | Mark the selected or visible activity read (activity pane) |
| `h` `F1` | Show the full keymap for the focused pane |
| `s` `S` | Sort the watched list; reverse the direction |
| `f` `F` | Filter activity to the selected pull request or stack |
| `u` | Show unread activity only |
| `c` `C` | Clear filters / reset the view |
| `,` | Open settings |
| `p` | Open the watch-scope picker |
| `q` `Esc` | Quit, or close the open overlay |
| `y` `n` `Esc` | In the save prompt: save, discard, or cancel the quit |

In a detail overlay, the movement keys scroll, `Enter` opens the pull request in
a browser, and `Esc` or `q` closes. Stacked pull requests that GitHub reports as
one stack are grouped with the stack base, size, and each member's position;
repositories without the stacked-PR preview show standalone rows.

## Notifications

Desktop notifications and sound are enabled by default; the bundled sound is
three short beeps within one second. One digest is delivered per check pass;
several pull requests become one notification naming the activity count.
`--no-desktop` and `--no-sound` mute one run without changing the configuration,
including `passuff-tui`. `passuff notify-test` reports each enabled surface; it
exits 1 when a surface fails and 2 when both are disabled.

All notification keys: [docs/reference.md](docs/reference.md).

## Configuration

```toml
[accounts.personal]
auth = "gh_oauth"
github_user = "octo-personal"
```

`gh_oauth` uses `gh auth token --user <github_user>`; an interactive watch runs
`gh auth login --web` once when no session exists. `check` and non-interactive
runs never open a browser.

`github_pat` accounts take exactly one of `token_env`, `token_command`, or
`token_file`. The token file must be absolute or start with `~`, and must be
readable only by its owner (`0600` or `0400`). A literal `token` is rejected.

Full schema, validation rules, and account semantics:
[docs/reference.md](docs/reference.md).

## Docs

[Reference](docs/reference.md) · [Architecture](docs/arc42/) · [Glossary](docs/glossary.md) · [Domain model](docs/domain-model.md)

<!-- givn:begin:run -->
## Run

Run the tool:

```sh
python -m passuff --help
```

<!-- givn:end:run -->

## Development

The Gherkin corpus under `givn/specs/` is the executable specification. Run
`./scripts/verify.sh` for the permanent non-E2E corpus, `./scripts/verify.sh
--e2e` for the E2E corpus, and `./scripts/measure-coverage.sh` followed by
`./scripts/merge-coverages.sh` for coverage. `./scripts/verify.sh --strictness`
proves the runner rejects undefined, skipped, and no-op steps.

Project workflow: `givn status` shows the next step; keep the `.feature` file as
the executable source of behaviour; run both verify scopes before review; create
one commit per coherent behaviour batch with `givn commit`.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

<!-- givn:begin:coverage -->
## Coverage

Merged report: [coverage/cobertura-coverage.xml](coverage/cobertura-coverage.xml)

Line coverage: 88% (2062/2349)
Branch coverage: 81% (719/886)

Reproduce the result:

```sh
./scripts/measure-coverage.sh
./scripts/merge-coverages.sh
```
<!-- givn:end:coverage -->
