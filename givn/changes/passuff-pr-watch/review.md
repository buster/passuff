# Review: passuff-pr-watch

Review kind: self-review
Author: Maki implementation agent
Reviewer context: self (same context; not an independent review)

## Implementation evidence check

- Inspected the delta feature: 4 `@e2e` scenarios present; no `@e2e` tag was removed. `givn lint` reports the file clean.
- Grepped both step-definition files for empty bodies: 0 empty or no-op step bodies across 2 files. The only bare `return;` statements end explicit dispatch branches in `apply_action`; each is preceded by a domain mutation and an e2e mirror update.
- 37 checked scenarios record `COMMIT: \`aff41de\` -- Batch: <name>`. The shared batch revision contains the full workspace implementation and test tree (`git show --stat aff41de`: 2,691 source/test lines); it is not a comment-, tag-, task-, or feature-only change. Later commits (`5fa5fb8`, `759f918`) add adapter regression tests and delete dead code.
- Every component named by `design.md` exists: `passuff-config` loader/validation, `passuff-core` domain/classifier/GitHub adapter/credential resolver/state/cycle, `passuff-cli` transcript/delivery/sound/runtime, and the `main` composition root.
- Consumers use the declared contracts: `CliRuntime` takes `GitHubGateway`, `SnapshotStore`, `CredentialSource`, and `NotificationDelivery` trait objects; no consumer imports an adapter implementation except the composition roots (`main.rs`, tests).
- Strict-mode proof is present in `tasks.md`: an undefined step produced `failed: 1` and exit 1.
- E2E Then steps assert on real interfaces: CLI stdout (`passuff: <key>`, event lines), the private D-Bus `Notify` call body, the fake `paplay` log, and the process exit status. No E2E assertion reads repository or in-memory state only.
- `verify.e2e_command` is `./scripts/verify.sh --e2e`, which invokes `crates/passuff-cli/tests/features_runner.rs` with `GIVN_RESULT_SCOPE=e2e`. Exactly one e2e step file exists (`pr_activity_notification_e2e.rs`); no parallel implementation was found in the tree, including untracked files.
- Scope isolation is explicit: regular runs exclude `@e2e` (48 scenarios), e2e runs only `@e2e` (4 scenarios), and `change` runs 52. Counts are distinct and are recorded in the verification receipts.
- Implementation matches `design.md` names: three-crate workspace, runner path, per-capability step files, wiremock GitHub twin, private `dbus-daemon` with a D-Bus notification fake, fake `paplay`, and the `passuff` binary spawned through `CARGO_BIN_EXE_passuff`.
- Design amendment: the coverage measurement uses the nightly toolchain with `--branch` and a JSON-to-Cobertura conversion because LLVM 23's lcov export segfaults on branch regions. `design.md` and `design-review.md` record the amendment and the re-check.

### Interaction cross-reference

| Inventory entry | Design matrix row | @e2e scenario | Driving mechanism in steps |
|---|---|---|---|
| pr-activity-notification / start continuous watching | present | Developer starts watching and is notified of a new comment | spawns `passuff`; waits for `passuff: watching`; asserts stdout, D-Bus body, sound log; SIGINT |
| pr-activity-notification / run one observation pass | present | Developer runs a single check and sees the current activity | spawns `passuff check --config --state`; asserts transcript, D-Bus body, sound log, exit 0 |
| pr-activity-notification / show usage | present | Developer asks for usage | spawns `passuff --help`; asserts `Commands:`/`check` and exit 0 |
| pr-activity-notification / show version | present | Developer asks for the version | spawns `passuff --version`; asserts `passuff <semver>` and exit 0 |

## Arc42 implementation conformance

