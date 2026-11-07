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
| `--verbose` | Diagnostic logs on stderr |
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

## Notification delivery

- Linux desktop notifications use the session D-Bus notification service;
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

## Files

| Path | Content |
|---|---|
| `$XDG_CONFIG_HOME/passuff/config.toml` | Configuration |
| `$XDG_STATE_HOME/passuff/state.json` | Activity, watched pull requests, and read state |
| Same directory as the state file, `ui.json` | TUI sort, filter, and notification preferences |

On macOS the platform configuration and data/state
directories instead of the XDG paths.
