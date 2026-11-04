# Proposal: passuff-observation-targets

## Release note

Observe only changed open pull requests and reconcile the open set hourly to cut check-cycle allowance use.

## Use-Case Context

- Use-case ID: `watch-pull-requests`
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — named by the owning use case and confirmed by the operator in this session.

## Review Context

- Review kind: self-review
- Reason: single-context change; the operator supplied and confirmed every policy decision in this session.

## Problem / Opportunity

A detected change runs a full observation of the 50 most recently updated authored pull requests plus the watched open and recently watched pull requests. Measured on the operator account, that is 31-40 GitHub GraphQL allowance points, although usually one or two pull requests changed. The operator's watched set is 52 pull requests, all merged or closed, so most of that cost buys activity on pull requests the operator explicitly does not want to see.

The change probe already fingerprints every in-scope authored pull request for about one point, but it reports only that *something* changed. The watcher cannot tell *what* changed, so it re-observes a large fixed slice.

The opportunity is to keep the probe as the cheap signal and let it identify the changed pull requests, so a check cycle observes only those. The saving only matters during change bursts; the probe tick remains the dominant idle cost by design.

## Proposed Solution

- The watcher stops reporting activity on an already-merged or already-closed watched pull request. Comments, reviews, and CI changes after the terminal state produce no notification.
- The transition itself still notifies: a watched open pull request that merges or closes, a pull request created and merged between checks, and a reopened pull request are observed and reported exactly as today.
- After a change signal, the watcher observes only the open pull requests that changed and any newly assigned or review-requested open pull requests. It does not re-observe the unchanged watched set.
- The watcher performs a reconciliation pass at a configured interval, default one hour, and on startup or watch-scope change. The pass refreshes every watched open pull request even when the change probe reports nothing, so activity on assigned or review-requested pull requests is bounded in time.
- The reconciliation interval is configurable.
- Merged and closed watched pull requests remain recorded, so a reopen is still observed and the activity history keeps its context.
- Observable outcome: a single open-pull-request change consumes a few allowance points instead of 31-40. The displayed remaining allowance makes the difference visible.

## Capability Routing

`givn spec route` reported no signal (top candidates tied). Decision recorded with rationale.

| Proposed capability | Route's recommendation | Decision | Rationale |
|---|---|---|---|
| `pr-activity-notification` | none (no signal) | `EXTEND pr-activity-notification` | The change alters which pull requests are observed and which activity notifies. It adds no new actor goal, channel, or surface; the existing capability already owns observation, classification, and notification. |
| `inbox-navigation` | none (no signal) | `EXTEND inbox-navigation` | The live inbox stops running a full check cycle on every change signal and gains the configurable reconciliation interval. The screen and its keys do not change; the live observation already belongs to this capability. |

## Out of Scope

- Hiding merged and closed rows from the watched pane. Kept records remain visible until the separate read-model change.
- Moving the watch scope into the observation queries; out-of-scope pull requests are still filtered after observation.
- Changing the probe cadence, the pacing formula, the probe query cost, or the event families.
- Pruning terminal watched pull requests or the activity history.
- Shortening the 30-day reopen tracking rule.

## Open Questions

### D1 — Coverage of terminal pull requests

- Question: should activity on already-merged or already-closed watched pull requests still notify?
- Options:
  1. **Retire terminal pull requests after their transition** — gain: cycles stop paying for unwanted activity on the operator's 52 terminal watch records, cost: a comment on a merged pull request no longer notifies, in practice: a comment on merged `buster/passuff#20` produces no notification and no observation.
  2. **Keep observing terminal pull requests** — gain: post-merge comments still notify, cost: every change cycle still observes the terminal set, in practice: the same comment runs a 31-40 point cycle.
- Disposition: answered — operator chose retirement in this session; only the terminal transition, short-lived, and reopen cases remain.

### D2 — Terminal rows in the inbox

- Question: what should happen to the 52 terminal watched records in the watched pane?
- Options:
  1. **Keep visible** — gain: no migration, cost: stale rows stay in the pane, in practice: merged rows remain listed and never change.
  2. **Keep the records, hide terminal rows by default with a toggle** — gain: the pane shows the current watch set and activity detail still resolves, cost: a visible behavior change and one view preference, in practice: a merged row disappears from the pane but its activity detail still shows the title.
  3. **Purge terminal records** — gain: smallest state, cost: activity detail loses title, state, CI, and stack, in practice: an old activity record can no longer show which pull request it belongs to.
- Disposition: answered — operator chose option 2 in this session; carried to a separate change, not this one.

### D3 — Reconciliation cadence

- Question: should the watcher run a full reconciliation pass, and how often?
- Options:
  1. **Hourly, configurable** — gain: bounded staleness for assigned/review-requested pull requests and signal blind spots, cost: one scheduled full pass per hour, in practice: with the operator's current 0 open pull requests the pass costs a few points.
  2. **No periodic pass** — gain: no scheduler, cost: known involved pull requests are never refreshed while the watcher runs, in practice: a review comment on a requested pull request waits for a restart.
  3. **Six-hourly, configurable** — gain: fewer passes, cost: up to six hours of staleness, in practice: the same review comment notifies up to six hours later.
- Disposition: answered — operator chose hourly and configurable in this session; the setting name and default are carried to `design.md`.

### D4 — Configuration surface

- Question: where does the reconciliation interval live and what is its unit?
- Options:
  1. **Top-level seconds, beside `poll_interval_secs`** — gain: matches the existing configuration style and is easy to validate, cost: a raw number in the file, in practice: `reconciliation_interval_secs = 3600`.
  2. **Human-readable duration string** — gain: reads better, cost: a new parser and a new failure mode, in practice: `reconciliation_interval = "1h"`.
- Disposition: carried — `design.md` decides; default must remain one hour.