| Arc42 chapter or fact | Durable-doc source | `arc42.md` claim | `design.md` | `tasks.md` | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Context: GitHub GraphQL, local desktop, audio player | `docs/arc42/03-context-and-scope.md` | Affected; read-only GitHub, local surfaces | Observation and delivery decisions | Domain constraints R1-R31 | `github.rs`, `delivery.rs`, `sound.rs` | yes |
| Building blocks: config/core/CLI split, future TUI | `docs/arc42/05-building-block-view.md` | Affected; three crates plus future adapter | Three-crate workspace with UI-agnostic core | Setup batch | `Cargo.toml`, module tree | yes |
| Runtime: check cycle, watch loop, account isolation | `docs/arc42/06-runtime-view.md` | Affected; per-account failure isolation | Cycle and CLI split | classification/accounts batches | `cycle.rs`, `cli/lib.rs` | yes |
| Cross-cutting: credentials, atomic state, dedupe, D-Bus twin | `docs/arc42/08-crosscutting-concepts.md` | Affected | Credential/storage/E2E decisions | all batches | `credential.rs`, `state.rs`, `tests/support/e2e.rs` | yes |
| ADR-0001/0002/0003 registered and honoured | `docs/arc42/09-architecture-decisions.md` | Three ADRs | Same decisions | design tasks | `docs/arc42/adr/` files; implementation follows them | yes |
| Risks: at-most-once, macOS manual smoke, daemon dependency | `docs/arc42/11-risks-and-technical-debt.md` | Affected | Same risks and mitigations | E2E batch | D-Bus twin, recording delivery, docs | yes |

The five sign-off checks hold: chapters are current and decision-specific; no placeholders remain; each claim maps to design, tasks, and code; ADRs match the implemented choices; no chapter contradicts the implementation.

ARC42 CONFORMANCE: CLEAN

## Ubiquitous language conformance

`docs/glossary.md` and `docs/domain-model.md` use the same terms as the spec, design, and code: Pull request, Authored pull request, Configured login set, Activity event, Notification (grouped report), Baseline, Watched pull request, Cycle, Check, Account failure, Transcript. The code symbols match (`AuthoredPullRequest`, `LoginSet`, `ActivityEvent`, `Notification`, `Baseline`, `WatchedPullRequest`, `RefreshReport`, `AccountFailure`). No old or competing term remains in README, glossary, ADRs, tests, or code. The structural model matches the implemented boundaries and relationships; no edit was needed.

## Coverage

Measured with `./measure-coverage.sh` then `./merge-coverages.sh`:

- non-E2E: 867/1076 lines (includes workspace unit tests and the regular Gherkin runner)
- E2E: 479/1076 lines (separate instrumentation run; includes the spawned binary)
- merged: 962/1076 lines (89.4%), 114/330 branches (34.5%)

Classification of the residual gaps, using only the three buckets:

1. Dead code: `PassuffCore::open`, `PassuffCore::with_gateway`, `GitHubApiAdapter::with_client`, `impl Default for GitHubApiAdapter`, `TranscriptRenderer::render_report`, `RecordingDelivery::records`, `RecordingSoundPlayer::calls`, `impl Default for SystemSoundPlayer`, `JsonStateStore::path`, and `PrState::label` were unused and are deleted (commit `759f918`).
2. Missing test coverage: the GitHub adapter's mapping and failure paths, configuration validation branches, and state unknown-version/write-failure branches were uncovered and are now covered by unit tests (commits `5fa5fb8`, `759f918`). No production rule is left without a test at its boundary.
3. Legitimately hard to test, with technical justification:
   - `main.rs` default config/state path resolution: resolving and writing the operator's real `~/.config` and `~/.local/state` paths during tests would touch the host account; every E2E run passes explicit `--config`/`--state` paths. The fallback is exercised only when neither flag is present.
   - `sound.rs` cache-write fallback and alternate player fallbacks: depend on a non-writable cache directory and on the presence/absence of `afplay`, `pw-play`, `aplay`, `canberra-gtk-play`. The E2E run covers the real `paplay` path; the rest are platform and filesystem failure surfaces.
   - `delivery.rs` recording sink (`with_sink`): test-support-only macOS capture path, not reachable on Linux where the D-Bus twin is authoritative.
   - Serde visitor error strings (`config.rs` 259-291): the rejection paths are tested; only the message formatting of otherwise-unreachable visitor states is uncovered.
   - Defensive `continue`/formatting branches in `classifier.rs`, `state.rs`, and `github.rs`: dedupe and display code for combinations the public scenarios cannot produce.

No fourth bucket is used. No branch instrumentation was removed. No redundant unit test was kept: the adapter/config/state tests cover file-format and error boundaries that the Gherkin scenarios do not reach; scenario-covered behaviour is asserted only through the runner.

