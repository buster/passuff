# passuff reference

Exhaustive commands, flags, keybindings, configuration keys, exit codes, and
observation rules. First-run use lives in [README.md](../README.md).

## Commands

| Command | Behavior |
|---|---|
| `passuff` | Watch until interrupted |
| `passuff check` | Run one observation pass and print notifications |
| `passuff status` | Show accounts, watch scope, watched pull requests, and unread count |
| `passuff inbox [--all]` | List unread activity; `--all` includes read activity |
| `passuff read <id>` | Mark one activity record read |
| `passuff read --all` | Mark every activity record read |
| `passuff setup` | Create the configuration interactively |
| `passuff notify-test` | Send one test notification and play the configured sound |

### Global options

| Option | Effect |
|---|---|
| `--config <PATH>` | Use a specific configuration file |
| `--state <PATH>` | Use a specific state file |
| `--verbose` | Diagnostic logs on stderr; `RUST_LOG` overrides the filter |
| `--no-desktop` | Suppress desktop notifications for this run |
| `--no-sound` | Suppress the notification sound for this run |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | One or more accounts failed |
| 2 | Configuration, usage, or argument error |
| 3 | State failure |

`notify-test` returns 1 when an enabled surface fails and 2 when both surfaces
are disabled. `setup` returns 2 for validation or cancellation and 3 for an I/O
failure. `passuff-tui` returns 3 when the terminal UI is requested without a
terminal or when state loading fails.

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
| `passuff-tui`, quit | 0 |
| `passuff-tui`, no terminal and no `--snapshot` | 3 |
| `passuff-tui`, invalid configuration | 2 |

The acceptance package asserts: `--help` 0, a successful `check` 0, a `check`
that hits a server error or an unreachable endpoint 1, and a TUI quit 0.

## passuff-tui options

| Option | Effect |
|---|---|
| `--config <PATH>` | Use a specific configuration file |
| `--state <PATH>` | Use a specific state file |
| `--snapshot` | Print one frame and exit |
| `--observe` | Observe once before opening or printing |
| `--observe-for <MILLIS>` | Observe for the given duration before opening or printing |
| `--scope-select <OWNER/REPO>` | Add a repository and switch to selected mode |
| `--scope-deselect <OWNER/REPO>` | Remove a repository from the selected scope; errors in all mode |
| `--scope-mode <all\|selected>` | Set the watch mode before running |
| `--focus-set <RELATIONS>` | Set the focus relations before running |
| `--picker` | Start with the watch-scope picker; with `--snapshot`, print it |
| `--keys <KEYS>` | Apply a comma-separated key sequence before `--snapshot` |
| `--no-desktop` | Suppress desktop notifications for this run |
| `--no-sound` | Suppress the notification sound for this run |

`--focus-set` accepts `authored`, `assigned`, `review-requested`, and
`team-review-requested`, comma-separated. `--keys` accepts `tab`, `backtab`,
`j`, `down`, `k`, `up`, `r`, `a`, `q`, `enter`, `esc`, `left`, `right`,
`pageup`, `pagedown`, `home`, `end`, `x`, `comma`, and `space`.

## TUI keymap

| Context | Key | Action |
|---|---|---|
| Any | `Tab`, `BackTab` | Switch focus between watched pull requests and activity |
| Any | `j`, `↓`, `k`, `↑` | Move in the focused pane |
| Any | `PgDn`, `PgUp` | Move by ten rows |
| Any | `Home`, `End` | Jump to the first or last row |
| Any | `q`, `Esc` | Quit |
| Watched pane | `Enter` | Open the pull request detail |
| Watched pane | `x` | Ignore the selected pull request, or unignore an ignored row |
| Watched pane | `←`, `→` | Show or hide ignored pull requests |
| Watched pane | `s`, `S` | Cycle the sort order; reverse it |
| Watched pane | `f` | Filter activity to the selected pull request |
| Watched pane | `F` | Filter activity to the selected pull request or stack |
| Activity pane | `Enter` | Open the activity detail |
| Activity pane | `r` | Mark the selected record read |
| Activity pane | `a` | Mark visible records read; with no filter, mark all read |
| Any pane | `u` | Toggle unread-only activity |
| Any pane | `c` | Clear the activity filter and unread-only view |
| Any pane | `C` | Reset sort, filters, unread-only, and ignored visibility |
| Any pane | `,` | Open settings |
| Any pane | `p` | Open the watch-scope picker |
| Any pane | `h`, `H`, `F1` | Show keys for the focused pane |

