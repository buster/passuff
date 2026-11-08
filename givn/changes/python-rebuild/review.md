# Review: python-rebuild

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-1

Scope: the working tree at `5b6b815174fb9713d0cc7e224e758221870ec6e1`. The review
gate reuses the committed verification receipt tested at
`624fe70a9f496fb7ead8d3169f2d63321c29b0ac`; the implementation fixes are in
`7a83b2db117a8bc5eb2f4fd5e670964c89a55e3d`. The operator forbids `git log`,
`git show`, `git diff`, reflog, and blame, so revisions were inspected only
with `git cat-file -t`, `git ls-tree`, `git ls-files`, and the working tree.

## Implementation evidence check

### 0. `@e2e` tag integrity

The delta carries one `@e2e` scenario: `@givn.modified @e2e Developer opens an
activity's details` (`givn/changes/python-rebuild/specs/review-activity-inbox/activity-inbox.feature:4-5`).
The permanent corpus carries 18 `@e2e` scenarios, one per row of the six use
cases' `## Interactions` inventories (18 rows). No scenario lost its `@e2e`
tag; the 18 permanent plus one delta entry equal the 19 E2E executions the
change records. Counts recompute from the feature files: permanent 215 entries
(197 non-E2E + 18 E2E), delta 33 entries (32 non-E2E + 1 E2E), change 230
(197 + 33), regular 197, E2E 19.

### 1. Empty step body audit

`python3 scripts/check_steps.py` exits 0: `check_steps: no no-op step bodies`.
It scans the seven per-capability regular files plus `givn/steps/common.py` and
`givn/steps/support.py`, and the seven per-capability E2E files plus
`givn/e2e_steps/common.py` — 0 empty or trivial bodies across 17 files.
`scripts/verify.sh:47` runs the same audit over `givn/steps` and
`givn/e2e_steps` in every normal scope (regular, change, E2E), so a no-op body
cannot bypass it. The `the developer reopens passuff` step that failed the
previous audit no longer exists.

### 2. Checked scenarios and their commits

`givn/changes/python-rebuild/tasks.md` records 206 checked tasks (204 numbered
scenario batches, including `151a`, plus two setup batches) and 206
`- [x] COMMIT: <hash> -- Batch: <name>` lines in the gate's shape;
`givn status` reports 412/412 and the review gate's hash-presence check passes.
Two revisions carry the evidence:
`14e0b99b19b4851e203c78ac375f3ce69a863da9` (205 lines) and
`7a83b2db117a8bc5eb2f4fd5e670964c89a55e3d` (the `boundary-body` batch).
`git ls-tree` shows 14e0b99 contains `passuff/` and the step bindings;
7a83b2d contains the seven regular step files, the seven E2E files,
`passuff/status.py`, `passuff/tui/session.py`, and
`tests/support/recording_delivery.py`. Every line names its batch, so the
shared revision is a batch revision. The prior `COMMIT`-format defect (F14) is
resolved.

### 3. Promised components

| `design.md` promise | Tree | Consumers |
|---|---|---|
| `WatchStatusReader` → `passuff/status.py` | `passuff/status.py:11` | `passuff/cli.py:21,86,94`; `passuff/tui/app.py:19,66,89`; `passuff/tui/frame.py:11,87` |
| `ObservationSession` → `passuff/tui/session.py` | `passuff/tui/session.py:15` | `passuff/tui/app.py:21,323,422` |
| `ActivityInboxService` | `passuff/inbox.py:70` | `passuff/cli.py`, `passuff/tui/app.py` |
| Seven regular step files | `givn/steps/{activity_inbox,inbox_navigation,initial_setup,pr_activity_notification,stacked_inbox,watch_scope,watch_status}_steps.py` | `givn/steps/common.py` shared phrases |
| Seven E2E step files | `givn/e2e_steps/{activity_inbox,inbox_navigation,initial_setup,pr_activity_notification,stacked_inbox,watch_scope,watch_status}_e2e.py` | each imports its capability's regular bindings |
| `RecordingDelivery` under `tests/support/` | `tests/support/recording_delivery.py:9` | `tests/support/fake_notifications.py:9,30` |
| macOS twins | `tests/support/bin/afplay`, `tests/support/bin/osascript` | reserved for the deferred macOS smoke (`design.md:551`) |
| `StackGrouper` / `composition.py` | absent; no importer | — |
| `render_activity_detail` | absent from `passuff/render.py` | — |
| production `recording_path` hook | absent from `passuff/notify.py` | — |

The per-capability step layout matches `design.md:368-390`; the E2E files
`activity_inbox_e2e.py`, `inbox_navigation_e2e.py`, `initial_setup_e2e.py`, and
`pr_activity_notification_e2e.py` define their E2E-only phrases and import the
regular bindings; `stacked_inbox_e2e.py`, `watch_scope_e2e.py`, and
`watch_status_e2e.py` are staged imports because those capabilities need no
E2E-only phrase. The prior missing-component findings (F2-F5) are resolved.

### 4. Strict-mode proof

`./scripts/verify.sh --strictness` exits 0: the undefined-step probe errors,
the skipped-scenario probe reports `skipped=1`, and the pass-only body is
rejected by `check_steps.py`. The fixtures under `tests/strictness/` are
retained as the reproducible proof, matching `design.md:330-337` and
`tasks.md:5`; `scripts/verify.sh:35-43` runs them and `:47` runs the durable
audit in every normal scope. The prior strictness-retention finding (F12) is
resolved.

### 5. Every `@e2e` Then asserts the real interface

Every `@e2e` Then asserts the command transcript, the rendered `--snapshot`
frame, the exit status, or a real-interface log:
`the terminal shows a notification for "<key>"` waits on the watcher's output
(`givn/steps/pr_activity_notification_steps.py`), `the desktop notification
reports "<text>"` reads the fake D-Bus delivery record
(`pr_activity_notification_steps.py:429-438`), `the configured sound is played
once` reads the fake player's sound log (`:393-396`), `the browser opened
"<url>"` reads the fake opener's log (`givn/e2e_steps/activity_inbox_e2e.py`),
and the TUI steps assert the printed frame. No `@e2e` Then asserts only the
state file or the repository, and no step calls the product over HTTP from
inside a page. The watcher scenario asserts the terminal notification, the
D-Bus summary/body, and one sound.

### 6. Exact `verify.e2e_command` and the files it invokes

`givn/commands.yaml` sets `verify.e2e_command: "./scripts/verify.sh --e2e"`.
`scripts/verify.sh:73-77` runs `behave --stage e2e --tags="@e2e"` over
`givn/specs/*/*.feature` and `givn/changes/python-rebuild/specs/*/*.feature`;
`--stage e2e` loads `givn/e2e_environment.py` and `givn/e2e_steps/` instead of
`givn/environment.py` and `givn/steps/`. The tree was searched for a parallel
implementation: only those two step trees exist (`git ls-files`; no untracked step
files), and no second E2E driver or HTTP-only implementation exists.

### 7. E2E scope selection

`--e2e` sets `SCOPE=e2e` and the runner also honors `GIVN_RESULT_SCOPE`;
`givn/commands.yaml` records the same E2E command. The receipt
`givn/changes/python-rebuild/verification/tasks-verification.json` records
change 230 total, 230 passed, 0 failed, 0 skipped at
`624fe70a9f496fb7ead8d3169f2d63321c29b0ac`; `tasks.md:19` records regular
197/197 and E2E 19/19, both with zero failures and skips. The two scopes select
different tags and are not the same execution. Review does not re-run the
suite.

### 8. E2E scope size

The six use cases' `## Interactions` sections hold 18 normalized actions:
watch-scope 3, watch-status 1, activity-inbox 5, inbox-navigation 2,
stacked-inbox 1, initial-setup 1, pr-activity-notification 5. Each has exactly
one matching `@e2e` scenario; no action has two. Variants and error cases are
regular or delta non-E2E scenarios. The E2E count (19 executions) is strictly
less than the change count (230).

### 9. Design conformance (commands, layout, framework, drivers)

- `givn/commands.yaml` matches `design.md:299-312` exactly.
- `scripts/verify.sh` implements the scopes, the empty-run-set skip
  (`design.md:312-314`), `--name`, the JSON merge, and the
  `total > 0 / failed = 0 / skipped = 0` exit rule.
- The step trees match `design.md:368-390` and the `--stage e2e` layout.
- The matrix drivers exist: `passuff/tui/app.py:359-371` defines
  `--snapshot`, `--observe`, `--observe-for`, `--scope-select`,
  `--scope-deselect`, `--scope-mode`, `--focus-set`, `--picker`, and `--keys`.
- The TUI-open matrix row now names the as-built `--observe --snapshot`
  mechanism and the regular pty smoke scenario (`design.md:502`);
  `givn/steps/activity_inbox_steps.py:87-95` runs that scenario through
  `tests/support/pty_driver.py`.
- The strictness fixtures are retained and the decision is recorded in
  `design.md` and `tasks.md`.

No implementation deviation from the reviewed design remains.

### 10. Interaction coverage cross-reference

| Use case | Inventory entry | Matrix row | `@e2e` scenario | Step file | Driving mechanism |
|---|---|---|---|---|---|
| change-watch-scope | toggle a repository | yes | Developer toggles a repository in the watch scope | `watch_scope_steps.py` | `--scope-select`/`--scope-deselect` + `--picker --snapshot` |
| change-watch-scope | switch the mode | yes | Developer switches the watch scope mode | `watch_scope_steps.py` | `--scope-mode selected --picker --snapshot` |
| change-watch-scope | narrow the focus | yes | Developer narrows the focus in the watch scope | `watch_scope_steps.py` | `--focus-set authored --picker --snapshot` |
| inspect-watch-status | show the watch state | yes | Developer sees the watch state | `watch_status_steps.py` | subprocess `passuff status` |
| review-activity-inbox | list unread | yes | Developer lists unread activity | `activity_inbox_steps.py` | subprocess `passuff inbox` |
| review-activity-inbox | mark read | yes | Developer marks activity read | `activity_inbox_steps.py` | subprocess `passuff read` |
| review-activity-inbox | open the TUI | yes | Developer opens the inbox TUI | `activity_inbox_e2e.py` | `--observe --snapshot`; pty smoke in the regular scenario |
| review-activity-inbox | open a detail | yes | Developer opens an activity's details | `common.py` + `inbox_navigation_steps.py` | `--observe --snapshot --keys "enter"` |
| review-activity-inbox | open the pull request | yes | Developer opens the pull request from an activity | `activity_inbox_e2e.py` | `--keys "enter,enter"` with fake `xdg-open` |
| review-activity-inbox | navigate panes | yes | Developer navigates the inbox panes | `common.py` + `inbox_navigation_steps.py` | `--snapshot --keys "tab,j"` |
| review-activity-inbox | live refresh | yes | The inbox shows activity observed while it is open | `inbox_navigation_e2e.py` | `--observe-for 4000 --snapshot`; injected comment |
| review-pull-request-stacks | review a stack | yes | Developer reviews a stacked inbox | `stacked_inbox_steps.py` | `--snapshot` with fake stack metadata |
| set-up-passuff | first run | yes | Developer sets up passuff on first run | `initial_setup_e2e.py` | `passuff setup` stdin, then TUI `--snapshot` |
| watch-pull-requests | start watching | yes | Developer starts watching and is notified of a new comment | `pr_activity_notification_e2e.py` | `timeout --signal=INT 10 passuff`; fake D-Bus/sound |
| watch-pull-requests | one check | yes | Developer runs a single check and sees the current activity | `pr_activity_notification_steps.py` | subprocess `passuff check` |
| watch-pull-requests | usage | yes | Developer asks for usage | `pr_activity_notification_e2e.py` | subprocess `passuff --help` |
| watch-pull-requests | version | yes | Developer asks for the version | `pr_activity_notification_e2e.py` | subprocess `passuff --version` |
| watch-pull-requests | notification test | yes | The developer tests notifications on the command line | `pr_activity_notification_e2e.py` | subprocess `passuff notify-test`; logs |