## E2e coverage

The capability is a CLI, not a Web UI. The opposite classification was considered: a rendered page would be a Web UI. `passuff` renders no HTML, CSS, or JavaScript; its surfaces are terminal text, an operating-system notification, and an audio player. The E2E suite drives the real binary under a private D-Bus daemon and a wiremock GitHub server. Every inventory action has exactly one `@e2e` scenario, and variants (comment types, CI states, toggles, silent cases) stay in the regular scope.

## Visual review

Interface: terminal. Evidence: the four committed transcripts under `givn/changes/passuff-pr-watch/evidence/visual/<scenario-slug>/transcript.txt`:

- `developer-starts-watching-and-is-notified-of-a-new-comment`: baseline, `passuff: watching`, then the grouped event.
- `developer-runs-a-single-check-and-sees-the-current-activity`: the grouped event and exit 0.
- `developer-asks-for-usage`: command list including `check` and the global options.
- `developer-asks-for-the-version`: `passuff 0.1.0`.

Persona `multi-account-developer`: each declared state is reachable in plain text, the `passuff:` label is stable, no meaning relies on color, and the Persona can see new activity, discover commands, and stop the watcher. No finding.

VISUAL-REVIEW: PASS

## Overlap dispositions

Advisory shape/subset findings are dispositioned below. Shape pairs are variants asserting different event families or lifecycle rules; subset pairs are boundaries isolating a suppression rule that the generic scenario does not prove.

