# Proposal: passuff-activity-inbox

## Release note

Review, filter, and mark read the pull-request activity that passuff records, and see which accounts, repositories, and pull requests are being watched.

## Use-Case Context

- Use-case IDs: `review-activity-inbox` (new) and `inspect-watch-status` (new); the watch scope extends `watch-pull-requests`.
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — confirmed by the operator's brief on 2026-11-02.

## Review Context

- Review kind: independent
- Reason: new public CLI commands, a persisted state schema change, and a new use-case contract — a major change.

## Problem / Opportunity

The watcher only reports changes during a cycle. The operator cannot list what is being watched, inspect the tracked pull requests and their CI state, see activity that was already notified, or acknowledge it. The state file is internal JSON, and notification history vanishes after delivery. The operator also cannot limit observation to a set of repositories, so a noisy account watches everything it authored.

## Proposed Solution

- `passuff status` shows the configured accounts and resolved logins, the active watch scope, every watched pull request with its state and CI result, and the number of unread activity records.
- `passuff inbox [--all]` lists recorded activity newest first, with time, pull request, event, and read state. Without `--all`, only unread records are listed.
- `passuff read <id>` marks one record read; `passuff read --all` marks every record read. Unknown IDs are reported and exit nonzero.
- The state persists a bounded activity history (the 500 most recent records) across restarts. Each record is an atomic event, not a grouped notification.
- Watch scope is an optional global repository allowlist owned by the watch use case. Empty means all authored pull requests. A pull request outside the allowlist is not observed, notified, or watched.
- An existing version-1 state file loads with an empty history; the state version becomes 2.
- Read state survives restarts. Recording history never causes a notification to replay.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| pr-activity-notification (watch scope) | EXTEND pr-activity-notification (use case `watch-pull-requests`, score 12.00, signal) | `EXTEND pr-activity-notification` | The repository allowlist changes which authored pull requests enter observation; it belongs to the watch capability. |
| watch-status | EXTEND pr-activity-notification (same route signal) | `NEW in inspect-watch-status` | Status advances a distinct read goal — inspecting the watch — rather than notification delivery. |
| activity-inbox | EXTEND pr-activity-notification (same route signal) | `NEW in review-activity-inbox` | Reviewing and marking recorded activity read is a distinct actor goal, not notification delivery. |

## Out of Scope

- The terminal UI. It is the immediately following change and consumes the same inbox queries.
- Editing configuration or state from the CLI.
- Remote synchronization of history or read state.
- Per-account watch scopes; the first version is global.
- Unbounded history or archival export.

## Open Questions

### D1 — How much activity history is retained?

- Question: Should the inbox keep a fixed number of records, a time window, or everything?
- Options:
  1. **Fixed bound (500 records)** — gain: predictable state size and pruning; cost: very old activity disappears. In practice: the 501st record drops the oldest.
  2. **Time window (30 days)** — gain: familiar retention; cost: pruning depends on clock and is harder to test deterministically. In practice: records older than 30 days vanish.
  3. **Unbounded** — gain: no loss; cost: state grows without limit.
- Disposition: answered 2026-11-02 (operator context: bounded history was the archived plan) — fixed bound of 500 records.

### D2 — Is read state per activity record or per pull request?

- Question: Does marking read acknowledge one event or every event on a pull request?
- Options:
  1. **Per event** — gain: precise acknowledgment; cost: more IDs and commands. In practice: mark the comment read while leaving the CI failure unread.
  2. **Per pull request** — gain: fewer actions; cost: hides distinct events. In practice: one pull request row clears all its activity.
- Disposition: answered 2026-11-02 — per event; the inbox lists atomic events.

### D3 — Where does the watch scope live?

- Question: Is the repository allowlist global, per account, or per pull request?
- Options:
  1. **Global** — gain: one list, simple status; cost: two accounts cannot have different scopes.
  2. **Per account** — gain: precise; cost: more configuration and a larger status surface.
- Disposition: carried to `design.md` (owner: design author) — the design decides and states the consequence.