Every inventory entry has one matrix row and one `@e2e` scenario, and each
step file uses the driving mechanism the matrix names. The prior pty-driver
mismatch (F13) is resolved.

### 11. Local run command

`python -m passuff --help` exits 0. The harness starts the fake GraphQL server,
the private D-Bus session, and the fake `gh`/player/opener binaries per
scenario (`tests/support/harness.py`, `tests/support/world.py:420-435`); the
full-stack execution evidence is the committed receipt, which review does not
re-run.

### 12. Additional evidence from this check

- The coverage measurement runs only the `regular` and `e2e` scopes
  (`scripts/measure-coverage.sh`), so the change delta's non-E2E scenarios are
  not instrumented in any Cobertura report. This is the declared process
  boundary at `design.md:572-589`; static inspection classified the delta-only
  regions.
- The four E2E step files that define phrases repeat their capability import
  line twice (`givn/e2e_steps/activity_inbox_e2e.py:17-18`,
  `inbox_navigation_e2e.py:17-18`, `initial_setup_e2e.py:17-18`,
  `pr_activity_notification_e2e.py:17-18`) and carry unused imports. This is
  cosmetic and does not affect behaviour or phrase ownership.
- The prior findings F1-F15 were each checked: F1 `check_steps.py` exit 0 and
  its call in every scope; F2 the seven-plus-seven step files; F3
  `ObservationSession`; F4 `passuff/status.py`; F5 the dead-code removal and
  `tests/support/recording_delivery.py`; F6 the 80-character row excerpt
  (`passuff/tui/frame.py:160`) and its delta scenario
  (`inbox-navigation.feature:5-14`, step
  `givn/steps/inbox_navigation_steps.py:248-256`); F7 the 4 KiB bound
  (`passuff/classify.py:47-54`) and `The stored activity body is bounded to
  4 KiB`; F8 `afplay` for darwin (`passuff/sound.py:11,31`) and the fake twins;
  F9 the watcher transcript; F10/F11 the arc42 and glossary symbols; F12 the
  retained strictness fixtures; F13 the matrix row; F14 the `COMMIT:` shape;
  F15 the tracked receipt. All are resolved.

## Arc42 implementation conformance

Affected chapters were re-derived from `design.md`, `tasks.md`, and the
implementation before accepting `givn/changes/python-rebuild/arc42.md`.

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Six use cases including `set-up-passuff` | `01-introduction-and-goals.md` | Affected | six use cases | all seven capabilities | six `usecase.md` files | yes |
| `behave` runner and the runner contract | `04-solution-strategy.md:60`, `08-crosscutting-concepts.md:227` | Affected | V2 | setup batches | `behave.ini`, `scripts/verify.sh` | yes |
| `curses` and the pure frame renderer | `04-solution-strategy.md:77` | Affected | V3 | TUI batches | `passuff/tui/app.py`, `frame.py` | yes |
| Separate E2E step tree per capability | `05-building-block-view.md:74,180` | Affected | `design.md:368-390` | step-file references | `givn/e2e_steps/` | yes |
| `ObservationSession` | `05-building-block-view.md:51,74`; `06-runtime-view.md:299`; `08:129`; glossary | Affected | `design.md:202` | TUI batches | `passuff/tui/session.py:15`, `app.py:21,323,422` | yes |
| `WatchStatusReader` | `05:20,70,163`; `08:127`; glossary | Affected | `design.md:186-205` | status batch | `passuff/status.py:11` | yes |
| `RecordingDelivery` under `tests/support/` | `05:77,179`; `04:76`; `08:235`; `10:44` | Affected | `design.md:103` | test-support tasks | `tests/support/recording_delivery.py`, used by the fake service | yes |
| 80-character list row; 4 KiB body | `05`/`08`; glossary | Affected | `design.md:447-469` | boundary batches | `frame.py:160`, `classify.py:47-54`, delta scenarios | yes |
| macOS `afplay` | `03:42`; `07:30` | Not affected | `design.md:551` | setup/E2E | `passuff/sound.py:11,31` | yes |
| Bundle deferred (2026-11-08); TD-007 | `07:58-61`; `11:53` | Affected | `design.md:276-282` | — | no bundle expected | yes |
| TD-005 and TD-006 resolved | `11:51-52` | Affected | coverage scripts; read models | setup/tasks | `scripts/merge-coverages.sh`, `passuff/status.py` | yes |
| QS-011 / QS-012 | `10:43-44` | Not affected | read-model contracts | — | `WatchStatusReader`, D-Bus twin | yes |

The chapter files are present, contain decision-specific content, and contain
no box-drawing characters or ASCII pseudo-diagrams (all matches are Markdown
tables; Mermaid diagrams are in chapters 03, 05, 06, 07, and 10). One mermaid
edge in `05-building-block-view.md:51` uses the node id `Session` with the edge
label `ObservationSession`; the building-block table names the same component,
so this is a diagram-label nit, not a contradictory fact.

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

The specs, design, and code reuse the glossary's terms. `docs/glossary.md:59`
names `passuff.status.WatchStatusReader` and `:62,67` name
`ObservationSession`; both exist and are the symbols the code and arc42
chapters use. `docs/domain-model.md:14` records the `python-rebuild` scope and
`:121` records the 4 KiB body and the 500-record cap; the model's boundaries
and relationships match the implemented aggregates. No stale glossary symbol
remains, and no no-op structural edit was made: the rebuild changes the
implementation language, not the structure.

## Use-case and Persona conformance

The six use cases are the owning contracts and pass the rejection criteria:
each `## Goal` is one outcome, each `## Trigger` names the developer, extensions
carry their main-flow step, preconditions are not re-established, and no goal,
trigger, or guarantee names a mechanism. Capability names are goal-shaped.
`review-activity-inbox`'s goal and success guarantee are single-outcome
(`givn/specs/review-activity-inbox/usecase.md:13-25`) and the watch-pull-requests
rules run R1-R46 in order.

Persona disposition:

| Persona | Confirmed | Outcome in this change |
|---|---|---|
| `multi-account-developer` (`givn/personas/multi-account-developer.md`) | yes, 2026-11-02 | The rebuild restores every user-visible outcome the Persona relies on: the watcher notification, status, inbox/read, the terminal inbox with details and stacks, scope editing, and first-run setup. The committed transcripts show the notification, inbox, detail, stack, picker, and status goals. Multi-account behavior is covered by the regular two-account scenarios (`Two accounts share the global scope`, `A failed account does not stop the other account`); the E2E corpus does not repeat it. |

No other Persona is invented or promoted; `developer` remains the interaction
role, not a Persona.

PERSONAS: multi-account-developer

## Coverage

Measured evidence: `coverage/cobertura-coverage.xml` (merged, 1879/2178 lines,
651/828 branches), `coverage/non-e2e-cobertura.xml` (1869/2178, 646/828), and
`coverage/e2e-cobertura.xml` (1411/2178, 361/828). The reports are distinct
files with distinct timestamps (`1794129613893`, `1794129648775`,
`1794129649172`) and distinct counts; branch instrumentation is present.

The measurement covers the regular and E2E scopes only, so the change delta's
non-E2E scenarios are not in the reports; their regions are classified by
static inspection, not by the report alone.

### Bucket 1 — dead code (delete)

None found. `passuff/stacking.py`, `passuff/composition.py`,
`render_activity_detail`, and the `recording_path` hook are gone; every
remaining production module has a consumer.

### Bucket 2 — missing test coverage (add tests)

