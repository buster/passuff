Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)
PERSONAS: multi-account-developer
README-IMPACT: none

# Review: passuff-activity-time

## Implementation evidence check

- **@e2e integrity:** the two delta `.feature` files contain 0 `@e2e` scenarios
  and no scenario's tags were changed. The change adds no user action; the
  affected capabilities keep their permanent `@e2e` scenarios unchanged.
- **Empty step bodies:** 0 across 10 step files. The four new phrases assert a
  concrete time field, a `~` marker, or the popup text; no `unimplemented!` or
  `todo!` remains in `crates/passuff-cli/tests/steps/`.
- **Scenario commits:** all 8 scenarios record
  `COMMIT: 9f3c9c5 -- Batch: activity-time`. `git show --stat 9f3c9c5` contains
  the classifier, cycle, read-model, renderer, step, and fixture changes.
- **Strict-mode proof:** `tasks.md` records the pre-implementation run exiting 1
  with `total 8, passed 0, failed 8, skipped 0` on undefined phrases. The runner
  keeps `.fail_on_skipped()` and fails on `total == 0`.
- **Components promised by `design.md`:** `ActivityRecord.occurred_at`,
  `AuthoredPullRequest.merged_at` / `closed_at`, the `mergedAt` / `closedAt`
  query fields, `ClassifiedEvent`, the cycle's record construction,
  `effective_time` / `is_approximate`, the transcript line, `short_record_time`,
  and `activity_time_line` all exist.
- **Contracts:** consumers use `InboxService`; `TuiApp` and `InboxCli` import the
  core's read-model types, not the `Inbox` implementation. `classify` remains an
  internal helper. No consumer imports an adapter implementation.
- **@e2e Then steps:** the permanent `@e2e` scenarios assert CLI stdout and the
  rendered TUI frame; none asserts repository state only.
- **Browser UI:** not applicable; the interface is a ratatui terminal frame.
- **E2E command:** `./scripts/verify.sh --e2e` exports `GIVN_RESULT_SCOPE=e2e`
  and runs `cargo test -p passuff --features test-support --test features_runner`.
  The e2e scope ran 17 scenarios, 0 failed, 0 skipped.
- **Explicit scope:** `scripts/verify.sh` selects the scope with
  `GIVN_RESULT_SCOPE` and falls back to the permanent corpus when the delta has
  no `@e2e` scenario.
- **Design conformance:** the implementation matches the named step files,
  commands, and rendering rules. One compatibility addition: the permanent
  phrase `the inbox shows an activity id and observation time` stays bound
  through a `(?:observation|occurrence)` alternation until the archive replaces
  the permanent scenario. This keeps the pre-merge corpus green and is recorded
  in `tasks.md`; it changes no reviewed decision.

### Interaction inventory cross-reference

| Inventory entry | Matrix row | @e2e scenario | Driving mechanism |
|---|---|---|---|
| activity-inbox / list unread activity | present | Developer lists unread activity (permanent) | real `passuff inbox` subprocess |
| activity-inbox / mark activity read | present | Developer marks activity read (permanent) | real `passuff read --all` subprocess |
| activity-inbox / open the inbox in the terminal UI | present | Developer opens the inbox TUI (permanent) | real `passuff-tui --snapshot --observe` |
| activity-inbox / open an activity's detail | present | Developer opens an activity's details (permanent) | real `passuff-tui --snapshot --keys "enter"` |
| activity-inbox / open the pull request from an activity | present | Developer opens the pull request from an activity (permanent) | real `passuff-tui --snapshot --keys "enter,enter"` with a fake opener |
| inbox-navigation / navigate the watched pull requests and activity, with automatic refresh | present | Developer navigates the inbox panes (permanent) | real `passuff-tui --snapshot --keys` |
| inbox-navigation / leave the inbox open while activity arrives | present | The inbox shows activity observed while it is open (permanent) | real `passuff-tui --snapshot --observe-for` against a live wiremock change |

No inventory entry is unmapped, no extra `@e2e` scenario was added, and no
driving mechanism changed.

## Architecture conformance from the first implementation

- The occurrence time is captured at classification, where the upstream
  timestamps already exist; no second request and no adapter change beyond two
  fields on the existing query.
- `ActivityEvent` and `Notification` are unchanged, so notification grouping and
  delivery keep their existing contract and tests.
- The CLI keeps rendering; `InboxService` exposes only `effective_time` and
  `is_approximate` as free helpers over the record, not a new component.
- `TuiApp` still reads only `InboxView` and the record; it writes no state and
  spawns no process.
