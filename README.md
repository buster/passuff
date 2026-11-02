# passuff

<!-- givn:begin:coverage-badge -->
[![Line Coverage: 89.4%](https://img.shields.io/badge/line%20coverage-89.4%25-brightgreen)](coverage/cobertura-coverage.xml)
[![Branch Coverage: 34.5%](https://img.shields.io/badge/branch%20coverage-34.5%25-yellow)](coverage/cobertura-coverage.xml)
<!-- givn:end:coverage-badge -->

`passuff` watches the GitHub pull requests you authored across several
accounts and notifies you only when something needs attention: comments and
reviews by other people, failing or recovered CI, and merged or closed pull
requests. Pushes, your own comments and reviews, and activity on other
people's pull requests stay silent. It runs on Linux and macOS, prints a
terminal transcript, raises a desktop notification, and plays a sound.

## Quick start

```sh
git clone <repository-url> passuff
cd passuff
cargo build --release
```

Create `~/.config/passuff/config.toml`:

```toml
poll_interval_secs = 60

[notifications]
desktop = true
sound = true
sound_file = "default"
urgency = "normal"

[events]
comments = true
reviews = true
ci_failure = true
ci_recovery = true
state_changes = true
bot_comments = false

[accounts.personal]
token_env = "PASSUFF_PERSONAL_TOKEN"

[accounts.work]
token_env = "PASSUFF_WORK_TOKEN"
```

Then run `passuff` (watch continuously) or `passuff check` (one pass).
Credentials resolve from the named environment variable, or from a token
command.

<!-- givn:begin:run -->
`givn run` starts `cargo run --release -p passuff-cli -- --help`
(`product_type: cli`).
<!-- givn:end:run -->

## Prerequisites

- Rust stable (see `rust-toolchain.toml`; `cargo`, `rustfmt`, `clippy`).
- A GitHub token with `read:user` and repository read access for every
  configured account.
- Linux: a session D-Bus and `paplay`, `pw-play`, `aplay`, or
  `canberra-gtk-play` for sound.
- macOS: Notification Center; sound is played with `afplay`.
- `cargo-llvm-cov` and the `llvm-tools` component for coverage reports.

## Usage

```
passuff                     watch until interrupted
passuff check               run one observation pass
passuff --config PATH       use a specific configuration file
passuff --state PATH        use a specific state file
passuff --help              show usage
passuff --version           show the version
```

Exit codes: `0` success, `1` one or more accounts failed, `2` configuration
error, `3` state failure.

The first observation of each pull request records a baseline and does not
replay historical activity. Short-lived pull requests created and merged
between two checks still notify their terminal state.

## Development

Run the project-owned scripts configured in `givn/commands.yaml`:

```sh
./scripts/verify.sh          # regular Gherkin suite (non-@e2e)
./scripts/verify.sh --e2e    # @e2e smoke suite against a fake GitHub and D-Bus
./measure-coverage.sh        # writes the non-E2E and E2E Cobertura reports
./merge-coverages.sh         # combines them into the merged report
```

The Gherkin runner is `crates/passuff-cli/tests/features_runner.rs`; feature
files live under `givn/specs/` and per-change delta specs. Step definitions are
split per capability under `crates/passuff-cli/tests/steps/`.

## Contributing

- Work through the givn lifecycle: `givn status` shows the next step.
- Keep the `.feature` file as the executable source of behaviour; do not
  weaken scenarios to fit the implementation.
- Run `./scripts/verify.sh` and `./scripts/verify.sh --e2e` before a change is
  reviewed.
- One commit per coherent behaviour batch, created with `givn commit`.

## License

GPL-3.0-or-later. See `LICENSE`.
