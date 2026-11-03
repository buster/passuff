# Proposal: passuff-watch-scope

## Release note

Change which repositories passuff observes from the terminal inbox, with hidden records and silent catch-up.

## Use-Case Context

- Use-case ID: `change-watch-scope` (new)
- Ideation topic: `inbox-scope-and-stacks`
- Seed path: `givn/ideation/inbox-scope-and-stacks/changes/passuff-watch-scope.md`
- Confirmed Personas: `multi-account-developer`

## Review Context

- Review kind: independent
- Reason: the change migrates the configuration schema, persists watch-scope decisions, changes watcher observation and notification behaviour, and adds a user-visible terminal picker.

## Problem / Opportunity

Changing which repositories are watched currently requires editing TOML and
restarting the watcher. The existing empty allowlist means "all repositories",
so deselecting the last repository would silently broaden observation. The
developer also has no defined behaviour for activity that happens while a
repository is deselected; reselecting can either replay a burst or lose the gap.

## Proposed Solution

- The terminal inbox opens a repository picker that lists known repositories with
  their selected state and accepts a manual `owner/name` entry.
- The watch scope has an explicit mode: all repositories, or selected
  repositories. A selected set may be empty, which observes nothing.
- Selecting a repository adds it to the selected set; deselecting removes it and
  hides its watched pull requests and activity from the inbox and status while
  retaining them in watcher state.
- Reselecting restores the hidden records and records activity that happened
  while the repository was deselected as unread, without a desktop notification
  or sound (catch-up).
- The selected set is retained while the mode is all; switching back restores it.
- A scope change is persisted atomically and applies to the next check cycle of a
  running watcher, without a restart.
- Repository entries are `owner/name`, case-insensitive. The format is validated;
  existence is not checked against GitHub.
- The scope remains global across configured accounts.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| watch-scope | no signal (tie) | `NEW in change-watch-scope` | The new use case owns scope editing; `watch-status` keeps read-only display and `activity-inbox` keeps review/read behaviour. |

## Out of Scope

- Per-account watch scope; the scope stays global.
- A full readable-repository catalog; the picker lists known repositories plus
  manual entry.
- GitHub Projects boards.
- Webhooks or streaming transport; polling stays.
- Changing stack grouping or notification delivery for normal cycles.

## Open Questions

### D1 — How is the scope mode represented and migrated?

- Question: which configuration shape carries all/selected and an empty selected
  set that means none, and how do legacy files map?
- Options:
  1. **Mode field** — a string or boolean mode plus the existing repository list.
     Gain: explicit. Cost: a schema addition and migration rules.
  2. **Optional list** — absence means all, an empty list means none.
     Gain: no mode field. Cost: existing explicit empty lists change meaning.
- Disposition: carried — `design.md` decides; owner: design.

### D2 — How does a running watcher learn about a scope change?

- Question: how does the next check cycle apply a persisted scope without a
  restart, and what happens when the reload fails?
- Options:
  1. **Reload configuration each cycle** — parse it before observation; a
     malformed reload keeps the previous scope.
  2. **State marker** — the terminal UI writes a marker and the watcher reloads.
     Gain: no parse per cycle. Cost: two writes and a consistency window.
- Disposition: carried — `design.md` decides; owner: design.

### D3 — How does the end-to-end test drive the picker?

- Question: the terminal picker is interactive; how does the real binary expose
  the same selection action for `--snapshot` E2E without a PTY?
- Options:
  1. **Scripted scope flags** — `--scope-select`, `--scope-deselect`, and
     `--scope-mode` before `--snapshot`, sharing the interactive service.
     Gain: deterministic real-process E2E. Cost: a non-interactive surface.
  2. **PTY driver** — script keystrokes. Gain: exercises raw mode. Cost: fragile
     and previously rejected.
- Disposition: carried — `design.md` decides; owner: design.

### D4 — Is the pre-existing TUI/watcher state-save race fixed here?

- Question: the TUI mark-read actions and the watcher both save the whole state
  file; should this change add file locking or a merge-on-save store?
- Options:
  1. **Defer** — the scope edit adds no state writer; record the pre-existing
     race and fix it in a separate change. Gain: bounded scope. Cost: the
     pre-existing window remains.
  2. **Fix now** — add file locking or a merge-on-save store. Gain: removes the
     window. Cost: a larger storage change unrelated to watch-scope editing.
- Disposition: answered — defer; the scope edit adds no writer, and the
  pre-existing race is recorded as R-024 for a separate change.