### Overlays

| Context | Key | Action |
|---|---|---|
| Pull request detail | movement keys | Scroll |
| Pull request detail | `Enter` | Open the pull request in a browser |
| Pull request detail | `Esc`, `q` | Close |
| Activity detail | movement keys | Scroll |
| Activity detail | `Enter` | Open the pull request in a browser |
| Activity detail | `Esc`, `q` | Close |
| Help | `Esc`, `q`, `h`, `H`, `F1` | Close |
| Settings | `j`, `↓`, `k`, `↑` | Select desktop notifications or sound |
| Settings | `Space`, `Enter` | Toggle the selected setting |
| Settings | `Esc`, `,`, `q` | Close |
| Scope picker | `j`, `↓`, `k`, `↑` | Move through focus relations and repositories |
| Scope picker | `Space` | Toggle the focused relation or repository |
| Scope picker | `m` | Switch between all and selected watch modes |
| Scope picker | `Esc`, `q` | Close |
| Save prompt | `y` | Save view and notification settings to `ui.json` |
| Save prompt | `n` | Discard the changes and quit |
| Save prompt | `Esc`, `q` | Return to the inbox |

The sort modes are updated, name, state, and age. Sort, filter, unread-only,
ignored visibility, and notification toggles are saved to `ui.json` next to the
state file when the save prompt is confirmed on exit.

## Configuration

Default path: `$XDG_CONFIG_HOME/passuff/config.toml`, normally
`~/.config/passuff/config.toml` on Linux. `--config` overrides it. The default
state file is `$XDG_STATE_HOME/passuff/state.json`, normally
`~/.local/state/passuff/state.json`; `--state` overrides it.

A minimal configuration needs one account:

```toml
[accounts.personal]
auth = "gh_oauth"
github_user = "octo-personal"
```

### Top-level keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `poll_interval_secs` | integer | `60` | Seconds between watch passes; minimum 1 |
| `reconciliation_interval_secs` | integer | `3600` | Seconds between full reconciliations; minimum 1 |

### `[watch]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `mode` | `all` or `selected` | `all`, or `selected` when `repositories` is non-empty | Which repositories are watched |
| `repositories` | array of `owner/repo` | empty | Selected watch scope |
| `ignored_pull_requests` | array of `owner/repo#number` | empty | Pull requests that produce no activity or notification |

`repositories` entries must contain exactly one `/` and no whitespace;
duplicates are rejected case-insensitively. `ignored_pull_requests` entries must
use `owner/repo#number` with a positive number; duplicates are rejected
case-insensitively. `mode = "selected"` with an empty `repositories` watches
nothing.

### `[focus]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `relations` | array | all four | Relations shown, counted, and notified |

Values: `authored`, `assigned`, `review-requested`, `team-review-requested`.
Focus never changes which pull requests passuff fetches; it filters the inbox,
counts, and notifications.

### `[events]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `comments` | boolean | `true` | Comments by other people |
| `reviews` | boolean | `true` | Reviews by other people |
| `ci_failure` | boolean | `true` | CI becoming failing |
| `ci_recovery` | boolean | `true` | CI recovering |
| `state_changes` | boolean | `true` | Merged, closed, and reopened pull requests |
| `bot_comments` | boolean | `false` | Comments by bots |

### `[notifications]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `desktop` | boolean | `true` | Desktop notification |
| `sound` | boolean | `true` | Notification sound |
| `sound_file` | `"default"` or path | `"default"` | Bundled sound or an existing file |
| `urgency` | `low`, `normal`, `critical` | `normal` | Linux notification urgency |

The bundled sound is three short beeps within one second.

### `[accounts.<label>]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `auth` | `gh_oauth` or `github_pat` | `github_pat` | Authentication mode |
| `github_user` | string | absent | Expected GitHub login |
| `token_env` | string | absent | Environment variable holding a PAT |
| `token_command` | string | absent | Command that prints a PAT |
| `token_file` | string | absent | File containing a PAT |
| `graphql_url` | URL | `https://api.github.com/graphql` | GitHub GraphQL endpoint |

