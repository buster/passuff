# 2. Architecture Constraints

## Technical constraints

| Constraint | Motivation |
|---|---|
| Rust stable toolchain, single distributable CLI binary | The project preference is Rust; distribution is a local executable. |
| UI-agnostic core library | `passuff-core` must not depend on `clap`, `notify-rust`, `ratatui`, `crossterm`, terminal formatting, or delivery; the CLI is the first adapter. |
| macOS and Linux only | The approved platform scope. Windows is explicitly out of scope. |
| No inbound network surface, server, or webhook receiver | The watcher is self-contained and must run behind NAT without a public endpoint. |
| GitHub GraphQL API over HTTPS | One typed query per account per cycle; no scraping. |
| Desktop notification through the platform surface | The notification must appear in the operating system's notification centre. |
| Sound through a system player, no audio library | Keeps the build free of ALSA or CoreAudio development dependencies. |
| Local configuration and state files | No database, no hosted state, no external account service. |
| The terminal UI may write the configuration file atomically | The watcher reloads the configuration before each cycle so a scope change applies without a restart. |
| GitHub rate limit of 5,000 GraphQL points per hour per token | Observation must remain cheap; normally one query per account per cycle. |
| GitHub stacked-PRs metadata is preview-gated and read-only | The inbox shows standalone rows where the preview is unavailable; a rejected stack query retries once without the stack fields, adding at most one retry of the account observation per account per cycle. |
| Tokens from a gh session, environment variable, helper command, or permission-checked token file | Secrets must not be stored in configuration or state; token files must be private to the operator. |
| Gherkin `.feature` files are the executable specification | The project's Givn workflow requires a Cucumber runner, not a parallel test suite. |
| Coverage gate produces separate non-E2E and E2E Cobertura reports | The project has the coverage addon enabled and archive gates on it. |

## Legal constraints

| Constraint | Motivation |
|---|---|
| Project license is GPL-3.0-or-later | Matches the sibling givn and watn projects. |
| Dependencies must be GPLv3-compatible | `notify-rust` (MIT OR Apache-2.0) and `user-notify` (LGPL-3.0-or-later) are compatible. The LGPL relinking obligation is satisfied by distributing source and build scripts under the project license. |

## Organisational constraints

| Constraint | Motivation |
|---|---|
| Single-operator project | There is no separate operations team, tenancy model, or support rotation. |
| Shared config and core for all UI adapters | Configuration lives in `passuff-config`; observation, classification, and state live in `passuff-core`; UI crates only present and deliver. |
| Spec-driven changes through Givn | Proposal, use case, delta feature, design, review, tasks, and archive are the project process. |
| Domain vocabulary is owned by `docs/glossary.md` | Specs, design, and code must use the same terms. |
| Diagrams are Mermaid only | Arc42 chapters and design documents must render without ASCII art. |

## Conventions

| Convention | Motivation |
|---|---|
| Config and state live under platform directories | Follows XDG on Linux and Application Support on macOS. |
| One grouped notification per pull request per cycle | Avoids notification bursts when several events arrive together. |
| Core persists before returning; CLI delivers | Preserves the no-replay guarantee at the cost of at-most-once delivery, while keeping the core free of delivery dependencies. |
| Account labels are local names; GitHub logins are resolved | A label can be `work` or `personal` without embedding a login in config. |
| Plain-text transcripts are the primary observable output | Makes every interaction testable without a desktop daemon. |
