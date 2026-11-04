Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Review: passuff-observation-targets

## Implementation evidence check

- **Empty step bodies:** 0 found across the 11 step files under
  `crates/passuff-cli/tests/steps/`. The four new phrases carry real
  assignments or assertions; the `@givn.removed` placeholder is filtered by
  `features_runner.rs` and has no step definition.
- **Scenario commits:**
  - `4dc4695` — Batch `terminal-retirement`; terminal-status and review
    scenarios; touches `crates/passuff-core/src/{cycle,github,domain}.rs` and
    the runner filter.
  - `3d9bcb0` — Batch `observation-targets` and Batch `reconciliation`;
    targeted observation, reconciliation scheduling, the gateway change, and
    the two live-inbox scenarios.
  - `42ccacf` — test-only: `relevant_targets`, `commit_probe`, and
    `reconciliation_interval_secs` unit tests.
  Both behavior commits contain production code, not comments or transcripts.
- **Promised components:** `Discovery` (`github.rs`), `ProbePullRequest`
  `state`/`created_at`, `ChangeProbe.label`, `ChangeSignal.failures`,
  `PassuffCore::{refresh_targeted, refresh_reconciliation}`,
  `relevant_targets`, `commit_probe`, the `last_check` cache, the
  `reconciliation_interval_secs` config field, `ObservationSession`
  scheduling, and the e2e query responder all exist. `MAIN_QUERY` and the
  old response structs are deleted.
- **Strict mode:** `.fail_on_skipped()` is set in `features_runner.rs`; the
  proof in `tasks.md` records exit 1 with 7 total, 0 passed, 7 failed before
  implementation.
- **@e2e tag integrity:** no delta scenario had an `@e2e` tag removed or
  added. The change adds no `@e2e` action; the permanent 16 e2e scenarios
  remain the action evidence.
- **E2E command and files:** `givn/commands.yaml` binds
  `verify.e2e_command: ./scripts/verify.sh --e2e`; the script exports
  `GIVN_RESULT_SCOPE=e2e`, so the scope is explicit. The invoked runner is
  `crates/passuff-cli/tests/features_runner.rs`, with e2e steps in
  `crates/passuff-cli/tests/steps/pr_activity_notification_e2e.rs` and the
  TUI-driven phrases in `inbox_navigation.rs`. No parallel e2e
  implementation exists in the tree.
- **E2E assertions:** every e2e Then asserts the real subprocess output
  (transcript lines, TUI snapshot text, or exit status), not repository
  state alone.
- **Design conformance:** the implementation matches the reviewed commands,
  file layout, and driver. The design's two illustrative type sketches were
  aligned with the code in `design.md` and design-review was re-run; the
  amendment records that no decision changed.

### Interaction coverage

| Inventory entry (usecase.md) | Design matrix row | `@e2e` scenario | Driver |
|---|---|---|---|
| watch-pull-requests / start continuous watching | yes | Developer starts watching and is notified of a new comment | real `passuff` subprocess + wiremock |
| watch-pull-requests / run one observation pass | yes | Developer runs a single check and sees the current activity | real `passuff check` |
| review-activity-inbox / leave the inbox open while activity arrives | yes | The inbox shows activity observed while it is open | real `passuff-tui --observe-for` + wiremock |
| review-activity-inbox / navigate the watched pull requests and activity | yes | Developer navigates the inbox panes | real `passuff-tui --snapshot` |

No inventory entry is unmapped, and no extra e2e action was introduced.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Probe-selected observation strategy | 04-solution-strategy | Yes | target selection | Batch `observation-targets` | `PassuffCore::probe`, `relevant_targets` | yes |
| Core, gateway, and session responsibilities | 05-building-block-view | Yes | Contracts and Components | Batches `observation-targets`, `reconciliation` | `cycle.rs`, `github.rs`, `live.rs` | yes |
| Check cycle and live observation flows | 06-runtime-view | Yes | Architecture impact | both batches | `run_cycle`, `ObservationSession::tick` | yes |
| Targeting and reconciliation concept | 08-crosscutting-concepts | Yes | Target selection, Scheduling | both batches | `relevant_targets`, `last_full` interval | yes |
| ADR-0007 amendment | 09-architecture-decisions | Yes | Decisions D1-D4 | design tasks | `ADR-0007` amendment section | yes |
| Thrift and freshness scenarios | 10-quality-requirements | Yes | Query shapes and costs | config and scenario tasks | delta scenarios, cost table | yes |
| TD-003 closure and new risks | 11-risks-and-technical-debt | Yes | Query shapes and costs | both batches | TD-003 removed, R-031/R-032/R-033 added | yes |
| Glossary terms and links | 12-glossary, docs/glossary.md | Yes | Data model, Decisions | docs task | glossary rows for the five terms | yes |

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