- Dependency direction is unchanged: `passuff-core` gains no CLI or terminal
  dependency; `cargo tree -p passuff-core` is unchanged.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Activity record carries both times | 08 Activity history / Activity time | Yes | Data model | Core batch | `domain.rs` `ActivityRecord.occurred_at` | Yes |
| Occurrence time per event kind | 08 Activity time | Yes | Occurrence time selection | Core batch | `classifier.rs` timestamp selection | Yes |
| Merge and close time from the source | 03 context (GitHub interface) | No (interface unchanged) | Technology decisions | Core batch | `github.rs` `mergedAt` / `closedAt` | Yes |
| Effective-time ordering | 06 Record activity | Yes | Rendering and ordering | Core batch | `inbox.rs` sort + `record_time` | Yes |
| Fetch-time fallback marker | 08 Activity time | Yes | Rendering and ordering | Core + TUI batches | `transcript.rs`, `short_record_time` | Yes |
| Popup shows both times | 06 Record activity / 05 Inbox | Yes | Rendering and ordering | TUI batch | `activity_time_line`, popup path | Yes |
| QS-032 accuracy scenario | 10 quality | Yes | Use-case traceability | Scenarios | delta scenarios + unit tests | Yes |
| R-035 ordering risk | 11 risks | Yes | Risks | Decisions | `~` marker and same-time ordering | Yes |
| Occurrence and fetch time terms | glossary | Yes | Data model | Core batch | `docs/glossary.md` rows | Yes |

Chapters 01, 02, 04, 07, and 09 remain unaffected as claimed: no goal,
constraint, strategy, deployment, or ADR boundary changed. `docs/domain-model.md`
describes the same aggregate; no relationship changed, so it is correctly
unchanged.

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

`docs/glossary.md` defines occurrence time and fetch time, and the activity
record and detail definitions now use both. The specs, design, code, and
transcript prose use those terms consistently; no old "observation time" phrase
remains in the delta, the use case, the arc42 chapters, or the CLI help. The
permanent scenario's old step phrase survives only as the documented pre-merge
compatibility alias.

## Coverage

`./measure-coverage.sh` (branch instrumentation, nightly toolchain) ran on the
candidate before this review: regular scope 8/8 scenarios, e2e scope 17/17, both
exit 0. Merged coverage is 6,192/7,269 lines (85%) and 826/2,322 branches
(36%). Gaps found by inspection:

| Region | Classification |
|---|---|
| `github.rs` `mergedAt` / `closedAt` mapping | Covered by `observe_maps_authored_pull_requests` and the new classifier timestamp tests. |
| `classifier.rs` empty `submittedAt` | Covered by `a_review_without_a_submitted_time_has_no_occurrence_time`. |
| `inbox.rs` unparseable timestamp fallback | Exercised through the existing malformed-state tests; no new branch added. |
| `tui.rs` popup line for a pull request not in view | Pre-existing branch; the record-only time line is rendered before the changes list in both branches. |

No dead code was introduced: `ClassifiedEvent` and both read-model helpers are
used by production code, and `effective_time` is used by the orchestrator and
both renderers.

## Findings

| ID | Finding | Disposition |
|---|---|---|
| F1 | The permanent step phrase would become undefined between the delta and the archive merge. | Bound both phrases with the `(?:observation|occurrence)` alternation; the permanent suite is green now (185/185). |
| F2 | The TUI approximate assertion could match a row other than the activity row. | The step requires the event summary on the same line as the `~`; the watched row carries the summary only in the CI cell without a tilde, so the assertion is specific to the activity row. |
| F3 | State-change occurrence times have no Gherkin scenario. | The classifier unit test covers the merge time, and the GraphQL mapping test covers `mergedAt`; the display rule is the same code path asserted by the comment and CI scenarios. Classified as covered, not a gap. |
| F4 | `measure-coverage.sh` and `scripts/verify.sh` detected `@e2e` in change prose, so the e2e coverage scope ran the delta (0 scenarios) instead of the permanent corpus and failed. | Fixed in `1296b4d` with an `--include='*.feature'` filter; the measured run then reported 8 regular and 17 e2e scenarios, both exit 0. |
| F5 | The corpus lint gate failed on a pre-existing unlisted `@e2e` scenario (`The developer tests notifications on the command line`). | Added the missing `notify-test` interaction row in `1768ba5`; `givn lint` exits 0. |

No critical finding remains.

VISUAL-REVIEW: PASS

REVIEW: PASS
