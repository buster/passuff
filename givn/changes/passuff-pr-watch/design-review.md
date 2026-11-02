# Design Review: passuff-pr-watch

Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: self-review — main authoring context; a read-only review analyst subagent supplied evidence; this is not an independent reviewer.

## Verdict

DESIGN-REVIEW: PASS

The five operator decisions are closed and the sixteen grilling findings are
resolved. Two hardening passes fixed the contract/component integrity,
traceability identifiers, runner and report testability, coverage probes, the
D-Bus E2E plan, and the durable documentation. The final focused verification
found no blockers.

## Independent arc42 assessment

The reviewer independently walked the twelve chapters before reading
`arc42.md`.

| # | Chapter | Reviewer says | arc42.md says | Note |
|---|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Yes | Requirements, quality goals, reusability, permanent specification path. |
| 2 | 02-architecture-constraints | Yes | Yes | Core/UI boundary and workspace package selection. |
| 3 | 03-context-and-scope | Yes | Yes | GitHub, desktop surface, audio player, config/state. |
| 4 | 04-solution-strategy | Yes | Yes | Polling, pure classifier, library-first core. |
| 5 | 05-building-block-view | Yes | Yes | `passuff-config` / `passuff-core` / `passuff-cli` split, including `with_ports` and the known closed/merged query. |
| 6 | 06-runtime-view | Yes | Yes | Core refresh, CLI delivery. |
| 7 | 07-deployment-view | Yes | Yes | Workstation CLI and the required macOS bundle for stable notification identity. |
| 8 | 08-crosscutting-concepts | Yes | Yes | Credentials, persistence, adapter boundary, delivery. |
| 9 | 09-architecture-decisions | Yes | Yes | ADR-0001/0002/0003 with aligned titles, durable links, and cheaper-home notes. |
| 10 | 10-quality-requirements | Yes | Yes | Reusability and Linux desktop-testability scenarios recorded. |
| 11 | 11-risks-and-technical-debt | Yes | Yes | Core API risk and missing inbox query API recorded. |
| 12 | 12-glossary | Yes | Yes | Notification redefined; Observation, Account failure, Event family, Short-lived pull request, and Reopen recorded. |

All twelve chapter files exist, contain decision-specific content, and use
Mermaid only. No ASCII-art diagrams were found.

## Grilling log

