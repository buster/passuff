---
status: accepted
date: 2026-11-03
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Probe before observing, target changed pull requests, and reconcile the open set

## Context and Problem Statement

The terminal inbox previously re-read the persisted watch state on a timer and
depended on a separate `passuff` process to contact GitHub. The developer
watching the inbox is the consumer who cares most about freshness, yet that
design could show nothing new unless another process happened to be running.
GitHub offers no push channel to a desktop application, so freshness is bounded
by how often the account may be queried.

GraphQL cost is charged in points against 5,000 per hour. Measured on the
operator's account on 2026-11-03: one full check cycle (100 pull requests, all
states, comment/review/thread/CI connections) costs 57 points; a change probe
(open pull requests, identity, update time, head commit, CI state) costs 1.
A full cycle every second would exhaust the allowance in minutes.

## Decision Drivers

- The inbox must show new activity without a keypress or another command.
- Observation must not exhaust the account's allowance.
- One external boundary (GitHub GraphQL) and one read model (`InboxService`).
- The watch command's configured interval must keep its existing meaning.

## Considered Options

- Run the full check cycle on a short fixed timer.
- Run a cheap change probe and a full check cycle only on a detected change,
  pacing both against the reported allowance.
- Use GitHub webhooks or another push channel.
- Keep the inbox passive and rely on an external watcher.

## Decision Outcome

Chosen option: "Run a cheap change probe and a full check cycle only on a
detected change, pacing both against the reported allowance", because it is the
only option that makes the common case affordable while still asking GitHub at
a useful rate.

### Consequences

- Good, because an unchanged account costs 1 point per probe rather than 57, so
  the inbox can probe about once per second within the allowance.
- Good, because the inbox no longer depends on a second process to be useful.
- Bad, because the original implementation paid a 31-40 point full cycle; the
  2026-11-04 amendment reduces the changed case to the selected targets.
- Bad, because the probe fingerprint covers update time, creation time, state,
  head commit, and CI state only; a change that moves none of those waits for
  the reconciliation pass.
- Bad, because the inbox and the watch command may both observe and write state;
  the overlap is documented and left uncoordinated.

### Confirmation

`The inbox keeps observation cheap while nothing changes`,
`The inbox paces observation against the account allowance`,
`The inbox observes only the open pull request that changed`,
`The inbox reconciles open pull requests when the interval elapses`, and
`The inbox shows activity observed while it is open` execute the probe,
targeting, reconciliation, and pacing paths through the Gherkin harness;
`PassuffCore::probe` and `ObservationSession::next_delay` carry unit tests for
the fingerprint and the delay formula.

## Pros and Cons of the Options

### Full check cycle on a short fixed timer

- Good, because a change is classified as soon as it is detected.
- Bad, because 57 points per cycle exceeds the 5,000-point hourly allowance
  after about 87 cycles, so the inbox would stall within minutes.

### Cheap probe, full cycle on change, paced by the allowance

- Good, because the common case is affordable and the allowance is respected.
- Good, because the full cycle still runs with all the classification data.
- Neutral, because detection latency now depends on the probe delay.
- Bad, because the probe adds a second query shape to maintain.

### Webhooks or another push channel

- Good, because detection would be near-instant and nearly free.
- Bad, because a desktop application has no inbound endpoint and no hosted
  service is part of the product; this contradicts the local-only constraint.

### Keep the inbox passive and rely on an external watcher

- Good, because no new observation path is needed.
- Bad, because it preserves the dependency that made the inbox feel stale.

## More Information

The measured costs and the delay formula are recorded in the
`passuff-tui-live` design (`givn/archive/passuff-tui-live/design.md`). The
full-cycle query cost is tracked as technical debt TD-003; reducing it would
raise the ceiling for how often a changed account can be fully observed.

## Amendment (2026-11-04)

The original decision ran a full observation after every change signal. The
`passuff-observation-targets` change keeps the probe-before-observing decision
and refines what the cycle observes:

- The accepted and pending probe fingerprints are diffed per account. The
  cycle observes changed open, new open, transitioned, and short-lived pull
  requests, plus newly discovered open assigned/review-requested pull requests.
- A known terminal (merged or closed) pull request stops being observed after
  its transition. Comments and reviews after the terminal state do not notify.
  The transition, a short-lived pull request, and a reopen are still reported.
- The live inbox runs a reconciliation pass at startup and every
  `reconciliation_interval_secs` (default 3600) to refresh every watched open
  pull request and every open assigned/review-requested pull request.
- An account's probe baseline advances only after that account's selected ids
  were observed. A missing or failed observation keeps the change pending.

Measured 2026-11-04: a sparse change costs about 3-5 points instead of 31-40;
the probe tick remains 1 point; a reconciliation pass with no open pull
request costs the two discovery searches.

Consequences of the amendment:

- Good, because the cost of a detected change is proportional to the change,
  not to the size of the watch history.
- Good, because terminal activity no longer consumes the allowance.
- Bad, because terminal activity is never reported; accepted by the operator.
- Bad, because a change the fingerprint cannot see is only found by the
  reconciliation pass; the default one hour bounds the delay.
- Bad, because an involved (assigned/review-requested) pull request is only as
  fresh as the reconciliation interval.
