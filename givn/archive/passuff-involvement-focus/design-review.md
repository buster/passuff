Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Design review: passuff-involvement-focus

## Grilling

### Q1 — Scope: does the spec contain exactly what the proposal asked for?

Proposal promises: record why a pull request is watched, persist a focus of four
relations, edit it on the watch-scope screen, keep fetching unchanged, filter the
inbox and notifications, keep unknown relations visible, and remove manual
entry. The deltas cover the picker rows and persistence, the e2e focus action,
the inbox and TUI filter with the unread count, the notification gate with the
retained activity and the widened focus, the modified toggle scenario, and the
removed manual-entry scenario. Unknown and empty focus are unit-tested. Nothing
extra is authored. Resolution: accepted.

### Q2 — Tech choices: is the smallest mechanism used?

Three candidates were considered for the notification gate: filtering in the
delivery layer, filtering in `InboxService`, and filtering in the cycle. The
cycle already has the stored relations and appends activity in the same step; a
delivery-layer gate would re-read the read model, and `InboxService` is not on
the notification path. The read model filter is a single map lookup. Discovery
returns relations with the node id instead of a second pass. Resolution:
accepted.

### Q3 — Missing scenarios: which observable behaviours are implied but absent?

Checked: (a) a pull request with several relations — covered by the discovery
deduplication unit test; (b) an unknown relation on an old state — unit test
(`includes` with an empty list) and the compatibility rule; (c) an empty focus —
unit test; (d) a failed team search — logged, no account failure, unit test of
the adapter's tolerant path; (e) a focus write failure — the existing scope write
failure path plus a `set_focus` unit test. None needs a Gherkin scenario because
each is a unit-level decision or an error path whose observable result is already
asserted elsewhere. Resolution: accepted.

### Q4 — Testability: can each scenario fail before the code exists?

The picker rows (`[x] authored`, `[x] team review`), the focus value in the
config, the absence of `Add:`/`owner/name`, the hidden pull request in the inbox
and the TUI, the unread count, the absent notification, and the retained
unread activity are concrete observable values. The new phrases start as
`unimplemented!`; the setup task records the failing run. The single-scenario
command is recorded. Resolution: accepted.

### Q5 — E2E fidelity: is the new `@e2e` scenario exactly one action?

The new action is "narrow the focus to a subset of involvement relations" on the
watch-scope screen. It is driven through the real binary
(`passuff-tui --snapshot --picker --focus-set ...`) and asserts the persisted
config and the rendered picker frame. The removed manual-entry action loses its
scenario; the modified toggle scenario keeps its `@e2e` tag and its assertions.
The inbox filter and the notification gate are variants of existing actions, so
they stay regular. No tag is removed to evade a gate. Resolution: accepted.

### Q6 — Risk: how does this most likely fail during implementation?

The most likely failure is a stale or missing relation on a pull request
observed in a targeted cycle, leaving it hidden when it should be visible. The
design states the refresh rule (replace when reported, keep otherwise, always
union authored) and the compatibility rule for empty relations. The second
likely failure is the permanent `Developer toggles a repository in the watch
scope` e2e scenario still using the manual-entry step after the step is deleted;
the design marks it modified, and the archive merge replaces it. Resolution:
accepted.

### Q7 — Use-case and Persona context

Three use cases are touched: `change-watch-scope` (R12–R15, R6, extensions,
interactions), `review-activity-inbox` (R9), and `watch-pull-requests` (R11,
minimal guarantee, R44). The confirmed Persona `multi-account-developer` is a
review lens only; the Gherkin actor stays "developer". No invented Persona.
Resolution: accepted.

### Q8 — Decision ledger

The proposal records no open question. The design records the three
implementation decisions it took. No seed question exists. Resolution: accepted.

### Q9 — Contracts and components

`InboxService` and `GitHubGateway` keep one implementing component each.
`WatchScope` keeps one implementing component (`ScopeEditor`) and changes its
surface: `set_focus` added, `add` removed. The picker and the scripted flags
remain its consumers. No component gains a second contract. Resolution:
accepted.

### Q10 — Decision timing

No carried decision and no speculative abstraction. The relation enum is the
current requirement for four known relations, not an extension point. The
unknown-relation case is a compatibility rule, not a future feature. Resolution:
accepted.

### Q11 — Visual design contract

`Interface: terminal`, 100x30 snapshot, the existing palette, a rendered picker
reference with the Focus section, and the confirmed Persona. A browser
classification does not hold: the ratatui frame is terminal text. Resolution:
accepted.

### Q12 — Deferrals

None. Resolution: no issue found.

### Q13 — ADR qualification

The assessment records no `NEW_ADR` and names the cheaper home: the capability
specs, `design.md`, and chapters 05/06/08. No component, authority, storage, or
compatibility boundary moves. Resolution: accepted.

### Q14 — Arc42 chapter diff

Re-derived before reading `arc42.md`: 05 (state), 06 (focus flow), 08
(involvement and focus), 10 (QS-033), 11 (R-036), 12 (glossary); 01, 02, 03, 04,
07, 09 no. `arc42.md` matches. All twelve chapter files exist with content and
no ASCII diagram. Resolution: accepted.

### Q15 — Ubiquitous language and structural model

The glossary gains involvement relation and focus and keeps the state/record
terms consistent. `docs/domain-model.md` is unchanged: the watched pull request
aggregate and its relationships are the same; the relation list is an attribute.
Resolution: accepted.

## Hardening applied

- `design.md` names `--focus-set` as the e2e surface, the exact removed
  surfaces, and the three search queries.
- `givn/specs/change-watch-scope/usecase.md` gains R12–R15 and the new
  interaction row; manual-entry extensions and examples are removed.
- `givn/specs/review-activity-inbox/usecase.md` gains R9.
- `givn/specs/watch-pull-requests/usecase.md` updates R11, the minimal
  guarantee, the out-of-scope note, and adds R44.

## Verdict

DESIGN-REVIEW: PASS
