# Change seed: passuff-watch-scope

Status: ready

- Use-case IDs: change-watch-scope
- Topic: inbox-scope-and-stacks
- Personas: multi-account-developer
- Suggested start: 2 of 2; no functional dependency on `passuff-stacked-inbox`,
  but run after it because both touch the terminal UI and the inbox read model.
- Handoff: ready for `/givn-propose`

## Accepted decisions

### Deselecting hides records but retains them

- Context: watcher state holds the repository's watched pull requests and
  activity when the developer deselects it. Retaining them preserves history;
  keeping them visible would contradict the scope.
- Options:
  1. **Hide** — stop observing; exclude records from the inbox and status.
     In practice: `acme/api` disappears; reselecting restores its rows.
     Gain: the inbox matches the scope; no data loss.
     Cost: a visibility rule; hidden records stay in state.
  2. **Retain visible** — keep records in the inbox.
     In practice: deselected rows remain next to the shorter scope line.
     Gain: complete history at a glance.
     Cost: the inbox shows records outside the scope.
  3. **Prune** — delete the repository's records.
     In practice: rows disappear permanently; reselect starts fresh.
     Gain: one rule, smallest state.
     Cost: history loss; no review of prior activity.
- Chosen: **Hide** because the inbox must match the scope without destroying
  history. Rejected: Retain visible because it contradicts the scope line;
  Prune because it destroys reviewable history.

### Reselecting catches up silently

- Context: observation stops while a repository is deselected. On reselect, the
  first cycle compares current GitHub activity with the stored baseline. With the
  old baseline it can produce a burst of delayed notifications; with a fresh
  baseline it loses the gap.
- Options:
  1. **Catch up silently** — record gap events as unread activity without a
     desktop notification or sound.
     In practice: reselecting after a week adds unread rows, no interruption.
     Gain: no burst, no history hole.
     Cost: a first-cycle suppression rule; many unread rows at once.
  2. **Resume with the old baseline** — notify the gap.
     In practice: notifications for the week's comments.
     Gain: nothing missed.
     Cost: a delayed burst of stale activity.
  3. **Fresh baseline** — suppress the gap entirely.
     In practice: reselect is quiet; the gap is never recorded.
     Gain: predictable quiet reselect.
     Cost: history hole; the developer cannot review the gap.
- Chosen: **Catch up silently** because deselection exists to stop
  interruptions, while the developer still wants to review the gap. Rejected:
  Resume because of the delayed burst; fresh baseline because it loses activity.

### The watch scope stays global across accounts

- Context: `watch.repositories` is one allowlist shared by every configured
  account; per-account scope changes the configuration shape.
- Options:
  1. **Global** — one scope for all accounts.
     In practice: selecting `acme/api` watches it for personal and work.
     Gain: no migration; one rule.
     Cost: over-broad observation.
  2. **Per account** — each account has its own scope.
     In practice: `acme/api` selected for work only.
     Gain: precise scope.
     Cost: configuration migration and a more complex picker.
- Chosen: **Global** because the adopted frame excludes per-account scope.
  Rejected: per account because it changes the configuration shape outside the
  frame.

### The picker lists known repositories and accepts manual entry

- Context: passuff stores watched pull requests, not a repository index. A full
  catalog needs pagination and extra API calls per account.
- Options:
  1. **Known plus manual entry** — list repositories seen in authored pull
     requests or the scope; add a missing one by owner/name.
     In practice: type `acme/new`; it appears selected.
     Gain: no catalog query; no TOML edit for a new repository.
     Cost: typing and validation.
  2. **Known only** — list seen repositories and scope entries.
     In practice: a repository without an authored pull request cannot be
     selected.
     Gain: no entry field.
     Cost: cannot preselect a new repository.
  3. **Full catalog** — query every readable repository per account.
     In practice: all accessible repositories appear.
     Gain: complete.
     Cost: pagination, extra calls, large list, differing visibility.
- Chosen: **Known plus manual entry** because it avoids both a catalog query and
  a TOML edit. Rejected: Known only because it cannot preselect a new
  repository; full catalog because of pagination and list size.

### Explicit all/selected scope mode; an empty selected set observes nothing

- Context: the current allowlist treats an empty list as "all repositories", so
  deselecting the last repository would silently broaden observation.
- Options:
  1. **Explicit all/selected mode; empty selected means none** — the scope has a
     mode; selected may be empty.
     In practice: deselect the last repository; status says watching nothing.
     Gain: deselecting never broadens; the default is explicit.
     Cost: configuration migration; a third observable state.
  2. **Keep empty = all; forbid the last deselection** — the picker disables the
     last toggle.
     In practice: the last repository cannot be deselected.
     Gain: no migration.
     Cost: "none" is not representable; the disabled toggle is opaque.
  3. **Keep empty = all; the last deselection switches to all** — with a notice.
     In practice: deselecting the last repository starts watching everything.
     Gain: current semantics survive.
     Cost: the action does the opposite of its label.
- Chosen: **Explicit all/selected mode; empty selected means none** because
  deselecting must never broaden observation. Rejected: Forbidding the last
  deselection because it cannot express none; switching to all because it
  reverses the action.

## Open questions

### How should the configuration represent the scope mode and migrate legacy files?

The current `[watch] repositories` list has no mode. The proposal must choose the
mode representation (`mode = "all" | "selected"`, a boolean, or an optional
list), and the migration rule for a legacy explicit empty list, which today
means all. The decision changes the configuration schema and how the terminal UI
writes the file.

### How does a running watcher learn that the scope changed?

The terminal UI writes configuration, while the watcher loads it at startup. The
next check cycle must apply the new scope without a restart. The proposal must
choose between reloading the configuration each cycle, a state-file marker, or a
signal. Each option changes failure behavior: a malformed reload must keep the
previous scope, not stop observation.

### What does the picker look like and how does it validate manual entry?

The screen needs a mode control, a known-repository list with selected state, and
a manual-entry field. The proposal must settle format-only validation (no GitHub
existence check), the error copy for a malformed entry, and the empty-known-list
state. These are design details with observable consequences in the terminal UI.
