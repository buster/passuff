# Questions: inbox-scope-and-stacks

## Q1 — What happens to stored history when a repository is deselected?

- Status: closed 2026-11-03 (operator)
- Blocks: the watch-scope edit rule and the reselect behaviour.
- Context: the watch scope is configuration that filters observation. The
  repository's watched pull requests and activity records already exist in
  watcher state when the developer deselects it. Keeping them preserves the
  inbox history and makes reselect cheap, but the selected scope and what the
  inbox shows diverge. Pruning them keeps the inbox equal to the scope, but
  destroys history and forces a fresh baseline on reselect, so activity that
  happened while deselected is never reported.
- Question: when the developer deselects a repository, should its watched pull
  requests and activity records remain, be hidden, or be deleted?
- Options:
  1. **Retain** — stop observing; keep records and history visible.
     In practice: deselecting `acme/api` stops new notifications, but its past
     activity stays in the inbox.
     Gain: no data loss; reselect resumes without a backlog surprise.
     Cost: the inbox shows records outside the current scope; the developer must
     remember what is still watched.
  2. **Hide** — stop observing; keep records but exclude them from the inbox and
     status.
     In practice: `acme/api` disappears from the inbox; reselecting it restores
     the old rows.
     Gain: the inbox matches the scope; no data loss.
     Cost: a second visibility rule; the state grows with hidden records.
  3. **Prune** — stop observing; delete the repository's watched pull requests
     and activity records.
     In practice: `acme/api` disappears and reselecting starts from a fresh
     baseline; changes made while deselected are never notified.
     Gain: one rule, smallest state, scope equals observable data.
     Cost: history loss; reselect can silently miss activity.
- Resolution: **Hide** because the inbox should match the current watch scope
  without destroying recorded history; hidden records remain available on
  reselect. Rejected: Retain because records outside the scope would still
  appear as watched pull requests, contradicting the scope line; Prune because
  it destroys history and prevents reviewing activity recorded before the
  deselection.

## Q2 — Where does stack membership come from?

- Status: closed 2026-11-03 (operator)
- Blocks: the stack term, the observation contract, and the state schema.
- Context: GitHub's stacked-PRs preview exposes a `stack` object on the
  GraphQL `PullRequest` and in pull request webhooks. Where the preview is not
  enabled, the field may be absent or null. A ref-chain fallback can infer
  membership from `baseRefName` and `headRefName`, but it only sees pull requests
  passuff already watches, and the state currently stores neither ref.
- Question: should the inbox derive stack membership only from GitHub's stack
  metadata, only from ref chains, or from metadata with a ref-chain fallback?
- Options:
  1. **Metadata only** — query `stack`; a null or absent value means standalone.
     In practice: a preview-enabled repository shows the chain; a
     non-preview repository shows flat rows.
     Gain: exact membership, size, position, and ultimate base; one code path.
     Cost: no grouping where the preview is off; depends on a preview API.
  2. **Ref-chain fallback only** — infer chains from base/head refs.
     In practice: `feature-a` → `feature-b` groups even without the preview.
     Gain: works everywhere; no preview dependency.
     Cost: sees only watched authored pull requests; merged boundaries and
     retargeted bases are ambiguous; needs both refs persisted.
  3. **Metadata with fallback** — use `stack` when present, otherwise ref chains.
     In practice: preview repositories get exact stacks; others get inferred
     chains.
     Gain: grouping everywhere available, exact where possible.
     Cost: two sources can disagree; tests cover both and the precedence rule.
- Resolution: **Metadata only** because exact membership, position, size, and
  ultimate base are worth more than grouping in repositories where the preview
  is unavailable; those repositories degrade to standalone rows. Rejected:
  Ref-chain fallback because it sees only watched authored pull requests and is
  ambiguous across merges and retargeted bases; Metadata with fallback because
  two disagreeing sources need a precedence rule and double the test surface for
  an approximation of the same fact.
## Q3 — Is the watch scope selection global or per account?

- Status: closed 2026-11-03 (operator, via frame adoption)
- Blocks: the selection screen's row model and the configuration shape.
- Context: `watch.repositories` is one global allowlist shared by all configured
  accounts. The developer uses a personal and a work account and may want
  `acme/*` only for work.
- Question: should the terminal UI edit one global repository allowlist, or a
  per-account allowlist?
- Options:
  1. **Global** — one list, current shape.
     In practice: selecting `acme/api` applies to every account.
     Gain: no configuration migration; one rule.
     Cost: a personal account also observes `acme/api`; over-broad observation.
  2. **Per account** — each account carries its own allowlist.
     In practice: `acme/api` is selected for work only.
     Gain: precise scope.
     Cost: configuration migration and a more complex selection screen.
- Resolution: **Global** because the adopted Frame excludes per-account watch
  scope and keeps the current one-list configuration shape. Rejected: per
  account because it changes the configuration shape and was explicitly outside
  the adopted frame.

## Q4 — On reselect, does observation resume from the old baseline or start fresh?

- Status: closed 2026-11-03 (operator)
- Blocks: the reselect rule and the notification behaviour after a scope change.
- Context: Q1 chose Hide: records are kept but excluded from the inbox while the
  repository is deselected. Observation stops at deselection. On reselect, the
  first check cycle compares current GitHub activity with the stored baseline and
  seen-activity identities. With the old baseline, every comment or review added
  while deselected is new and can produce one grouped notification per pull
  request; with a fresh baseline, the first observation absorbs the current
  state and the gap is not notified.
