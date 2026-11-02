# Review: passuff-activity-inbox

Review kind: independent
Author: passuff maintainer (operator)
Reviewer context: independent review analyst (fresh context), re-review after the second FAIL
PERSONAS: multi-account-developer

This pass ran in its own fresh context with no author history. It changed no
production code, test, spec, design, task, ADR, glossary, coverage, or README
file. It re-ran the suites and the mechanical checks below; the only file it
wrote is this review.

Result: **PASS**. The five blockers from the prior FAIL are closed and no new
blocker was found. `givn check review --change passuff-activity-inbox` exits 0
after this verdict.

## Blocker closure

| ID | Prior blocker | Current evidence | Closed |
|---|---|---|---|
| B1 | `proposal.md` Capability Routing first column used the route's capability instead of the bare watch-scope capability; gate reported an undeclared capability | `proposal.md:37` now reads `\| pr-activity-notification \|` in the first column, with `EXTEND pr-activity-notification` in the decision column. `givn check review` no longer reports an undeclared capability; it prints `capability 'pr-activity-notification' already owned by givn/specs/watch-pull-requests`. | yes |
| B2 | `reopens` was an empty step; the restart scenario named an action it never took | `crates/passuff-cli/tests/steps/activity_inbox.rs:45-55` asserts the on-disk state (`World::state_json`, `support/mod.rs:870-873`) contains an activity record with `read == true` before clearing the transcript, then `activity-inbox.feature:94-95` re-runs `status` and asserts `0 unread of 1`. The scenario passes in the live delta run (`Read state survives a restart`, part of 32/32). | yes |
| B3 | Dead `TranscriptRenderer::render_activity_event` | `grep -rn render_activity_event` over the workspace returns only this review's historical text; the function is absent from `crates/passuff-cli/src/transcript.rs`. `cargo clippy --workspace --all-targets` reports no `dead_code` warning (only 5 pre-existing style/default-impl warnings). | yes |
| B4 | Four change-introduced behaviours had no scenario: config errors, inbox/read state errors, `read` with no target, labels `closed`/`CI pending`/`CI not started` | New scenarios: `watch-status.feature:76-87` Scenario Outline `A read command rejects invalid configuration` (4 examples: status, inbox, inbox --all, read --all; asserts `configuration error`, exit 2); `activity-inbox.feature:137-144` `Inbox reports a state error` (exit 3); `:146-153` `Read reports a state error` (exit 3); `:155-160` `Read requires an activity id or all` (asserts `read requires exactly one`, exit 2); `watch-status.feature:66-74` `Status uses human labels for closed and pending` (asserts `acme/api#42 closed CI pending`). Label unit test `human_labels_cover_every_state_and_ci_result` at `crates/passuff-cli/src/transcript.rs:200-210` covers `open`/`merged`/`closed` and `CI passing`/`CI failing`/`CI pending`/`CI not started`. All five scenarios and the unit test are in `8ffbe0b`. | yes |
| B5 | Chapter 05 omitted `Inbox`/`InboxCli` and contradicted the migration rule | `docs/arc42/05-building-block-view.md`: level-1 nodes `Inbox["Inbox / InboxService"]` (`:20`) and `InboxCli["InboxCli"]` (`:25`) with edges `CLI --> InboxCli`, `InboxCli --> Inbox`, `Inbox --> StatePort` (`:38-41`); building-block rows `InboxService / Inbox` (`:65`) and `InboxCli` (`:66`); `### Inbox` section (`:145-156`). State text (`:111`) now reads "version 1 migrates to version 2; corrupt or unknown-version files are quarantined", and `Versioning` (`:118`) reads "Migrate version 1 additively; reject unknown versions rather than guessing". Matches `08-crosscutting-concepts.md` and ADR-0005. | yes |

## Implementation evidence check

### Step bodies

102 annotated step definitions across four files (`activity_inbox.rs` 12,
`watch_status.rs` 13, `pr_activity_notification.rs` 71,
`pr_activity_notification_e2e.rs` 6). Searching for empty/whitespace-only
bodies (`fn ... (_world: &mut World) {}`) returns 0 matches. The former empty
`reopens` body is now a real assertion.

### `@e2e` tag integrity

| Feature file | `@e2e` | Change |
|---|---|---|
| `.../inspect-watch-status/watch-status.feature` | 1 | unchanged (planned) |
| `.../review-activity-inbox/activity-inbox.feature` | 2 | unchanged (planned) |
| `.../watch-pull-requests/pr-activity-notification.feature` | 0 | unchanged; the watch scope adds no consumer action |
| `givn/specs/watch-pull-requests/pr-activity-notification.feature` | 4 | unchanged |

No tag removal. No `@wip` remains.

### Scenario commit evidence

