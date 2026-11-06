# passuff

<!-- givn:begin:coverage-badge -->
[![Line Coverage: 88%](https://img.shields.io/badge/line%20coverage-88%25-brightgreen)](coverage/cobertura-coverage.xml)
[![Branch Coverage: 47%](https://img.shields.io/badge/branch%20coverage-47%25-brightgreen)](coverage/cobertura-coverage.xml)
<!-- givn:end:coverage-badge -->

[![crates.io](https://img.shields.io/crates/v/passuff?logo=rust)](https://crates.io/crates/passuff)

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

```sh
cargo install passuff
```

Installs `passuff` and `passuff-tui`. Releases are source-only: `passuff-config`,
`passuff-core`, and `passuff` are published to crates.io in dependency order; the
GitHub release carries the changelog.

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
| Rust stable (`rust-toolchain.toml`) | Build and install |
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
| `q` `Esc` | Quit, or close the open overlay |

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
cargo run --release -p passuff -- --help
```

<!-- givn:end:run -->

## Development

```sh
./scripts/verify.sh          # formatting, clippy, library tests, regular Gherkin suite
./scripts/verify.sh --e2e    # @e2e smoke suite against a fake GitHub and D-Bus
./measure-coverage.sh        # non-E2E and E2E Cobertura reports
./merge-coverages.sh         # merged report
```

Coverage requires `cargo-llvm-cov` and the `llvm-tools` component. The Gherkin
runner is `crates/passuff-cli/tests/features_runner.rs`; feature files live under
`givn/specs/`, and step definitions are split per capability under
`crates/passuff-cli/tests/steps/`.

Project workflow: `givn status` shows the next step; keep the `.feature` file as
the executable source of behaviour; run both verify scopes before review; create
one commit per coherent behaviour batch with `givn commit`.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

<!-- givn:begin:coverage -->
## Coverage

Merged report: [coverage/cobertura-coverage.xml](coverage/cobertura-coverage.xml)

Line coverage: 88% (6584/7443)
Branch coverage: 47% (1118/2362)

Reproduce the result:

```sh
./measure-coverage.sh
./merge-coverages.sh
```
<!-- givn:end:coverage -->
