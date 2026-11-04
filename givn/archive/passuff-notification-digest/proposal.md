# Proposal: passuff-notification-digest

## Release note

Deliver one digest notification per check pass and play a one-second three-beep chime.

## Use-Case Context

- Use-case ID: `watch-pull-requests`
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — named by the owning use case and confirmed by the operator.

## Review Context

- Review kind: self-review
- Reason: single-context change; the operator supplied the behaviour in this session.

## Problem / Opportunity

A watcher started after a pause can report activity for many pull requests in one pass. The watcher currently delivers one desktop notification and one sound per pull request (`crates/passuff-cli/src/lib.rs:352-369`). Dozens of pull requests therefore produce dozens of banners and dozens of plays of the 0.35 s bundled sound, which is disruptive and impossible to read.

The terminal transcript already carries the per-event detail. The problem is the delivery volume, not the information.

## Proposed Solution

- When one check pass reports activity for more than one pull request, the watcher delivers **one** notification for the whole pass. It names the total activity count and the number of pull requests, e.g. `passuff: 5 activities in 3 pull requests`, and the body names each pull request's activity count. The sound plays once for that digest.
- A single pull request with several events keeps one notification and names the activity count, e.g. `passuff: 3 activities on acme/api#42`.
- A single activity on a single pull request is unchanged: the existing key title and event text.
- The terminal transcript keeps the per-event detail under the digest heading; the digest adds the count, it does not remove the detail.
- The default sound becomes three short beeps within one second, each shorter than the current 0.35 s beep. A custom `sound_file` is unaffected.

## Capability Routing

`givn spec route` recommends `EXTEND pr-activity-notification` (score 14, signal).

| Proposed capability | Route's recommendation | Decision | Rationale |
|---|---|---|---|
| `pr-activity-notification` | `EXTEND pr-activity-notification` | `EXTEND pr-activity-notification` | Notification grouping, desktop delivery, and the sound are already owned by this capability. The change adds no new actor goal or surface. |

## Out of Scope

- Changing which events notify, or the per-pull-request classification and grouping.
- The terminal inbox: it shows persisted activity and does not deliver notifications.
- Notification history or a digest inbox.
- A user-configurable digest size or per-PR body length beyond the fixed compact list.
- Custom sound files: only the bundled default changes.

## Open Questions

### D1 — Digest scope and transcript

- Question: when more than one pull request has activity, should the terminal transcript collapse too, or keep the detail?
- Options:
  1. **One digest heading, per-PR detail under it** — gain: one delivery and one sound while the transcript stays readable, cost: a longer transcript block, in practice: one heading names `5 activities in 3 pull requests`, followed by each key and its events.
  2. **One compact digest everywhere** — gain: the shortest output, cost: the terminal loses the event detail, in practice: only the count and per-PR counts appear.
- Disposition: answered — operator asked for one notification with the count; the detail stays in the terminal transcript (option 1).

### D2 — Single pull request with several events

- Question: should a single pull request with multiple events also name the count?
- Options:
  1. **Name the count** — gain: consistent with the digest and immediately readable, cost: a title change, in practice: `passuff: 3 activities on acme/api#42`.
  2. **Keep the current key title** — gain: no change for the common single-PR case, cost: inconsistent with the multi-PR digest, in practice: `passuff: acme/api#42` with three event lines.
- Disposition: answered — name the count.

### D3 — Desktop body size

- Question: how many pull requests should the desktop digest body list?
- Options:
  1. **Cap at five with `+N more`** — gain: a bounded banner, cost: some PRs appear only in the transcript, in practice: five count lines plus `+4 more pull requests`.
  2. **List every pull request** — gain: complete banner, cost: an unbounded body the desktop may truncate, in practice: dozens of lines.
- Disposition: answered — cap the desktop body; the transcript lists all.

### D4 — Sound shape

- Question: what should the new default sound be?
- Options:
  1. **Three short beeps in one second** — gain: noticeable but brief, cost: replaces the existing chime, in practice: three 120 ms tones with gaps inside a 1.0 s file.
  2. **Keep the current 0.35 s chime** — gain: no asset change, cost: one longer tone, in practice: the current asset.
- Disposition: answered — three short beeps, overall one second.
