# 4. Solution Strategy

## Key decisions

- Poll the pull requests authored by each configured account through one
  GraphQL query per account per cycle; a rejected stack-field query retries
  once without the stack fields. This removes the need for a public endpoint
  and makes the "only my pull requests" filter deterministic. See
  [ADR-0001](adr/ADR-0001-poll-authored-pull-requests.md).
- Keep classification pure and separate from I/O. The classifier receives the
  previous observation and the current snapshot and returns activity events;
  it has no network, clock, or file dependency.
- Persist observations in a versioned JSON file, written atomically before the
  core returns its report. This buys the no-replay guarantee and accepts
  at-most-once delivery. See [chapter 05](05-building-block-view.md).
- Deliver through the local desktop notification surface and a bundled WAV
  played by a system player. One notification is delivered per check pass: a
  single pull request keeps its events together, several become one digest with
  the total count, and the sound plays once. The bundled default is three short
  beeps within one second. No hosted push channel, audio library, or
  third-party account is involved. See
  [ADR-0002](adr/ADR-0002-local-desktop-notifications.md).
- Record GitHub-reported stack membership read-only with the observation and
  project stack groups in the terminal inbox. The flat CLI inbox and status are
  unchanged. See [chapter 05](05-building-block-view.md).
- Keep the watch scope explicit: all repositories or selected repositories,
  where an empty selected set observes none. The terminal picker writes the
  configuration atomically, and a running watcher reloads it before each
  cycle. See [chapter 08](08-crosscutting-concepts.md).
- Provide a line-based first-run setup shared by `passuff setup` and the
  terminal UI; it validates the configuration, never stores a token, and writes
  atomically. See [chapter 06](06-runtime-view.md).
- Let the open terminal inbox act as a watcher: a cheap change probe paces
  against the account's allowance and selects the changed open pull requests,
  a check cycle observes only those targets, and an hourly (configurable)
  reconciliation pass refreshes every watched open pull request. The view
  updates without a keypress while preserving the selected pull request. See
  [ADR-0007](adr/ADR-0007-probe-before-observing.md).
- Retire terminal pull requests from observation: once a watched pull request
  is merged or closed, only the transition itself, a short-lived pull request,
  and a reopen are observed; comments and reviews after the terminal state do
  not notify and do not cost an observation.
- Let an activity open a detail overlay and its pull request: `TuiApp` returns
  an open intent, and a small opener adapter launches the platform browser;
  the key interpreter never spawns a process. See [chapter 06](06-runtime-view.md).
- Persist a bounded activity history in the watch state and expose a read model
  (`InboxService`) to the CLI and the TUI. Status, inbox, and read never
  contact GitHub. See [ADR-0005](adr/ADR-0005-state-version-2-migration.md).
- Delegate GitHub OAuth to the gh CLI and keep personal access tokens for
  headless use. passuff reads an existing session with
  `gh auth token --user <login>` and starts `gh auth login --web` only in an
  interactive watch. See [ADR-0004](adr/ADR-0004-gh-delegated-authentication.md).
- Keep the core UI-agnostic: `passuff-config` owns the shared configuration
  schema; `passuff-core` owns observation, classification, and state;
  `passuff` is the adapter package; its `passuff-tui` binary is the terminal adapter over
  the same contracts. See [ADR-0003](adr/ADR-0003-library-first-core.md).
- Use ports for GitHub and state in the core, and for notification and sound in
  the CLI. The test suite replaces the external surfaces with local digital
  twins.
- Use a custom Cucumber runner so the `.feature` files are the executable
  tests and the Givn runner contract is satisfied.

## Technology choices

| Concern | Choice | Rationale (see ADR) |
|---|---|---|
| Language and runtime | Rust stable + Tokio | Single distributable CLI, predictable resource use, strong typing at the GitHub boundary. |
| Workspace boundary | `passuff-config` library, `passuff-core` library, `passuff` binary | Shared config schema; UI and delivery dependencies stay out of the core; the TUI binary reuses config and core. |
| GitHub access | GraphQL via `reqwest` with rustls | One query returns authored pull requests, comments, reviews, and CI state. |
| Configuration | TOML via `serde` | Readable, typed, with unknown-key rejection. |
| Authentication | gh CLI session, environment, helper command, or token file | Reuses existing OAuth sessions, keeps tokens out of config, and leaves a headless path. |
| State | Versioned JSON with atomic replacement | Small local dataset; inspectable; no database dependency. |
| Notifications | `notify-rust` | Covers Linux D-Bus and macOS notifications. |
| Sound | Bundled WAV + system player | Avoids ALSA/CoreAudio build dependencies. |
| CLI | `clap` derive | Stable help, version, and error handling. |
| Tests | `cucumber` + `wiremock` + `FakeGateway` + Linux D-Bus fake + recording delivery | Real CLI in E2E; real `notify-rust` path on Linux; core tested without a process. |
| Terminal UI | `ratatui` 0.30 + `crossterm` 0.29 | Matches the watn terminal design language; a second binary in `passuff`. |
| Coverage | `cargo-llvm-cov` with separate scopes | Produces the required non-E2E and E2E Cobertura reports. |

## Approach to quality goals

| Quality attribute | Approach |
|---|---|
| Accuracy | Authored-only observation, configured login set, explicit event rules, pure classifier tests, and one delivered notification per check pass with the per-pull-request detail in the transcript. |
| No replay | Seen-ID sets persisted before the core returns; the CLI delivers afterwards; restart scenarios in the Gherkin suite. |
| Reliability | Per-account isolation, atomic state writes, corrupt-state quarantine, timeouts, and rate-limit reporting. |
| Timeliness | Default 60-second cycle; configurable interval; the first cycle starts immediately. |
| Security | Tokens resolved at runtime, never persisted or logged; HTTPS except loopback test addresses; read-only GitHub scope. |
| Portability | Ports and platform adapters for notification, sound, and paths; Linux E2E plus a recorded macOS smoke test. |
| Reusability | A UI-agnostic `passuff-core` contract; the CLI and TUI are adapters; `cargo tree -p passuff-core` excludes UI crates. |