| ID | Branch | Finding | Recommended action | Status |
|---|---|---|---|---|
| Q1 | Missing scenarios / correctness | A pull request first seen on a later account cycle has no baseline; step 3 treats existing comments as new. | Baseline the first observation of each pull request, not the first account cycle. Add a first-seen-later scenario. | Resolved — hardened |
| Q2 | Use-case / traceability | The design cites `G1`–`G8`; the use case has unlabelled Minimal/Success guarantees and `R1`–`R22`. `docs/domain-model.md` cites a phantom `G5`. | Replace `G` IDs with `R` IDs and explicit guarantee labels; add capability and Persona rows. | Resolved — hardened |
| Q3 | Contracts and components | `NotificationDelivery` names two implementing components; `RecordingSoundPlayer` is unlisted; transcript rendering has no contract; split triggers are replacement triggers; `ConfigLoader` combines parsing and credential process spawning; `accounts()` has no consumer; UI settings live in core `Config`; `Notification.title` and `AccountFailure.message` are presentation strings; `CoreError` is undefined. | One production component per contract; classify recording adapters as test doubles; add `TranscriptRenderer`; split `CredentialSource`; remove `accounts()` until used; split `CoreConfig`/`UiConfig`; return typed data and define `CoreError`. | Resolved — hardened |
| Q4 | Missing scenarios | Extensions 2a, 7a, 7b, R20 desktop disable, first-seen-later, short-lived PR, reopen, CI `ERROR`, own-review, and exit 1/3 lack scenarios. | Add the missing regular scenarios; parameterise CI failure; assert exit 1 in the failed-account scenario. | Resolved — hardened |
| Q5 | Scope | Proposal promises "choose the sound", "toggle each event family", and silence for label/draft/assignee/milestone/base-branch changes; the use case has no such rules. | Operator decision: expand the spec or narrow the proposal. | Resolved — expand spec |
| Q6 | Missing scenarios / behaviour | Design notifies for a PR created and merged between checks, and resets `state_reported` on reopen; neither behaviour is in the use-case rules or the feature. | Operator decision: keep and specify, or remove. | Resolved — keep both |
| Q7 | Testability | The strict-mode proof names APIs and flags that need verification against cucumber-rs 0.23; the design does not name `[[test]] harness = false`, the `passuff` bin target, or the RED stub pattern. | Verify the cucumber API; prove strictness with an undefined/`unimplemented!()` step; name the test and bin targets; add the stub pattern. | Resolved — hardened |
| Q8 | Testability | `RefreshReport` has no baseline field, but regular scenarios assert "Fresh baseline recorded"; transcript strings and report assertions are mixed. | Add `RefreshReport.baselines_recorded`; add a `TranscriptRenderer` contract; align exact strings. | Resolved — hardened |
| Q9 | Coverage | E2E probes do not match what the E2E harness or spawned binary actually exercises; the child `LLVM_PROFILE_FILE` ownership is unclear; the regular/E2E feature paths are unspecified for a first change. | Fix the probes; state who sets the child profile path; point `GIVN_FEATURES` at the delta; name the actual classifier function. | Resolved — hardened |
| Q10 | Deferral / E2E | The real desktop adapter is only exercised by a manual smoke test; E2E uses `RecordingDelivery`. | Operator decision: accept the deferral, record the condition and closure date, and require the manual smoke before archive. | Resolved — notify-rust + Linux D-Bus fake |
| Q11 | ADR / arc42 | ADR-0001 title mismatch; ADR confirmations overclaim evidence; chapter 01/07/10 wording is stale or unverified; chapter 09 and ADR links point at the transient change. | Align titles; soften confirmations; fix wording; repoint links to permanent artifacts; add the cheaper-home sentence per NEW_ADR. | Resolved — hardened |
| Q12 | Ubiquitous language / structure | `Notification` is defined by desktop and transcript delivery, conflicting with the UI-agnostic core; Observation and Account failure are undefined; the domain model cites a phantom `G5` and omits the short-lived PR path. | Confirm the new Notification definition; add Observation and Account failure; reconcile code symbols; fix the domain-model relationship. | Resolved — redefine Notification |
| Q13 | Visual contract / Persona | No Persona disposition, accessibility line, or explicit Web UI rejection; the rendered-reference pointer is wrong. | Add Persona disposition, accessibility, and the opposite-classification sentence; point at a real transcript sample. | Resolved — hardened |
| Q14 | Scope / future TUI | The design records TUI visual details and pins ratatui/crossterm versions; this could read as a silent commitment. | Mark the TUI details as non-binding follow-up context or trim them. | Resolved — keep as non-binding context |
| Q15 | Risk | The custom cucumber runner, strict mode, result contract, and coverage scopes are the likely setup failure point. | Setup task must prove the runner, strictness, four-count result, package coverage, and a real binary before GREEN. | Resolved — hardened |
| Q16 | Tech choices / runnability | The design says `givn/commands.yaml` declares the runner and run commands; the file still contains sentinels. | Reword to "the setup task sets" and name the exact values, or update `commands.yaml`. | Resolved — hardened |

## Operator decisions

### Q5 — Scope: proposal promises not in the use case

- Status: closed 2026-11-02 (operator)
- Finding: `proposal.md` promises sound choice, per-event-family toggles, and
  silence for label, draft, assignee, milestone, and base-branch changes. The
  use case has only desktop/sound disable (R20) and the bot toggle (R12). The
  design's config has six event toggles and a `sound_file` setting.
- Options:
  1. **Expand the spec (recommended).** Add use-case rules and regular
     scenarios for the six event-family toggles, the configured sound, and the
     non-notified change types. Gain: proposal, use case, feature, and config
     agree; the configuration surface is tested. Cost: a few more scenarios and
     a slightly larger first change. Example: `events.comments = false`
     suppresses a new comment; a label-only change produces no notification.
  2. **Narrow the proposal.** Remove sound choice and per-event-family toggles
     from the proposal; keep desktop/sound on/off and bot-comment configuration.
     Gain: a smaller spec. Cost: the design's six toggles become undocumented
     or unimplemented. Example: `events.comments` is removed and comments always
     notify.
- Recommended: option 1.
- Resolution: expand the spec. Add use-case rules and regular scenarios for the six event-family toggles, the configured sound, and silence for label, draft, assignee, milestone, and base-branch changes.
- Rejected: narrow the proposal — it would leave the design's six event toggles and `sound_file` undocumented or unimplemented and remove configuration the operator wants.

### Q6 — Short-lived pull requests and reopen behaviour