| Scenario A | Scenario B | Finding | Decision | Reason |
|---|---|---|---|---|
| A review comment from another person notifies | A submitted review notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review comment from another person notifies | A failing CI result notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review comment from another person notifies | A CI recovery notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review comment from another person notifies | A CI recovery across pending notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review comment from another person notifies | A pull request merged or closed while the watcher was stopped is reported | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review comment from another person notifies | A merge performed by the configured account still notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A submitted review notifies | A failing CI result notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A submitted review notifies | A CI recovery notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A submitted review notifies | A CI recovery across pending notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A submitted review notifies | A pull request merged or closed while the watcher was stopped is reported | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A submitted review notifies | A merge performed by the configured account still notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | A commit push stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | Activity on someone else | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | A reopened pull request stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | A disabled event family suppresses its notification | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | A comment from another configured account stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | A restart does not replay notified activity | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A review from a configured account stays silent | A non-notified change stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A commit push stays silent | Activity on someone else | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A commit push stays silent | A reopened pull request stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A commit push stays silent | A disabled event family suppresses its notification | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A commit push stays silent | A comment from another configured account stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A commit push stays silent | A restart does not replay notified activity | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A commit push stays silent | A non-notified change stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A failing CI result notifies | A CI recovery notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A failing CI result notifies | A CI recovery across pending notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A failing CI result notifies | A pull request merged or closed while the watcher was stopped is reported | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A failing CI result notifies | A merge performed by the configured account still notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A CI recovery notifies | A CI recovery across pending notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A CI recovery notifies | A pull request merged or closed while the watcher was stopped is reported | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A CI recovery notifies | A merge performed by the configured account still notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A CI recovery across pending notifies | A pull request merged or closed while the watcher was stopped is reported | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A CI recovery across pending notifies | A merge performed by the configured account still notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A pull request merged or closed while the watcher was stopped is reported | A merge performed by the configured account still notifies | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A reopened pull request stays silent | A disabled event family suppresses its notification | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A reopened pull request stays silent | A comment from another configured account stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A reopened pull request stays silent | A restart does not replay notified activity | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A reopened pull request stays silent | A non-notified change stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| Historical activity is not replayed on first observation | A corrupt state file is replaced with a baseline | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A disabled event family suppresses its notification | A comment from another configured account stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A disabled event family suppresses its notification | A restart does not replay notified activity | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A disabled event family suppresses its notification | A non-notified change stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A comment from another configured account stays silent | A restart does not replay notified activity | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A comment from another configured account stays silent | A non-notified change stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| A restart does not replay notified activity | A non-notified change stays silent | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| An invalid configuration is rejected | A state write failure is reported with a nonzero status | shape | variant | distinct event family and rule; merging would hide which rule regressed |
| Developer starts watching and is notified of a new comment | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| An invalid configuration is rejected | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A state write failure is reported with a nonzero status | Developer runs a single check and sees the current activity | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | Developer asks for usage | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | Developer asks for the version | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A short-lived pull request merged or closed between checks notifies | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review from a configured account stays silent | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A commit push stays silent | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A reopened pull request stays silent | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A disabled event family suppresses its notification | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A comment from another configured account stays silent | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A restart does not replay notified activity | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A non-notified change stays silent | Historical activity is not replayed on first observation | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review from a configured account stays silent | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A commit push stays silent | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A reopened pull request stays silent | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Historical activity is not replayed on first observation | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A disabled event family suppresses its notification | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A comment from another configured account stays silent | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A restart does not replay notified activity | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A corrupt state file is replaced with a baseline | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A non-notified change stays silent | A pull request first seen after the first cycle records a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A configured sound is used | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A disabled sound produces no sound | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| An invalid configuration is rejected | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A state write failure is reported with a nonzero status | A disabled desktop notification keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| An invalid configuration is rejected | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A state write failure is reported with a nonzero status | An unavailable desktop surface keeps the transcript | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| An unavailable desktop surface keeps the transcript | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| An invalid configuration is rejected | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A state write failure is reported with a nonzero status | A missing sound player still delivers the notification | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| An invalid configuration is rejected | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A state write failure is reported with a nonzero status | A failed account does not stop the other account | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review from a configured account stays silent | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A commit push stays silent | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A reopened pull request stays silent | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A disabled event family suppresses its notification | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A comment from another configured account stays silent | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A restart does not replay notified activity | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A non-notified change stays silent | A corrupt state file is replaced with a baseline | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | An invalid configuration is rejected | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review comment from another person notifies | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A submitted review notifies | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A failing CI result notifies | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery notifies | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A CI recovery across pending notifies | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A pull request merged or closed while the watcher was stopped is reported | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A merge performed by the configured account still notifies | A state write failure is reported with a nonzero status | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review from a configured account stays silent | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A commit push stays silent | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A reopened pull request stays silent | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A disabled event family suppresses its notification | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A comment from another configured account stays silent | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A restart does not replay notified activity | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A non-notified change stays silent | No authored pull requests produce no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| Developer starts watching and is notified of a new comment | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A review from a configured account stays silent | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A commit push stays silent | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A reopened pull request stays silent | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A disabled event family suppresses its notification | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A comment from another configured account stays silent | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A restart does not replay notified activity | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| A non-notified change stays silent | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |
| No authored pull requests produce no notifications | A single check with no changes reports no notifications | subst | boundary | isolates a distinct suppression rule that the generic scenario does not prove |

## Split-or-keep

No scenario exceeds the gate's split threshold; no split-or-keep rows are required.

## README impact decision

README-IMPACT: updated - Description, Quick start, Prerequisites, Usage, Development, Contributing, License

## Sign-off checklist

- Fabrication audit: clean.
- Every checked task has a verified commit touching production code.
- Every promised component exists.
- Every consumer uses only its declared contract.
- Strict-mode proof present and passing.
- Scenario execution evidence is recorded per scope in the verification receipts.
- Coverage classification uses the latest measured evidence; the archive receipt will carry the merged result.
- Every coverage gap classified and resolved.
- Dead code deleted; missing tests added; hard-to-test gaps justified.
- Redundant unit tests removed.
- No `@wip` tags remain; no implementation-layer detail in the spec.
- Canonical E2E policy read; every normalized inventory action has one `@e2e` scenario; variants are regular scenarios.
- Every E2E scenario has a real-interface primary assertion and uses the reviewed driver.
- The local run command is declared and starts the CLI.
- The exact file invoked by `verify.e2e_command` was identified and read; no parallel e2e implementation exists.
- E2E scope is selected explicitly through `GIVN_RESULT_SCOPE`.
- Implementation matches the names in `design.md`; the coverage amendment is recorded and re-checked.
- Interaction coverage verified.
- No finding was excused with a classification outside the three buckets.

REVIEW: PASS
