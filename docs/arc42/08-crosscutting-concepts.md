# 8. Cross-cutting Concepts

## Configuration and credentials

Configuration is TOML, validated with `deny_unknown_fields`. Each account
declares exactly one credential source: an environment variable name or a
credential command. Tokens are resolved at runtime and are never written to
config or state, never logged, and never included in an error message. Plain
HTTP GitHub addresses are accepted only for loopback test addresses.

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
terminal dependency. `passuff-core` is UI-agnostic. It exposes
`PassuffCore::open`, `with_gateway`, and `with_ports`, and `refresh`, and
returns a `RefreshReport` containing notifications, account failures, and
baselines recorded. It has no `clap`, `notify-rust`, `ratatui`, `crossterm`,
terminal formatting, or delivery dependency. `passuff-cli` is the first adapter;
it owns the transcript, desktop delivery, sound, signals, and exit codes. A
future `passuff-tui` consumes `passuff-config` and the same core contract. The
regular Gherkin scenarios call `CliRuntime::run_check` with injected ports; the
E2E scenarios spawn the CLI binary.

## Error handling and isolation

Each account is an isolation boundary. Authentication failures, rate limits,
timeouts, malformed responses, and transport errors are mapped to an account
failure, reported with the account label, and do not stop other accounts.
Configuration errors are returned by `passuff-config` and mapped by the CLI to
status 2. A core state read/write failure in `check` maps to status 3. `check`
exits 1 when the report contains an account failure but other work completed.
Continuous mode reports failures and retries on the next cycle.

## Notification delivery

One notification is produced per pull request per cycle. The core returns a
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

## Security

The watcher performs read-only GraphQL requests with the minimum scopes needed
for the configured accounts. State and config files are local; the state file
is created with private Unix permissions. There is no inbound network surface,
no telemetry, and no third-party notification service. Tokens are redacted from
all logs and errors.

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
