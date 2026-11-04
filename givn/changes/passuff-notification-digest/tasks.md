# Tasks: passuff-notification-digest

Release note for every batch commit: `Deliver one digest notification per check pass and play a one-second three-beep chime.`

### Batch: `notification-digest`

- [ ] Setup: confirm the Gherkin runner and strict mode.
  - `GIVN_FEATURES=$PWD/givn/changes/passuff-notification-digest/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh` must fail with the two `@wip` scenarios until implemented; `.fail_on_skipped()` is already set.
  - No new step phrases; the delta reuses `pr_activity_notification.rs` and the existing support fixtures. The change adds no `@e2e` scenario.

- [ ] Renderer: count and digest.
  - [ ] `render_notification`: when a single pull request has more than one event, title and body start with `<n> activities on <key>`; the single-event text is unchanged.
  - [ ] `render_digest`: title and body first line `<n> activities in <m> pull requests`; body lists `<key>: <count> activit(y|ies)` for the first five pull requests and then `+<k> more pull requests`; transcript lists every key and event with the existing indentation; pluralization for 1.
  - [ ] Unit tests: single event unchanged, single-PR count, multi-PR count, body cap, plural forms, transcript detail.
- [ ] `process_report`: render once (`render_digest` when more than one notification, otherwise `render_notification`), call `deliver` exactly once, and append the single delivery's errors. Unit/support test: two notifications produce one delivery record and one sound call.
- [ ] Tooling: `cliff.toml` gains a `Changes` group for non-conventional Givn release-note commits and skips `docs|chore|ci|test|style|refactor`.
- [ ] Docs: use case R13/R24/R42/R43 and main-flow steps; glossary `Notification digest`; arc42 04/05/06/08/10/11/12; README notification/sound text.

- [ ] Scenario: `Multiple events on one pull request produce one notification` (modified)
  - [ ] **Design constraints:** one pull request, three events; one report notification; the title/body names `3 activities on acme/api#42`; the event lines remain; one sound. The count is the number of `ActivityEvent`s, not PRs.
  - [ ] COMMIT: `<pending>` -- Batch: `notification-digest`

- [ ] Scenario: `Activity on two pull requests produces one digest notification`
  - [ ] **Design constraints:** two authored pull requests, one comment each, one check; one delivered notification; title/body `2 activities in 2 pull requests`; body `acme/api#42: 1 activity` and `acme/web#7: 1 activity`; the transcript keeps both event lines; one sound call.
  - [ ] COMMIT: `<pending>` -- Batch: `notification-digest`

### Batch: `sound-asset`

- [ ] Replace `crates/passuff-cli/assets/notification.wav` with a PCM mono 44.1 kHz 16-bit, 1.0 s file containing three 120 ms 880 Hz beeps at 0/340/680 ms with 20 ms fades and about 0.3 peak amplitude.
- [ ] Unit test in `crates/passuff-cli/src/sound.rs`: parse the bundled WAV, assert 1.0 s and three non-silent regions separated by silence; the player selection and custom `sound_file` path stay unchanged.
  - [ ] COMMIT: `<pending>` -- Batch: `sound-asset`

### Closeout

- [ ] Run the delta scope, the permanent suite, and the e2e suite; record the totals.
- [ ] Confirm no `@wip` remains and the release note subject is used for both batch commits.
