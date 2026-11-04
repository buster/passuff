Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Design review: passuff-observation-targets

The operator directed this session to proceed through archive without further
questions. Each grilling question below was resolved from the recorded operator
decisions, the use-case contract, or the codebase; none remain open.

## Grilling

Questions are ranked by how likely they are to invalidate the plan.

### Q1 — Does target selection miss a change the developer cares about?

Open changes are covered by the fingerprint (`updatedAt`, `headOid`, CI state,
state). Transitions are covered by the state diff. Short-lived pull requests are
covered by a new terminal id whose `createdAt` is after the account's last
check. Reopens are a terminal-to-open transition. Assigned/review-requested
pull requests are covered by discovery and the reconciliation pass. Terminal
activity is deliberately not covered (R40, operator decision).

Residual gap: a watched open pull request that is missing from the probe (for
example beyond the probe page cap) is still refreshed by the reconciliation
pass from the state's watched-open set, so its transition is observed. No
blocker.

### Q2 — Does the probe carry enough data to select targets without another query?

Yes. `state` and `createdAt` are added to `ProbePullRequest`; the fingerprint
already carries `updatedAt`, `headOid`, and CI state. `merged` maps the terminal
state. No extra query is needed; the probe was measured at 1 point and the added
fields do not change the shape.

### Q3 — Can a failed or partial observation silently consume a change?

No by design, and the plan now protects it explicitly:

- per-account baseline commitment: a failed account keeps its previous probe
  entry;
- a requested id missing from the response keeps its previous entry (or stays
  absent when new), so it is targeted again;
- a state-save failure commits nothing.

A new Gherkin scenario, `A failed observation keeps the change pending`, asserts
the retry. This was the most likely failure mode; it is now both an invariant
and a scenario.

### Q4 — Does the headless watcher change cadence?

No. `poll_interval_secs` still governs `passuff watch`, and each tick is a
reconciliation pass. `passuff check` performs one reconciliation pass. The
configurable hourly interval governs the live inbox, whose probe cadence is
allowance-paced. The proposal and design state this split.

### Q5 — Does removing `MAIN_QUERY` lose stack handling or discovery?

No. The stack retry moves to `NODES_QUERY`; discovery keeps both search queries,
now restricted to open pull requests. The design records both facts.

### Q6 — Is the reconciliation interval observable and testable?

Yes. `reconciliation_interval_secs` is top-level, default 3600, minimum 1. The
live inbox schedules startup and interval passes; the delta scenario configures
2 seconds and observes for 6 seconds. The below-100-point deferral prevents a
short interval from bypassing the allowance. The default has a config unit test
rather than a one-hour Gherkin wait.

### Q7 — Where do the 52 terminal watch records go?

They stay in watcher state and remain visible in the watched pane. Hiding them
is the separately recorded D2 decision and is out of scope here. Activity
detail keeps resolving them.

### Q8 — Does the interaction inventory change?

No new consumer action is introduced. The reconciliation is a timing variant of
"leave the inbox open while activity arrives"; the existing `@e2e` scenario
remains the action evidence, and the new scenarios are regular. The matrix maps
both new behaviors to that inventory entry. No `@e2e` tag is removed.

### Q9 — Does the expected cost hold?

A probe tick is 1 point. A sparse targeted pass is two searches plus one to
three observations, about 3-5 points. A reconciliation with no open pull request
is two search points; with open and involved pull requests it scales with their
count. The old page observed up to 75 ids; the new pass never fetches a terminal
record. The budget formula and the below-100-point deferral are unchanged.

### Q10 — What is the single most likely implementation failure?

A probe/refresh desynchronization: the accepted probe advances while a selected
id was not observed, so the next probe no longer reports the change. The
per-account commit and missing-id retention close it; Q3's scenario and unit
tests protect it.

## Branch findings

- **Scope:** the proposal asks for terminal retirement, targeted observation,
  and a configurable reconciliation sweep. The specs contain exactly those and
  nothing extra; hiding terminal rows is explicitly out of scope. No mismatch.
- **Tech choices:** the existing Rust, tokio, reqwest, cucumber-rs, wiremock,
  and ratatui stack fits. No new dependency. Simpler alternative (a fixed
  smaller page) does not deliver the terminal retirement or the target
  identity, so it was rejected.
