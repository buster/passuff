---
status: accepted
date: 2026-11-02
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Poll authored pull requests through the GitHub GraphQL API

## Context and Problem Statement

passuff must observe only pull requests authored by one or more configured
GitHub accounts, across all readable repositories, and must work behind NAT
without a public endpoint. The observation mechanism determines the deployment
boundary, the credential model, the notification latency, and the rate-limit
budget. The alternatives are polling authored pull requests, reading the
GitHub notifications feed, and receiving webhooks.

## Decision Drivers

- Deterministic "only pull requests I authored" filtering.
- No inbound network surface or hosted relay.
- Multiple accounts with independent credentials.
- Read-only GitHub access.
- Low operational complexity for a single-operator local tool.
- Predictable behaviour under restarts and missed events.

## Considered Options

- Poll authored pull requests through the GitHub GraphQL API.
- Read the GitHub notifications feed.
- Receive GitHub webhooks through a local or hosted endpoint.

## Decision Outcome

Chosen option: "Poll authored pull requests through the GitHub GraphQL API",
because it provides deterministic authored-only filtering without requiring an
inbound endpoint or relying on GitHub notification delivery settings.

### Consequences

- Good, because the watcher has no public endpoint and works behind NAT.
- Good, because the configured login set is applied to the actual author, so
  other people's pull requests cannot enter the notification path.
- Good, because one query per account per cycle returns comments, reviews, CI
  state, and pull request state together.
- Bad, because a change can take up to one observation interval to appear.
- Bad, because polling consumes the per-token GraphQL rate limit.
- Bad, because the adapter is coupled to the GitHub GraphQL schema.

### Confirmation

The `GitHubApiAdapter` integration tests assert one outbound query per account
per cycle and exercise a 429 response. The E2E start and check scenarios run the
real binary against the local fake GitHub, while the regular other-PR scenario
verifies the authored-only filter. The binary binds no listening port, and the
dependency tree contains no server framework.

## Pros and Cons of the Options

### Poll authored pull requests through the GitHub GraphQL API

- Good, because authored-only filtering is explicit and testable.
- Good, because no public endpoint, webhook secret, or GitHub App is required.
- Good, because the same query supports comments, reviews, CI, and state.
- Neutral, because the polling interval is configurable.
- Bad, because latency is bounded by the interval.
- Bad, because the GraphQL schema is an external dependency.

### Read the GitHub notifications feed

- Good, because GitHub already aggregates activity.
- Bad, because the feed depends on repository watch settings and read state.
- Bad, because it includes review requests, mentions, and other people's pull
  requests, which the product must exclude.
- Bad, because `ci_activity` does not deterministically cover every CI result
  on an authored pull request.

### Receive GitHub webhooks

- Good, because delivery is near real time.
- Good, because GitHub performs the event classification.
- Bad, because it requires a public or relayed endpoint, which violates the
  self-contained local constraint.
- Bad, because it requires a webhook secret, delivery retries, and a server
  lifecycle.
- Bad, because local development and testing need a tunnel or hosted relay.

## More Information

- [Use case: watch-pull-requests](../../../givn/specs/watch-pull-requests/usecase.md)
- [Building block view](../05-building-block-view.md)
- [ADR register](../09-architecture-decisions.md)