29 checked scenario tasks, 29 `COMMIT:` entries, every one naming a batch:

| Revision | Batches named | Scenarios |
|---|---|---|
| `1f9a3c8` | `state-history`, `watch-scope`, `inbox-service`, `cli-read` | 24 |
| `8ffbe0b` | `cli-read`, `inbox-service` | 5 |

`git show --stat 8ffbe0b` touches test code and production code:
`crates/passuff-cli/tests/steps/{activity_inbox,watch_status}.rs`,
`crates/passuff-cli/tests/support/mod.rs`, the two delta feature files,
`crates/passuff-cli/src/transcript.rs` (label logic), plus design/proposal/docs.
No scenario commit is comment-, tag-, task-, feature-, or transcript-only.
`givn check review` reports `scenario commits passed (29 scenario(s))`.

## Execution evidence (re-run in this review context)

| Scope | Command | Result | Exit |
|---|---|---|---|
| delta | `./scripts/verify.sh` | `givn test-result: total 32, passed 32, failed 0, skipped 0` | 0 |
| permanent | `GIVN_FEATURES=givn/specs ./scripts/verify.sh` | `total 80, passed 80, failed 0, skipped 0` | 0 |
| `@e2e` | `./scripts/verify.sh --e2e` | `total 4, passed 4, failed 0, skipped 0` | 0 |
| unit | `./scripts/verify.sh` (lib) | 6 + 16 + 23 = **45** passed | 0 |

Strict proof is recorded at `tasks.md:11-18` (undefined `Given`,
`total 1, passed 0, failed 1`, `cargo exit status: 1`), and
`.fail_on_skipped()` is set in the runner. The verification receipt
(`verification/tasks-verification.json`, `status: passed`, `runner_exit: 0`)
is reused for the documentation-only revision `0782168` with the recorded
tested head `33ec292`.

## Coverage

Merged report `coverage/cobertura-coverage.xml` root attributes:

| Metric | Value |
|---|---|
| Lines | 1758/2007 (87.6%) |
| Branches | 201/582 (34.5%) |

The README badge (`README.md:4-5`) matches. Branch instrumentation is intact in
both scopes.

Bucket classification:

- **Bucket 1 — dead code:** empty. `render_activity_event` is deleted (B3).
- **Bucket 2 — missing tests:** the four change-introduced rows from the prior
  review are now covered by the five new scenarios and the label unit test
  (B4). Remaining uncovered regions are pre-existing save/observation/credential
  arms (`state.rs`, `cycle.rs`, `github.rs`, `classifier.rs`, `credential.rs`,
  `config`), carried unchanged from the archived review, plus the pre-existing
  config read/token-path branches; none is introduced by this change.
- **Bucket 3 — legitimately hard to test:** none new.

Redundant unit tests: `inbox::tests::view_merges_accounts_and_orders_activity`
(`crates/passuff-core/src/inbox.rs:242-253`) remains. Prior finding N6. This pass
keeps it: it asserts the `InboxService::view` contract at the component
boundary (account login, newest-first order, unread count, watched-PR title),
which is a distinct requirement from the CLI transcript assertions and is the
contract `InboxCli` consumes. Recorded as a keep decision, not an escape.

## E2e coverage

- Inventory mapping: 3 delta consumer actions (status, inbox, read), 3 delta
  `@e2e` scenarios, one each; 4 unchanged permanent `@e2e` scenarios (4/4 in the
  live run). `givn lint --change` reports the capability-aware inventory clean.
- Primary assertions read the spawned binary's stdout and exit status
  (`support/e2e.rs:392-412`); no scenario asserts only on the state file.
- Scope isolation: `scripts/verify.sh:19-22` maps `--e2e` to
  `GIVN_RESULT_SCOPE=e2e`; the runner filters on it
  (`features_runner.rs:31,58-64`). Both scopes report zero failures and zero
  skips.

## Visual review

VISUAL-REVIEW: PASS

Interface: terminal. Evidence committed under
`givn/changes/passuff-activity-inbox/evidence/visual/` (7 transcripts, refreshed
by `0782168`; tree clean). The status transcript shows the account with login,
the watch scope, `acme/api#42 open CI passing`, and the unread count; the inbox
transcript shows `a1 <time> acme/api#42 unread New comment from @alice`; the
read transcript shows `no unread activity` with exit 0. No enum token, raw
comment body, or command text leaks into a transcript; `passuff:` headings are
stable; the activity id is the documented CLI argument. Persona
`multi-account-developer` can inspect the watch, review activity, and see
acknowledgement take effect.

Note: re-running the e2e suite regenerates the two timestamped transcripts; this
review restored them so the reviewed tree stays clean.

## Arc42 conformance

ARC42 CONFORMANCE: CLEAN