- Question: when a repository is reselected, should observation resume from the
  old baseline and report the gap, or start from a fresh baseline and stay silent
  about it?
- Options:
  1. **Resume from the old baseline** — report activity that happened while
     deselected.
     In practice: reselecting `acme/api` after a week produces notifications for
     comments added in that week.
     Gain: no activity is missed; no baseline replacement rule.
     Cost: a notification burst, including stale comments that may no longer
     matter; delayed notifications mix with current ones.
  2. **Fresh baseline** — the first observation after reselect records a new
     baseline and suppresses the gap.
     In practice: reselecting `acme/api` is quiet; only activity after the next
     observation notifies.
     Gain: predictable, quiet reselect; matches "start watching again".
     Cost: activity during the gap is never notified; the retained history has a
     hole.
  3. **Catch up silently** — record the gap events as activity records but
     deliver no desktop notification or sound for the first cycle.
     In practice: reselecting `acme/api` adds unread inbox rows for the week's
     comments without interrupting the developer.
     Gain: no notification burst and no history hole.
     Cost: a first-cycle suppression rule; the inbox can gain many unread rows
     at once; it diverges from the current notification contract.
- Resolution: **Catch up silently** because deselection exists to stop
  interruptions, while the developer still wants to review what happened during
  the gap in the inbox. Rejected: Resume from the old baseline because it
  produces a delayed notification burst of possibly stale activity; Fresh
  baseline because it creates a history hole and discards activity the developer
  explicitly wants to review.

## Q5 — Where does the repository picker get its candidates?

- Status: closed 2026-11-03 (operator)
- Blocks: the read model behind the watch-scope edit and the TUI screen's contents.
- Context: the watch scope is a global allowlist. passuff stores watched pull
  requests, not a repository index. The observation query returns authored pull
  requests across all readable repositories and then filters by scope, so
  passuff can see repositories that are currently outside the scope. A picker
  can offer only repositories already seen, query GitHub for the full readable
  catalog, or combine a list with manual entry.
- Question: which repositories should the picker offer for selection?
- Options:
  1. **Known repositories** — every repository seen in an authored pull request,
     in or out of scope, plus current scope entries.
     In practice: a repository with an authored pull request appears even when
     deselected; a repository with no authored pull request never appears.
     Gain: no extra GitHub calls; fast; works offline.
     Cost: cannot preselect a repository before the first authored pull request
     there; needs a small repository index in state.
  2. **Full catalog** — query every repository readable by each configured
     account.
     In practice: all readable repositories appear, including those with no
     authored pull requests.
     Gain: complete selection.
     Cost: pagination and extra API calls per account; a large list; visibility
     differences across accounts; slower.
  3. **Known plus manual entry** — list known repositories; add a missing one by
     typing owner/name.
     In practice: a new repository is added by typing `acme/new`; it appears
     selected in the scope.
     Gain: covers new repositories without a catalog.
     Cost: typing and validation; less "select/deselect".
- Resolution: **Known plus manual entry** because it avoids both a full catalog
  query and a TOML edit for a new repository. Rejected: Known repositories
  because a repository with no authored pull request could not be selected;
  Full catalog because per-account pagination, visibility differences, and a
  large list outweigh completeness.

## Q6 — What does an empty watch scope mean?

- Status: closed 2026-11-03 (operator)
- Blocks: the `change-watch-scope` rule for deselecting the last repository and
  the configuration schema.
- Context: the current watch scope is an optional allowlist; an empty list means
  "all authored pull requests" (`WatchConfig::is_all`). A picker that deselects
  the last repository would therefore silently broaden observation to every
  repository, the opposite of the developer's action. The picker can show
  selected repositories, but the current configuration cannot distinguish "all"
  from "none" as an explicit choice.
- Question: when every repository is deselected, should the watch scope mean all
  repositories, no repositories, or should the last deselection be prevented?
- Options:
  1. **Explicit all/selected mode; empty selected list means none** — the watch
     scope gains a mode: all repositories, or selected repositories. Selecting
     from all mode switches to selected; deselecting the last selected
     repository leaves selected mode with no entries and no observation.
     In practice: deselect the last repository and the status says "watching
     nothing"; the developer can switch back to "all repositories" explicitly.
     Gain: the picker represents the default and every selection; deselecting
     never broadens observation.
     Cost: configuration schema change and migration; a third observable state
     (none); the picker needs a mode control.
  2. **Keep empty = all; forbid deselecting the last repository** — the picker
     disables the last toggle.
     In practice: the last selected repository cannot be deselected; to stop
     observing it, the developer stops the watcher.
     Gain: no schema migration.
     Cost: the UI cannot express "none" or "all" explicitly; the last toggle is
     unavailable with no visible reason.
  3. **Keep empty = all; deselecting the last switches to all with a notice** —
     the last deselection clears the allowlist and the picker shows "all
     repositories".
     In practice: deselecting the last repository starts observing every
     repository, with a notice.
     Gain: current configuration semantics survive.
     Cost: the last action does the opposite of its label; "none" is not
     representable.
- Resolution: **Explicit all/selected mode; empty selected list means none**
  because deselecting must never broaden observation, and the picker must be
  able to show the default and every selection. Rejected: Forbid the last
  deselection because it cannot express "none" and hides the reason; Switch to
  all with a notice because the last action would do the opposite of its label.
