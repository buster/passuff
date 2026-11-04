# 8. Cross-cutting Concepts

## Configuration and credentials

Configuration is TOML, validated with `deny_unknown_fields`. Each account
declares one credential mode: `gh_oauth` with an expected GitHub login, or
`github_pat` with exactly one of an environment variable, a helper command, or a
token file. A configuration without `auth` keeps the legacy personal-access-token
form. A literal token field is rejected. Token files must be absolute or
`~`-prefixed and must not be accessible to group or other users. Tokens are
resolved at runtime and are never written to config or state, never logged, and
never included in an error message. Plain HTTP GitHub addresses are accepted
only for loopback test addresses.

The interactive gh login is available only to an interactive continuous watch;
`check` and non-interactive runs resolve credentials without starting a
process that waits for a browser. The credential source attempts a login at most
once per gh login per process.

## Activity history and watch scope

Watch state version 2 carries up to 500 activity records: one per event, with a
stable `a<n>` id, the observation time, the pull request key, and a read flag.
The oldest records are dropped on append. Version-1 files load with their
watched pull requests and are rewritten as version 2; unknown versions are
quarantined. Read state persists and is independent of delivery. The optional
global watch scope filters observation and the inbox/status read model. Excluded
watched pull requests and activity are hidden but retained in state; reselecting
them records a silent catch-up cycle.

## Persistence and deduplication

The watcher state is a versioned JSON file containing, per account, the resolved
login, the last check time, and per pull request the baseline, seen comment,
review, and review-comment IDs, the head OID, the last CI state, and the last
known pull request state. Seen sets are bounded to the observation window.
Writes use a temporary file in the same directory, flush, and rename. A corrupt
file is quarantined and replaced with an empty baseline.

The core persists state before it returns a `RefreshReport`. This is
at-most-once delivery: a crash between the core return and CLI delivery can
lose a notification, but a restart cannot replay one. The no-replay guarantee
is the deliberate priority.

## UI adapter boundary

`passuff-config` owns the shared TOML schema and validation; it has no UI or
terminal dependency. `passuff-core` is UI-agnostic. It exposes `PassuffCore::with_ports` and
`refresh`, and returns a `RefreshReport` containing notifications, account failures, and
baselines recorded. It has no `clap`, `notify-rust`, `ratatui`, `crossterm`,
terminal formatting, or delivery dependency. `passuff` is the first adapter;
it owns the transcript, desktop delivery, sound, signals, and exit codes. A
`passuff-tui` consumes `passuff-config` and the same core contract. The
regular Gherkin scenarios call `CliRuntime::run_check` with injected ports; the
E2E scenarios spawn the CLI binary.

## Error handling and isolation

Each account is an isolation boundary. Authentication failures, identity
mismatches, rate limits, timeouts, malformed responses, and transport errors are
mapped to an account failure, reported with the account label, and do not stop other accounts.
Configuration errors are returned by `passuff-config` and mapped by the CLI to
status 2. A core state read/write failure in `check` maps to status 3. `check`
exits 1 when the report contains an account failure but other work completed.
Continuous mode reports failures and retries on the next cycle.

## Notification delivery

The core produces one notification per pull request per cycle; the CLI delivers
one notification per cycle. Several pull requests become one digest naming the
total activity count and the per-pull-request counts, and the terminal transcript
keeps the event detail. The sound plays once per delivered notification. The core returns a
`Notification` value; the CLI formats the transcript block and the desktop
title and body. The transcript is the primary observable output and is always
printed. The desktop adapter uses `notify-rust`. The sound adapter plays a
bundled WAV through the platform player and falls back to the XDG sound hint.
Delivery and playback failures are reported but do not affect core state or the
transcript.

## Logging and transcript

The transcript uses a stable `passuff:` prefix, one header per pull request, and
indented event lines. Errors go to stderr with the account label and a
remediation. `--verbose` adds diagnostic logs without changing the notification
content. No internal identifiers, enum names, or state names appear in visible
text.

## Terminal UI

The TUI is a second adapter over `InboxService` and a second watcher. It renders
the same view as the read commands, marks records through the same service, and
runs check cycles through `ObservationSession` while it is open. A blocking input
thread feeds keys; observation updates arrive over a channel and redraw the
frame. `--snapshot` uses ratatui's `TestBackend` and prints the frame for scripts
and E2E; `--observe` performs one probe and cycle first, and `--observe-for`
runs the real loop against the test backend. Interactive mode uses crossterm raw
mode and the alternate screen. No token, body, command text, or enum name is
rendered.

## Watch scope editing

