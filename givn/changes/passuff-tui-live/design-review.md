Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Design review: passuff-tui-live

## Grilling

Questions are ranked by how likely they are to invalidate the plan.

### Q1 — Does the change probe detect every event the classifier can report?

An update or comment on an open pull request changes `updatedAt`; a CI transition
changes `statusCheckRollup.state`; a merge or close removes the pull request from
the open set. An update on an already-closed pull request is invisible to the
probe. Resolution: accepted and recorded in ADR-0007; the next full cycle
triggered by any other change sees it. No scenario asserts closed-PR freshness,
so the limitation does not contradict the spec.

### Q2 — Is the pacing formula affordable on this account?

Measured: full cycle 57 points, probe 1 point, 5,000 per hour. The formula
spends at most half the remaining allowance across the window: 5,000 remaining
gives 1.44 s, 100 gives 72 s, 10 gives 720 s. The full cycle is deferred below
100. No arithmetic path schedules past the reset. Resolution: accepted.

### Q3 — What if the account changes constantly?

Every detected change pays 57 points, so the delay widens after each cycle.
TD-003 records the query-cost reduction. The plan degrades to slower updates, not
to a stall or an overrun. Resolution: accepted as documented debt.

### Q4 — Does the open inbox duplicate the watch command?

Yes. Both observe and write the whole state file. The plan permits it and
documents it as redundant; R-028 records the possible duplicate notification and
last-writer-wins state. No lock is added; D2 records why. Resolution: accepted.

### Q5 — Can every new scenario fail in RED?

All new step phrases are undefined; the scenarios carry `@wip` and the runner
enables `.fail_on_skipped()`, so the suite fails until the steps exist. The
single-scenario command is `GIVN_SCENARIO="<title>"` with the change's features
directory. Resolution: verified in the design and by `givn lint` (only expected
`@wip` findings).

### Q6 — Is the observation loop testable without real time?

`next_delay` is a pure function over the reported allowance and a supplied
`now`; the cheap-observation and failure scenarios drive ticks directly. Only
the E2E live-update scenario uses real time, bounded at 4 seconds against a
1.44-second delay. Resolution: accepted.

### Q7 — Does the E2E drive the real interface?

The E2E invokes the real `passuff-tui` binary with `--snapshot --observe` or
`--snapshot --observe-for`, against `wiremock` as GitHub, and asserts the printed
frame. It is not a PTY; that gap is pre-existing R-019, and the interface is a
terminal UI, not a browser UI. Resolution: accepted.

### Q8 — Is every interaction inventory entry covered?

The matrix in `design.md` maps all five entries. The modified
`Developer opens the inbox TUI` scenario proves opening with observation; the new
`The inbox shows activity observed while it is open` proves the live update.
Resolution: matrix added (finding F1).

### Q9 — Does opening the inbox change the stored state?

The first observation records baselines and activity exactly as a check cycle
does; the state format is unchanged and the version stays 2. Resolution: no
change.

### Q10 — Who owns the configuration after an edit?

The observation task reloads the configuration path on every tick and sends the
new `AppConfig` with the update; the main loop swaps it before redrawing. A
scope edit in the picker takes effect on the next tick. Resolution: recorded
(finding F4).

### Q11 — What if GitHub omits the allowance?

The delay falls back to 5 seconds; the measured 1-point cost is never assumed.
Resolution: recorded (finding F5).

### Q12 — Is the visual change approved?

The layout is unchanged; the only addition is one footer line in the existing
Controls block. The existing rendered snapshots are the reference; no style tile
is warranted for a status line. Resolution: contract recorded with interface and
viewport.

### Q13 — Does every proposal question resolve?

D1 (cadence) resolves in the design and ADR-0007; D2 (watch coexistence) resolves
to no coordination; D3 (config) resolves to watch-only `poll_interval_secs`. No
carried question remains. Resolution: ledger complete.

### Q14 — Do the contracts pass the component checks?

