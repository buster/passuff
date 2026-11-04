Review kind: self-review
Author: passuff maintainer (operator)
Reviewer context: passuff maintainer (operator)

# Review: passuff-notification-digest

## Implementation evidence check

- **Empty step bodies:** 0 found. The change adds no step definition; the two delta scenarios reuse the existing `pr_activity_notification.rs` phrases, all with assertions or real setup.
- **Scenario commits:** `2325069` (Batch `notification-digest`) implements `render_notification`/`render_digest`, the single `process_report` delivery, the transcript unit tests, and the delta scenarios; `17fa820` (Batch `sound-asset`) replaces the WAV and adds the asset test. Both commits touch production code or a regression test.
- **Components:** `TranscriptRenderer::render_digest`, the multi-event count in `render_notification`, the one-delivery `process_report`, the 1.0 s WAV, and `the_bundled_sound_is_three_short_beeps_in_one_second` exist.
- **Strict mode:** `.fail_on_skipped()` is set; the setup task records the two failing delta scenarios before implementation (0 passed, 2 failed).
- **@e2e tag integrity:** no tag was added or removed. The digest is a variant of the existing check action, covered by the regular scenario and the existing real-interface e2e scenarios.
- **E2E command:** `givn/commands.yaml` binds `./scripts/verify.sh --e2e` with `GIVN_RESULT_SCOPE=e2e`; the e2e steps remain in `pr_activity_notification_e2e.rs` and `inbox_navigation.rs`. The permanent e2e suite passes 17/17.
- **Design conformance:** the implementation matches the design's digest format, cap, one-delivery rule, and asset shape. No deviation.

### Interaction coverage

| Inventory entry (usecase.md) | Design matrix row | `@e2e` scenario | Driver |
|---|---|---|---|
| watch-pull-requests / run one observation pass | yes | Developer runs a single check and sees the current activity | real `passuff check` |
| watch-pull-requests / start continuous watching | yes | Developer starts watching and is notified of a new comment | real `passuff` + wiremock + D-Bus |

No new action was introduced; the digest is an output variant covered by the regular scenario.

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| One delivered notification per pass | 04-solution-strategy | Yes | Delivery | Batch `notification-digest` | `process_report` single call | yes |
| Renderer and delivery responsibilities | 05-building-block-view | Yes | Architecture impact | Batch `notification-digest` | `render_digest`, `DesktopDelivery` | yes |
| Check-cycle delivery step | 06-runtime-view | Yes | Delivery | Batch `notification-digest` | `process_report` | yes |
| Notification/sound crossing concept | 08-crosscutting-concepts | Yes | Delivery | both batches | digest text, sound asset | yes |
| Digest and sound quality scenarios | 10-quality-requirements | Yes | Digest format, Sound asset | both batches | QS-030/QS-031 | yes |
| Digest body cap risk | 11-risks-and-technical-debt | Yes | Digest format | batch `notification-digest` | R-034 | yes |
| Notification digest term | 12-glossary, docs/glossary.md | Yes | Digest format | batch `notification-digest` | glossary row | yes |
| No new ADR | 09-architecture-decisions | No | Decisions | design-review | ADR-0002 unchanged | yes |

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

`docs/glossary.md` holds `Notification digest`; the specs, design, and code use "digest", "notification", "activity", and "pull request" consistently. The structural model is unchanged: the core still produces one `Notification` per pull request; the CLI composes the delivered digest. The assessment is recorded in `arc42.md`.

## Coverage classification

- **Dead code:** none introduced; the previous per-notification loop is replaced, not left unused.
- **Missing test coverage:** none. `render_notification`/`render_digest` have five unit tests (single event, single-PR count, digest counts, plural forms, five-PR cap); the sound test asserts the 1.0 s three-beep asset; the two delta scenarios assert the user-visible delivery.
- **Hard to test:** none. The desktop body and sound calls are recorded by the in-process fake.

Measured with the project command: 6078/7147 lines, 739/2306 branches in the merged report. The archive re-measures the committed candidate.

## E2E coverage

The product is a terminal UI/CLI with a desktop notification surface, not a browser UI; no browser driver applies. The digest scenario is a regular variant of the observation action, and the existing e2e scenarios assert the real subprocess output and D-Bus delivery. Both required executions pass with zero failures or skips (regular 201/201 permanent, e2e 17/17).

## Deferrals

None.

## Use-case and Persona conformance

`watch-pull-requests` R13/R24 now describe one delivered notification per pass; R42/R43 record the digest body cap and the three-beep one-second default. The goal, actors, trigger, preconditions, extensions, and capability placement are unchanged. Persona `multi-account-developer` remains the confirmed lens.

## Visual review

- Interface: terminal and desktop notification text; the TUI layout is unchanged.
- Evidence: the committed transcripts for the permanent e2e scenarios under `givn/changes/passuff-notification-digest/evidence/visual/`; the digest text is asserted by the regular scenario.
- The reviewer read the transcripts; no screen state changed.

VISUAL-REVIEW: PASS

## Overlap dispositions

| Finding pair | Decision |
|---|---|
| `Multiple events on one pull request produce one notification` (modified) / `Activity on two pull requests produces one digest notification` (added) | boundary — one pull request versus several |
| The digest scenario / permanent single-notification e2e scenario | variant — output variant of the same check action |

## Split-or-keep

No delta scenario exceeds the long-scenario threshold.

## README impact

README-IMPACT: updated - Description

The description now states one notification per pass, the digest count, and the three-beep one-second default sound.

## Sign-off

- Fabrication audit clean; scenario commits touch production code.
- Every promised component exists; contracts unchanged.
- Strict-mode proof present.
- Coverage gaps classified and resolved.
- No `@wip` remains.
- Interaction coverage and the real interface verified.
- No finding excused outside the three buckets.

PERSONAS: multi-account-developer
README-IMPACT: updated - Description
VISUAL-REVIEW: PASS
REVIEW: PASS
