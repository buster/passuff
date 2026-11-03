Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Design review: passuff-activity-detail

## Grilling

### Q1 — Does the popup have enough data, and is any of it unpublished?

The popup reads `ActivityRecord` and `PullRequestView`. `PullRequestView` lacks
the URL, so the design adds `url` from `WatchedPullRequest.url`. Comment and
review bodies remain absent from the read model, so the popup cannot leak them;
T7 still holds. Resolution: accepted.

### Q2 — What happens when no pull request view matches the activity?

The activity records a `PullRequestKey`; the watched view may lack it (outside
the scope, or removed). `ActivityDetail.url` stays `None`, Enter in the popup
does nothing, and the popup shows the activity without the PR section. No crash.
Resolution: recorded in the data model (`Option<PullRequestView>` / `Option<String>`).

### Q3 — Does Escape still quit?

The modal branch precedes the existing match; Escape closes the popup only when
one is open, and keeps the quit meaning otherwise. `q` still quits. Resolution:
accepted; T2 updated.

### Q4 — Is the side effect testable without a browser?

`handle_key` returns `OpenUrl`; only the binary adapter spawns the opener. The
E2E injects a fake `xdg-open` in `PATH` and reads its log; the in-process World
uses a recording/failing opener. Resolution: accepted.

### Q5 — What if the opener is missing or exits non-zero?

`SystemBrowserOpener::open` maps spawn errors and non-zero exits to a message;
the binary sets the footer status and keeps the popup. R-030 records it.
Resolution: accepted.

### Q6 — Does an observation update disturb the popup?

`TuiApp::reload` replaces the view but leaves `popup` untouched; the overlay is a
snapshot taken at Enter. Resolution: recorded in the design.

### Q7 — Are both new actions covered by exactly one E2E each?

Yes: `Developer opens an activity's details` and `Developer opens the pull
request from an activity`. The Escape, empty-activity, and failure cases are
regular scenarios: variants of the same two actions. Resolution: accepted.

### Q8 — Does the popup fit 100x30?

70% x 9 cells leaves the inbox visible around it; six content lines fit five
inner rows plus the border. Long titles clip. Resolution: accepted.

### Q9 — Is the new adapter inside the architecture boundary?

`TuiApp` stays process-free; `SystemBrowserOpener` is a CLI-package adapter with
one external mechanism, built at the composition root (`tui_main`). No core
change beyond the URL field. Resolution: accepted.

### Q10 — Does every decision resolve?

D1 (success closes, failure keeps), D2 (activity-only Enter), D3 (`xdg-open` /
`open`, failures reported) each name their realization. No carried question.
Resolution: ledger complete.

## Findings and hardening

| ID | Finding | Fix applied |
|---|---|---|
| F1 | The read model did not expose the pull request URL, so the popup could not open the browser. | Added `PullRequestView.url` to the data model and the view traceability. |
| F2 | The design did not say what happens to an open popup when an observation update reloads the view. | Recorded that the popup survives as a snapshot. |
| F3 | The in-process steps had no defined opener, so the failure scenario could not exercise the side effect. | The World holds a recording/failing opener; the binary uses the system adapter. |
| F4 | The delta's `--keys` values (`enter`, `esc`) were not part of the existing mapping. | Added to the snapshot surface in the design. |

## Branch checklist

- Scope: the proposal's two actions plus three invariants; nothing extra.
- Tech choices: existing Rust/ratatui; no new dependency; opener as a child process.
- Missing scenarios: detail content, open, close, empty, failure — all present.
- Testability: modal state is deterministic; the opener is an injected adapter; strict mode and single-scenario command named.
- E2E fidelity: real binary and fake platform opener; assertions on the rendered frame and the recorded URL.
- Interaction coverage: two inventory rows, two E2E scenarios.
- Risk: missing opener (R-030).
- Use case and Persona: `review-activity-inbox`, `activity-inbox`, `multi-account-developer`.
- Decision ledger: D1–D3 resolved.
- Contracts and components: `TuiApp` and `SystemBrowserOpener` split by side effect.
- Decision timing: all foundational.
- Visual contract: terminal, 100x30, existing styles.
- Deferrals: none.
- Persona disposition: confirmed and relevant.
- Interface classification: terminal; no browser UI exists (the browser only receives a URL).
- ADR qualification: no new ADR; the opener is a small adapter under ADR-0003.
- Arc42: 10 of 12 chapters affected; 2 and 7 unaffected; chapters updated.
- Ubiquitous language: "Activity detail" added to the glossary.
- Structural model: no aggregate change; read-model field only.

DESIGN-REVIEW: PASS
