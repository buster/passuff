Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)
PERSONAS: multi-account-developer
README-IMPACT: updated - terminal inbox keys, watch-scope picker, focus configuration, scripted flags

# Review: passuff-involvement-focus

## Implementation evidence check

- **@e2e integrity:** the delta contains 2 `@e2e` scenarios (the modified
  repository toggle and the new focus action). No scenario lost a tag; the
  pre-existing `@e2e` scenarios of the touched capabilities remain.
- **Empty step bodies:** 0 across 10 step files; no `unimplemented!` or
  `todo!` remains. The new focus, focus-state, picker-absence, and team-review
  steps all assert or drive real behavior.
- **Scenario commits:** all 8 scenarios record
  `COMMIT: 8359900 -- Batch: involvement-focus`. The commit contains the
  config schema, the discovery split, the cycle gate, the read-model filter,
  the picker rewrite, the flag replacement, the tests, and the README.
- **Strict-mode proof:** `tasks.md` records the pre-implementation run exiting 1
  with `total 7, passed 1, failed 6, skipped 0` on undefined phrases.
- **Components promised by `design.md`:** `Involvement` with `all`/`label`/
  `as_str`/`parse`, `FocusConfig` with `includes`, `CoreConfig.focus`,
  `DiscoveredPullRequest`, the three discovery searches, the union rule in the
  cycle gate, `PullRequestView.involvements`, the read-model filter,
  `ScopeEditor::set_focus`, the picker focus rows, and `--focus-set` all exist.
- **Contracts:** `InboxService`, `GitHubGateway`, and `WatchScope` keep one
  implementing component each. `TuiApp` and `InboxCli` consume the read model;
  `ScopePicker` and the binary flags consume `ScopeEditor`; no consumer imports
  an adapter implementation.
- **@e2e Then steps:** the focus e2e asserts the persisted configuration and the
  rendered picker frame; the modified toggle asserts configuration, state, CLI
  output, and the TUI frame. None asserts repository state only.
- **Browser UI:** not applicable; the interface is a ratatui terminal frame.
- **E2E command:** `./scripts/verify.sh --e2e` exports `GIVN_RESULT_SCOPE=e2e`
  and runs the features runner. The delta scope ran 2 e2e scenarios; the
  permanent e2e corpus ran 17; both exit 0.
- **Explicit scope:** `scripts/verify.sh` selects the scope with
  `GIVN_RESULT_SCOPE`; the coverage script now checks `*.feature` files only.
- **Design conformance:** the implementation matches the design's named step
  files, flags, queries, and gates. Two compatibility notes: the pre-merge
  corpus keeps the `enters` phrase bound through a selecting shim, and the
  `removed` scenario stays in the permanent file until archive removes it; both
  are recorded in `tasks.md` and change no reviewed decision.

### Interaction inventory cross-reference

| Inventory entry | Matrix row | @e2e scenario | Driving mechanism |
|---|---|---|---|
| watch-scope / toggle a repository in the watch scope | present | Developer toggles a repository in the watch scope (modified) | real `passuff-tui --snapshot --picker --scope-select/--scope-deselect` |
| watch-scope / switch the watch scope mode | present | Developer switches the watch scope mode (permanent) | real `passuff-tui --snapshot --picker --scope-mode` |
| watch-scope / narrow the focus to a subset of involvement relations | present | Developer narrows the focus in the watch scope | real `passuff-tui --snapshot --picker --focus-set authored,assigned` |
| actor / start continuous watching | present | Developer starts watching and is notified of a new comment (permanent) | real `passuff` watch process against the wiremock twin |
| actor / run one observation pass | present | Developer runs a single check and sees the current activity (permanent) | real `passuff check` |
| actor / show usage | present | Developer asks for usage (permanent) | real `passuff --help` |
| actor / show version | present | Developer asks for the version (permanent) | real `passuff --version` |
| actor / test the notification surfaces on the command line | present | The developer tests notifications on the command line (permanent) | real `passuff notify-test` |
| activity-inbox / list unread activity | present | Developer lists unread activity (permanent) | real `passuff inbox` |
| activity-inbox / mark activity read | present | Developer marks activity read (permanent) | real `passuff read --all` |
| activity-inbox / open the inbox in the terminal UI | present | Developer opens the inbox TUI (permanent) | real `passuff-tui --snapshot --observe` |
| activity-inbox / open an activity's detail | present | Developer opens an activity's details (permanent) | real `passuff-tui --snapshot --keys "enter"` |
| activity-inbox / open the pull request from an activity | present | Developer opens the pull request from an activity (permanent) | real `passuff-tui --snapshot --keys "enter,enter"` |
| inbox-navigation / navigate the watched pull requests and activity | present | Developer navigates the inbox panes (permanent) | real `passuff-tui --snapshot --keys` |
| inbox-navigation / leave the inbox open while activity arrives | present | The inbox shows activity observed while it is open (permanent) | real `passuff-tui --snapshot --observe-for` |