- **Missing scenarios:** the failure-retry scenario was added during hardening.
  Terminal retirement, targeted observation, and reconciliation are covered.
  Reopen, short-lived, and terminal-transition behavior remain pinned by the
  permanent scenarios.
- **Testability:** every Then-step asserts a transcript, a TUI snapshot, an
  observed-id set, or an exit status. `.fail_on_skipped()` is set in
  `features_runner.rs`; delta scenarios stay `@wip` until implemented. The
  single-scenario command is `GIVN_SCENARIO="<title>" GIVN_FEATURES=<dir> cargo
  test -p passuff-cli --features test-support --test features_runner`.
- **E2E fidelity:** read `givn instructions specs --change
  passuff-observation-targets`. The real interface is the `passuff-tui`
  subprocess driven with `--observe-for` against wiremock; the assertions are
  on the rendered TUI. No browser driver applies. No duplicate action coverage.
- **Interaction coverage:** the inventory lives in the use-case documents; the
  design matrix covers every entry touched. No new entry is needed.
- **Risk:** Q10; mitigated by the commit invariant, the retry scenario, and
  unit tests. Residual risks R-031/R-032/R-033 are recorded.
- **Use-case and Persona:** use case `watch-pull-requests` (capability
  `pr-activity-notification`) and `review-activity-inbox` (capability
  `inbox-navigation`); Persona `multi-account-developer` is confirmed in both
  documents. No invented Persona. No Persona biography is used as an actor.
- **Decision ledger:** D1-D3 are answered with the operator's decisions; D4 is
  decided in `design.md` with default, trigger, switch cost, and owner. No
  unresolved `D<n>`.
- **Contracts and components:** `GitHubGateway` (one adapter), `PassuffCore`
  (one component), `LiveObservation` (`ObservationSession`), and
  `InboxService` (`Inbox`) each have one contract and one responsibility. No
  consumer imports an implementation. The core replaces its adapter without
  consumer changes.
- **Decision timing:** all D questions are settled; no scenario depends on a
  carried answer. The config field is decided, not carried. No speculative
  abstraction.
- **Visual design contract:** the screen is unchanged; `Interface: terminal`,
  100x30, no new rendered reference. The change is behavioral, so the section
  records that no rendered state changes.
- **Deferrals:** none. The terminal-row hiding is out of scope by decision, not
  a deferred mandatory check. No `@e2e` tag is removed.
- **Persona disposition:** `multi-account-developer` — relevance is allowance
  spend and freshness of the watched open set; evidence is the live-inbox
  scenarios and the cost measurement.
- **Interface classification:** the surface is a ratatui terminal, not HTML/CSS/
  JS; no browser exists. The real driver is the TUI subprocess. The opposite
  classification fails because no browser or rendered web document exists.
- **ADR qualification:** `AMEND_ADR` for ADR-0007. The qualification is in
  `arc42.md`; the cheaper home (ADR-0007 plus specs and design) is named. No
  `NEW_ADR`.
- **Architecture documentation:** I re-derived all 12 rows independently before
  opening `arc42.md`; the recorded rows match. Chapters 04, 05, 06, 08, 09, 10,
  11, and 12 were read after the edits and match the design. All 12 chapter
  files exist with real content; no placeholder text. The box-drawing scan is
  empty. ADR-0007 is in the register and amended; R-031/R-032/R-033 are in
  chapter 11.
- **Ubiquitous language:** `docs/glossary.md` now holds changed pull request,
  observation target, reconciliation pass, reconciliation interval, and
  terminal retirement, and the updated check cycle and change probe definitions.
  The specs and design use those terms.
- **Structural model:** `docs/domain-model.md` records that targets and the
  reconciliation pass are transient selections over the existing aggregates;
  no boundary or aggregate changes. The assessment is explicit.

## Hardening applied

- Target selection applies the current watch scope and ignore list.
- `MAIN_QUERY` removal and the `NODES_QUERY` stack retry are recorded.
- R40/R41 and the failure-retry scenario were added to the traceability.
- `A failed observation keeps the change pending` was added to the delta specs.
- No `@e2e` tag was removed; no mandatory check was deferred.

DESIGN-REVIEW: PASS
