Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Design review: passuff-notification-digest

## Grilling

### Q1 — Does the digest change the core notification contract?

No. `PassuffCore` keeps one `Notification` per pull request; classification, the `RefreshReport`, and the TUI read model are untouched. The change is the CLI delivery adapter: `process_report` renders one digest from the report's notifications and calls the delivery port once. This keeps the core free of delivery policy and the per-PR events available to the transcript.

### Q2 — What is counted?

The number of `ActivityEvent`s in the pass and the number of per-pull-request notifications. A review comment is one event; a CI failure with two failed checks is one event. This matches the existing `ActivityEvent` model and the transcript lines.

### Q3 — What happens with one pull request and one event?

The previous title, body, and transcript are preserved. The new count and digest paths only activate for `events.len() > 1` on one pull request, or for more than one notification. Existing scenarios and UX stay stable.

### Q4 — Is the desktop body bounded?

Yes: five count lines then `+N more pull requests`, matching D3. The transcript is complete. Without a cap, a backlog with dozens of PRs would recreate a long banner. The transcript carries the detail.

### Q5 — Does the sound play once?

Yes. `deliver` is called once per pass, so the sound adapter is invoked once. A sound player failure is reported once through the existing `render_delivery_errors` path. `notify-test` keeps its own single delivery.

### Q6 — Is the three-beep asset testable?

Yes. The WAV is PCM mono 44.1 kHz 16-bit; a unit test parses the header, asserts the 1.0 s duration, and counts three non-silent regions. The player selection and custom `sound_file` path are unchanged.

### Q7 — Will the next release changelog contain this change?

No with the current `cliff.toml`: it groups `feat`/`fix`-prefixed commits and skips everything else, while Givn commits carry the release note as the subject without a type prefix. The 0.1.1 changelog would be empty again. Hardening: add a `Changes` group for non-conventional, non-noise commits and skip `docs|chore|ci|test|style|refactor`. This is a tooling fix in the same change; it has no user-visible behavior and is not a scenario.

### Q8 — Is a new e2e scenario required?

No. The digest is an output variant of the existing "run one observation pass" action. The in-process fake records the desktop body and sound calls; the existing `@e2e` scenarios keep the real-interface evidence. No interaction inventory entry is added.

### Q9 — What is the most likely implementation failure?

Calling `deliver` inside the notification loop by accident, which restores one sound per PR. The `process_report` change must render once and deliver once; the digest scenario's `the configured sound is played once` and the recording-delivery log protect it.

## Branch findings

- **Scope:** the request is one digest per pass with counts and a short sound. The specs contain exactly that; the event classification and inbox are out of scope.
- **Tech choices:** existing Rust, renderer, delivery, and sound ports; no new dependency.
- **Missing scenarios:** the two delta scenarios cover the multi-PR digest and the single-PR count; the sound asset is a unit test; the single-event path is covered by the permanent scenarios.
- **Testability:** all Then steps assert transcript text, the recorded desktop body, the sound call count, or the WAV structure. Strict mode is already proven; no new step phrases.
- **E2E fidelity:** the digest is a variant; no new e2e action. The existing e2e check/watch scenarios remain.
- **Interaction coverage:** the inventory entries map to the existing check/watch e2e scenarios; the digest is a regular variant.
- **Risk:** Q9; the one-delivery rule and the sound-once assertion.
- **Use-case and Persona:** `watch-pull-requests`, Persona `multi-account-developer`; R13/R24 updated, R42/R43 added. The goal, actor, trigger, and extensions are unchanged.
- **Decision ledger:** D1-D4 are answered in the proposal and decided in the design.
- **Contracts and components:** the core contract is unchanged; `TranscriptRenderer` and `NotificationDelivery` keep their responsibilities; the digest is a renderer concern.
- **Decision timing:** all decisions are settled; no carried question.
- **Visual design contract:** terminal/desktop text only; no screen layout change.
- **Deferrals:** none.
- **Persona disposition:** `multi-account-developer` — relevance is a readable, quiet backlog notification; evidence is the digest scenario and the sound test.
- **Interface classification:** terminal plus desktop notification text; no HTML/CSS/JS or browser.
- **ADR qualification:** no new ADR; ADR-0002 owns the delivery channel and sound, and the lower-level home is the use case, specs, and design.
- **Architecture documentation:** all 12 rows were re-derived; chapters 04, 05, 06, 08, 10, 11, and 12 are updated; chapter 09 records no new ADR. The chapter files are non-placeholder and contain no ASCII art.
- **Ubiquitous language:** `Notification digest` is added to `docs/glossary.md`; the specs and design use it consistently with `Notification`.
- **Structural model:** no aggregate or relationship change; recorded in the arc42 structural assessment.

## Hardening applied

- Added the `cliff.toml` `Changes` group and noise skips so Givn release-note commits reach the release changelog.
- Added QS-030/QS-031 and R-034.

DESIGN-REVIEW: PASS
