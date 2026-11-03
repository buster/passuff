---
status: accepted
date: 2026-11-03
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Probe before observing and pace the inbox against the account allowance

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
- Bad, because a changed account pays the 57-point full cycle; a busy account
  observes more slowly than an idle one.
- Bad, because the probe covers the 100 most recently updated pull requests in
  any state; a comment on an older merged or closed pull request is noticed by
  the next periodic full cycle rather than by the probe.
- Bad, because the inbox and the watch command may both observe and write state;
  the overlap is documented and left uncoordinated.

### Confirmation

`The inbox keeps observation cheap while nothing changes`,
`The inbox paces observation against the account allowance`,
`A comment on a merged pull request reaches the inbox`, and
`The inbox shows activity observed while it is open` execute the probe, pacing,
and update paths through the Gherkin harness; `PassuffCore::probe` and
`ObservationSession::next_delay` carry unit tests for the fingerprint and the
delay formula.

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