The specs, design, and code use the glossary terms: changed pull request,
observation target, reconciliation pass, reconciliation interval, and terminal
retirement. A sweep-scan of the change artifacts, docs, and README found no
remaining use of "sweep" as a competing term. The code symbols named by the
glossary exist (`relevant_targets`, `AppConfig::reconciliation_interval_secs`).

`docs/domain-model.md` records the structural assessment: observation targets
and the reconciliation pass are transient selections over the existing
aggregates; no boundary, aggregate, or relationship changed.

## Coverage classification

- **Dead code (bucket 1):** `MAIN_QUERY`, `GqlResponse`, `GqlData`, `GqlViewer`,
  and `GqlConnection` were deleted with the old page path. No new dead code was
  introduced in the change.
- **Missing test coverage (bucket 2):** none remaining. `relevant_targets` has
  eight unit tests covering open/new/unchanged/changed/terminal
  transition/reopen/short-lived/old-terminal. `commit_probe` has three tests
  for failed-account, missing-target, and observed-target commitment. The
  config field has default, explicit, and zero-value tests. The six delta
  scenarios cover the user-visible outcomes.
- **Hard to test (bucket 3):** none identified.

Measured before the added unit tests with the project coverage command
(branch instrumentation): 5263/6302 lines and 708/2148 branches in the merged
report. The archive re-measures the committed candidate and its receipt is
authoritative.

## E2E coverage

The interface is a terminal UI and CLI, not HTML/CSS/JS; the opposite
classification (Web UI) fails because no browser renders the product. The real
driver is the `passuff-tui`/`passuff` subprocess against wiremock, and the
primary assertions are on the subprocess output. No delta action needs a new
e2e scenario; the existing four inventory actions remain the smoke evidence.
Both required executions report zero failed and skipped: regular
`givn test-result: total 6, passed 6`, e2e `total 16, passed 16`.

## Deferrals

None. No mandatory check was deferred or tag removed.

## Use-case and Persona conformance

`watch-pull-requests` gains R40 (post-terminal activity is silent) and R41
(selected targets and reconciliation). `review-activity-inbox` T10 now names
targeted observation and the configurable reconciliation interval. The goal,
actors, trigger, preconditions, extensions, and capability placement are
unchanged; no mutation was added under the observing goal. Actor and Persona
roles stay distinct. The scope rejection criteria were checked: no goal
conjunction, no missing actor, no channel-shaped capability, no inconsistent
precondition, and no extension without a main-flow reference.

Persona: `multi-account-developer` (confirmed) — relevance is allowance spend
and freshness of the watched open set; evidence is the live-inbox scenarios
and the measured cost change.

## Visual review

- Interface: terminal (ratatui); the change does not alter the rendered screen.
- Evidence: the committed transcripts under
  `givn/changes/passuff-observation-targets/evidence/visual/` for the e2e
  scenarios (watch state, TUI open, activity detail, browser launch, stack,
  inbox panes, live activity, unread list).
- The reviewer read the transcripts; the watched and activity panes, focus
  frame, and footer remain reachable and unchanged for the Persona.
- No screen was introduced or visibly changed, so no new style tile applies.

VISUAL-REVIEW: PASS

## Overlap dispositions

| Finding pair | Decision |
|---|---|
| Removed `A comment on a merged pull request reaches the inbox` / added `Activity on a merged or closed pull request does not notify` | variant — the added scenario supersedes the removed behavior for the same interaction |
| `Activity on a merged or closed pull request does not notify` / `A review on a merged pull request does not notify` | variant — one event family each |
| `The inbox reconciles open pull requests when the interval elapses` / permanent `The inbox shows activity observed while it is open` | boundary — the new scenario proves the interval path, the permanent one the probe path |

## Split-or-keep

| Scenario | Decision |
|---|---|
| The inbox reconciles open pull requests when the interval elapses | keep — the interval setup, the startup pass, and the unseen comment are one timing behavior |

## README impact

README-IMPACT: updated - Quick start, Run, Usage, Coverage

The configuration example names `reconciliation_interval_secs`, the inbox
paragraph describes targeted observation and the reconciliation interval, the
baseline paragraph records terminal retirement, and the coverage numbers were
refreshed.

## Sign-off

- Fabrication audit: clean; no empty step bodies or tag manipulation.
- Every checked task maps to a commit that touches production code or a
  regression test.
- Every promised component exists; consumers use the declared contracts.
- Strict-mode proof present.
- Scenario execution evidence: delta 6/6, e2e 16/16, zero skips.
- Coverage gaps: classified and resolved; branch instrumentation used.
- No `@wip` tags remain; no implementation detail in the specs.
- Interaction coverage and the real-interface driver verified.
- No finding was excused outside the three coverage buckets.

PERSONAS: multi-account-developer
README-IMPACT: updated - Quick start, Run, Usage, Coverage
VISUAL-REVIEW: PASS
REVIEW: PASS