The watch scope is configuration with an explicit mode: all repositories or a
selected set; an empty selected set observes none. The terminal picker edits the
scope through `ScopeEditor`, which preserves unrelated configuration with
`toml_edit` and writes atomically. Deselecting a repository stops observation
and hides its records from the inbox and status while retaining them in state;
reselecting restores them. Reselecting a repository with retained records marks
a catch-up cycle: activity is recorded as unread without a desktop notification
or sound. A running watcher reloads the configuration before each cycle; a
failed reload keeps the previous configuration.

## Inbox navigation and observation pacing

The terminal inbox has one focus state (`watched` or `activity`). Movement keys
apply to the focused pane; the watched table uses a `TableState` and scrolls to
the selected PR, while activity keeps its clamp. Observation runs independently
of input: the probe delay is `clamp(time_to_reset / max(remaining / 2, 1),
1s, time_to_reset)`, so the inbox spends at most half the remaining allowance
across the window and never schedules past the reset. A targeted or
reconciliation pass is deferred below 100 remaining points. A failed probe
backs off exponentially and keeps the view. Updates preserve the selected PR
key and never change focus. The `--keys` and `--observe` snapshot surfaces
drive the same handler for E2E.

Observation targeting is the other crosscutting rule: the probe fingerprint
selects changed open, new open, transitioned, and short-lived pull requests; a
known terminal pull request whose update time changed alone is ignored. The
live inbox reconciles every watched open id and every open
assigned/review-requested id on startup and every `reconciliation_interval_secs`
(default one hour), which bounds staleness for signals the fingerprint cannot
see. The headless watcher reconciles on its `poll_interval_secs` cadence.

## Activity detail and browser launch

The activity pane's Enter opens a modal overlay built only from persisted data:
the activity record and its pull request view. Modal key precedence runs before
the inbox keys, so Escape closes the overlay instead of quitting and no other
key moves the selection underneath it. The key interpreter returns an
`OpenUrl` intent; only the binary adapter spawns the platform opener, so widget
logic stays testable and free of process launches. A non-zero exit, a missing
opener, or a spawn error is a footer status line and the overlay stays open.

## First-run setup

Setup is line-based and shared by `passuff setup` and the first-run terminal
UI. It validates the configuration with `ConfigLoader::parse` before writing,
creates parent directories, writes a temporary file plus rename, and never
reads or stores a token: only an environment variable name, a command, or a
permission-checked token-file path is recorded. A missing terminal and a
missing configuration produce the `passuff setup` hint and exit 2.

## Stack metadata

GitHub reports stack number, size, base, and position on pull requests where
the stacked-PRs preview is enabled. The observation records the membership with
the watched pull request; incomplete or rejected metadata degrades to a
standalone row. A GraphQL error naming the stack fields retries the account
observation once without them. Grouping is a read-only terminal projection over
`InboxView`; the CLI's flat inbox is unchanged.

## Security

The watcher performs read-only GraphQL requests with the minimum scopes needed
for the configured accounts. State and config files are local; the state file
is created with private Unix permissions. There is no inbound network surface,
no telemetry, and no third-party notification service. Tokens are redacted from
all logs and errors.

### Credentials and identities

A gh OAuth token comes from the operator's gh session; passuff never sees or
stores the OAuth refresh material. A personal access token comes from an
environment variable, a helper process, or a file whose group and other bits are
clear. The resolved token's GitHub login is compared with the account's expected
login before any pull request is processed; a mismatch names both logins and
skips the account. No token, token-file content, or Authorization header is
written to the transcript, state, or evidence artifacts.

## Rate limiting

One GraphQL query per account per cycle, plus a targeted node query only when
known open pull requests fall outside the recent window. The default 60-second
interval stays far below the 5,000-points-per-hour limit per token. A rate-limit
response reports the reset time and skips only that account for the cycle.

## Testing and coverage

The Gherkin `.feature` files are the executable specification. A custom
cucumber-rs runner in `crates/passuff-cli/tests` honors the Givn
`GIVN_FEATURES`, `GIVN_RESULT_SCOPE`, and `GIVN_RESULT_FILE` contract and fails
on skipped or undefined steps. On Linux, E2E scenarios spawn the real CLI binary
against a local wiremock GitHub and a private `dbus-daemon` session with a mock
`org.freedesktop.Notifications` service, so the real `notify-rust` path is
exercised. On macOS, E2E uses `RecordingDelivery` and the manual desktop smoke
covers the real surface. A fake sound player records playback. Regular scenarios
drive `CliRuntime::run_check`, which in turn calls `PassuffCore::refresh`, with a
`FakeGateway` in process. Coverage runs the regular and E2E scopes separately,
instruments the `passuff-config` and `passuff-core` path dependencies, and
produces two distinct Cobertura reports.

## Platform paths

Linux uses XDG config and state directories. macOS uses Application Support.
The `directories` crate resolves both. The bundled sound cache uses the
platform cache directory.