`gh_oauth` requires `github_user` and no token source. It reads the token from
`gh auth token --user <github_user>`; an interactive watch starts
`gh auth login --web` once when no session exists. `check` and non-interactive
runs never open a browser.

`github_pat` (or an absent `auth`) requires exactly one of `token_env`,
`token_command`, or `token_file`. A literal `token` key is rejected. A token
file must be absolute or start with `~`, must be a regular file, and must be
readable only by its owner: `0600` or `0400`. Group or other access is rejected.

When `github_user` is set, the resolved token must belong to that login,
case-insensitively. A mismatch is an account failure naming both logins.

`graphql_url` accepts `https` URLs and `http` only for `localhost` or
`127.0.0.1`.

### Validation

The configuration is rejected, with exit 2, when:

- no `[accounts.<label>]` entry exists;
- a label is empty;
- `poll_interval_secs` or `reconciliation_interval_secs` is below 1;
- a watch repository is not `owner/repo`, or is duplicated;
- an ignored pull request is not `owner/repo#number`, or is duplicated;
- an account uses an unknown auth mode;
- a PAT account does not select exactly one credential source;
- a `gh_oauth` account lacks `github_user` or sets a token source;
- `github_user` contains whitespace;
- a token file is relative, is not a regular file, or is not owner-only;
- a `sound_file` path is not a readable file;
- `graphql_url` uses plain HTTP outside loopback.

## Setup

`passuff setup` writes the configuration file. With an existing file it asks
`Configuration exists. Replace? [y/N]`; any answer other than `y` or `yes`
leaves the file unchanged and exits 0.

Prompts, in order:

1. Account label.
2. Auth: `gh_oauth`, `token_env`, `token_command`, or `token_file`; default
   `gh_oauth`.
3. Expected GitHub login; the prompt requires a value.
4. The selected credential value, variable, command, or file.
5. Add another account.
6. Watch mode: `all` or `selected`; default `all`.
7. Repositories, comma-separated, when `selected`.

The generated configuration contains the accounts, watch mode, and selected
repositories. Validation failures exit 2; write failures exit 3. `passuff-tui`
runs the same prompts when the configuration file is missing and both stdin and
stdout are terminals; otherwise it exits 2 with a setup instruction.

## Observation model

- `passuff` runs one pass every `poll_interval_secs`; `check` runs one pass and
  exits.
- The first observation of each pull request records a baseline and does not
  replay historical activity.
- A pull request created and merged between two passes still notifies its
  terminal state.
- After a merge or close, later comments and reviews produce no notification.
  The transition itself and a reopen are reported.
- A failed account keeps its previous baseline.
- Passuff records your own comments and reviews as read without a notification.
  Pushes and activity on pull requests outside the focus stay silent.
- One notification digest is delivered per pass. Several pull requests become
  one notification naming the activity count.
- Pull requests GitHub reports as one stack are grouped with the stack base,
  size, and member position. Repositories without the stacked-PR preview show
  standalone rows.
- `passuff-tui` observes the configured accounts itself: it asks GitHub for a
  cheap change signal, observes only changed open pull requests, and reconciles
  every watched open pull request at `reconciliation_interval_secs`. The delay
  follows the account's remaining API allowance. Running `passuff` at the same
  time is redundant.

## GitHub GraphQL contract

`passuff` reads GitHub through one GraphQL endpoint per account: the account's
`graphql_url`, defaulting to `https://api.github.com/graphql`. It sends one
`POST` per operation with `Authorization: Bearer <token>`, `User-Agent:
passuff`, and a JSON body of `{"query": <text>, "variables": <object>}`. The
acceptance package answers `POST /graphql` and dispatches on the `query` text
only; it ignores `variables`.

### Operations

Every operation is named. Dispatch is a substring match on the query text, so
the name has to appear verbatim. A query that carries none of these names
returns `{"data": {}}` with HTTP 200 and no `errors`: an account with no pull
requests and no diagnostic.