`GitHubGateway` gains one operation for the same external boundary; it keeps one
contract. `ObservationSession` owns pacing and delivery and is consumed only by
the terminal loop. `TuiApp` owns input and rendering. `PassuffCore` owns probes
and cycles. No unit owns two contracts. Resolution: accepted.

### Q15 — Does the arc42 assessment match an independent re-derivation?

Re-derived all 12 rows: 1, 3, 4, 5, 6, 8, 9, 10, 11, 12 affected; 2 and 7
unaffected. `arc42.md` matches. Chapter 09 registers ADR-0007; chapter 11
records the ADR's bad consequences (R-002, R-028, R-029, TD-003, TD-004). No
ASCII diagrams. Resolution: verified.

### Q16 — Is the vocabulary consistent?

`Change probe` is added to the glossary; `Watcher` and `Terminal UI` are widened
to include the open inbox. The design and specs use those terms. The domain model
needs no structural edit. Resolution: verified.

## Findings and hardening

| ID | Finding | Fix applied |
|---|---|---|
| F1 | `design.md` had no Interaction Coverage Matrix, so an inventory entry could be unowned. | Added the matrix with all five entries and their evidence layers. |
| F2 | The modified E2E reused the shared `the developer opens the inbox TUI` step, which would have made every view-only snapshot scenario perform a network observation. | Added the distinct phrase `the developer opens the inbox TUI observing the account` and kept the shared phrase view-only. |
| F3 | The failure scenario asserted a retained view without a preceding successful observation and without an observation trigger. | Added the observing open and an explicit `the inbox observes the account` step. |
| F4 | The design did not define the update payload, the configuration handoff, or the failure status line. | Added `ObservationUpdate`, per-tick config reload, and footer failure text. |
| F5 | A probe response without an allowance had no defined delay. | Added the 5-second fallback and the rule that the measured cost is never assumed. |
| F6 | The scope picker was still described as its own polling loop, which would have re-introduced the removed timer. | Stated that the picker stays in the same `select!` loop. |
| F7 | Persona disposition and interface classification were not recorded. | Added both to `design.md`. |
| F8 | The quality requirements were inserted out of numeric order. | Reordered QS-024, QS-025, QS-026. |

## Branch checklist

- Scope: the proposal's behaviours map to the modified scenario plus four new
  scenarios; nothing extra.
- Tech choices: existing Rust, tokio, crossterm, ratatui, reqwest, wiremock; no
  new dependency.
- Missing scenarios: opening performs the first observation (modified); live
  update, cheap probe, allowance pacing, and failure fallback (added). Error
  paths: failure fallback covered; state errors keep existing exit codes.
- Testability: strict mode, `@wip` stubs, single-scenario command named.
- E2E fidelity: real binary, no browser, one action per E2E; the view-only
  phrase remains separate.
- Interaction coverage: matrix complete.
- Risk: allowance exhaustion and duplicate observers, both mitigated or
  documented.
- Use case and Persona: `review-activity-inbox`, `inbox-navigation` and
  `activity-inbox`; `multi-account-developer` confirmed and relevant.
- Decision ledger: D1–D3 resolved with realization points.
- Contracts and components: one contract per component; ports and adapters
  named.
- Decision timing: all decisions are foundational or current; no speculative
  abstraction.
- Visual design contract: present; terminal; 100x30; no new screen.
- Deferrals: none. Query-cost reduction is out of scope, not a deferred
  mandatory check.
- Persona disposition: recorded.
- Interface classification: terminal; no browser path exists.
- ADR qualification: ADR-0007 with the design and use case as cheaper homes for
  the surrounding detail.
- Arc42: all 12 rows re-derived and matched; chapter files contain real content;
  no ASCII diagrams.
- Ubiquitous language: glossary updated; terms used consistently.
- Structural model: no aggregate change; no-op edit avoided.

DESIGN-REVIEW: PASS
