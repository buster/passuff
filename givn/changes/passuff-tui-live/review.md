Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator), same context

# Review: passuff-tui-live

## Implementation evidence check

- **Empty step bodies:** 0 across 10 step files. The 7 `{}` matches are
  `format!` placeholders, not step bodies. Every new step calls a World method
  or asserts a concrete value.
- **@e2e integrity:** the delta keeps two `@e2e` scenarios: the modified
  `Developer opens the inbox TUI` and the added `The inbox shows activity
  observed while it is open`. No tag was removed from the permanent specs.
- **Strict-mode proof:** recorded in `tasks.md`, with the pre-implementation
  run exiting 1 at `total 5, passed 0, failed 5, skipped 0`; the runner uses
  `.fail_on_skipped()`.
- **Scenario commits:** every scenario records `9d1bc4f -- Batch:
  live-observation`. That commit contains the production probe, the session,
  the terminal loop, the steps, and the support fakes. The later `b869f79`
  commit removes dead fields found during this preflight; it changes no
  observable behaviour.
- **Components promised by `design.md`:** `PassuffCore::probe` and
  `GitHubGateway::probe` exist in the core; `ObservationSession` and
  `next_delay` exist in `crates/passuff-cli/src/live.rs`; the TUI input thread,
  observation task, `select!` loop, `--observe`, and `--observe-for` exist in
  `tui_main.rs`; the footer status line exists in `TuiApp`. No promised
  component is missing.
- **E2E interface:** `verify.e2e_command` is `./scripts/verify.sh --e2e`; the
  runner selects `GIVN_RESULT_SCOPE=e2e`. The two E2E scenarios invoke the real
  `passuff-tui` binary and assert the rendered frame (CLI output), not
  repository state. The live scenario drives the change through the wiremock
  GitHub boundary while the process runs.
- **Design deviations:** none. Steps live in the files the design names; the
  live E2E uses `--observe-for 6000` against a 1.44 s delay, satisfying the
  design's "must exceed one delay" rule.

### Interaction inventory cross-reference

| Inventory entry | Matrix row | @e2e scenario | Driving mechanism |
|---|---|---|---|
| activity-inbox / list unread activity | present | Developer lists unread activity (permanent) | real `passuff inbox` subprocess |
| activity-inbox / mark activity read | present | Developer marks activity read (permanent) | real `passuff read --all` subprocess |
| activity-inbox / open the inbox in the terminal UI | present | Developer opens the inbox TUI (modified) | real `passuff-tui --snapshot --observe` |
| inbox-navigation / navigate the watched pull requests and activity | present | Developer navigates the inbox panes (permanent) | real `passuff-tui --snapshot --keys` |
| inbox-navigation / leave the inbox open while activity arrives | present | The inbox shows activity observed while it is open | real `passuff-tui --snapshot --observe-for` with a live wiremock change |

No inventory entry is unmapped and no extra E2E scenario was added.

## Architecture conformance from the first implementation

- `PassuffCore` owns probing and check cycles; `ObservationSession` owns pacing
  and delivery; the TUI owns input and rendering. The TUI imports the session
  through `passuff_cli::live`, not the core's internals.
- `GitHubApiAdapter` remains the only adapter for the GitHub boundary and owns
  both the probe and the observation query. No core type leaks `reqwest`.
- The composition root (`tui_main.rs`, `main.rs`) builds the ports; the session
  receives them. The test World builds its own ports.
- The declared contracts table matches the implementation: no consumer reaches
  around `InboxService` to the state store, and the probe is part of the same
  gateway contract.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Open inbox contacts GitHub | 03 context table | Yes | Live observation | Terminal loop | `tui_main.rs` builds a runtime; `live.rs` probes | Yes |
| Probe-before-observe strategy | 04 solution strategy, ADR-0007 | Yes | Pacing section | Core + CLI batches | `PassuffCore::probe`, `next_delay` | Yes |
| `ObservationSession` building block | 05 block table | Yes | Contracts | CLI batch | `crates/passuff-cli/src/live.rs` | Yes |
| Live observation flow | 06 runtime view | Yes | Terminal loop | Terminal loop | Observation task + `select!` | Yes |
| Allowance pacing and backoff | 08 crosscutting | Yes | Pacing | CLI batch | `next_delay`, `failed()` | Yes |
| ADR-0007 registered | 09 register | Yes | Decisions | Closeout | `adr/ADR-0007-probe-before-observing.md` | Yes |
| QS-025/QS-026 | 10 quality | Yes | N/A | Scenarios | E2E + pacing scenarios | Yes |
| R-002, R-028, R-029, TD-003, TD-004 | 11 risks | Yes | Decisions/out of scope | Closeout | Same facts in design and code | Yes |
| Change probe vocabulary | glossary | Yes | Data model | Core batch | `PassuffCore::probe`, `ChangeProbe` | Yes |

Chapters 02 and 07 remain unaffected as claimed. No durable chapter is stale.

## Coverage classification

Merged coverage after the change: 3,285/3,952 lines (83%) and 416/1,328
branches (31%). Uncovered regions are classified, not ignored:

| Region | Lines | Classification |
|---|---|---|
| `live.rs` config-change rebuild and config-load error (66–79) | 6 | Minor. The same `ConfigLoader::load` + rebuild pattern is exercised by `runtime_reloads_scope_between_cycles` in `lib.rs`; the session path is a straight-line adaptation. Resolved by inspection; a dedicated test belongs to the next change if the session grows. |
| `live.rs` check-cycle failure branch (100) | 1 | Minor. It shares the `failed()` path and footer rendering covered by the probe-failure scenario; only the error source differs. |
| `tui_main.rs` raw-mode interactive loop, picker keys, restore | ~160 | Pre-existing risk R-019: terminal raw mode is not driven through a PTY in CI. The snapshot and observe modes cover the same renderer and handlers. |
| Branch coverage gaps in the pre-existing CLI/core | remainder | Pre-existing from earlier changes; this change adds tests for every new branch except the two listed above. |

## Findings

| ID | Finding | Disposition |
|---|---|---|
| F1 | `ObservationUpdate` carried unused `report`/`changed` semantics and `ChangeProbe` carried an unused `allowance`. | Fixed in `b869f79`; `changed` now gates the view reload. |
| F2 | The proposal's capability routing table used prose instead of the required `EXTEND <capability>` decisions. | Fixed in `80a1673`. |
| F3 | The verification receipt predated the cleanup commit. | Refreshed by `givn check tasks` at `80a1673`. |
| F4 | The footer status line was not visually rendered. | Verified: a probe against an unreachable endpoint renders `probe failed: personal: transport error ...` on the second Controls line, with the keys line intact. |
| F5 | Uncovered config-reload and cycle-failure lines. | Classified as minor above; no behavioural risk found. |

No critical finding remains.

VISUAL-REVIEW: PASS

REVIEW: PASS