- Status: closed 2026-11-02 (operator)
- Finding: Design step 2 notifies for an authored PR created and merged
  between two checks. Design step 6 resets `state_reported` on reopen. Neither
  is in the use-case rules or the feature.
- Options:
  1. **Keep and specify.** Add a rule and scenario for a short-lived PR and a
     unit/scenario for reopen. Gain: the implementation matches the design.
     Cost: more spec surface. Example: a PR created and merged between two
     checks produces one `Merged` notification.
  2. **Remove both.** Notify only for PRs that were observed while open; do not
     reset on reopen. Gain: simpler spec. Cost: the short-lived edge is missed;
     a reopened-and-merged PR may not notify. Example: a PR created and merged
     between checks produces nothing.
- Recommended: option 1 for short-lived PRs; keep reopen reset with a focused
test.
- Resolution: keep both. Add a use-case rule and regular scenario for a
  short-lived PR, and a focused test for the reopen reset.
- Rejected: removing both would miss a PR created and merged between checks and
  change reopen semantics; keeping only one would leave the other design
  behaviour unspecified.

### Q10 — Manual desktop verification deferral

- Status: closed 2026-11-02 (operator)
- Finding: the real desktop adapter is exercised only by a manual macOS/Linux
  smoke test; E2E uses `RecordingDelivery`.
- Project license decision (2026-11-02): GPL-3.0-or-later, matching givn and
  watn. `notify-rust` (MIT OR Apache-2.0) and `user-notify`
  (LGPL-3.0-or-later) are both GPLv3-compatible. The LGPL relinking obligation
  is satisfiable by distributing the source and build scripts under the
  project license. The license does not decide the crate.
