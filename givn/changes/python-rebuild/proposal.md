# Proposal: python-rebuild

## Release note

passuff is rebuilt from its permanent specification and keeps every specified
behavior unchanged.

## Use-Case Context

- Use-case ID: `watch-pull-requests`, `inspect-watch-status`,
  `review-activity-inbox`, `review-pull-request-stacks`, `change-watch-scope`,
  `set-up-passuff` — all six permanent contracts.
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — confirmed 2026-11-02 by the
  operator's brief; the rebuild preserves this Persona's outcomes without
  changing them.

## Review Context

- Review kind: independent
- Reason: The rebuild carries no user-visible behavior change, but it touches
  all seven capabilities and the whole implementation blast radius, which the
  review instruction treats as a major change. The boundary delta completes
  executable coverage of behavior already specified at the rule and reference
  level. One fresh reviewer context covers design-review and final review; the
  author does not review its own plan.

## Problem / Opportunity

The product's implementation no longer exists in the working tree. Only the
permanent specification survives: seven feature files, six use cases, the arc42
chapters, the glossary, the domain model, and the reference. The specified
product cannot run, cannot be installed, and cannot demonstrate that its
behavior is still available.

The specification claims to be sufficient to rebuild the product. That claim is
currently untested. The rebuild is the Ise test: delete the code, keep the
specification, rebuild, and see whether the same product emerges. If the
rebuild needs knowledge that exists nowhere in the specification, that
knowledge is a specification defect, not an implementation detail.

Impact: until the rebuild, the developer has no watcher, inbox, setup flow, or
release. After the rebuild, every specified behavior is available again,
unchanged, and the specification has evidence that it is complete.

## Proposed Solution

Rebuild the product from the permanent specification alone:

- The seven permanent feature files remain the behavioral acceptance
  contract. A bounded delta adds executable scenarios for behavior already
  specified in the use-case rules and the reference: invalid and unknown
  configuration, GitHub response failures and the stack retry, state recovery
  boundaries, CLI usage errors, delivery-surface failures, and the
  80-character list-row excerpt. No delta scenario changes an existing
  behavior.
- The rebuilt product installs and runs through the specified surfaces: the
  command-line watcher, the one-check command, status, inbox and read
  commands, setup, the notification test, and the terminal inbox.
- Acceptance requires the 185 permanent scenarios and the bounded delta to
  pass: 167 permanent non-E2E, 18 permanent E2E, and every delta scenario. A
  scenario that is undefined, pending, or skipped fails acceptance.
- The activity detail keeps showing the selected comment or review body, as
  T14 and the long-comment scenario already require; the bounded excerpt
  applies to the list row. The glossary and runtime view are corrected to
  match the executable contract.
- The specification's stale implementation-symbol references are refreshed to
  the rebuilt symbols, the architecture introduction is corrected to route all
  six use cases, and duplicate rule numbers and a compound use-case goal are
  corrected. These are corrections to the durable specification, not behavior
  changes.
- The macOS `Passuff.app` bundle is deferred to a separate packaging change.
- Acceptance does not require a live GitHub account or a desktop session.

## Capability Routing

`givn spec route` reported no signal: the top candidates tie, and no ranked
list is printed. The decision below is recorded directly.

| Proposed capability | Decision | Rationale |
|---|---|---|
| `pr-activity-notification` | `EXTEND pr-activity-notification` | The behavior is permanent; the delta makes already-specified configuration, GitHub, state, CLI, and delivery error paths executable. |
| `watch-status` | `REUSE` permanent capability, no delta | The status read model is already specified; the rebuild reproduces it. |
| `activity-inbox` | `EXTEND activity-inbox` | The delta adds the E2E body assertion that T14 and the long-comment scenario already require. |
| `inbox-navigation` | `EXTEND inbox-navigation` | The delta adds the 80-character list-row excerpt and the browser-opener failure path. |
| `stacked-inbox` | `EXTEND stacked-inbox` | The delta makes the already-specified stack-field rejection and one-retry fallback executable. |
| `watch-scope` | `EXTEND watch-scope` | The delta makes the already-specified all-mode deselect rejection and duplicate-scope rejection executable. |
| `initial-setup` | `REUSE` permanent capability, no delta | The first-run prompt flow is already specified; the rebuild reproduces it. |

The change no longer skips the `specs` artifact. The delta is a bounded spec
completion: it adds scenarios for already-specified behavior, not new behavior.

## Out of Scope

- No user-visible behavior change. The boundary delta only makes
  already-specified error and boundary behavior executable; the activity-detail
  body correction aligns the durable documentation with the existing T14 and
  long-comment scenario.
- Changing configuration keys, the state schema version, exit codes,
  keybindings, notification wording, or terminal layout.
- Windows support, hosted services, webhooks, or new delivery channels.
- The macOS `Passuff.app` bundle; it is deferred to a separate packaging
  change.
- Changing the permanent scenario behavior or removing any `@e2e` tag.
- Reworking the architecture described by arc42. Only stale
  implementation-symbol references, duplicate rule numbers, a compound
  use-case goal, and the use-case route in the architecture introduction are
  corrected.

## Open Questions

None. The independent design review resolved the boundary-delta, activity-body,
E2E-step, unknown-key, macOS-bundle, and use-case-integrity decisions with the
operator; the resolutions are recorded in `design-review.md`. Technology
choices are design-level and are recorded in `design.md`.