| Operation | Issued by | Request | Response paths read |
|---|---|---|---|
| `PassuffViewer` | account resolution, once per account per pass, before any observation | `query PassuffViewer { viewer { login } }`, no variables | `data.viewer.login` |
| `PassuffProbe` | the open inbox change probe | `viewer.login` and `viewer.pullRequests(first: 100, orderBy: {field: UPDATED_AT, direction: DESC}).nodes` with `id`, `number`, `state`, `createdAt`, `updatedAt`, `headRefOid`, `statusCheckRollup { state }`, `repository { name owner { login } }`; plus `rateLimit { remaining resetAt cost }` | `data.viewer.pullRequests.nodes`, `data.rateLimit` |
| `PassuffAuthored` | `check`, each `passuff` pass, and inbox reconciliation | the same `viewer.pullRequests` connection with the shared node selection | `data.viewer.pullRequests.nodes` |
| `PassuffDiscovery` | every pass | three aliases, each a `search` with the shared node selection: `assigned`, `reviewRequested`, `teamReviewRequested` | `data.assigned.nodes`, `data.reviewRequested.nodes`, `data.teamReviewRequested.nodes` |
| `PassuffObserve` | the open inbox, for changed open pull requests only | `query PassuffObserve($ids: [ID!]!) { nodes(ids: $ids) { ... on PullRequest { <shared node selection> } } }` with variables `{"ids": ["<node id>", ...]}` taken from the most recent probe or full node | `data.nodes` |

Discovery qualifiers, one per alias, in this order:
`is:open is:pr assignee:@me`, `is:open is:pr review-requested:@me`,
`is:open is:pr team-review-requested:@me`. The discovery response may be empty
for all three aliases and must not fail the account.

The shared node selection, used inside `PassuffAuthored`,
`PassuffDiscovery`, and `PassuffObserve`:

```graphql
id
number
title
body
state
isDraft
createdAt
updatedAt
mergedAt
closedAt
url
headRefOid
author { login __typename }
assignees(first: 10) { nodes { login } }
statusCheckRollup { state }
comments(first: 100) { nodes { id body createdAt author { login __typename } } }
reviews(first: 100) { nodes { id body state submittedAt author { login __typename } } }
reviewThreads(first: 50) { nodes { comments(first: 50) { nodes { id body createdAt author { login __typename } } } } }
repository { name owner { login } }
stack { number size baseRefName }
stackEntry { position }
```

The pull request key is `repository.owner.login`, `/`, `repository.name`, `#`,
then `number`. Those two fields are the only accepted source. A response that
carries `nameWithOwner` instead yields no key, and the node is dropped.

`PassuffProbe` nodes carry only the fields listed in the table above. A probe
node has no title, body, author, comments, reviews, assignees, `url`,
`isDraft`, `mergedAt`, or `closedAt`; each is read as absent, not as an error.

`state` is `OPEN`, `MERGED`, or `CLOSED`. `statusCheckRollup.state` is
`SUCCESS`, `FAILURE`, `ERROR`, `PENDING`, or `EXPECTED`. A bot author is either
`author.__typename = "Bot"` or a login ending in `[bot]`.

### Error envelope

| Response | Result |
|---|---|
| Connection refused, DNS failure, or timeout | `GitHub is unavailable: <detail>` |
| HTTP 401 or 403 | `GitHub authentication failed` |
| HTTP 429 | `GitHub rate limit reached`, or `GitHub rate limit reached until <resetAt>` when the reset time is known |
| Any other non-2xx status, an unparseable body, or a non-empty `errors` array | `GitHub returned a malformed response: <detail>` |

`errors` is a top-level array of objects carrying a `message` string; the
messages are joined with `; `. A message containing `rate limit`,
case-insensitively, maps to the rate-limit result. `PassuffDiscovery` is the
exception: a non-empty `errors` array uses the partial data instead of failing
the account, so one rejected search alias cannot discard the authored
observation.

A malformed-response message that contains `stack` retries the same query once
with the `stack { number size baseRefName }` and `stackEntry { position }`
selections removed and the variables unchanged.

Every failure becomes one stderr line, the pass continues with the remaining
accounts, and `check` exits 1:

```text
passuff: account <label> failed: <error>
```

Token resolution failures use the same wrapper, before any request:

```text
passuff: account <label> failed: credential environment variable <NAME> is not set
passuff: account <label> failed: credential environment variable <NAME> is empty
passuff: account <label> failed: gh auth token failed for @<login>
passuff: account <label> failed: GitHub authentication failed
```

## Terminal output

Every line below is what the product writes to stdout or stderr. The acceptance
package asserts the listed substrings; anything beyond them is the reference
wording and may differ as long as the substrings still appear on one line.

### `passuff status`

```text
passuff: accounts
  work @tester  last check 2026-11-01T09:00:00Z
passuff: watch scope: all authored pull requests
passuff: watched pull requests
  octo/repo#7 open CI passing  Pull request octo/repo#7 - Add retry loop
passuff: 0 unread of 0
```

- **Account line:** two spaces, the label, one space, `@<login>` or
  `not resolved`, two spaces, `last check `, then an RFC 3339 UTC timestamp with
  second precision or `never`.
- **Scope line:** `all authored pull requests`, `watching nothing`, or the
  comma-separated selected repositories.
- **Watched heading:** `passuff: watched pull requests`, or
  `passuff: no watched pull requests` when the list is empty.
- **Watched row:** two spaces, the key, one space, the state
  (`open`, `merged`, `closed`), ` CI `, the CI result (`passing`, `failing`,
  `pending`, `not started`), two spaces, `Pull request <key>`, then
  ` - <title>` when the title is non-empty.
- **Last line:** `passuff: <unread> unread of <total>`.

Asserted: `passuff: 0 unread of 0`, `watched pull requests`,
`octo/repo#7 open CI passing`.

### `passuff check`

One fresh baseline and no notification:

```text
passuff: Fresh baseline recorded for octo/repo#7
passuff: no notifications
```

One pull request with one activity:

```text
passuff: notification for octo/repo#7
passuff: 1 activity on octo/repo#7
  New comment from @reviewer
```

One pull request with several activities: the same two header lines with
`<n> activities on <key>`, then one two-space-indented summary line per event.

Several pull requests in one pass:

```text
passuff: 3 activities in 2 pull requests
passuff: notification for octo/repo#7
  octo/repo#7: 2 activities
    New comment from @reviewer
    Review approved by @reviewer
passuff: notification for octo/api#9
  octo/api#9: 1 activity
    CI failing: build
```

Activity summaries:

| Event | Summary |
|---|---|
| Comment | `New comment from @<login>` |
| Review comment | `Review comment from @<login>` |
| Review | `Review <approved\|changes requested\|commented\|pending\|dismissed> by @<login>` |
| First-seen open pull request created after the last check | `New pull request by @<login>` |
| CI becoming failing | `CI failing: build` |
| CI recovering | `CI recovered` |
| Merged, closed | `Merged`, `Closed` |

Asserted: `Fresh baseline recorded for octo/repo#7`, `New comment from
@reviewer`.

### `passuff inbox`

```text
  a1  2026-11-01T11:00:00Z  octo/repo#7  New comment from @reviewer  unread
```

- **Row:** two spaces, the activity id, two spaces, the event time, two spaces,
  `owner/repo#number`, two spaces, the summary, two spaces, `read` or `unread`.
- **Time:** the event time as RFC 3339 UTC with second precision, or
  `~<observed time>` when the event carries no time of its own.
- **Ids:** `a<number>` allocated from a persisted counter, starting at `a1`.
- **Empty:** `passuff: no unread activity`, or `passuff: no activity` with
  `--all` when every record is read.

Asserted: `New comment from @reviewer`, `unread`.

### `passuff read`

```text
passuff: 1 activity record read
passuff: 2 activity records read
```

Singular `record` only for one changed record. An unknown id exits 2 with
`passuff: no activity with id <id>` on stderr.

Asserted: `activity record read`.

### `passuff-tui` layout

The frame is a 3-line header, the watched pane, the activity pane, and a 3-line
footer. The panes share the remaining height evenly and both are visible
without an overlay at 80x24 (9 rows each, 7 inner lines) and at 120x40 (17 rows
each, 15 inner lines). Below 40x12 the frame holds one line,
`Terminal too small. Resize to at least 40x12.`

Header:

```text
Accounts: work @tester
watch scope: all authored pull requests
1 unread of 1    sort: updated ↑
```

The default sort is `updated` ascending; `s` cycles the order and `S` reverses
the direction.

Pane titles: `Watched pull requests`, and `Activity · <filter>`, where the
filter is `all` unless a pull request, stack, or unread filter is active.

Watched pane: a column header
`  Key               Pos  State    CI          ☺`, then one row per pull request:

```text
  ▶ octo/repo#7      1/1   open     CI passing     ☺ @tester  Add retry loop
```

The key column is 16 characters, truncated to `…` plus the last 15 characters
when longer. `▶ ` marks the selection.

Activity pane: the first inner line is `<unread> unread of <total>`, plus
` (filtered from <n>)` when filters hide records. Each row:

```text
  ▶ a1  11-01 11:00  Pull request octo/repo#7  comment by @reviewer
```

The row display text is `comment by @<login>`, `review comment by @<login>`,
`review <state> by @<login>`, `new pull request by @<login>`, or the summary
for CI and state changes.

Footer: `Focus: PRs` or `Focus: activity`, followed by `Tab switch pane`; then
the key hints for the focused pane; then the last status or error line.

`q` exits 0. With unsaved view or notification changes, `q` opens the overlay
titled `Save` whose first line is `Save view and notification settings?`; `y`
saves and exits, `n` discards and exits, `esc` or `q` returns to the inbox.

Asserted at 80x24: `1 unread of 1`, `octo/repo#7`.

### Activity detail overlay

The overlay is centred, `clamp(width - 4, 20, 100)` by
`clamp(height - 2, 6, 30)`, with an all-borders block whose title repeats the
first line. Content lines, in order:

```text
Activity repo#7
When: 11-01 11:00
Fetched: 11-01 11:03
Pull request octo/repo#7
Title: Add retry loop
stack 4
unread
Event: comment by @reviewer
Enter open in browser    esc/q close
────
Comment by @reviewer
Looks good now.
```

- **`When`:** `%m-%d %H:%M` in UTC. `Fetched` appears only when the event time
  differs from the observation time.
- **Optional lines:** `Title` only when the title is non-empty, `stack` only
  for a stacked pull request.
- **Body label:** `Comment by @<login>`, `Review comment by @<login>`,
  `Review <state> by @<login>`, or the summary when the event has no body.

Asserted at 120x40: `comment by @reviewer`, then after `Enter`,
`Event: comment by @reviewer` and `Pull request octo/repo#7`.

### Configuration the acceptance package writes

```toml
poll_interval_secs = 1
reconciliation_interval_secs = 600

[notifications]
desktop = true
sound = false

[accounts.work]
github_user = "tester"
token_env = "PASSUFF_E2E_TOKEN"
graphql_url = "http://127.0.0.1:<port>/graphql"
```

Every value here must load without validation errors: an unsuffixed auth mode
with one token source, a loopback plain-HTTP endpoint, and the defaults for
every absent key.

## Notification delivery

- Linux desktop notifications use the session D-Bus through `notify-rust`;
  `urgency` maps to the notification urgency.
- Linux sound uses the first available player: `paplay`, `pw-play`, `aplay`, or
  `canberra-gtk-play`.
- macOS desktop notifications use Notification Center through `osascript`.
  Allow notifications for the app that runs the script, usually Script Editor or
  the terminal.
- macOS sound uses `afplay`.
- `notify-test` sends one notification and plays the sound for each enabled
  surface, then names each result. A surface failure exits 1; both surfaces
  disabled exits 2.

### Notification log (test seam)

With `PASSUFF_NOTIFICATION_LOG` set to a file path, every desktop notification
is appended to that file as one JSON object per line and no D-Bus or
Notification Center call is made. The path is opened in append mode and created
when missing. One line is written per digest, not per event, and no line is
written for a pass that produces no notification, a baseline pass included.

```json
{"title":"New activity on octo/repo#7","body":"New comment from @reviewer","urgency":"normal"}
```

| Field | Meaning |
|---|---|
| `title` | Notification summary |
| `body` | Newline-separated event summaries, trailing newline trimmed |
| `urgency` | `low`, `normal`, or `critical`, from `[notifications] urgency` |

Digest titles, per pass:

| Pass contained | Title | Body |
|---|---|---|
| One pull request, one activity | `New activity on <key>` | The single summary |
| One pull request, several activities | `<n> activities on <key>` | `<n> activities on <key>`, then one line per summary |
| Several pull requests | `<total> activities in <count> pull requests` | The title line, one `<key>: <n> activities` line per pull request (first five), then `and <remaining> more pull requests` |

The acceptance package asserts exactly one log line, titled
`New activity on octo/repo#7`, after a baseline pass followed by a pass that
observes one comment.

## Environment seams

| Variable | Effect | Acceptance dependency |
|---|---|---|
| `PASSUFF_NOTIFICATION_LOG` | Append desktop notifications as JSON lines instead of delivering them | Required: the notification scenario reads this file |
| `PASSUFF_SOUND_PLAYER` | Command used to play the sound file, replacing player detection | Optional |
| `PASSUFF_GRAPHQL_URL` | Overrides every account's `graphql_url` | Optional |

The acceptance package starts both binaries with a cleared environment holding
only `HOME`, `PATH=/usr/local/bin:/usr/bin:/bin`, the account's `token_env`
value, and any seam it needs. A product that requires another variable to start
fails every scenario.

## Discrepancies with the v0.1.3 implementation

The suite is the acceptance authority. These values are asserted by the
acceptance package and are not what `main` at v0.1.3 sends or prints.

| Area | v0.1.3 | Required |
|---|---|---|
| Operation names | Unnamed operations: `query($ids: [ID!]!)`, `query($after: String)`, `query($query: String!)` | `PassuffViewer`, `PassuffProbe`, `PassuffAuthored`, `PassuffDiscovery`, `PassuffObserve` |
| Repository field | `repository { nameWithOwner }` | `repository { name owner { login } }` |
| Status count line | `passuff: activity: <unread> unread of <total>` | `passuff: <unread> unread of <total>` |
| Watched row | `  <key> <state> <ci> <title>` | `  <key> <state> CI <ci>  Pull request <key> - <title>` |
| CLI inbox heading | `passuff: inbox` before the rows | Not asserted; the reference prints rows only |
| Discovery | One `search` per query | Three aliases in one query: `assigned`, `reviewRequested`, `teamReviewRequested` |
| Rate-limit location | Reads `data.rateLimit` | The suite places `rateLimit` beside `data`; neither location is asserted, so read either and tolerate absence |

## Sources

Operation names, payload shapes, the `repository { name owner { login } }`
shape, the `rateLimit` placement, the `{"data": {}}` fallthrough, and the HTTP
500 path come from `crates/passuff-e2e/tests/harness/github.rs`. Asserted
strings and exit codes come from `crates/passuff-e2e/features/boundary.feature`.
The written configuration and the `PASSUFF_NOTIFICATION_LOG` hook come from
`crates/passuff-e2e/tests/harness/world.rs`; the cleared child environment from
`crates/passuff-e2e/tests/harness/bin.rs`.

Reference wordings for lines the suite does not assert come from the reference
implementation used to qualify the suite, on branch `ise/trial`:
`crates/passuff-core/src/github.rs` (queries, error texts),
`crates/passuff-core/src/core.rs` (the `account <label> failed` wrapper),
`crates/passuff-cli/src/render.rs` (status, check, and inbox transcripts),
`crates/passuff-cli/src/runtime.rs` (read output, exit codes),
`crates/passuff-cli/src/delivery.rs` and `crates/passuff-cli/src/main.rs` (the
log seam), `crates/passuff-cli/src/tui/render.rs` and
`crates/passuff-cli/src/tui/mod.rs` (layout, overlay, detail lines), and
`crates/passuff-core/src/model.rs` plus
`crates/passuff-core/src/classifier.rs` (activity summaries and display text).

## Files

| Path | Content |
|---|---|
| `$XDG_CONFIG_HOME/passuff/config.toml` | Configuration |
| `$XDG_STATE_HOME/passuff/state.json` | Activity, watched pull requests, and read state |
| Same directory as the state file, `ui.json` | TUI sort, filter, and notification preferences |

On macOS the `directories` crate uses the platform configuration and data/state
directories instead of the XDG paths.