| Chapter | Fact | Match |
|---|---|---|
| 05 | Level-1 diagram, block table, `### Inbox`, and `InboxCli` name the new components | yes (`05-building-block-view.md:20,25,38-41,58,65-66,145-156`) |
| 05 | State load/versioning: v1 migrates, unknown version quarantines | yes (`:111,118`) |
| 06 | Status/inbox/read runtime flows | yes (`06-runtime-view.md` flows match `cycle.rs`, `inbox.rs`) |
| 08 | History retention, migration, read persistence, scope filtering | yes |
| 09 | ADR-0005 registered and linked | yes |
| 10/11/12 | QS-016…QS-019, R-016…R-018, glossary terms | yes |

## Lint

- `givn lint --change passuff-activity-inbox`: exit 0, `3 file(s) checked — clean`.
- `givn check review` runs the same scoped lint and reports clean.
- Full-tree `givn lint` (no `--change`) exits 1 with three findings: the two new
  use-case directories carry their durable `usecase.md` while their capability
  `.feature` files exist only in the delta until the archive merge, so the
  permanent paths `givn/specs/{inspect-watch-status/watch-status,review-activity-inbox/activity-inbox}.feature`
  are absent. This is the documented pre-archive layout for `NEW in <usecase>`
  capabilities (the archive merges the delta into the permanent spec), not a
  defect in this change; no action is available without duplicating the delta.

## Non-blocking findings

| ID | Finding | Disposition |
|---|---|---|
| N6 | `inbox::tests::view_merges_accounts_and_orders_activity` overlaps scenario assertions. | Kept: component-contract test at the `InboxService::view` boundary; distinct from CLI transcript assertions. |
| N7 | Managed `## Coverage` block (`README.md:146-160`) still shows the previous archive's 91% (1303/1429). | Archive-owned block, rewritten by the archive gate; the badge and merged report agree at 87.6% / 34.5%. Carried unchanged. |
| N8 | Full-tree `givn lint` fails on the two new use-case directories before archive. | Expected pre-archive layout (see Lint); resolved by the archive merge. |

## README impact

README-IMPACT: updated - Quick start, Usage

- Quick start: `README.md:42-43` documents the `[watch] repositories` block and
  the empty-means-all rule.
- Usage: `README.md:99-105` lists `status`, `inbox`, `inbox --all`, `read <id>`,
  `read --all`; exit codes at `:108-109` now name `2` as configuration or usage
  error and `3` as state failure, consistent with the new usage error.
- Badge matches the merged report. The managed coverage block remains
  archive-owned. No placeholder text is open.

## Sign-off checklist

- [x] Fabrication audit: 0 empty step bodies; the restart step asserts persisted
      read state. (B2 closed)
- [x] Every checked scenario has a commit touching its implementation or
      regression test (`1f9a3c8`, `8ffbe0b`; 29 scenarios, batches named).
- [x] Every promised component exists (`Inbox`, `InboxService`, `InboxCli`,
      `JsonStateStore` v2, `WatchConfig`, `InboxView`, `ReadOutcome`).
- [x] Every consumer uses only its declared contract (`InboxService`,
      `SnapshotStore`; composition root excepted).
- [x] Strict-mode proof present and passing (`tasks.md:11-18`).
- [x] Scenario execution evidence re-verified live: delta 32/32, permanent 80/80,
      e2e 4/4, unit 45.
- [x] Coverage classified from the merged report; branch instrumentation intact.
- [x] Every coverage gap classified and resolved; the four change-introduced
      rows are now covered. (B4 closed)
- [x] Dead code deleted; clippy reports no `dead_code`. (B3 closed)
- [x] Redundant unit tests reviewed; one kept with a recorded contract-level
      rationale (N6).
- [x] No `@wip` tags; no `@e2e` tag removed; no implementation-layer detail in
      the spec beyond persisted-state facts.
- [x] Canonical E2E policy: every normalized inventory action has one `@e2e`
      scenario; variants are regular scenarios.
- [x] Every E2E scenario asserts the real CLI interface (stdout, exit status).
- [x] The run declaration passes; `passuff --help` transcript present.
- [x] E2E scope selected explicitly; both executions report zero failures and
      zero skips.
- [x] Capability routing gate passes; the watch-scope row names the bare
      `pr-activity-notification`. (B1 closed)
- [x] Interaction coverage maps every row for the three new/changed
      interactions; the three unchanged watch actions have matrix rows.
- [x] Arc42 conformance clean; chapter 05 names the new components and states
      the correct migration rule. (B5 closed)
- [x] Ubiquitous language and structural model current
      (`docs/domain-model.md:13-14` names `passuff-activity-inbox`).

## Verdict

All five blockers are closed and no new blocker was found. The change is
signable; `givn check review --change passuff-activity-inbox` exits 0.

REVIEW: PASS