No inventory entry is unmapped, the new focus action has exactly one `@e2e`
scenario, and no driving mechanism changed.

## Architecture conformance from the first implementation

- The relation is captured where the search result is known; the cycle stores it
  and gates notifications in the same step that appends activity, so no consumer
  re-reads state to decide.
- `ActivityEvent` and `Notification` are unchanged.
- `Inbox` owns the read-model filter; `ScopeEditor` owns the persisted focus;
  the picker is a consumer of both, not a second source of truth.
- The core still has no CLI or terminal dependency; `cargo tree -p
  passuff-core` is unchanged.
- The removed `add` path had no observation effect; deleting it removes no
  reachable behavior.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Watched pull request records its relations | 08 Involvement and focus | Yes | Data model | Core batch | `WatchedPullRequest.involvements` | Yes |
| Discovery tags each search result | 03 context (GitHub interface) | No (interface unchanged) | Discovery table | Core batch | `github.rs` three searches | Yes |
| Focus filters the read model and unread | 08 Involvement and focus | Yes | Read model | Core batch | `inbox.rs` filter + unread | Yes |
| Focus gates notifications after recording | 06 Focus the inbox | Yes | Gate placement | Core batch | `cycle.rs` gate | Yes |
| Unknown relations stay visible | 08 Involvement and focus | Yes | Focus config | Unit tests | `FocusConfig::includes` | Yes |
| QS-033 | 10 quality | Yes | Traceability | Scenarios | `Inbox hides activity outside the focus` | Yes |
| R-036 team search availability | 11 risks | Yes | Discovery | Core batch | `search_pull_request_ids` warn path | Yes |
| Involvement relation and focus terms | glossary | Yes | Data model | Core batch | `docs/glossary.md` rows | Yes |

Chapters 01, 02, 04, 07, and 09 remain unaffected as claimed: no goal,
constraint, strategy, deployment, or ADR boundary changed.
`docs/domain-model.md` is unchanged because the aggregate and its relationships
are unchanged; the relation list is an attribute.

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

`docs/glossary.md` defines involvement relation and focus. The specs, design,
README, picker labels, configuration keys, and code use them consistently. The
removed `owner/name` manual entry no longer appears in the README, the picker
footer, or the capability interactions. The pre-merge `enters` step is a
documented compatibility binding, not a competing term.

## Coverage

`./measure-coverage.sh` ran on the candidate before this review: regular scope
5/5 scenarios, delta e2e 2/2, permanent e2e 17/17, all exit 0. Merged coverage
is 6,326/7,443 lines (85%) and 791/2,362 branches (33%). Gaps found by
inspection:

| Region | Classification |
|---|---|
| `FocusConfig::includes` empty-list branch | Covered by `set_focus_persists_the_relation_set` and the config unit test. |
| Discovery deduplication | Covered by `discovery_tags_each_search_result_with_its_relation`. |
| Notification gate | Covered by the notification scenario and the read-model unit test. |
| Picker relation toggle and cursor arithmetic | Covered by `picker_switches_mode_toggles_and_narrows_the_focus`. |
| `--focus-set` invalid value | Exercised by the parse-and-exit branch; no scenario for the error path, which mirrors the existing `--scope-mode` handling. |

No dead code remains: `add`, `--scope-add`, the insert state, and the add
rendering are gone; `Involvement::parse` is used by the flags and the steps.

## Overlap dispositions

| Scenario | Decision | Reason |
|---|---|---|
| Developer toggles a repository in the watch scope | KEEP | It exercises the picker's repository toggle through the real binary: config write, picker frame, TUI frame, and retained state. The similar-shaped permanent scenarios assert either the picker only ("The picker shows the known repositories") or a scope outcome after a check ("Deselecting hides status and activity"). Merging would drop the manual toggle leg through the real interface. |
| Inbox hides activity outside the focus | KEEP | It asserts the focus rule (R9): a team-requested pull request is watched, recorded, hidden from `inbox --all`, excluded from the unread count, and the authored pull request stays visible. The repository-scope scenarios assert R2 with a deselected repository, a different rule and a different observable. Merging would conflate two independent filters. |

## Findings

| ID | Finding | Disposition |
|---|---|---|
| F1 | The permanent corpus still uses the manual-entry phrase until archive removes its scenario. | Bound to `select` as a shim, recorded in `tasks.md`; the delta removes the scenario and the modified toggle drops the phrase at archive. |
| F2 | The focus action needed a deterministic real-process surface. | Added `--focus-set` with the accepted values in the error, used by the e2e scenario. |
| F3 | The pre-existing unlisted `notify-test` e2e scenario was cleared in the previous change. | Already fixed; the lint gate is clean. |

No critical finding remains.

VISUAL-REVIEW: PASS

REVIEW: PASS