- Crate investigation (2026-11-02):
  - `notify-rust 4.18.2` is the abstraction already in the design. It is
    MIT OR Apache-2.0, has about 18.3M downloads, and wraps Linux XDG D-Bus and
    macOS notifications. It has no mock backend.
  - `user-notify 0.4.2` is a higher-level manager with permission handling, a
    mock backend, and macOS `UNUserNotificationCenter`. It is
    LGPL-3.0-or-later (compatible with the project's GPL-3.0-or-later), has
    about 16.6k downloads, wraps `notify-rust` on Linux, requires a signed app
    bundle and an Apple developer account on macOS, and its builder has no
    explicit sound field. It is not selected.
  - `tauri-plugin-notification` is Tauri-only. `libnotify` is Linux-only.
    `mac-notification-sys` is macOS-only.
- Options:
  1. **Keep `notify-rust` and accept the manual smoke (recommended).** Record
     the manual smoke command and make a passing run a condition before
     archive. Gain: the mature, license-compatible abstraction stays; E2E
     stays deterministic. Cost: a desktop regression can escape CI until the
     smoke runs. Example: `passuff check` shows a real toast and sound on Linux
     and macOS during the release check.
  2. **Keep `notify-rust` and add a Linux D-Bus fake.** Start a private session
     bus and a mock `org.freedesktop.Notifications` service, then keep macOS
     manual. Gain: the Linux desktop path is automated. Cost: extra test
     infrastructure and time. Example: the E2E harness starts the mock service
     and asserts the title and body.
  3. **Switch to `user-notify` and use its mock backend.** Gain: a
     manager-level mock and permission APIs; the LGPL license is compatible
     with the project's GPL-3.0-or-later. Cost: a much younger crate, a signed
     macOS bundle plus Apple developer account, no explicit sound field, and
     the real OS surface still needs a smoke test. Example: the test uses the
     mock manager, but release still needs a signed macOS bundle and a real
     notification check.
- Recommended: option 1. The `NotificationDelivery` port is the real
  abstraction; the underlying crate is replaceable without changing the core.
- Resolution: keep `notify-rust`; add a Linux-only private D-Bus fake for the
  E2E scope. The E2E starts `dbus-daemon --session`, registers a mock
  `org.freedesktop.Notifications` service, and points the child at the private
  bus. `DesktopDelivery` and the real `notify-rust` adapter are used on Linux
  E2E; macOS keeps `RecordingDelivery` plus the manual smoke. The manual macOS
  smoke remains a condition before archive.
- Rejected: manual-only acceptance (operator wants Linux E2E coverage);
  switching to `user-notify` (younger crate, signed macOS bundle and Apple
  developer account, no explicit sound field; the license is compatible);
  automating both platforms now (macOS infrastructure cost is disproportionate
  for this change).

### Q12 — Notification term redefinition

- Status: closed 2026-11-02 (operator)
- Finding: `docs/glossary.md` defines Notification by desktop and transcript
  delivery. The core now returns a grouped report; delivery belongs to the
  adapter.
- Options:
  1. **Redefine Notification (recommended).** A Notification is the grouped
     report of activity events for one pull request; the CLI or TUI delivers it.
     Gain: the domain term matches the core contract. Cost: a glossary edit.
     Example: `PassuffCore::refresh` returns notifications; `passuff-cli`
     renders and delivers them.
  2. **Keep the old definition.** Gain: no glossary change. Cost: the term
     contradicts the UI-agnostic core and the design.
- Recommended: option 1.
- Resolution: redefine Notification as the grouped report of activity events for one pull request. Delivery is the adapter's concern, not part of the term.
- Rejected: introducing a separate `ActivityReport`/`NotificationBatch` core term (two terms for one user-facing thing); keeping the old definition (contradicts the UI-agnostic core).

### Q14 — Future TUI context

- Status: closed 2026-11-02 (operator)
- Finding: the design records TUI visual details and pins ratatui/crossterm
  versions before the TUI change exists.
- Options:
  1. **Keep as non-binding context (recommended).** Label the TUI section as
     direction only; the TUI change will record its own contract and versions.
     Gain: the user's style direction is preserved. Cost: a future design must
     still confirm it.
  2. **Trim it.** Remove the visual details and version pins; keep only the
     core/UI boundary. Gain: no implied commitment. Cost: the watn style
     reference is lost from the durable plan.
- Recommended: option 1.
- Resolution: keep the future TUI section as non-binding context. The TUI change records its own Visual Design Contract, layout, palette, and crate versions; this change only fixes the core/UI boundary.
- Rejected: trimming the section — it would lose the operator's watn-style direction from the durable plan.

## Hardening plan

| Area | Action |
|---|---|
| Classification | Baseline every first-seen authored PR; keep the short-lived exception only if Q6 keeps it; add the first-seen-later scenario. |
| Traceability | Replace `G1`–`G8` with `R` IDs and guarantee names; add capability and Persona rows. |
| Contracts | One production component per contract; recording adapters as test doubles; add `TranscriptRenderer`; split `CredentialSource`; remove `accounts()` until used; split `CoreConfig`/`UiConfig`; define `CoreError`. |
| Report types | Add `RefreshReport.baselines_recorded`; return `Notification.key` plus events; type `AccountFailure`; align transcript strings. |
| Runner | Verify cucumber-rs 0.23 APIs; name `[[test]] harness = false` and `[[bin]] name = "passuff"`; add the RED stub pattern; remove or handle `--nocapture`. |
| Coverage | Fix probes and child profile ownership; point `GIVN_FEATURES` at the delta for the first change; name actual functions. |
| E2E desktop | Add a private `dbus-daemon` session and a mock `org.freedesktop.Notifications` service for Linux E2E; use the real `DesktopDelivery`/`notify-rust` path there; keep `RecordingDelivery` for macOS and non-desktop scenarios; record the macOS manual smoke condition. |
| Visual contract | Add Persona disposition, accessibility, and Web UI rejection; fix the rendered-reference pointer. |
| Glossary / domain model | Redefine Notification; add Observation and Account failure; reconcile code symbols; fix the phantom `G5` and short-lived-PR relationship. |
| Arc42 / ADR | Fix stale wording, titles, transient links, unverified CI claim, bundle "optional" wording, and cheaper-home notes. |
| Runnability | Reword `commands.yaml` claims to future setup; name exact verify/e2e/run values. |
| TUI | Mark the future TUI section as non-binding context and remove environment-specific paths. |
| License | Add `LICENSE` with GPL-3.0-or-later, set the workspace `license`, and update the README License section. |

## Second hardening pass

The hardened artifacts now include the CLI library entry (`CliRuntime`, `run_check`,
`run_watch`), the three-crate config/core/CLI split, the `CredentialSpec` /
`CredentialSource` distinction, the nested baseline/current state, the
`createdAt` short-lived rule, the `ci_failed` recovery flag, 30-day reopen
tracking, the Linux private D-Bus fake with explicit address and readiness,
the `test-support` feature gate, the unit-test coverage process, and the typed
`RenderedNotification`/`SoundOutcome` delivery surface.

Rejected finding: `Level: !` is the valid Givn leaf level, not a placeholder.

The focused final verification passed with no remaining blockers. The three
non-blocking wording notes it recorded were corrected in the design and chapter
08.

## Sign-off

DESIGN-REVIEW: PASS
