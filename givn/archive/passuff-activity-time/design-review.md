Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Design review: passuff-activity-time

## Grilling

### Q1 — Scope: does the spec contain exactly what the proposal asked for?

The proposal promises the occurrence time in the table, the `passuff inbox`
line, and the popup, the `~` marker for unreported times, ordering by the shown
time, and compatibility with earlier state. The delta covers each: the CLI
line (`Inbox shows when a comment was written`), the marker
(`Inbox marks an event without a reported time as approximate`), ordering
(`Inbox orders activity by when it happened`), the TUI cell
(`The activity table shows when the activity happened`), the marker in the
table, and both popup lines. Compatibility is covered by the serde default and
a unit test, not by a scenario: an old state file has no occurrence time and
therefore renders exactly like the approximate case already asserted.
Resolution: accepted.

### Q2 — Tech choices: is the smallest mechanism used?

Three candidates were considered. Storing the time inside `ActivityEvent`
changes notification construction and every event literal. A parallel vector of
times desynchronizes from the events. `ClassifiedEvent` pairs the two at the one
place that has both, and leaves `ActivityEvent`, `Notification`, and the
transcript renderer untouched. `mergedAt`/`closedAt` are added to the existing
node query instead of a second request. Resolution: accepted, no simpler option
satisfies both a real merge time and unchanged grouping.

### Q3 — Missing scenarios: which observable behaviours are implied but absent?

Four were checked. (a) An exact occurrence time that equals the fetch time: the
popup then omits `Fetched:`; not a scenario because it is a display variant of
the asserted rule and would require a clock equal to the upstream time. (b) A
state file written before this change: unit-tested, no scenario, because the
observable result is identical to the approximate case. (c) A review with an
empty `submittedAt`: unit-tested in the classifier. (d) The CLI `inbox --all`
line: the same renderer as `inbox`, already asserted through the shared line.
Resolution: accepted; no scenario added.

### Q4 — Testability: can each scenario fail before the code exists, and are the
Then-steps concrete?

The four new scenarios assert concrete text (`2026-11-02T09:15:00Z`, an
approximate marker, `11-02 09:15`, `When:`/`Fetched:`) and the runtime order of
two named comments. The new step bodies start as `unimplemented!`; the setup
task records the failing run. The single-scenario command is recorded in
`design.md`. Strict mode is `.fail_on_skipped()` plus the runner's `total == 0`
failure. Resolution: accepted.

### Q5 — E2E fidelity: is the absence of a new `@e2e` scenario justified?

The change adds no user action. The table cell, the popup line, and the CLI line
are variants of the existing actions already covered by `Developer opens the
inbox TUI`, `Developer opens an activity's details`, and `Developer lists unread
activity`. The Interaction Coverage Matrix keeps one row per inventory entry and
no entry is added or removed. Marking a second scenario `@e2e` for the same
action would be over-coverage. Resolution: accepted.

### Q6 — Risk: how does this most likely fail during implementation?

The most likely failure is an ordering regression in the permanent
`Inbox lists unread activity newest first` scenario: its comment fixture is
created at 12:00:01 and its check runs at 12:00:02, so the CI failure (fetch
time) still sorts first. The design states the effective-time ordering keeps
that outcome; the modified scenario is carried in the delta so the merge cannot
silently drop the assertion. The runner change for `record_lines` is named in
the testing strategy so the approximate record still counts. Resolution:
accepted.

### Q7 — Use-case and Persona context

The owning use case is `review-activity-inbox`; the capabilities are
`activity-inbox` and `inbox-navigation`. R1 and T14 are amended in place, and
T28 is added; the goal, guarantees, Persona reference, and interaction inventory
are preserved. The confirmed Persona `multi-account-developer` appears only as
a review lens, never as a Gherkin actor. Resolution: accepted.

### Q8 — Decision ledger

The proposal records no open question, so the ledger has nothing to resolve; the
design records the two implementation decisions it took. No seed question
exists. Resolution: accepted.

### Q9 — Contracts and components

`InboxService` and `GitHubGateway` are the only contracts with consumers
outside their components; each has exactly one implementing component. The
initial draft listed `classify` as a contract; it is an internal helper of the
cycle with no external consumer and was removed from the table. No component
gains a second responsibility: the core keeps classification and state, the CLI
keeps rendering. Resolution: accepted after the hardening edit.

### Q10 — Decision timing

No carried decision and no speculative abstraction: the record field is the
current requirement, not a hook for a future feature. Resolution: accepted.

### Q11 — Visual design contract

`Interface: terminal`, viewport 100x30, the existing palette rules (T25), a
rendered table and popup reference, and the confirmed Persona. A browser
classification does not hold: the surface is the ratatui frame rendered into a
terminal, not HTML/CSS/JS. Resolution: accepted.

### Q12 — Deferrals

None. The change defers no mandatory check. Resolution: no issue found.

### Q13 — ADR qualification

The assessment records no `NEW_ADR`, and the cheaper home is named: the record
shape and the marker are owned by `design.md`, the capability spec, and chapters
05/06/08. No component, authority, storage, or compatibility boundary moves.
Resolution: accepted.

### Q14 — Arc42 chapter diff

Re-derived independently before reading `arc42.md`: 05 yes (state description),
06 yes (record flow), 08 yes (time concept), 10 yes (quality scenario), 11 yes
(ordering risk), 12 yes (glossary); 01, 02, 03, 04, 07, 09 no. `arc42.md`
matches every row. All twelve chapter files exist with real content and no
ASCII diagram; chapter 12 links the glossary, which now holds occurrence time
and fetch time. Resolution: accepted.

### Q15 — Ubiquitous language and structural model

`docs/glossary.md` gains occurrence time and fetch time and updates the activity
record and detail definitions. `ActivityRecord` keeps its identity and
ownership; `docs/domain-model.md` describes the same record aggregate, so no
relationship changes. Resolution: accepted.

## Hardening applied

- `design.md` `## Contracts and Components`: `classify` removed from the
  contract table and recorded as an internal helper.
- `design.md` popup rule: `Fetched:` appears only when the formatted fetch
  time differs from the occurrence time, matching the proposal.
- `design.md` testing strategy: names the `record_lines` timestamp-regex update.
- `givn/specs/review-activity-inbox/usecase.md`: R1, T14 amended, T28 added to
  match the final display rule.

## Verdict

DESIGN-REVIEW: PASS