| Region | Evidence | Gap |
|---|---|---|
| `passuff/config.py:121-125` `reconciliation_interval_secs` below 1 | `coverage/cobertura-coverage.xml` hits `0` at line 125; no feature contains the message | `docs/reference.md:221` specifies exit 2; no scenario covers it (the delta covers only `poll_interval_secs`) |
| `passuff/config.py:139-140` watch repository not `owner/repo` | hits `0` at line 140; no feature contains the message | `docs/reference.md:222`; the permanent repository-entry scenario drives `ScopeEditor`, not configuration load |
| `passuff/config.py:154-158` ignored pull request not `owner/repo#number` | hits `0` at 155,158; no feature contains the message | `docs/reference.md:223` |
| `passuff/config.py:160-161` duplicate ignored pull request | hits `0` at 161; no feature contains the message | `docs/reference.md:223` |
| `passuff/config.py:201` no accounts | hits `0`; no feature contains the message | `docs/reference.md:219` |
| `passuff/config.py:204` empty account label | hits `0`; no feature contains the message | `docs/reference.md:220` |
| `passuff/config.py:218-219` `github_user` contains whitespace | hits `0`; no feature contains the message | `docs/reference.md:227` |
| `passuff/config.py:187-188` `sound_file` not a readable file | hits `0`; no feature contains the message | `docs/reference.md:229` |
| `passuff/config.py:131,136,146-148,151,167,171,176,182,191,206,234` structural validation | hits `0`; no feature configures a non-table `watch`/`focus`/`events`/`notifications`, an invalid `watch.mode`, an invalid `notifications.urgency`, a non-table account, or a non-HTTP(S) `graphql_url` | the same bucket-2 class: configuration input validation with no scenario |
| `passuff/config.py:110-111` configuration read error | hits `0`; no scenario makes the file unreadable | the same bucket-2 class (the prior review's "read errors") |

These are reachable, user-visible exit-2 rules (verified by loading crafted
configurations through `passuff.config.load_config`); they are missing test
coverage, not dead code, and not a bucket-3 exception.

### Bucket 3 — legitimately hard to test (concrete technical justification)

| Region | Justification |
|---|---|
| `passuff/notify.py:53-63` `_send_macos` | the real `osascript` Notification Center surface requires a macOS host and a logged-in session; the suite runs on Linux. |
| `passuff/browser.py:15-19` the `open` branch | the same macOS-only process call; Linux covers `xdg-open`. |
| `passuff/sound.py:31,40-43` the `afplay` selection and player failures | `sys.platform == "darwin"` selects it; the Linux suite cannot take that branch without a production platform seam. The `tests/support/bin/afplay` twin is reserved for the deferred macOS smoke (`design.md:551`, Q10). |
| `passuff/tui/app.py:319-354` the interactive curses loop | the regular pty smoke starts it (and presses `q`); driving every interactive key path through a pty is the manual smoke's scope, and the key dispatch it calls is covered through `--keys`. |

### E2E coverage

The canonical E2E policy was read from `givn instructions specs --change
python-rebuild`. The interface is a terminal: CLI subprocesses and a curses
TUI. Every inventory action has one `@e2e` scenario; the primary assertions are
on the transcript, the printed frame, the exit status, or the twin logs;
drivers are real subprocesses, `--snapshot`, `--observe-for`, the private
D-Bus service, the fake sound player, and the fake opener. No `@e2e` scenario
was downgraded or over-produced, and the scope is explicit.

### Redundant unit tests

There is no separate unit-test tree; the `.feature` corpus is the only test
surface, so no redundant unit test exists to remove.

## Visual review

The design declares `Interface: terminal` (`design.md:447-469`) with viewports
80x24 / 120x40 and a transcript per `@e2e` scenario. All 18 committed
transcripts under
`givn/changes/python-rebuild/evidence/visual/<scenario-slug>/transcript.txt`
were read:

| Scenario slug | State seen |
|---|---|
| `developer-starts-watching-and-is-notified-of-a-new-comment` | watcher: `passuff: watching`, baseline, `no notifications`, then `acme/api#42: New comment from @alice` |
| `developer-opens-the-inbox-tui` | live-observe frame with unread activity and detail body |
| `developer-opens-an-activity-s-details` | activity detail overlay: metadata, rule, body, `Enter open in browser` |
| `developer-opens-the-pull-request-from-an-activity` | frame with the detail and the open affordance |
| `developer-navigates-the-inbox-panes` | two-pane frame, focus on PRs, selection moved |
| `the-inbox-shows-activity-observed-while-it-is-open` | frame with the injected activity |
| `developer-reviews-a-stacked-inbox` | stack `☰ 1/2`, `☰ 2/2`, `stack 7 base main` |
| `developer-toggles-a-repository-in-the-watch-scope` | picker, `watching nothing` empty state |
| `developer-switches-the-watch-scope-mode` | picker, `Mode: selected`, focus marks |
| `developer-narrows-the-focus-in-the-watch-scope` | picker, focus marks, watched row |
| `developer-sees-the-watch-state` | status transcript with account, scope, last check |
| `developer-lists-unread-activity` | inbox list row: id, summary, key, time, read state |
| `developer-marks-activity-read` | `0 unread of 1`, `no unread activity` |
| `developer-runs-a-single-check-and-sees-the-current-activity` | check transcript |
| `developer-sets-up-passuff-on-first-run` | first-run setup then empty inbox |
| `developer-asks-for-usage` / `developer-asks-for-the-version` | CLI usage and version |
| `the-developer-tests-notifications-on-the-command-line` | `desktop notification sent`, `sound played` |

Persona judgment (`multi-account-developer`): the watcher transcript shows the
Persona's primary goal — a new comment on their own pull request produces the
notification line after the baseline — and the remaining transcripts show the
inbox, read state, detail body, stack positions, scope picker, status, and both
notification surfaces. No finding names a failed Persona goal. The macOS
desktop attribution is outside the terminal transcript and is covered by the
deferred manual smoke; it is not a failed Persona goal in the committed
evidence.

VISUAL-REVIEW: PASS

## Scenario execution evidence

Receipt status: proven. `givn check review --change python-rebuild` reports
`change verification receipt reused across documentation-only revision
5b6b815174fb9713d0cc7e224e758221870ec6e1 (tested at
624fe70a9f496fb7ead8d3169f2d63321c29b0ac, 230 scenario(s))`. The receipt
`givn/changes/python-rebuild/verification/tasks-verification.json` records
change 230/230, 0 failed, 0 skipped; `tasks.md:19` records regular 197/197 and
E2E 19/19. Review does not re-run the suite.

## Overlap dispositions

One row per `[SUBST]` and `[SHAPE]` finding reported by
`givn lint --change python-rebuild` involving this change's delta (614 pairs).
Decision rule: `duplicate` only when the same rule and the same Given/When
path produce the same assertion and observable state; `boundary` when the two
scenarios are the minimum/maximum or inside/outside cases of one rule;
`variant` otherwise — a distinct rule or distinct invariant that shares step
vocabulary. No pair meets the duplicate test: every shared Then is the generic
`the exit status is <n>` or a shared transcript phrase whose Given/When path
differs. The finding kind ([SUBST] or [SHAPE]) is recorded in the lint output;
the disposition is per scenario pair.

| Scenario A | Scenario B | Decision |
|---|---|---|
| A CI recovery across pending notifies | A CLI usage error exits 2 | `variant` |
| A CI recovery across pending notifies | A GitHub response failure is an account failure | `variant` |
| A CI recovery across pending notifies | A delivery surface failure keeps the transcript | `variant` |
| A CI recovery across pending notifies | A missing configuration is a configuration error | `variant` |
| A CI recovery across pending notifies | A missing or failing gh is an account failure | `variant` |
| A CI recovery across pending notifies | A quarantine failure is a state error | `variant` |
| A CI recovery across pending notifies | An invalid configuration is reported with exit 2 | `variant` |
| A CI recovery across pending notifies | An unknown state version is quarantined and replaced | `variant` |
| A CI recovery across pending notifies | Deselecting a repository in all mode is refused | `variant` |
| A CI recovery across pending notifies | Duplicate watch scope entries are rejected | `variant` |
| A CI recovery notifies | A CLI usage error exits 2 | `variant` |
| A CI recovery notifies | A GitHub response failure is an account failure | `variant` |
| A CI recovery notifies | A delivery surface failure keeps the transcript | `variant` |
| A CI recovery notifies | A missing configuration is a configuration error | `variant` |
| A CI recovery notifies | A missing or failing gh is an account failure | `variant` |
| A CI recovery notifies | A quarantine failure is a state error | `variant` |
| A CI recovery notifies | An invalid configuration is reported with exit 2 | `variant` |
| A CI recovery notifies | An unknown state version is quarantined and replaced | `variant` |
| A CI recovery notifies | Deselecting a repository in all mode is refused | `variant` |
| A CI recovery notifies | Duplicate watch scope entries are rejected | `variant` |
| A CLI usage error exits 2 | A GitHub response failure is an account failure | `variant` |
| A CLI usage error exits 2 | A delivery surface failure keeps the transcript | `variant` |
| A CLI usage error exits 2 | A disabled desktop notification keeps the transcript | `variant` |
| A CLI usage error exits 2 | A failed account does not stop the other account | `variant` |
| A CLI usage error exits 2 | A missing or failing gh is an account failure | `variant` |
| A CLI usage error exits 2 | A missing sound player still delivers the notification | `variant` |
| A CLI usage error exits 2 | A missing token file is an account failure | `variant` |
| A CLI usage error exits 2 | A one-shot check never starts a browser login | `variant` |
| A CLI usage error exits 2 | A personal access token for a different login is an account failure | `variant` |
| A CLI usage error exits 2 | An invalid configuration is reported with exit 2 | `variant` |
| A CLI usage error exits 2 | An unavailable desktop surface keeps the transcript | `variant` |
| A CLI usage error exits 2 | An unknown state version is quarantined and replaced | `variant` |
| A CLI usage error exits 2 | Developer lists unread activity | `variant` |
| A CLI usage error exits 2 | Developer runs a single check and sees the current activity | `variant` |
| A CLI usage error exits 2 | Developer sees the watch state | `variant` |
| A CLI usage error exits 2 | Duplicate watch scope entries are rejected | `variant` |
| A CLI usage error exits 2 | Inbox with no state shows no activity | `variant` |
| A CLI usage error exits 2 | Status with no state shows empty sections | `variant` |
| A CLI usage error exits 2 | The developer tests notifications | `variant` |
| A CLI usage error exits 2 | The developer tests notifications on the command line | `variant` |
| A CLI usage error exits 2 | The notification test can mute the desktop notification | `variant` |
| A CLI usage error exits 2 | The notification test can mute the sound | `variant` |
| A CLI usage error exits 2 | The notification test reports an unavailable surface | `variant` |
| A CLI usage error exits 2 | The notification test reports disabled surfaces | `variant` |
| A GitHub response failure is an account failure | A personal access token for a different login is an account failure | `variant` |
| A delivery surface failure keeps the transcript | A missing sound player still delivers the notification | `variant` |
| A delivery surface failure keeps the transcript | The developer tests notifications | `variant` |
| A deselected repository stops observation | A rejected stack query retries once and shows standalone rows | `variant` |
| A deselected repository stops observation | The activity list row is bounded to the recorded excerpt | `variant` |
| A failed account does not stop the other account | A GitHub response failure is an account failure | `variant` |
| A failed browser launch keeps the activity detail open | A missing browser opener is reported and keeps the detail | `variant` |
| A failed browser launch keeps the activity detail open | Developer opens an activity | `variant` |
| A failed browser login does not stop the other account | A GitHub response failure is an account failure | `variant` |
| A failed observation keeps the change pending | A CLI usage error exits 2 | `variant` |
| A failed observation keeps the change pending | A GitHub response failure is an account failure | `variant` |
| A failed observation keeps the change pending | A delivery surface failure keeps the transcript | `variant` |
| A failed observation keeps the change pending | A missing configuration is a configuration error | `variant` |
| A failed observation keeps the change pending | A missing or failing gh is an account failure | `variant` |
| A failed observation keeps the change pending | A quarantine failure is a state error | `variant` |
| A failed observation keeps the change pending | An invalid configuration is reported with exit 2 | `variant` |
| A failed observation keeps the change pending | An unknown state version is quarantined and replaced | `variant` |
| A failed observation keeps the change pending | Deselecting a repository in all mode is refused | `variant` |
| A failed observation keeps the change pending | Duplicate watch scope entries are rejected | `variant` |
| A failing CI result notifies | A CLI usage error exits 2 | `variant` |
| A failing CI result notifies | A GitHub response failure is an account failure | `variant` |
| A failing CI result notifies | A delivery surface failure keeps the transcript | `variant` |
| A failing CI result notifies | A missing configuration is a configuration error | `variant` |
| A failing CI result notifies | A missing or failing gh is an account failure | `variant` |
| A failing CI result notifies | A quarantine failure is a state error | `variant` |
| A failing CI result notifies | An invalid configuration is reported with exit 2 | `variant` |
| A failing CI result notifies | An unknown state version is quarantined and replaced | `variant` |
| A failing CI result notifies | Deselecting a repository in all mode is refused | `variant` |
| A failing CI result notifies | Duplicate watch scope entries are rejected | `variant` |
| A gh OAuth account uses the existing gh session | A GitHub response failure is an account failure | `variant` |
| A gh OAuth account uses the existing gh session | A reopen within the retention window is observed | `variant` |
| A gh OAuth account uses the existing gh session | A replayed pull request node does not duplicate activity | `variant` |
| A legacy command credential remains a personal access token account | A GitHub response failure is an account failure | `variant` |
| A legacy command credential remains a personal access token account | A reopen within the retention window is observed | `variant` |
| A legacy command credential remains a personal access token account | A replayed pull request node does not duplicate activity | `variant` |
| A legacy environment credential remains a personal access token account | A GitHub response failure is an account failure | `variant` |
| A legacy environment credential remains a personal access token account | A reopen within the retention window is observed | `variant` |
| A legacy environment credential remains a personal access token account | A replayed pull request node does not duplicate activity | `variant` |
| A long pull request key keeps its number visible | A missing browser opener is reported and keeps the detail | `variant` |
| A long pull request key keeps its number visible | A rejected stack query retries once and shows standalone rows | `variant` |
| A long pull request key keeps its number visible | Developer opens an activity | `variant` |
| A long pull request key keeps its number visible | The activity list row is bounded to the recorded excerpt | `variant` |
| A merge performed by the configured account still notifies | A CLI usage error exits 2 | `variant` |
| A merge performed by the configured account still notifies | A GitHub response failure is an account failure | `variant` |
| A merge performed by the configured account still notifies | A delivery surface failure keeps the transcript | `variant` |
| A merge performed by the configured account still notifies | A missing configuration is a configuration error | `variant` |
| A merge performed by the configured account still notifies | A missing or failing gh is an account failure | `variant` |
| A merge performed by the configured account still notifies | A quarantine failure is a state error | `variant` |
| A merge performed by the configured account still notifies | An invalid configuration is reported with exit 2 | `variant` |
| A merge performed by the configured account still notifies | An unknown state version is quarantined and replaced | `variant` |
| A merge performed by the configured account still notifies | Deselecting a repository in all mode is refused | `variant` |
| A merge performed by the configured account still notifies | Duplicate watch scope entries are rejected | `variant` |
| A merged stack member remains in the group | Developer opens an activity | `variant` |
| A missing browser opener is reported and keeps the detail | Developer opens an activity | `variant` |
| A missing browser opener is reported and keeps the detail | Developer reviews a stacked inbox | `variant` |
| A missing configuration is a configuration error | A CLI usage error exits 2 | `variant` |
| A missing configuration is a configuration error | A GitHub response failure is an account failure | `variant` |
| A missing configuration is a configuration error | A delivery surface failure keeps the transcript | `variant` |
| A missing configuration is a configuration error | A disabled desktop notification keeps the transcript | `variant` |
| A missing configuration is a configuration error | A failed account does not stop the other account | `variant` |
| A missing configuration is a configuration error | A missing or failing gh is an account failure | `boundary` |
| A missing configuration is a configuration error | A missing sound player still delivers the notification | `variant` |
| A missing configuration is a configuration error | A missing token file is an account failure | `boundary` |
| A missing configuration is a configuration error | A one-shot check never starts a browser login | `boundary` |
| A missing configuration is a configuration error | A personal access token for a different login is an account failure | `variant` |
| A missing configuration is a configuration error | A quarantine failure is a state error | `boundary` |
| A missing configuration is a configuration error | An invalid configuration is reported with exit 2 | `boundary` |
| A missing configuration is a configuration error | An unavailable desktop surface keeps the transcript | `variant` |
| A missing configuration is a configuration error | An unknown state version is quarantined and replaced | `boundary` |
| A missing configuration is a configuration error | Developer lists unread activity | `variant` |
| A missing configuration is a configuration error | Developer runs a single check and sees the current activity | `variant` |
| A missing configuration is a configuration error | Developer sees the watch state | `variant` |
| A missing configuration is a configuration error | Duplicate watch scope entries are rejected | `boundary` |
| A missing configuration is a configuration error | Inbox with no state shows no activity | `variant` |
| A missing configuration is a configuration error | Status with no state shows empty sections | `variant` |
| A missing configuration is a configuration error | The developer tests notifications | `variant` |
| A missing configuration is a configuration error | The developer tests notifications on the command line | `variant` |
| A missing configuration is a configuration error | The notification test can mute the desktop notification | `variant` |
| A missing configuration is a configuration error | The notification test can mute the sound | `variant` |
| A missing configuration is a configuration error | The notification test reports an unavailable surface | `variant` |
| A missing configuration is a configuration error | The notification test reports disabled surfaces | `variant` |
| A missing or failing gh is an account failure | A GitHub response failure is an account failure | `variant` |
| A missing or failing gh is an account failure | A delivery surface failure keeps the transcript | `variant` |
| A missing or failing gh is an account failure | A disabled desktop notification keeps the transcript | `variant` |
| A missing or failing gh is an account failure | A failed account does not stop the other account | `variant` |
| A missing or failing gh is an account failure | A missing sound player still delivers the notification | `variant` |
| A missing or failing gh is an account failure | A missing token file is an account failure | `boundary` |
| A missing or failing gh is an account failure | A one-shot check never starts a browser login | `boundary` |
| A missing or failing gh is an account failure | A personal access token for a different login is an account failure | `variant` |
| A missing or failing gh is an account failure | An invalid configuration is reported with exit 2 | `boundary` |
| A missing or failing gh is an account failure | An unavailable desktop surface keeps the transcript | `variant` |
| A missing or failing gh is an account failure | An unknown state version is quarantined and replaced | `boundary` |
| A missing or failing gh is an account failure | Developer lists unread activity | `variant` |
| A missing or failing gh is an account failure | Developer runs a single check and sees the current activity | `variant` |
| A missing or failing gh is an account failure | Developer sees the watch state | `variant` |
| A missing or failing gh is an account failure | Duplicate watch scope entries are rejected | `boundary` |
| A missing or failing gh is an account failure | Inbox with no state shows no activity | `variant` |
| A missing or failing gh is an account failure | Status with no state shows empty sections | `variant` |
| A missing or failing gh is an account failure | The developer tests notifications | `variant` |
| A missing or failing gh is an account failure | The developer tests notifications on the command line | `variant` |
| A missing or failing gh is an account failure | The notification test can mute the desktop notification | `variant` |
| A missing or failing gh is an account failure | The notification test can mute the sound | `variant` |
| A missing or failing gh is an account failure | The notification test reports an unavailable surface | `variant` |
| A missing or failing gh is an account failure | The notification test reports disabled surfaces | `variant` |
| A missing token file is an account failure | A delivery surface failure keeps the transcript | `variant` |
| A missing token file is an account failure | An invalid configuration is reported with exit 2 | `boundary` |
| A missing token file is an account failure | Duplicate watch scope entries are rejected | `boundary` |
| A personal access token whose login differs only in case is accepted | A GitHub response failure is an account failure | `variant` |
| A personal access token whose login differs only in case is accepted | A reopen within the retention window is observed | `variant` |
| A personal access token whose login differs only in case is accepted | A replayed pull request node does not duplicate activity | `variant` |
| A pull request I did not author is watched | A GitHub response failure is an account failure | `variant` |
| A pull request I did not author is watched | A reopen within the retention window is observed | `variant` |
| A pull request I did not author is watched | A replayed pull request node does not duplicate activity | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A CLI usage error exits 2 | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A GitHub response failure is an account failure | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A delivery surface failure keeps the transcript | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A missing configuration is a configuration error | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A missing or failing gh is an account failure | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | A quarantine failure is a state error | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | An invalid configuration is reported with exit 2 | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | An unknown state version is quarantined and replaced | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | Deselecting a repository in all mode is refused | `variant` |
| A pull request merged or closed while the watcher was stopped is reported | Duplicate watch scope entries are rejected | `variant` |
| A pull request without stack metadata stays standalone | A rejected stack query retries once and shows standalone rows | `boundary` |
| A pull request without stack metadata stays standalone | The activity list row is bounded to the recorded excerpt | `variant` |
| A quarantine failure is a state error | A CLI usage error exits 2 | `variant` |
| A quarantine failure is a state error | A GitHub response failure is an account failure | `variant` |
| A quarantine failure is a state error | A delivery surface failure keeps the transcript | `variant` |
| A quarantine failure is a state error | A disabled desktop notification keeps the transcript | `variant` |
| A quarantine failure is a state error | A failed account does not stop the other account | `variant` |
| A quarantine failure is a state error | A missing or failing gh is an account failure | `boundary` |
| A quarantine failure is a state error | A missing sound player still delivers the notification | `variant` |
| A quarantine failure is a state error | A missing token file is an account failure | `boundary` |
| A quarantine failure is a state error | A one-shot check never starts a browser login | `boundary` |
| A quarantine failure is a state error | A personal access token for a different login is an account failure | `variant` |
| A quarantine failure is a state error | An invalid configuration is reported with exit 2 | `boundary` |
| A quarantine failure is a state error | An unavailable desktop surface keeps the transcript | `variant` |
| A quarantine failure is a state error | An unknown state version is quarantined and replaced | `boundary` |
| A quarantine failure is a state error | Developer lists unread activity | `variant` |
| A quarantine failure is a state error | Developer runs a single check and sees the current activity | `variant` |
| A quarantine failure is a state error | Developer sees the watch state | `variant` |
| A quarantine failure is a state error | Duplicate watch scope entries are rejected | `boundary` |
| A quarantine failure is a state error | Inbox with no state shows no activity | `variant` |
| A quarantine failure is a state error | Status with no state shows empty sections | `variant` |
| A quarantine failure is a state error | The developer tests notifications | `variant` |
| A quarantine failure is a state error | The developer tests notifications on the command line | `variant` |
| A quarantine failure is a state error | The notification test can mute the desktop notification | `variant` |
| A quarantine failure is a state error | The notification test can mute the sound | `variant` |
| A quarantine failure is a state error | The notification test reports an unavailable surface | `variant` |
| A quarantine failure is a state error | The notification test reports disabled surfaces | `variant` |
| A read command rejects invalid configuration | A CLI usage error exits 2 | `variant` |
| A read command rejects invalid configuration | A GitHub response failure is an account failure | `variant` |
| A read command rejects invalid configuration | A delivery surface failure keeps the transcript | `variant` |
| A read command rejects invalid configuration | A missing configuration is a configuration error | `variant` |
| A read command rejects invalid configuration | A missing or failing gh is an account failure | `variant` |
| A read command rejects invalid configuration | A quarantine failure is a state error | `variant` |
| A read command rejects invalid configuration | An invalid configuration is reported with exit 2 | `variant` |
| A read command rejects invalid configuration | An unknown state version is quarantined and replaced | `variant` |
| A read command rejects invalid configuration | Deselecting a repository in all mode is refused | `variant` |
| A read command rejects invalid configuration | Duplicate watch scope entries are rejected | `variant` |
| A reload failure keeps the previous scope | A CLI usage error exits 2 | `variant` |
| A reload failure keeps the previous scope | A GitHub response failure is an account failure | `variant` |
| A reload failure keeps the previous scope | A delivery surface failure keeps the transcript | `variant` |
| A reload failure keeps the previous scope | A missing configuration is a configuration error | `variant` |
| A reload failure keeps the previous scope | A missing or failing gh is an account failure | `variant` |
| A reload failure keeps the previous scope | A quarantine failure is a state error | `variant` |
| A reload failure keeps the previous scope | An invalid configuration is reported with exit 2 | `variant` |
| A reload failure keeps the previous scope | An unknown state version is quarantined and replaced | `variant` |
| A reload failure keeps the previous scope | Deselecting a repository in all mode is refused | `variant` |
| A reload failure keeps the previous scope | Duplicate watch scope entries are rejected | `variant` |
| A reopen after the retention window is not observed | A personal access token for a different login is an account failure | `variant` |
| A reopen within the retention window is observed | A GitHub response failure is an account failure | `variant` |
| A reopen within the retention window is observed | A failed account does not stop the other account | `variant` |
| A reopen within the retention window is observed | A failed browser login does not stop the other account | `variant` |
| A reopen within the retention window is observed | A personal access token for a different login is an account failure | `variant` |
| A reopen within the retention window is observed | A pull request opened after the last check is an activity | `variant` |
| A reopen within the retention window is observed | A replayed pull request node does not duplicate activity | `variant` |
| A reopen within the retention window is observed | A short-lived pull request merged or closed between checks notifies | `variant` |
| A reopen within the retention window is observed | Each configured account is observed | `variant` |
| A review comment from another person notifies | A CLI usage error exits 2 | `variant` |
| A review comment from another person notifies | A GitHub response failure is an account failure | `variant` |
| A review comment from another person notifies | A delivery surface failure keeps the transcript | `variant` |
| A review comment from another person notifies | A missing configuration is a configuration error | `variant` |
| A review comment from another person notifies | A missing or failing gh is an account failure | `variant` |
| A review comment from another person notifies | A quarantine failure is a state error | `variant` |
| A review comment from another person notifies | An invalid configuration is reported with exit 2 | `variant` |
| A review comment from another person notifies | An unknown state version is quarantined and replaced | `variant` |
| A review comment from another person notifies | Deselecting a repository in all mode is refused | `variant` |
| A review comment from another person notifies | Duplicate watch scope entries are rejected | `variant` |
| A stack is marked with a symbol and its position | A missing browser opener is reported and keeps the detail | `variant` |
| A stack is marked with a symbol and its position | Developer opens an activity | `variant` |
| A standalone pull request shows no stack symbol | A rejected stack query retries once and shows standalone rows | `variant` |
| A standalone pull request shows no stack symbol | The activity list row is bounded to the recorded excerpt | `variant` |
| A state write failure is reported | A CLI usage error exits 2 | `variant` |
| A state write failure is reported | A GitHub response failure is an account failure | `variant` |
| A state write failure is reported | A delivery surface failure keeps the transcript | `variant` |
| A state write failure is reported | A missing configuration is a configuration error | `variant` |
| A state write failure is reported | A missing or failing gh is an account failure | `variant` |
| A state write failure is reported | A quarantine failure is a state error | `variant` |
| A state write failure is reported | An invalid configuration is reported with exit 2 | `variant` |
| A state write failure is reported | An unknown state version is quarantined and replaced | `variant` |
| A state write failure is reported | Deselecting a repository in all mode is refused | `variant` |
| A state write failure is reported | Duplicate watch scope entries are rejected | `variant` |
| A state write failure is reported with a nonzero status | A CLI usage error exits 2 | `variant` |
| A state write failure is reported with a nonzero status | A GitHub response failure is an account failure | `variant` |
| A state write failure is reported with a nonzero status | A delivery surface failure keeps the transcript | `variant` |
| A state write failure is reported with a nonzero status | A missing configuration is a configuration error | `boundary` |
| A state write failure is reported with a nonzero status | A missing or failing gh is an account failure | `boundary` |
| A state write failure is reported with a nonzero status | A quarantine failure is a state error | `variant` |
| A state write failure is reported with a nonzero status | An invalid configuration is reported with exit 2 | `boundary` |
| A state write failure is reported with a nonzero status | An unknown state version is quarantined and replaced | `boundary` |
| A state write failure is reported with a nonzero status | Deselecting a repository in all mode is refused | `variant` |
| A state write failure is reported with a nonzero status | Duplicate watch scope entries are rejected | `boundary` |
| A submitted review notifies | A CLI usage error exits 2 | `variant` |
| A submitted review notifies | A GitHub response failure is an account failure | `variant` |
| A submitted review notifies | A delivery surface failure keeps the transcript | `variant` |
| A submitted review notifies | A missing configuration is a configuration error | `variant` |
| A submitted review notifies | A missing or failing gh is an account failure | `variant` |
| A submitted review notifies | A quarantine failure is a state error | `variant` |
| A submitted review notifies | An invalid configuration is reported with exit 2 | `variant` |
| A submitted review notifies | An unknown state version is quarantined and replaced | `variant` |
| A submitted review notifies | Deselecting a repository in all mode is refused | `variant` |
| A submitted review notifies | Duplicate watch scope entries are rejected | `variant` |
| A token file path starting with ~ is expanded | A GitHub response failure is an account failure | `variant` |
| A token file path starting with ~ is expanded | A reopen within the retention window is observed | `variant` |
| A token file path starting with ~ is expanded | A replayed pull request node does not duplicate activity | `variant` |
| A token file readable by other local users is refused | A CLI usage error exits 2 | `variant` |
| A token file readable by other local users is refused | A GitHub response failure is an account failure | `variant` |
| A token file readable by other local users is refused | A delivery surface failure keeps the transcript | `variant` |
| A token file readable by other local users is refused | A missing configuration is a configuration error | `boundary` |
| A token file readable by other local users is refused | A missing or failing gh is an account failure | `boundary` |
| A token file readable by other local users is refused | A quarantine failure is a state error | `boundary` |
| A token file readable by other local users is refused | An invalid configuration is reported with exit 2 | `boundary` |
| A token file readable by other local users is refused | An unknown state version is quarantined and replaced | `boundary` |
| A token file readable by other local users is refused | Deselecting a repository in all mode is refused | `variant` |
| A token file readable by other local users is refused | Duplicate watch scope entries are rejected | `boundary` |
| A token file readable only by the operator is used | A GitHub response failure is an account failure | `variant` |
| A token file readable only by the operator is used | A reopen within the retention window is observed | `variant` |
| A token file readable only by the operator is used | A replayed pull request node does not duplicate activity | `variant` |
| A version-1 state loads with empty history | A CLI usage error exits 2 | `variant` |
| A version-1 state loads with empty history | A GitHub response failure is an account failure | `variant` |
| A version-1 state loads with empty history | A delivery surface failure keeps the transcript | `variant` |
| A version-1 state loads with empty history | A missing configuration is a configuration error | `variant` |
| A version-1 state loads with empty history | A missing or failing gh is an account failure | `variant` |
| A version-1 state loads with empty history | A quarantine failure is a state error | `variant` |
| A version-1 state loads with empty history | An invalid configuration is reported with exit 2 | `variant` |
| A version-1 state loads with empty history | An unknown state version is quarantined and replaced | `variant` |
| A version-1 state loads with empty history | Deselecting a repository in all mode is refused | `variant` |
| A version-1 state loads with empty history | Duplicate watch scope entries are rejected | `variant` |
| An ignored pull request produces no activity | A CLI usage error exits 2 | `variant` |
| An ignored pull request produces no activity | A GitHub response failure is an account failure | `variant` |
| An ignored pull request produces no activity | A delivery surface failure keeps the transcript | `variant` |
| An ignored pull request produces no activity | A missing configuration is a configuration error | `variant` |
| An ignored pull request produces no activity | A missing or failing gh is an account failure | `variant` |
| An ignored pull request produces no activity | A quarantine failure is a state error | `variant` |
| An ignored pull request produces no activity | An invalid configuration is reported with exit 2 | `variant` |
| An ignored pull request produces no activity | An unknown state version is quarantined and replaced | `variant` |
| An ignored pull request produces no activity | Deselecting a repository in all mode is refused | `variant` |
| An ignored pull request produces no activity | Duplicate watch scope entries are rejected | `variant` |
| An invalid configuration is rejected | A CLI usage error exits 2 | `variant` |
| An invalid configuration is rejected | A GitHub response failure is an account failure | `variant` |
| An invalid configuration is rejected | A delivery surface failure keeps the transcript | `variant` |
| An invalid configuration is rejected | A missing configuration is a configuration error | `variant` |
| An invalid configuration is rejected | A missing or failing gh is an account failure | `boundary` |
| An invalid configuration is rejected | A quarantine failure is a state error | `boundary` |
| An invalid configuration is rejected | An invalid configuration is reported with exit 2 | `boundary` |
| An invalid configuration is rejected | An unknown state version is quarantined and replaced | `boundary` |
| An invalid configuration is rejected | Deselecting a repository in all mode is refused | `variant` |
| An invalid configuration is rejected | Duplicate watch scope entries are rejected | `boundary` |
| An invalid configuration is reported with exit 2 | A delivery surface failure keeps the transcript | `variant` |
| An invalid configuration is reported with exit 2 | A missing sound player still delivers the notification | `variant` |
| An invalid configuration is reported with exit 2 | The developer tests notifications | `variant` |
| An unavailable desktop surface keeps the transcript | A delivery surface failure keeps the transcript | `variant` |
| An unavailable desktop surface keeps the transcript | An invalid configuration is reported with exit 2 | `variant` |
| An unavailable desktop surface keeps the transcript | Duplicate watch scope entries are rejected | `variant` |
| Deselecting a repository in all mode is refused | A CLI usage error exits 2 | `variant` |
| Deselecting a repository in all mode is refused | A GitHub response failure is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A delivery surface failure keeps the transcript | `variant` |
| Deselecting a repository in all mode is refused | A disabled desktop notification keeps the transcript | `variant` |
| Deselecting a repository in all mode is refused | A failed account does not stop the other account | `variant` |
| Deselecting a repository in all mode is refused | A missing configuration is a configuration error | `variant` |
| Deselecting a repository in all mode is refused | A missing or failing gh is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A missing sound player still delivers the notification | `variant` |
| Deselecting a repository in all mode is refused | A missing token file is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A one-shot check never starts a browser login | `variant` |
| Deselecting a repository in all mode is refused | A personal access token for a different login is an account failure | `variant` |
| Deselecting a repository in all mode is refused | A quarantine failure is a state error | `variant` |
| Deselecting a repository in all mode is refused | An invalid configuration is reported with exit 2 | `variant` |
| Deselecting a repository in all mode is refused | An unavailable desktop surface keeps the transcript | `variant` |
| Deselecting a repository in all mode is refused | An unknown state version is quarantined and replaced | `variant` |
| Deselecting a repository in all mode is refused | Developer lists unread activity | `variant` |
| Deselecting a repository in all mode is refused | Developer runs a single check and sees the current activity | `variant` |
| Deselecting a repository in all mode is refused | Developer sees the watch state | `variant` |
| Deselecting a repository in all mode is refused | Duplicate watch scope entries are rejected | `variant` |
| Deselecting a repository in all mode is refused | Inbox with no state shows no activity | `variant` |
| Deselecting a repository in all mode is refused | Status with no state shows empty sections | `variant` |
| Deselecting a repository in all mode is refused | The developer tests notifications | `variant` |
| Deselecting a repository in all mode is refused | The developer tests notifications on the command line | `variant` |
| Deselecting a repository in all mode is refused | The notification test can mute the desktop notification | `variant` |
| Deselecting a repository in all mode is refused | The notification test can mute the sound | `variant` |
| Deselecting a repository in all mode is refused | The notification test reports an unavailable surface | `variant` |
| Deselecting a repository in all mode is refused | The notification test reports disabled surfaces | `variant` |
| Developer lists unread activity | A delivery surface failure keeps the transcript | `variant` |
| Developer lists unread activity | An invalid configuration is reported with exit 2 | `variant` |
| Developer lists unread activity | Duplicate watch scope entries are rejected | `variant` |
| Developer marks activity read | A CLI usage error exits 2 | `variant` |
| Developer marks activity read | A GitHub response failure is an account failure | `variant` |
| Developer marks activity read | A delivery surface failure keeps the transcript | `variant` |
| Developer marks activity read | A missing configuration is a configuration error | `variant` |
| Developer marks activity read | A missing or failing gh is an account failure | `variant` |
| Developer marks activity read | A quarantine failure is a state error | `variant` |
| Developer marks activity read | An invalid configuration is reported with exit 2 | `variant` |
| Developer marks activity read | An unknown state version is quarantined and replaced | `variant` |
| Developer marks activity read | Deselecting a repository in all mode is refused | `variant` |
| Developer marks activity read | Duplicate watch scope entries are rejected | `variant` |
| Developer navigates the inbox panes | A missing browser opener is reported and keeps the detail | `variant` |
| Developer navigates the inbox panes | Developer opens an activity | `variant` |
| Developer opens the inbox TUI | A missing browser opener is reported and keeps the detail | `variant` |
| Developer opens the inbox TUI | Developer opens an activity | `variant` |
| Developer reviews a stacked inbox | Developer opens an activity | `variant` |
| Developer sees the watch state | A delivery surface failure keeps the transcript | `variant` |
| Developer sees the watch state | An invalid configuration is reported with exit 2 | `variant` |
| Developer sees the watch state | Duplicate watch scope entries are rejected | `variant` |
| Developer sets up passuff on first run | A missing browser opener is reported and keeps the detail | `variant` |
| Developer sets up passuff on first run | A rejected stack query retries once and shows standalone rows | `variant` |
| Developer sets up passuff on first run | Developer opens an activity | `variant` |
| Duplicate watch scope entries are rejected | A delivery surface failure keeps the transcript | `variant` |
| Duplicate watch scope entries are rejected | A missing sound player still delivers the notification | `variant` |
| Duplicate watch scope entries are rejected | An invalid configuration is reported with exit 2 | `boundary` |
| Duplicate watch scope entries are rejected | The developer tests notifications | `variant` |
| Enter with no activity shows no detail | A rejected stack query retries once and shows standalone rows | `variant` |
| Enter with no activity shows no detail | The activity list row is bounded to the recorded excerpt | `variant` |
| Inbox hides activity outside the focus | A CLI usage error exits 2 | `variant` |
| Inbox hides activity outside the focus | A GitHub response failure is an account failure | `variant` |
| Inbox hides activity outside the focus | A delivery surface failure keeps the transcript | `variant` |
| Inbox hides activity outside the focus | A missing configuration is a configuration error | `variant` |
| Inbox hides activity outside the focus | A missing or failing gh is an account failure | `variant` |
| Inbox hides activity outside the focus | A quarantine failure is a state error | `variant` |
| Inbox hides activity outside the focus | An invalid configuration is reported with exit 2 | `variant` |
| Inbox hides activity outside the focus | An unknown state version is quarantined and replaced | `variant` |
| Inbox hides activity outside the focus | Deselecting a repository in all mode is refused | `variant` |
| Inbox hides activity outside the focus | Duplicate watch scope entries are rejected | `variant` |
| Inbox reports a state error | A CLI usage error exits 2 | `variant` |
| Inbox reports a state error | A GitHub response failure is an account failure | `variant` |
| Inbox reports a state error | A delivery surface failure keeps the transcript | `variant` |
| Inbox reports a state error | A missing configuration is a configuration error | `variant` |
| Inbox reports a state error | A missing or failing gh is an account failure | `variant` |
| Inbox reports a state error | A quarantine failure is a state error | `variant` |
| Inbox reports a state error | An invalid configuration is reported with exit 2 | `variant` |
| Inbox reports a state error | An unknown state version is quarantined and replaced | `variant` |
| Inbox reports a state error | Deselecting a repository in all mode is refused | `variant` |
| Inbox reports a state error | Duplicate watch scope entries are rejected | `variant` |
| Read all marks every activity read | A CLI usage error exits 2 | `variant` |
| Read all marks every activity read | A GitHub response failure is an account failure | `variant` |
| Read all marks every activity read | A delivery surface failure keeps the transcript | `variant` |
| Read all marks every activity read | A missing configuration is a configuration error | `variant` |
| Read all marks every activity read | A missing or failing gh is an account failure | `variant` |
| Read all marks every activity read | A quarantine failure is a state error | `variant` |
| Read all marks every activity read | An invalid configuration is reported with exit 2 | `variant` |
| Read all marks every activity read | An unknown state version is quarantined and replaced | `variant` |
| Read all marks every activity read | Deselecting a repository in all mode is refused | `variant` |
| Read all marks every activity read | Duplicate watch scope entries are rejected | `variant` |
| Read marks one activity read | A CLI usage error exits 2 | `variant` |
| Read marks one activity read | A GitHub response failure is an account failure | `variant` |
| Read marks one activity read | A delivery surface failure keeps the transcript | `variant` |
| Read marks one activity read | A missing configuration is a configuration error | `variant` |
| Read marks one activity read | A missing or failing gh is an account failure | `variant` |
| Read marks one activity read | A quarantine failure is a state error | `variant` |
| Read marks one activity read | An invalid configuration is reported with exit 2 | `variant` |
| Read marks one activity read | An unknown state version is quarantined and replaced | `variant` |
| Read marks one activity read | Deselecting a repository in all mode is refused | `variant` |
| Read marks one activity read | Duplicate watch scope entries are rejected | `variant` |
| Read refuses an unknown activity id | A CLI usage error exits 2 | `variant` |
| Read refuses an unknown activity id | A GitHub response failure is an account failure | `variant` |
| Read refuses an unknown activity id | A delivery surface failure keeps the transcript | `variant` |
| Read refuses an unknown activity id | A missing configuration is a configuration error | `variant` |
| Read refuses an unknown activity id | A missing or failing gh is an account failure | `variant` |
| Read refuses an unknown activity id | A quarantine failure is a state error | `variant` |
| Read refuses an unknown activity id | An invalid configuration is reported with exit 2 | `variant` |
| Read refuses an unknown activity id | An unknown state version is quarantined and replaced | `variant` |
| Read refuses an unknown activity id | Deselecting a repository in all mode is refused | `variant` |
| Read refuses an unknown activity id | Duplicate watch scope entries are rejected | `variant` |
| Read reports a state error | A CLI usage error exits 2 | `variant` |
| Read reports a state error | A GitHub response failure is an account failure | `variant` |
| Read reports a state error | A delivery surface failure keeps the transcript | `variant` |
| Read reports a state error | A missing configuration is a configuration error | `variant` |
| Read reports a state error | A missing or failing gh is an account failure | `variant` |
| Read reports a state error | A quarantine failure is a state error | `variant` |
| Read reports a state error | An invalid configuration is reported with exit 2 | `variant` |
| Read reports a state error | An unknown state version is quarantined and replaced | `variant` |
| Read reports a state error | Deselecting a repository in all mode is refused | `variant` |
| Read reports a state error | Duplicate watch scope entries are rejected | `variant` |
| Read requires an activity id or all | A CLI usage error exits 2 | `variant` |
| Read requires an activity id or all | A GitHub response failure is an account failure | `variant` |
| Read requires an activity id or all | A delivery surface failure keeps the transcript | `variant` |
| Read requires an activity id or all | A missing configuration is a configuration error | `variant` |
| Read requires an activity id or all | A missing or failing gh is an account failure | `variant` |
| Read requires an activity id or all | A quarantine failure is a state error | `variant` |
| Read requires an activity id or all | An invalid configuration is reported with exit 2 | `variant` |
| Read requires an activity id or all | An unknown state version is quarantined and replaced | `variant` |
| Read requires an activity id or all | Deselecting a repository in all mode is refused | `variant` |
| Read requires an activity id or all | Duplicate watch scope entries are rejected | `variant` |
| Read state survives a restart | A CLI usage error exits 2 | `variant` |
| Read state survives a restart | A GitHub response failure is an account failure | `variant` |
| Read state survives a restart | A delivery surface failure keeps the transcript | `variant` |
| Read state survives a restart | A missing configuration is a configuration error | `variant` |
| Read state survives a restart | A missing or failing gh is an account failure | `variant` |
| Read state survives a restart | A quarantine failure is a state error | `variant` |
| Read state survives a restart | An invalid configuration is reported with exit 2 | `variant` |
| Read state survives a restart | An unknown state version is quarantined and replaced | `variant` |
| Read state survives a restart | Deselecting a repository in all mode is refused | `variant` |
| Read state survives a restart | Duplicate watch scope entries are rejected | `variant` |
| Recording activity survives a delivery failure | A CLI usage error exits 2 | `variant` |
| Recording activity survives a delivery failure | A GitHub response failure is an account failure | `variant` |
| Recording activity survives a delivery failure | A delivery surface failure keeps the transcript | `variant` |
| Recording activity survives a delivery failure | A missing configuration is a configuration error | `variant` |
| Recording activity survives a delivery failure | A missing or failing gh is an account failure | `variant` |
| Recording activity survives a delivery failure | A quarantine failure is a state error | `variant` |
| Recording activity survives a delivery failure | An invalid configuration is reported with exit 2 | `variant` |
| Recording activity survives a delivery failure | An unknown state version is quarantined and replaced | `variant` |
| Recording activity survives a delivery failure | Deselecting a repository in all mode is refused | `variant` |
| Recording activity survives a delivery failure | Duplicate watch scope entries are rejected | `variant` |
| Reselecting catches up silently | A missing browser opener is reported and keeps the detail | `variant` |
| Reselecting catches up silently | A rejected stack query retries once and shows standalone rows | `variant` |
| Reselecting catches up silently | Developer opens an activity | `variant` |
| Reselecting catches up silently | The activity list row is bounded to the recorded excerpt | `variant` |
| Selecting a repository with no records records a baseline | A CLI usage error exits 2 | `variant` |
| Selecting a repository with no records records a baseline | A GitHub response failure is an account failure | `variant` |
| Selecting a repository with no records records a baseline | A delivery surface failure keeps the transcript | `variant` |
| Selecting a repository with no records records a baseline | A missing configuration is a configuration error | `variant` |
| Selecting a repository with no records records a baseline | A missing or failing gh is an account failure | `variant` |
| Selecting a repository with no records records a baseline | A quarantine failure is a state error | `variant` |
| Selecting a repository with no records records a baseline | An invalid configuration is reported with exit 2 | `variant` |
| Selecting a repository with no records records a baseline | An unknown state version is quarantined and replaced | `variant` |
| Selecting a repository with no records records a baseline | Deselecting a repository in all mode is refused | `variant` |
| Selecting a repository with no records records a baseline | Duplicate watch scope entries are rejected | `variant` |
| Stack membership updates after a change | A rejected stack query retries once and shows standalone rows | `variant` |
| Stack membership updates after a change | The activity list row is bounded to the recorded excerpt | `variant` |
| Status lists configured accounts before the first check | A delivery surface failure keeps the transcript | `variant` |
| Status lists configured accounts before the first check | An invalid configuration is reported with exit 2 | `variant` |
| Status lists configured accounts before the first check | Duplicate watch scope entries are rejected | `variant` |
| Status lists multiple accounts | A delivery surface failure keeps the transcript | `variant` |
| Status lists multiple accounts | An invalid configuration is reported with exit 2 | `variant` |
| Status lists multiple accounts | Duplicate watch scope entries are rejected | `variant` |
| Status reports unread and total activity | A CLI usage error exits 2 | `variant` |
| Status reports unread and total activity | A GitHub response failure is an account failure | `variant` |
| Status reports unread and total activity | A delivery surface failure keeps the transcript | `variant` |
| Status reports unread and total activity | A missing configuration is a configuration error | `variant` |
| Status reports unread and total activity | A missing or failing gh is an account failure | `variant` |
| Status reports unread and total activity | A quarantine failure is a state error | `variant` |
| Status reports unread and total activity | An invalid configuration is reported with exit 2 | `variant` |
| Status reports unread and total activity | An unknown state version is quarantined and replaced | `variant` |
| Status reports unread and total activity | Deselecting a repository in all mode is refused | `variant` |
| Status reports unread and total activity | Duplicate watch scope entries are rejected | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A CLI usage error exits 2 | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A GitHub response failure is an account failure | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A delivery surface failure keeps the transcript | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A missing configuration is a configuration error | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A missing or failing gh is an account failure | `variant` |
| Status shows the watch scope when a repository allowlist is configured | A quarantine failure is a state error | `variant` |
| Status shows the watch scope when a repository allowlist is configured | An invalid configuration is reported with exit 2 | `variant` |
| Status shows the watch scope when a repository allowlist is configured | An unknown state version is quarantined and replaced | `variant` |
| Status shows the watch scope when a repository allowlist is configured | Deselecting a repository in all mode is refused | `variant` |
| Status shows the watch scope when a repository allowlist is configured | Duplicate watch scope entries are rejected | `variant` |
| Status shows the watch scope when all repositories are watched | A CLI usage error exits 2 | `variant` |
| Status shows the watch scope when all repositories are watched | A GitHub response failure is an account failure | `variant` |
| Status shows the watch scope when all repositories are watched | A delivery surface failure keeps the transcript | `variant` |
| Status shows the watch scope when all repositories are watched | A missing configuration is a configuration error | `variant` |
| Status shows the watch scope when all repositories are watched | A missing or failing gh is an account failure | `variant` |
| Status shows the watch scope when all repositories are watched | A quarantine failure is a state error | `variant` |
| Status shows the watch scope when all repositories are watched | An invalid configuration is reported with exit 2 | `variant` |
| Status shows the watch scope when all repositories are watched | An unknown state version is quarantined and replaced | `variant` |
| Status shows the watch scope when all repositories are watched | Deselecting a repository in all mode is refused | `variant` |
| Status shows the watch scope when all repositories are watched | Duplicate watch scope entries are rejected | `variant` |
| Status uses human labels for closed and pending | A CLI usage error exits 2 | `variant` |
| Status uses human labels for closed and pending | A GitHub response failure is an account failure | `variant` |
| Status uses human labels for closed and pending | A delivery surface failure keeps the transcript | `variant` |
| Status uses human labels for closed and pending | A missing configuration is a configuration error | `variant` |
| Status uses human labels for closed and pending | A missing or failing gh is an account failure | `variant` |
| Status uses human labels for closed and pending | A quarantine failure is a state error | `variant` |
| Status uses human labels for closed and pending | An invalid configuration is reported with exit 2 | `variant` |
| Status uses human labels for closed and pending | An unknown state version is quarantined and replaced | `variant` |
| Status uses human labels for closed and pending | Deselecting a repository in all mode is refused | `variant` |
| Status uses human labels for closed and pending | Duplicate watch scope entries are rejected | `variant` |
| Status with no state shows empty sections | A delivery surface failure keeps the transcript | `variant` |
| Status with no state shows empty sections | An invalid configuration is reported with exit 2 | `variant` |
| Status with no state shows empty sections | Duplicate watch scope entries are rejected | `variant` |
| Switching to all observes every repository | A missing browser opener is reported and keeps the detail | `variant` |
| Switching to all observes every repository | Developer opens an activity | `variant` |
| Switching to selected with no retained set observes nothing | A missing browser opener is reported and keeps the detail | `variant` |
| Switching to selected with no retained set observes nothing | A rejected stack query retries once and shows standalone rows | `variant` |
| Switching to selected with no retained set observes nothing | Developer opens an activity | `variant` |
| Switching to selected with no retained set observes nothing | The activity list row is bounded to the recorded excerpt | `variant` |
| The activity detail marks an activity without a reported time as approximate | A missing browser opener is reported and keeps the detail | `variant` |
| The activity detail marks an activity without a reported time as approximate | Developer opens an activity | `variant` |
| The activity detail shows and scrolls a long comment | A rejected stack query retries once and shows standalone rows | `variant` |
| The activity detail shows and scrolls a long comment | The activity list row is bounded to the recorded excerpt | `variant` |
| The activity detail shows when the activity happened and when it was fetched | A missing browser opener is reported and keeps the detail | `variant` |
| The activity detail shows when the activity happened and when it was fetched | Developer opens an activity | `variant` |
| The activity keeps the stack label | A missing browser opener is reported and keeps the detail | `variant` |
| The activity keeps the stack label | Developer opens an activity | `variant` |
| The activity table shows when the activity happened | A missing browser opener is reported and keeps the detail | `variant` |
| The activity table shows when the activity happened | A rejected stack query retries once and shows standalone rows | `variant` |
| The activity table shows when the activity happened | Developer opens an activity | `variant` |
| The activity table shows when the activity happened | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer closes dialogs with q | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer closes dialogs with q | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer filters the activity to a pull request | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer filters the activity to a pull request | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer filters the activity to a stack | A missing browser opener is reported and keeps the detail | `variant` |
| The developer filters the activity to a stack | Developer opens an activity | `variant` |
| The developer filters the activity to unread records | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer filters the activity to unread records | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer ignores and restores a pull request | A missing browser opener is reported and keeps the detail | `variant` |
| The developer ignores and restores a pull request | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer ignores and restores a pull request | Developer opens an activity | `variant` |
| The developer ignores and restores a pull request | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer mutes notifications from the settings dialog | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer mutes notifications from the settings dialog | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer opens the key reference | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer opens the key reference | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer scrolls the pull request detail | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer scrolls the pull request detail | The activity list row is bounded to the recorded excerpt | `variant` |
| The developer sorts the watched list | A missing browser opener is reported and keeps the detail | `variant` |
| The developer sorts the watched list | A rejected stack query retries once and shows standalone rows | `variant` |
| The developer sorts the watched list | Developer opens an activity | `variant` |
| The developer sorts the watched list | The activity list row is bounded to the recorded excerpt | `variant` |
| The focus cycles between watched pull requests and activity | A missing browser opener is reported and keeps the detail | `variant` |
| The focus cycles between watched pull requests and activity | A rejected stack query retries once and shows standalone rows | `variant` |
| The focus cycles between watched pull requests and activity | Developer opens an activity | `variant` |
| The focus cycles between watched pull requests and activity | The activity list row is bounded to the recorded excerpt | `variant` |
| The inbox reconciles open pull requests when the interval elapses | A missing browser opener is reported and keeps the detail | `variant` |
| The inbox reconciles open pull requests when the interval elapses | A rejected stack query retries once and shows standalone rows | `variant` |
| The inbox reconciles open pull requests when the interval elapses | Developer opens an activity | `variant` |
| The inbox reconciles open pull requests when the interval elapses | The activity list row is bounded to the recorded excerpt | `variant` |
| The inbox shows activity observed while it is open | A missing browser opener is reported and keeps the detail | `variant` |
| The inbox shows activity observed while it is open | Developer opens an activity | `variant` |
| The last deselection observes nothing | A CLI usage error exits 2 | `variant` |
| The last deselection observes nothing | A GitHub response failure is an account failure | `variant` |
| The last deselection observes nothing | A delivery surface failure keeps the transcript | `variant` |
| The last deselection observes nothing | A missing configuration is a configuration error | `variant` |
| The last deselection observes nothing | A missing or failing gh is an account failure | `variant` |
| The last deselection observes nothing | A quarantine failure is a state error | `variant` |
| The last deselection observes nothing | An invalid configuration is reported with exit 2 | `variant` |
| The last deselection observes nothing | An unknown state version is quarantined and replaced | `variant` |
| The last deselection observes nothing | Deselecting a repository in all mode is refused | `variant` |
| The last deselection observes nothing | Duplicate watch scope entries are rejected | `variant` |
| The notification test can mute the desktop notification | A delivery surface failure keeps the transcript | `variant` |
| The notification test can mute the desktop notification | An invalid configuration is reported with exit 2 | `variant` |
| The notification test can mute the desktop notification | Duplicate watch scope entries are rejected | `variant` |
| The notification test can mute the sound | A delivery surface failure keeps the transcript | `variant` |
| The notification test can mute the sound | An invalid configuration is reported with exit 2 | `variant` |
| The notification test can mute the sound | Duplicate watch scope entries are rejected | `variant` |
| The notification test reports an unavailable surface | A delivery surface failure keeps the transcript | `variant` |
| The notification test reports an unavailable surface | An invalid configuration is reported with exit 2 | `variant` |
| The notification test reports an unavailable surface | Duplicate watch scope entries are rejected | `variant` |
| The notification test reports disabled surfaces | A delivery surface failure keeps the transcript | `variant` |
| The notification test reports disabled surfaces | An invalid configuration is reported with exit 2 | `variant` |
| The notification test reports disabled surfaces | Duplicate watch scope entries are rejected | `variant` |
| The pull request detail shows its metadata | Developer opens an activity | `variant` |
| The terminal UI shows the empty inbox | A missing browser opener is reported and keeps the detail | `variant` |
| The terminal UI shows the empty inbox | Developer opens an activity | `variant` |
| The watched pull request selection scrolls | A missing browser opener is reported and keeps the detail | `variant` |
| The watched pull request selection scrolls | Developer opens an activity | `variant` |
| The watched table hides a pull request outside the focus | A rejected stack query retries once and shows standalone rows | `variant` |
| The watched table hides a pull request outside the focus | The activity list row is bounded to the recorded excerpt | `variant` |
| The watched table shows the creator and the assignee | A missing browser opener is reported and keeps the detail | `variant` |
| The watched table shows the creator and the assignee | Developer opens an activity | `variant` |
| Two accounts share the global scope | A rejected stack query retries once and shows standalone rows | `boundary` |
| Two accounts share the global scope | The activity list row is bounded to the recorded excerpt | `variant` |

## Split-or-keep

`givn lint --change python-rebuild` reports zero `[LONG]` findings. No scenario
in the change or the permanent corpus exceeds the long-scenario threshold, so
no split-or-keep row is required. No scenario was removed and re-added by this
change: the single `@givn.modified` scenario supersedes the permanent
`Developer opens an activity's details` by title at archive time, and the
permanent copy remains until then.

## README impact

README-IMPACT: updated - Install, Prerequisites, Development, Coverage

The README was refreshed for the rebuilt product: Install names Python 3.11 and
the `passuff`/`passuff-tui` entry points; Prerequisites names `gh` 2.40+, the
Linux sound players, and Notification Center/`afplay`; Development names the
`behave` runner scopes and the strictness proof; Coverage names the two
Cobertura reports.

## Findings

The previous review's findings F1-F15 are resolved:

| ID | Previous finding | Current evidence | State |
|---|---|---|---|
| F1 | no-op step `the developer reopens passuff` | `scripts/check_steps.py` exit 0; `scripts/verify.sh:47` in every scope | resolved |
| F2 | per-capability step layout missing | seven `givn/steps/*_steps.py`, seven `givn/e2e_steps/*_e2e.py`, shared `common.py`; `design.md:368-390` matches | resolved |
| F3 | `ObservationSession` missing | `passuff/tui/session.py:15`, used at `app.py:21,323,422` | resolved |
| F4 | `passuff/status.py` missing | `passuff/status.py:11`, imported by CLI and TUI | resolved |
| F5 | dead code and missing `RecordingDelivery` | `stacking.py`, `composition.py`, `render_activity_detail`, `recording_path` absent; `tests/support/recording_delivery.py` used by `fake_notifications.py:9,30` | resolved |
| F6 | 80-character list-row excerpt | `passuff/tui/frame.py:160`; delta scenario and `givn/steps/inbox_navigation_steps.py:248-256` | resolved |
| F7 | 4 KiB body bound | `passuff/classify.py:47-54`; delta `The stored activity body is bounded to 4 KiB` | resolved |
| F8 | macOS `afplay` | `passuff/sound.py:11,31`; fake `afplay` and `osascript` twins present | resolved (macOS branch classified bucket 3) |
| F9 | watcher transcript missing | `evidence/visual/developer-starts-watching-and-is-notified-of-a-new-comment/transcript.txt` shows the notification | resolved |
| F10/F11 | arc42 and glossary symbols | `passuff.status.WatchStatusReader`, `ObservationSession`, no `StackGrouper`/`composition.py`/production recording hook; TD-005/TD-006 resolved; QS-011/QS-012 corrected | resolved |
| F12 | strictness fixtures retained without a decision | retained and recorded in `design.md:330-337` and `tasks.md:5`; `verify.sh --strictness` exits 0 | resolved |
| F13 | pty-driver mismatch in the matrix | `design.md:502` names `--observe --snapshot` and the regular pty scenario; `activity_inbox_steps.py:87-95` implements it | resolved |
| F14 | task `COMMIT` shape | 206 `- [x] COMMIT: <hash> -- Batch: <name>` lines; `givn check review` passes hash presence | resolved |
| F15 | receipt untracked | `givn/changes/python-rebuild/verification/tasks-verification.json` is tracked (`git ls-files`) | resolved |

Open findings:

| ID | Severity | Finding | Fix |
|---|---|---|---|
| C1 | major | Coverage classification leaves eight specified configuration rules without any scenario (bucket 2, missing test coverage): `docs/reference.md:219-229` specifies exit 2 for no accounts, an empty label, `reconciliation_interval_secs` below 1, a watch repository not `owner/repo`, an ignored pull request not `owner/repo#number` or duplicated, `github_user` whitespace, and an unreadable `sound_file`. The measured report shows `hits=0` at `passuff/config.py:110-111,125,131,136,140,146-148,151,155,158,161,167,171,176,182,188,191,201,204,206,219,234`, and no feature under `givn/specs/` or `givn/changes/python-rebuild/specs/` contains the corresponding messages. The Q1 delta closed the enumerated boundary list but not these rules. | Add the cases to the existing `Given a configuration with <case>` outline (or add a parallel outline), record the batch commit, regenerate the receipt, and re-run the affected scope. Do not defer: external-input validation is bucket 2, never bucket 3. |

Non-blocking observation: the four E2E step files that define phrases repeat
their capability import line twice and carry unused imports
(`givn/e2e_steps/activity_inbox_e2e.py:17-18`, `inbox_navigation_e2e.py:17-18`,
`initial_setup_e2e.py:17-18`, `pr_activity_notification_e2e.py:17-18`). This is
style polish for the backlog and does not affect behaviour or phrase ownership.

## Sign-off

- [x] Fabrication audit: clean — `check_steps.py` reports 0 no-op bodies across 17 step files; the sampled steps assert transcripts, frames, logs, or state, not "it ran".
- [x] Every checked task has a verified commit touching production code — 206 tasks, 206 batch `COMMIT:` lines; `14e0b99` contains `passuff/` and the bindings, `7a83b2d` the fix batch and step layout.
- [x] Every promised component exists — `WatchStatusReader`, `ObservationSession`, `ActivityInboxService`, the seven-plus-seven step files, `RecordingDelivery`, the macOS twins.
- [x] Every consumer uses only its declared contract — the CLI, TUI, and frame renderer consume `WatchStatusReader`/`ActivityInboxService`; no consumer imports an implementation-only surface; `status.py` reuses `inbox.py`'s internal helpers, which is the reviewed split.
- [x] Strict-mode proof present and passing — `./scripts/verify.sh --strictness` exits 0 with all three probes failing as required.
- [x] Scenario execution evidence is recorded per scope — receipt 230/230, regular 197/197, E2E 19/19, zero failed or skipped; the gate reuses the receipt at `624fe70`.
- [x] Coverage classification uses the most recent measured evidence — three distinct Cobertura reports with branch instrumentation; the delta scope is classified by static inspection because the declared measurement boundary excludes it.
- [ ] Every coverage gap classified and resolved — **fails**: C1's eight specified validation rules are classified bucket 2 and remain untested.
- [ ] Dead code deleted. Missing tests added. Hard-to-test gaps justified. — dead code and hard-to-test rows are clean; the missing tests are C1.
- [x] Redundant unit tests removed — there is no separate unit-test tree.
- [x] No `@wip` tags remain. No implementation-layer detail in the spec.
- [x] Canonical E2E policy was read from the resolved specs instruction; every normalized inventory action has one `@e2e` scenario and variants are regular scenarios.
- [x] Every E2E scenario has a real-interface primary assertion and uses the reviewed driver; no browser UI scenario is replaced by HTTP/fetch.
- [x] The local run command starts the full stack cleanly — `python -m passuff --help` exits 0; the twins start per scenario under the runner.
- [x] The exact files `verify.e2e_command` invokes were identified and read; the tree (including untracked files) has no second E2E implementation.
- [x] The E2E scope is selected explicitly (`GIVN_RESULT_SCOPE` and `--e2e`); the receipt reports zero failures and skips.
- [x] Implementation (commands, file layout, framework/driver) matches what `design.md` explicitly named; the step layout, `status.py`, `session.py`, recording double, strictness retention, and matrix row were all conformed and the design records them.
- [x] Interaction coverage verified: every inventory entry maps to a matrix row, an existing `@e2e` scenario, and step definitions using the promised driving mechanism.
- [x] No finding was excused with a classification outside the three buckets — C1 is bucket 2, not a deferral or an invented exception.

## Result

The previous review's F1-F15 are all resolved, the verification receipt is
tracked and valid, the design conformance is clean, the E2E evidence uses the
real interfaces, the arc42 and glossary match the tree, and the Visual Design
Contract's transcript evidence supports the Persona. The change still cannot
sign off: the coverage classification leaves eight specified, reachable exit-2
configuration rules with no scenario (C1), so the "executable gate is complete"
claim is not yet true for those behaviours. Add the scenarios, regenerate the
receipt, and re-run the review. This is a missing-test finding, not a
production defect.

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-1

REVIEW: FAIL
