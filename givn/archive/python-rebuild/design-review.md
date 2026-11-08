Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-1

# Design Review: python-rebuild

Reviewed artifacts: `proposal.md`, `design.md`, `arc42.md`, the six
`givn/specs/*/usecase.md` files, the seven permanent `.feature` files,
`docs/arc42/` (twelve chapters, `README.md`, `adr/`), `docs/glossary.md`,
`docs/domain-model.md`, `docs/reference.md`, `givn/commands.yaml`,
`givn/config.yaml`, and `README.md`. No git history was used.

Independent verdict: the rebuild is the right change, and the permanent corpus
is the right gate. Two blockers stop sign-off: the corpus is not a complete gate
for the behaviors the design itself commits to (Q1), and the activity-detail
body is contradicted across the use case, feature, glossary, runtime view, and
design data model (Q2). Four further operator decisions remain (Q3, Q4, Q10,
Q11). The remaining findings are resolved by inspecting the tree and require
design/durable-document edits before the artifacts return for sign-off.

## Ranked questions

### Q1 — The `specs` artifact is skipped, but the permanent corpus is not a complete gate

- Status: OPERATOR DECISION — blocker
- Blocks: the honesty of the acceptance claim, the final review's external-boundary check, and the proposal's no-delta premise.

**Context.** `proposal.md:55-56` makes all 185 permanent scenarios the acceptance
gate (167 non-E2E, 18 E2E). `proposal.md:78-80` and `design.md:5-7` say the
`specs` artifact is skipped because the rebuild has no behavior delta. The count
is correct: `givn spec tree` reports 185 scenarios and 18 `@e2e`; the feature
counts are 18 + 11 + 37 + 35 + 7 + 8 + 69. The corpus is not, however, complete
against the design's own boundary table (`design.md:458-469`) or the permanent
reference:

- unknown configuration key (`docs/reference.md:230`; no scenario),
- invalid TOML, missing configuration on `check`, plain HTTP outside loopback,
  duplicate scope entries (`docs/reference.md:211-212, 216-230`; no explicit
  scenario; `watch-scope.feature:128` proves malformed `owner/repo`, not
  duplicate or loopback),
- unknown command or option, missing option value, invalid `--focus-set`,
  invalid `--keys`, `--scope-deselect` in all mode (`design.md:465`; the only
  "usage" scenario is `pr-activity-notification.feature:46` invoking `--help`,
  which is a success path),
- malformed or partial GitHub response, timeout, non-2xx, GraphQL `errors`
  payload, replayed duplicate node, stack-field rejection with one retry
  (`design.md:462`; `stacked-inbox.feature` has no rejected-stack scenario),
- unknown state version, quarantine failure, duplicate activity ids, 30-day
  retirement boundary, `poll_interval_secs` below 1 (`design.md:463`; only
  corrupt JSON, version 1, and state write failure are covered),
- D-Bus `Notify` error, empty summary, long body; sound player non-zero exit;
  `gh` binary missing/non-zero (`design.md:466-468`; only service-unavailable,
  failed browser launch, and wrong-login cases are covered),
- the 80-character activity excerpt boundary (`usecase.md:54` T7; no list-row
  scenario; the long-comment detail scenario at
  `inbox-navigation.feature:245-261` proves the opposite: the detail body is
  longer than 80 characters).

The design says the boundary evidence is "Usage and exit-2 scenarios" and
"scenarios for account failure, partial data, stack fallback"
(`design.md:462-465`). Those scenarios do not exist. The reference validation
list is documentation, not an executable assertion. An implementation that
silently ignores an unknown config key, accepts plain HTTP, or returns a wrong
exit code for an unknown option passes all 185 scenarios. That is the
"zero delta scenarios report GREEN" trap in a different form.

**Question.** Should the change add a small delta that closes the missing
boundary scenarios, or keep the `specs` skip and weaken the acceptance claim?

**Options.**

1. **Add a minimal delta (`@givn.added @wip`) for the missing boundary
   scenarios and amend the proposal's "no scenario changes" clause.**
   Meaning: the behavior does not change; the executable gate becomes complete.
   Gain: the rebuild's Ise claim is testable; the final review's boundary check
   has evidence. Cost: a spec delta and archive merge; the proposal's Out of
   Scope at `proposal.md:83-87` must be amended by the operator; the reviewer
   must verify the new scenarios. Example: a new scenario "An unknown
   configuration key is rejected" with `Given a configuration with an unknown
   key`, `When passuff runs one check`, `Then the transcript reports
   "configuration error"`, `And the exit status is 2` — the existing "An invalid
   configuration is rejected" scenario at `pr-activity-notification.feature:457`
   does not distinguish this case.
2. **Keep no delta; narrow the acceptance claim.** Meaning: the change accepts
   that the 185 scenarios pass and some reference behaviors are un-gated.
   Gain: no spec change. Cost: the rebuild cannot prove those behaviors; the
   final review's external-boundary check will still record them; the
   "specification is complete" claim is false. Example: a config typo
   `poll_interval_sec` is silently ignored and the watcher runs at the default
   interval, yet the whole suite is green.
3. **Remove the un-gated behaviors from the reference and design.** Meaning:
   accept unknown keys, allow plain HTTP, drop the stack-retry rule, and so on.
   Gain: reference and corpus agree. Cost: a user-visible behavior change,
   contradicting `docs/reference.md:211-230`, `docs/arc42/04:70`,
   `docs/arc42/05:137`, and `docs/arc42/08:5`; this is no longer a no-delta
   rebuild. Example: an operator with an unknown key no longer gets exit 2.

**Recommendation.** Option 1. The rebuild's premise is that the permanent
specification is sufficient; the missing scenarios are already specified
behaviors, not new ones. Adding them is a spec completion, not a behavior
change. The operator must amend the proposal's no-scenario clause because the
proposal currently forbids this. If the operator refuses, record Option 2
explicitly in `design.md` and `arc42.md` and accept that the Ise test is not
fully proven. Option 3 requires a separate behavior-change proposal.

**Resolution (operator, 2026-11-08):** Option 1. Add a minimal
`@givn.added @wip` delta for the missing boundary scenarios, unskip the
`specs` artifact, and amend the proposal's "no scenario changes" clause. The
delta covers the reference validation list, the GitHub response-failure and
stack-retry cases, state unknown-version/quarantine/duplicate-id/retention
boundaries, CLI usage errors, delivery-surface failures, and the 80-character
list-row excerpt. The non-E2E scenarios remain non-E2E; no `@e2e` tag is
removed.

**Rejected:** Option 2 (narrow the acceptance claim) — it leaves the
specification's completeness claim false and the final review's boundary check
without evidence. Option 3 (remove the un-gated behaviors) — it changes
user-visible behavior and turns the rebuild into a behavior change.

---

### Q2 — The activity-detail body contradicts the use case, the feature, the glossary, the runtime view, and the design data model

- Status: OPERATOR DECISION — blocker
- Blocks: the activity-detail implementation, the state schema, and the T14/T7 rules.

**Context.** The permanent use case requires the activity detail to show the
selected comment or review body in full: `review-activity-inbox/usecase.md:61`
(T14) says "the selected comment or review body in full, rendered as Markdown".
The feature proves it: `inbox-navigation.feature:245-261` ("The activity detail
shows and scrolls a long comment") asserts `comment line 1` and, after `End`,
`comment line 40`. But:

- `docs/glossary.md:23` defines Activity detail as never containing a comment
  or review body.
- `docs/arc42/06-runtime-view.md:316-319` says "No comment or review body is
  shown."
- `design.md:197` persists `activity[]` with only a "bounded summary", not the
  body.
- `docs/arc42/08-crosscutting-concepts.md:23-25` describes the activity record
  without a body; `docs/arc42/05-building-block-view.md:165` likewise.
- `review-activity-inbox/usecase.md:54` (T7) says the screen contains "no more
  than the recorded 80-character event excerpt", which conflicts with a long
  body in the detail popup.
- The E2E scenario `activity-inbox.feature:326-339` asserts only metadata, not
  the body; the body is covered only by a regular scenario.

The design's `## Use-case traceability` row for `review-activity-inbox` claims
"Markdown detail" (`design.md:344`), but the design's persisted shape cannot
supply it. An implementer following `design.md:197` will omit the body and pass
the E2E scenario; the regular long-comment scenario will fail only if the body
is not persisted. This is a specification contradiction, not an implementation
detail.

**Question.** Should the activity record persist and render the body (aligning
to T14 and the feature), or should the body be removed from the detail
(aligning to the glossary and chapter 6)?

**Options.**

1. **Persist the body (bounded) and render it; scope the 80-character excerpt
   to the list row.** Meaning: the activity record keeps the event summary and
   the body up to a record-size bound (for example 4 KiB); the list shows the
   excerpt; the detail shows the stored body as Markdown. Gain: satisfies T14
   and `inbox-navigation.feature:245-261`; the detail works offline from
   persisted data; the E2E can assert the body. Cost: a state-schema addition
   and a size bound; the 500-record cap plus body storage grows the state file;
   `docs/glossary.md`, `docs/arc42/05`, `docs/arc42/06`, `docs/arc42/08`, T7,
   and `design.md:197` must be edited. Example: a 40-line comment is stored and
   scrolled in the detail; the list row still shows `Comment by @alice` and the
   80-character excerpt.
2. **Remove the body from the detail.** Meaning: the detail shows only the
   summary and metadata; T14 and the long-comment feature are changed. Gain: no
   body storage; smaller state. Cost: a behavior change; the current permanent
   feature explicitly requires line 40; users lose the detail body; the use-case
   rule and scenario must be replaced. Example: the detail shows `Comment by
   @alice` and the metadata, but `comment line 40` is never visible.
3. **Fetch the body live from GitHub when the detail opens.** Meaning: the
   activity record stores only a reference; the detail calls GitHub. Gain: no
   body storage. Cost: the inbox/status contract says the TUI reads persisted
   state and does not contact GitHub (`usecase.md:44` S5 is status, but the TUI
   detail is advertised as offline); a failed fetch leaves an empty detail; the
   E2E needs a live twin call. Example: opening a 30-day-old activity fails when
   GitHub no longer returns the comment.

**Recommendation.** Option 1. Two artifacts (T14 and the feature) require the
body; the executable feature is the stronger contract. Record the body-size
bound and update T7 to apply the 80-character excerpt to the list row, not the
detail. The E2E activity-detail scenario should assert a non-empty body and the
Markdown rendering, not only the metadata.

**Resolution (operator, 2026-11-08):** Option 1. Persist the selected comment
or review body with a 4 KiB per-record bound, render it as Markdown in the
detail, and scope T7's 80-character excerpt to the list row. Correct the
glossary, arc42 chapters 5/6/8, T7, and the design data model. Add the body
assertion to the `@e2e` activity-detail scenario.

**Rejected:** Option 2 (remove the body) — it changes a behavior the executable
feature requires. Option 3 (fetch live) — it breaks the offline read model and
fails for retained activity.

---

### Q3 — Unknown configuration keys: specified in the reference, absent from the corpus, and plausibly Rust-era residue

- Status: OPERATOR DECISION
- Blocks: the configuration contract and the law-of-Ise claim.

**Context.** `docs/reference.md:230` includes "an unknown configuration key" in
the exhaustive rejection list; `docs/arc42/04:70`, `docs/arc42/05:137`, and
`docs/arc42/08:5` repeat it; `design.md:180-186` makes it normative and says the
reference "is corrected to name the unknown-key rejection". No feature scenario
covers it. The behavior looks like the Rust `deny_unknown_fields` decoding
policy carried into the reference rather than a use-case rule: the
`watch-pull-requests` rules R32-R38 (`usecase.md:79-85`) do not mention unknown
keys. The design simultaneously says "No key is added or removed"
(`design.md:178`) and "unknown keys are rejected" (`design.md:180`), so the
behavior is either already in the permanent contract (reference) or is an
unrecorded delta. It cannot be both.

**Question.** Does the rebuild keep unknown-key rejection as a specified,
tested behavior, or is it Rust-era residue to remove?

**Options.**

1. **Keep rejection and add a scenario** (Q1 Option 1). Meaning: the reference
   and the use-case rules own the behavior; the corpus proves it. Gain: the
   durable docs and the executable gate agree; a typo cannot silently change
   behavior. Cost: a delta scenario; exit 2 for unknown keys. Example: a config
   with `poll_interval_sec` is rejected with "configuration error" and exit 2.
2. **Remove rejection.** Meaning: unknown keys are ignored. Gain: no un-gated
   behavior; less Rust residue; a future key can be added without breaking old
   binaries. Cost: a behavior change; four durable documents must be corrected;
   a typo silently falls back to the default. Example: `poll_interval_sec = 5`
   is ignored and the watcher runs at 60 seconds.
3. **Keep rejection as reference-only and record it as un-gated.** Gain: no
   spec delta. Cost: the rebuild cannot prove it; the final review's
   boundary check records it. Example: an implementation that ignores unknown
   keys passes all 185 scenarios.

**Recommendation.** Option 1. The durable documents already specify rejection;
removing it is the bigger change. If the operator decides the rule is Rust-era
residue, choose Option 2 and update the reference and three arc42 chapters in
the same change; do not leave it un-gated.

**Resolution (operator, 2026-11-08):** Option 1, through Q1's accepted Option 1.
Keep unknown-key rejection as a specified behavior and add its scenario to the
boundary delta; the reference and arc42 keep the rule.

**Rejected:** Option 2 (remove rejection) — it would reject the durable
reference and silently accept a typo. Option 3 (un-gated reference-only) — Q1
Option 1 requires a complete executable gate.

---

### Q4 — E2E step definitions are not separate from the regular step definitions

- Status: OPERATOR DECISION
- Blocks: the canonical E2E policy and the final review's step-definition check.

**Context.** `givn instructions specs` says "`@e2e` scenarios use separate step
definitions ... separate from the unit/integration step definitions."
`givn instructions design` repeats "E2E step definition location: separate from
unit/integration steps, one file per capability." `design.md:414-419` says the
opposite: "the same permanent per-capability files in `givn/steps/`. There are
no unit-level step definitions: every Gherkin scenario drives the real CLI or
TUI. Behave binds a phrase once, so a second E2E step tree would create
ambiguous bindings." The design acknowledges the deviation but does not record
an operator decision. The justification is a tooling constraint, not a product
constraint: behave can load nested step modules (`use_nested_step_modules`) or
a staged step directory, and the regular scenarios already use the same real
CLI/TUI, so the separation is about organization, not a second mock layer.

**Question.** Should the change conform to the separate-E2E-step rule, or record
the single-tree interpretation as an operator-approved deviation?

**Options.**

1. **Separate the E2E step modules.** Meaning: `givn/steps/` keeps the regular
   bindings; an `givn/steps/e2e/` tree (or a `--stage`/nested module layout)
   holds the E2E bindings; no phrase is bound twice. Gain: canonical policy;
   E2E and regular failure modes are distinguishable. Cost: a nested/staged
   behave layout; the design must name the exact directory and the phrase
   ownership; more files. Example: `givn/steps/e2e/pr_activity_notification_e2e.py`
   owns `passuff starts watching`, while the regular file owns
   `passuff runs one check`.
2. **Keep the single tree and record the operator's decision.** Meaning: there
   is no unit-level step set; every scenario drives the real interface; the
   separate-set rule is vacuous. Gain: no duplicate bindings; the
   real-interface intent is met. Cost: a recorded deviation from two served
   instructions; a future unit-level step must split the tree. Example:
   `givn/steps/pr_activity_notification_steps.py` binds both the regular and
   `@e2e` phrases; the `@e2e` tag is only a scope marker.

**Recommendation.** Option 1 unless the operator explicitly records Option 2.
"Behave binds a phrase once" is true only when both trees are loaded; it is not
a reason to override the canonical policy. The design's stronger claim ("every
scenario drives the real CLI or TUI") does satisfy the real-interface intent,
but it does not satisfy the separate-file rule.

**Resolution (operator, 2026-11-08):** Option 1. Use behave's `--stage e2e`
layout: `givn/steps/` owns the regular bindings, `givn/e2e_steps/` owns the
E2E bindings, and `givn/environment.py` / `givn/e2e_environment.py` are the
hooks. Shared phrases are imported from one common module so no phrase is bound
twice. Update the design's Step Definitions, E2E infrastructure, runner, and
matrix drivers.

**Rejected:** Option 2 (single tree with a recorded deviation) — the operator
chose conformance over the documented deviation.

---

### Q5 — The Visual Design Contract names T25 but no durable design system file

- Status: EVIDENCE-RESOLVED
- Finding: `design.md:385-408` has `Interface: terminal` and viewports, but its
  "Design system" line points at T25 of the `review-activity-inbox` use case and
  says "No separate `docs/design/design-system.md` exists and none is
  introduced". `givn instructions design` requires the durable design system in
  `docs/design/` and requires the Visual Design Contract's "Design system" line
  to name it. `docs/design/` does not exist.
- Resolution: create `docs/design/design-system.md` as a thin durable index:
  link T25 (`givn/specs/review-activity-inbox/usecase.md:72`) as the normative
  palette source, record the terminal tokens and component primitives without
  copying the palette, and point at the `--snapshot` capture mechanism. The
  T25 authority is honest; the missing durable file is not. The design's
  "a second copy would drift" concern is satisfied by a link-and-index file,
  not a duplicate.

### Q6 — E2E driver mechanisms contain three concrete mismatches

- Status: EVIDENCE-RESOLVED
- Findings:
  - `design.md:437` drives the watch-scope toggle with `--scope-select`, which
    is a pre-run CLI flag, not the picker action the scenario names
    (`watch-scope.feature:4-21`, Trigger at `usecase.md:17`). The picker action
    is `--picker --snapshot --keys "space"` on the repository row.
  - `design.md:438` drives the mode switch with `--keys "m"`, but
    `docs/reference.md:64-66` does not list `m` in the accepted `--keys` set.
    The same list omits `s`, `S`, `f`, `F`, `u`, `c`, `C`, `p`, `h`, `H`, and
    `F1`, which the regular sort/filter/help/settings scenarios need. The
    reference's TUI keymap at `docs/reference.md:109` includes `m`, so the
    `--keys` list is the inconsistent artifact.
  - `design.md:447` uses `--observe-for 4`, but `docs/reference.md:53` defines
    the value as milliseconds and the scenario
    (`inbox-navigation.feature:344-351`) says "4 seconds". The driver must use
    `--observe-for 4000` and set `poll_interval_secs = 1`.
  - `design.md:439` says "`--keys "space"` over the focus rows" without the
    movement keys or the starting focus; the key sequence is not reproducible.
  - `design.md:443` drives "Developer opens the inbox TUI" with
    `--observe --snapshot`; the pty smoke is described separately and is not
    tied to the scenario's assertion. Either make the pty smoke the action's
    E2E driver or rename the inventory action to "print the inbox TUI frame".
  - `design.md:450` starts `passuff` but never says how the continuous watcher
    is bounded, stopped, and asserted; the scenario
    (`pr-activity-notification.feature:4-12`) needs a concrete start/wait/SIGINT
    sequence and a timeout.
- Resolution: correct the matrix and the reference. The fake GitHub twin,
  private D-Bus twin, fake `gh`/sound/browser, and `--snapshot`/`--keys`
  drivers are real-interface drivers when used as above; the three mismatches
  are not downgrades of the twin model, they are incorrect driver details.
  `RecordingDelivery` is a test double (Q10).

### Q7 — The persisted shape is incomplete for the TUI read model

- Status: EVIDENCE-RESOLVED
- Findings: `design.md:188-198` lists the state shape. It omits:
  - the activity body (Q2);
  - the pull request title, description, creator, assignees, and involvement
    relations needed by T23/T24 (`review-activity-inbox/usecase.md:70-71`) and
    asserted by `inbox-navigation.feature:229-243`; the design's
    "last observation" (`design.md:196`) is not defined as containing them;
  - the 500-record cap and the oldest-drop rule (`usecase.md:42` R5);
  - the 80-character list excerpt scope (T7).
  `design.md:211-213` lists `ui.json` fields but omits the sort direction,
  which T18/T22 require to persist (`usecase.md:65,69`). `design.md:182` says a
  `setup` write preserves comments and unrelated known configuration; the
  permanent setup contract (`docs/reference.md:232-252`,
  `set-up-passuff/usecase.md:44-52`) does not describe that behavior and setup
  creates or replaces the document.
- Resolution: expand the persisted shape and the UI preferences; remove "or
  setup" from `design.md:182` unless the reference is amended. These are
  design-level additions, not new behavior, except the setup-preservation
  clause, which is invented.

### Q8 — Contracts and components fail three checks

- Status: EVIDENCE-RESOLVED
- Findings:
  - `design.md:172` records `TerminalInbox` as a contract whose consumer is
    "Operator". That is the product's external terminal interface, not a
    callable contract between components. It also serves three use cases with
    one surface. Remove it from the contract table and record it as the external
    interface, or replace it with per-use-case internal contracts.
  - `design.md:161` gives `InboxReadModel` to both `inspect-watch-status` and
    `review-activity-inbox` while the contract includes mark-read. The status
    consumer does not use mark-read. Split a read-only status view from the
    inbox read/mark contract (ISP). The `review-pull-request-stacks` use case
    has no internal contract at all: its only row is the external
    `TerminalInbox`, while the stack grouping lives in `InboxService`/`TuiApp`
    (`design.md:346`, `docs/arc42/06:238`). Add the stacks consumer to the read
    model or add a stack-view contract.
  - `design.md:164,166` give `ConfigStore` to "all six" and `SnapshotStore` to
    read-only and read/write consumers. The readers depend on write methods they
    do not use (ISP). Split reader/writer contracts or record the consumer roles
    and the accepted narrow surface.
  - `design.md:167` makes `SetupSession` consume `CredentialSource`; the setup
    contract explicitly does not verify credentials
    (`set-up-passuff/usecase.md:97`) and validates token-file permissions as
    configuration. Remove the `SETUP --> CRED` edge (`design.md:145`) unless a
    setup scenario needs credential resolution.
  - `design.md:65-66` places `DesktopDelivery`, `RecordingDelivery`,
    `SystemSoundPlayer`, and `SystemBrowserOpener` in one module. Three
    external adapters plus a test double in one module is a multi-adapter unit.
    Split the adapters into separate modules or justify the co-location; move
    `RecordingDelivery` to `tests/support/` (Q10).
  - `design.md:54` gives `cli.py` argument parsing, composition root, and exit
    codes. The composition root is allowed; argument parsing is a separate
    responsibility. Split the parser from the composition root.
  - The port and adapter pairs are co-located in one file
    (`design.md:55-66`). The architecture principles prefer separate files; the
    design should either separate them or add a mechanical import check so a
    consumer cannot import `TomlConfigStore`, `JsonStateStore`,
    `GitHubApiAdapter`, or `SystemCredentialSource`.
- Resolution: apply the splits/renames above. The external interface
  classification and the `RecordingDelivery` move also resolve the ch5 diagram
  findings in the Architecture documentation section.

### Q9 — The verification runner deviates from the commands.yaml contract and does not name the coverage keys

- Status: EVIDENCE-RESOLVED
- Findings:
  - `design.md:273-278` gives the `change` scope delta features plus permanent
    non-E2E, and the `e2e` scope permanent `@e2e` plus **delta** `@e2e`.
    `givn/commands.yaml` says `change` executes every delta scenario including
    `@e2e`, and `e2e` selects only `@e2e` from permanent specs. Delta `@e2e`
    therefore runs twice when both scopes run. Either remove delta `@e2e` from
    the `e2e` scope or record the deliberate duplicate and its cost.
  - `design.md:259-265` names the `verify` keys but not the `coverage` keys.
    The commented template in `givn/commands.yaml` points at
    `./measure-coverage.sh` and `./merge-coverages.sh`, while `design.md:314-315`
    and `design.md:507-524` use `scripts/measure-coverage.sh` and
    `scripts/merge-coverages.sh`. The design must give the exact
    `coverage.measure_command`, `coverage.merge_command`, and the three report
    paths, or the archive coverage gate will point at missing scripts.
  - The `change` scope's delta run with no delta features has no defined path
    (`design.md:273-278`). The runner must skip the empty delta invocation and
    still run the permanent non-E2E set; otherwise behave is invoked with no
    feature path and may fail before the union is reported.
  - `design.md:312` documents `./scripts/verify.sh --name ...` but the runner
    contract at `design.md:267-286` does not define `--name`. Either add it or
    use the raw `behave -n` command.
  - `design.md:515` gives the CLI subprocess profile output as
    `coverage/non-e2e/.coverage.*` only; the CLI also runs in the E2E scope, as
    the TUI row at `design.md:516` acknowledges. The CLI profile output should
    be `coverage/{non-e2e,e2e}/.coverage.*`.
  - `scripts/check_steps.py` (`design.md:298-300`) rejects `pass`, `...`, and a
    no-effect `return`, but a step body containing only a docstring or a
    constant is also a no-op and passes the AST check. Add the docstring/constant
    case, and note that the `pass`-only strictness fixture fails
    `check_steps.py`, not the behave runner (`design.md:302-304` says "must
    each fail the runner").
  - `design.md:301` says "any skipped scenario fails the runner, so `@wip` and
    `@skip` cannot ship". Behave does not skip `@wip` by itself; the givn
    `check tasks` WIP protection is the mechanism. State the actual skip
    mechanism and keep the tag protection separate.
- Resolution: apply these runner corrections. The permanent-corpus union and
  the `total > 0` exit condition do close the original zero-scenario GREEN
  trap; the cost is the permanent non-E2E suite on every change, which the
  design records at `design.md:288-290`.

### Q10 — macOS bundle, Python 3.11 floor, and `RecordingDelivery`

- Status: OPERATOR DECISION
- Blocks: scope, verification evidence, and the production package boundary.

**Context.** `design.md:244-253` adds `scripts/build-macos-app.sh` and the
`Passuff.app` bundle; `docs/arc42/07:41,50-53` and `docs/arc42/11:7` already
document the bundle, so it is not invented from nowhere, but it is not in the
proposal's specified surfaces (`proposal.md:52-54`) and it is not exercised by
the 185 scenarios. `design.md:252-253` delegates it to a manual macOS smoke
with no owner, date, or acceptance condition. The Python
`requires-python = ">=3.11"` floor (`design.md:27`) narrows the durable "Python
3" constraint (`docs/arc42/02:7`, `README.md:51`) to 3.11; the design records
`tomllib` as the reason but no rejected alternative (`tomli` would support 3.10).
`RecordingDelivery` (`design.md:65`) is a test double in the production
package; `docs/arc42/08:231` and `docs/arc42/10:44` say macOS E2E uses it, while
`design.md:252` says the bundle is not exercised by the corpus and
`docs/arc42/04:76` lists "recording delivery for macOS".

**Question.** What is in scope: the macOS bundle, the 3.11 floor, and
`RecordingDelivery`?

**Options.**

1. **Keep the bundle; record the manual smoke as a non-gated deliverable with
   an owner and condition; keep 3.11 and record `tomli`/3.10 as the rejected
   alternative; move `RecordingDelivery` to `tests/support/` and stop calling
   it E2E evidence.** Gain: matches ch7/R-001; the Linux acceptance is honest;
   the test double is out of production. Cost: an unverified macOS artifact;
   the floor excludes 3.10; a documented manual check. Example: the bundle is
   built by `scripts/build-macos-app.sh` and the manual smoke is recorded
   before archive; the 185 scenarios never touch it.
2. **Defer the bundle to a macOS packaging change.** Gain: a focused Linux
   rebuild with evidence for every acceptance item. Cost: ch7/R-001 remain
   unfulfilled; macOS notifications attribute to the terminal until the
   follow-up. Example: the rebuild ships the CLI/TUI; the bundle is a later
   change.
3. **Lower the floor to 3.10 with `tomli`; keep the bundle.** Gain: broader
   compatibility. Cost: one more dependency; not needed by the single operator.
   Example: a 3.10 machine can install the package.

**Recommendation.** Option 1 if the bundle is required now; Option 2 if the
operator wants a Linux-only acceptance. Either way, record the 3.11 floor
decision and its rejected alternative, and move `RecordingDelivery` out of the
production package. Do not claim macOS E2E fidelity from a test double.

**Resolution (operator, 2026-11-08):** Option 2. Defer `Passuff.app` to a
separate macOS packaging change. Remove the bundle script and section from the
design; add the bundle to the proposal's out-of-scope; add a dated deferral note
to arc42 chapter 7 and a technical-debt entry for the unbuilt bundle. Keep the
macOS CLI notification path. Keep the Python 3.11 floor and record `tomli`/3.10
as the rejected alternative. Move `RecordingDelivery` to `tests/support/` and
stop calling it E2E evidence.

**Rejected:** Option 1 (include the bundle with a manual smoke) — the operator
chose a Linux-complete acceptance with the bundle deferred. Option 3 (lower the
floor to 3.10) — one more dependency for no operator need.

### Q11 — Permanent use-case scope and rule numbering fail the review criteria

- Status: OPERATOR DECISION
- Blocks: the use-case scope check and the design's rule traceability.

**Context.** `givn instructions review` rejects a goal conjunction: `## Goal`
joining two verb phrases, or `## Success guarantee` naming two outcomes.
`review-activity-inbox/usecase.md:13` says "The developer reviews the activity
passuff recorded and marks it read" and `:89` says "The developer can review the
retained activity and mark it read; the history and read state survive
restarts...". That is two outcomes (review and mark read). The main flow at
`:26-28` likewise combines listing and marking. The same use case duplicates
R9 at `:46-47`. `watch-pull-requests/usecase.md:90-93` duplicates R42 and R43
and places R44 at `:89` between R41 and R42. The design's traceability at
`design.md:342` cites "R1-R38", but the permanent use case has R1-R44 (with the
duplicates); R39-R44 are not mapped. These are pre-existing permanent-contract
defects, but the rebuild claims to reproduce the permanent contract and the
design-review instruction requires the criteria to be run against it.

**Question.** Fix the permanent use-case documents now, or record an
operator-approved deferral?

**Options.**

1. **Fix the permanent use cases as a documentation-only correction.** Meaning:
   split the `review-activity-inbox` goal into "review the retained activity"
   and "mark activity read" (or move mark-read to a separate use case); renumber
   the duplicate rules; update the design traceability to the full rule set.
   Gain: the use-case criteria pass; the rebuild's contract is clean; no product
   behavior changes. Cost: permanent-contract edits; the proposal's "no
   use-case rule changes" clause at `proposal.md:83-85` must be amended; the
   domain model/glossary references need a sweep. Example: the goal becomes
   "The developer reviews the retained activity" and a separate capability owns
   "mark activity read"; the scenarios remain the same.
2. **Record a deferral.** Gain: no scope change. Cost: the review instruction's
   use-case criteria fail; the rebuild's "specification is complete" claim is
   weaker; the operator must own the deferral. Example: the design notes that
   the pre-existing goal conjunction is accepted for the rebuild.

**Recommendation.** Option 1. The defects are documentation-level and the
rebuild is the moment to make the permanent corpus internally consistent. If
the operator refuses, Option 2 must be recorded explicitly; a silent deferral
fails the review.

**Resolution (operator, 2026-11-08):** Option 1, in place. Reword the
`review-activity-inbox` goal to one outcome and its success guarantee to one
outcome; renumber the duplicate `R9` to `R10`; renumber the duplicate `R42` and
`R43` in `watch-pull-requests` to `R45` and `R46`, move `R44` into order, and
update the design traceability to the full rule set. No use-case split and no
scenario behavior change.

**Rejected:** Option 2 (split the use case) — a structural change the rebuild
does not need. Option 3 (defer) — it leaves the review criteria failing.

### Q12 — Durable documentation contains stale contradictions and an ADR index drift

- Status: EVIDENCE-RESOLVED
- Findings: see the Architecture documentation section below. The highest-impact
  items are `docs/arc42/06:342` (`TestBackend`), `docs/arc42/06:316` (`Clear`),
  `docs/arc42/08:89-90` (regular scenarios with injected ports), and
  `docs/arc42/11:51` (TD-005 "done" before implementation). The ADR register
  also carries an invalid status and two different ADR-0007 titles.
  `docs/reference.md:42` says `passuff-tui` returns 3 when requested without a
  terminal, while `set-up-passuff/initial-setup.feature:88-92` requires exit 2
  when the configuration is missing; the reference must scope the 3 to the
  configuration-present case. `docs/arc42/README.md:6-8` still names
  `passuff-pr-watch` as the current baseline.
- Resolution: apply the chapter edits listed in the table and the integrity
  scan. No new ADR is required; the ADR-0007 register/README inconsistency is a
  durable-index correction, not a qualification change. Fix the reference's
  no-terminal exit-code wording and update the arc42 README baseline line.

## Grilling branches

### Scope

- The rebuild's 185-scenario count is correct: 167 non-E2E and 18 E2E, exactly
  matching `proposal.md:55-56` and the tree.
- The proposal's "no scenario changes" (`proposal.md:83-87`) is not honest while
  the design's boundary table (`design.md:458-469`) lists behaviors with no
  scenario. Q1 is the blocker.
- The design adds behaviors not in the proposal: `setup` write preservation
  (`design.md:182`), a `--snapshot` default size of 100x30
  (`design.md:388-389`) not in `docs/reference.md:51`, the macOS bundle
  (`design.md:248-253`), and `RecordingDelivery` (`design.md:65`). The first is
  invented; the second is a design choice that should be recorded; the third
  and fourth are Q10.
- The "clipboard transcript" in `design.md:398` is not a specified screen or
  behavior; no use case, feature, glossary, or reference term exists. Remove it
  or replace it with "CLI transcript".
- `README.md:22-24,140-158` is target-state text ("being rebuilt", "until they
  exist", "Coverage is regenerated"). The design does not name the README
  sections to refresh for the completed product. The final review's
  `README-IMPACT` decision will require Install, Development, and Coverage to be
  edited; plan them now.

### Tech choices

- The stack fits the observable behaviors: Python 3, `behave`, synchronous
  `httpx`, `tomlkit`, stdlib `curses`, `jeepney`, `coverage.py`, and local
  loopback/bus twins. The rejected alternatives in `design.md:359-381` are
  real.
- The 3.11 floor is the only unrecorded narrowing; Q10.
- `httpx` over stdlib `urllib` is acceptable for explicit timeouts and typed
  request construction; no simpler alternative is required.
- `curses` plus a pure frame renderer is justified by the `--snapshot`
  contract (`docs/reference.md:51`) and avoids a third-party TUI. `Textual` is
  a real rejected alternative.
- `RecordingDelivery` in the production package is not a tech choice that
  survives the "test double" label; Q10.

### Missing scenarios

No issue found for the 18 normalized interactions; the matrix count is exact.
The gaps are boundary and error cases:

- unknown config key, invalid TOML, plain HTTP outside loopback, duplicate
  scope entries, `poll_interval_secs < 1`;
- unknown CLI option, missing option value, invalid `--focus-set`, invalid
  `--keys`, `--scope-deselect` in all mode;
- malformed/partial/timeout/non-2xx/`errors` GitHub responses, replayed node,
  stack-field rejection with retry;
- unknown state version, quarantine failure, duplicate activity ids, 30-day
  retirement, 80-character list excerpt;
- D-Bus `Notify` error, empty summary, long body; sound non-zero exit; `gh`
  missing/non-zero;
- activity detail body and Markdown rendering (Q2).
The generic "An invalid configuration is rejected" and "A corrupt state file
is replaced with a baseline" scenarios do not distinguish these cases. Q1
covers the decision.

### Testability

- The design names the strict-mode mechanism (`behave` undefined steps plus the
  runner's undefined/skipped failure), the not-implemented stub
  (`raise NotImplementedError`), and a single-scenario command
  (`behave -n` / `./scripts/verify.sh --name`). Good.
- Holes: `scripts/check_steps.py` does not reject a docstring-only body; the
  `pass`-only strictness fixture fails `check_steps.py`, not the runner;
  `@wip` does not skip by itself; `verify.sh --name` is undocumented in the
  runner contract; the `--keys` set is incomplete; `--observe-for 4` is the
  wrong unit; the continuous `passuff` scenario has no bounded stop; the
  `change` scope with zero delta features has no defined path.
- The `@e2e` scenarios assert concrete transcript/frame/D-Bus values in most
  rows. The activity-detail E2E does not assert the body (Q2).

### E2E fidelity

- The fake GitHub GraphQL twin on loopback, the private D-Bus session with a
  fake `org.freedesktop.Notifications`, the fake `gh`/sound/browser binaries,
  and the `--snapshot`/`--keys`/`--observe-for` TUI modes are real-interface
  drivers, not downgraded repository-only assertions.
- `design.md:437` uses `--scope-select` for a picker toggle; `--keys "m"` is
  not in the accepted set; `--observe-for 4` is milliseconds; the open-TUI
  action is driven by `--snapshot` while the pty smoke is untied; the
  continuous `passuff` scenario has no stop. These are Q6 findings.
- `design.md:414-419` silently redefines the separate-E2E-step rule; Q4.
- `RecordingDelivery` on macOS is a test double, not a real notification
  surface; Q10.

### Interaction Coverage

- Inventory count: watch-pull-requests 5, inspect-watch-status 1,
  review-activity-inbox 7, review-pull-request-stacks 1, change-watch-scope 3,
  set-up-passuff 1 = 18. The matrix at `design.md:437-454` has 18 rows and the
  seven feature files have 18 `@e2e` titles. No missing or extra row.
- The rows that need correction: toggle (Q6), mode switch (Q6), focus key
  sequence (Q6), open TUI (Q6), activity detail body (Q2), live refresh unit
  (Q6), start watching start/stop (Q6).
- No use-case inventory entry is unmapped. The matrix's "every entry has a
  non-empty driving mechanism" claim is true but not sufficient; three
  mechanisms are not the scenario's action or not executable as written.

### Risk

- The design's risk list (`design.md:526-535`) names the twin-shape, D-Bus,
  step-location, and coverage-install risks. The single most likely failure is
  not listed: the rebuild discovers that the permanent corpus does not contain
  the behavior it needs (the Ise test fails), and the implementer either invents
  behavior or weakens the scenario. That is Q1/Q2/Q3. The second is scope: 185
  scenarios, 18 E2E, a hand-written curses TUI, a D-Bus twin, a GraphQL twin,
  a pty smoke, coverage, and a macOS bundle in one change.
- Mitigation: add the missing boundary scenarios before implementation; keep a
  written spec-gap log and stop for an operator decision when a scenario needs
  unspecified behavior; do not weaken a driver to make a scenario pass.
- The macOS bundle/smoke is a risk with no owner/condition; Q10.

### Use-case and Persona context

- Identity and capability ownership are clear: six use cases, seven
  capabilities, no invented capability; `review-pull-request-stacks` Extends
  `review-activity-inbox` with the condition R9 (`usecase.md:54`), and the
  other typed relationships are `none`.
- Persona: `multi-account-developer` is confirmed (`personas/...md:3-5`);
  the design records the impact at `design.md:350-352`; Gherkin actors are
  "developer", not the Persona biography. No invented Persona. No issue found
  for Persona disposition.
- Scope criteria: the `review-activity-inbox` goal/success guarantee joins
  review and mark-read; duplicate R9 in that use case; duplicate R42/R43 and
  R44 ordering in `watch-pull-requests`; Q11.
- The activity-detail body contradiction is a cross-artifact use-case defect;
  Q2.

### Decision ledger

- `proposal.md:92-96` records no open `D<n>`; `design.md:354-383` records
  V1-V5 and "Carried decisions: none". There is no seed and no missing `D<n>`.
  No issue found.
- The design-local V1-V5 are answered now; the verification union, runner, TUI,
  notification fidelity, and GitHub twin are used by scenarios, so they are
  foundational decisions and are correctly not carried.

### Contracts and components

- Every consumer use case has a contract and at least one implementing
  component, but the checks fail on classification, ISP, and component
  responsibility: `TerminalInbox` is an external interface; `InboxReadModel`
  serves status and inbox with mark-read; `ConfigStore` and `SnapshotStore`
  expose writes to read-only consumers; `SetupSession` consumes
  `CredentialSource` without a setup need; `notify.py` contains three adapters
  plus a test double; `cli.py` contains argument parsing, composition, and exit
  codes; port and adapter pairs share one module. Q8.
- `review-pull-request-stacks` has no internal contract; its only row is the
  external `TerminalInbox`. Add the stack read model to `InboxReadModel` or add
  a stack-view contract. Q8.
- The composition root and implementation tests are the only allowed
  implementation imports; the design does not add a mechanical import check for
  the co-located port/adapter files. Q8.

### Decision timing

- No carried decision is recorded; V1-V5 are answered now. Good.
- The macOS bundle/smoke, `RecordingDelivery`, and the 3.11 floor are
  effectively carried or deferred without the four fields (default, trigger,
  switch cost, owner). Q10 and the Deferrals section.
- The contracts are current requirements, not open questions. Good.

### Visual design contract

- The section exists with `Interface: terminal` and viewports
  (`design.md:385-389`); the capture mechanism is a transcript per `@e2e`
  scenario (`design.md:404-406`), matching the terminal evidence rule.
- The durable `docs/design/design-system.md` is missing; the T25 authority is
  honest but not the required durable home. Q5.
- "Clipboard transcript" is not a specified screen; remove or rename.
- `Interface: terminal` is correct; the opposite Web UI classification is not
  stated. Add: no browser, no HTML/CSS/JS; the interface is a curses TUI that
  renders text in a terminal. Q14.
- The visual contract's accessibility ("text plus symbols, never color alone")
  and motion ("none beyond cursor movement and redraw") are appropriate.

### Deferrals

- The macOS bundle and manual notification-attribution smoke are deferred
  without an operator decision, owner, date, or condition (`design.md:252-253`).
- The missing boundary scenarios are effectively deferred if Q1 Option 2 is
  chosen; no operator decision is recorded.
- The unknown-key behavior is reference-only if Q3 Option 3 is chosen; no
  operator decision.
- `RecordingDelivery` is a deferred real macOS surface; no operator decision.
- The review instruction requires a recorded operator decision for any
  deferred mandatory check; none is present.

### Persona disposition

- `multi-account-developer` confirmed 2026-11-02 (`personas/...md:3-5`);
  relevance: personal and work accounts, macOS/Linux, terminal, desktop
  notification and sound; evidence: the design's impact statement at
  `design.md:350-352`. The visual design contract has at least one confirmed
  Persona. No issue found.

### Interface classification

- The design declares `Interface: terminal`; the opposite classification (Web
  UI) does not hold because the interface is a curses TUI, not HTML/CSS/JS in a
  browser. The design does not state this rationale; add it. Q14.
- The GitHub GraphQL twin is an external-service twin, not a browser; the real
  driver is `httpx` against loopback. The D-Bus twin is a real bus protocol
  with a fake service; the sound/browser/gh binaries are real subprocesses.
  Good.

### ADR qualification

- No candidate qualifies. The verification union is process guidance owned by
  `docs/arc42/08` and `scripts/verify.sh`; `behave`, `curses`, `tomlkit`, and
  the twins are implementation choices owned by `design.md`; ADR-0003 owns the
  core/adapter boundary and ADR-0005 the state migration. The assessment
  records the cheaper home for each (`arc42.md:20-32`). No issue found.
- The ADR-0007 register status "accepted; amended 2026-11-04"
  (`docs/arc42/09:17`) is not an allowed MADR status (`accepted`, `proposed`,
  `rejected`, `deprecated`, or `superseded by ADR-NNNN`), and
  `docs/arc42/adr/README.md:14` carries a different title. This is a durable
  index correction, not a new ADR; see Q12.

### Architecture documentation

The 12-row verdict was derived before comparing with `arc42.md`. My verdict
matches every row; the content check finds stale contradictions in the rows
marked Yes and in two No rows.

| # | Chapter | My verdict | `arc42.md` | Match? | Content check |
|---|---|---|---|---|---|
| 1 | Introduction and Goals | Yes | Yes | Yes | `docs/arc42/01:33-40` lists six use cases including `set-up-passuff`; the goals are unchanged. Good. |
| 2 | Architecture Constraints | No | No | Yes | `docs/arc42/02:7` records Python 3; no new constraint. The motivation phrase "The rebuild target is Python" is a rebuild reference, not a new constraint. |
| 3 | Context and Scope | No | No | Yes | `docs/arc42/03:9-51` already covers GitHub, D-Bus, sound, browser, gh, files, and the terminal inbox. No new external system. |
| 4 | Solution Strategy | Yes | Yes | Yes | `docs/arc42/04:60-78` records `behave`, `curses`, and the twins. Content issue: `docs/arc42/04:76` says "recording delivery for macOS", which conflicts with the "test double" classification and with `design.md:252-253`. |
| 5 | Building Block View | Yes | Yes | Yes | Stale content: `docs/arc42/05:53` uses `LiveObservation` and an undeclared `Session` where the design's contract is `ObservationSchedule`/`ObservationSession`; `docs/arc42/05:52` routes `TUI -->|InboxService| Core`, but `InboxService` is a separate component; `docs/arc42/05:43` puts `RecordingDelivery` in the CLI boundary although `docs/arc42/05:79` calls it "Not a production component"; `docs/arc42/05:74` gives `TuiApp` a run-on responsibility list. |
| 6 | Runtime View | Yes | Yes | Yes | Stale content: `docs/arc42/06:342` uses `TestBackend` (ratatui) and `:316` uses a `Clear` widget; the design uses stdlib `curses` plus a pure frame renderer. `:51-56` duplicates step 9 and contradicts itself ("delivery adapter is called once for the pass" vs "once per group"). `:199,221` route `InboxService.view` through `Core`; `:316-319` says no comment/review body is shown, contradicting T14 and `inbox-navigation.feature:245-261` (Q2). `:298` says "the CLI runtime delivers" in the TUI live-inbox flow. |
| 7 | Deployment View | No | No | Yes | `docs/arc42/07:41,50-53` already documents the `Passuff.app` bundle and the macOS paths; the build script is implementation. The manual smoke is unverified; Q10. |
| 8 | Cross-cutting Concepts | Yes | Yes | Yes | Stale content: `docs/arc42/08:89-90` says regular scenarios "call the CLI runtime's check entry point with injected ports", contradicting `design.md:415-416` and the same chapter's `:227` ("Every scenario drives the real CLI or TUI as a subprocess"). `:174` uses `OpenUrl`; `:231` says macOS E2E uses `RecordingDelivery`; `:23-25` omits the activity body (Q2). |
| 9 | Architecture Decisions | No | No | Yes | No new ADR is correct. Index issue: `docs/arc42/09:17` has the invalid status "accepted; amended 2026-11-04"; `docs/arc42/adr/README.md:14` has a different ADR-0007 title from the body (`ADR-0007...md:9`). |
| 10 | Quality Requirements | No | No | Yes | No new quality scenario is required, but `docs/arc42/10:43` still calls the TUI a "follow-up" and `:44` says macOS uses `RecordingDelivery`; both are stale relative to this rebuild. |
| 11 | Risks and Technical Debt | Yes | Yes | Yes | `docs/arc42/11:51` says TD-005 is "done in python-rebuild" before implementation; `:52` names the wrong change (`passuff-activity-inbox` instead of `review-activity-inbox`); `:25` R-019 says the TUI is "not driven through a PTY in CI" while its mitigation names a pty smoke. |
| 12 | Glossary | Yes | Yes | Yes | `docs/arc42/12` links the glossary. The glossary itself (`docs/glossary.md:7-10`) lists five behavior owners and omits `set-up-passuff`; `:23` says Activity detail never contains a body (Q2); `:58` uses `Inbox` while the design uses `InboxService`. |

Two durable-document findings sit outside the chapter table:
`docs/arc42/README.md:6-8` still names `passuff-pr-watch` as the current
baseline, and `docs/reference.md:42` gives a blanket exit 3 for a terminal-less
TUI, while `initial-setup.feature:88-92` requires exit 2 when the configuration
is missing. The design's `set-up-passuff` R8 (`usecase.md:51`) and
`review-activity-inbox` T5 (`usecase.md:52`) already distinguish the cases; the
reference is the outlier.

**Chapter-file integrity scan (all 12 files).** All twelve files exist under
`docs/arc42/`; `README.md` and `adr/` exist. Every file contains real content or
a deliberate link/register, not a scaffold placeholder. Zero box-drawing
characters (`┌┐└┘│─├┤┬┴┼╔╗╚╝║═╠╣╦╩╬╭╮╰╯`) were found. Zero non-table ASCII
pseudo-diagrams (`+---+`, `===`, multi-line `|` blocks) were found; the `|`
matches are Markdown tables. Mermaid diagrams are present in chapters 03, 05,
06, 07, and 10. The integrity failures are content, not file absence or ASCII
art: chapter 05 (stale component names), chapter 06 (`TestBackend`, `Clear`,
duplicate step 9, body contradiction), chapter 08 (injected-port sentence,
`OpenUrl`, macOS `RecordingDelivery`), chapter 10 (follow-up TUI, macOS
`RecordingDelivery`), chapter 11 (premature TD-005, wrong change ID, R-019),
chapter 09/ADR README (ADR-0007 status/title), and the glossary/domain model
issues below.

### Ubiquitous language

- The glossary is the vocabulary authority; the design uses most of its terms.
  Failures:
  - `docs/glossary.md:23` "Activity detail ... never contains a comment or
    review body" contradicts T14 and `inbox-navigation.feature:245-261`; Q2.
  - `docs/glossary.md:7-10` omits `set-up-passuff` from the behavior owners.
  - `docs/glossary.md:58` uses code symbol `Inbox`; the design uses
    `InboxService` (`design.md:61`).
  - `docs/arc42/05:53` uses `LiveObservation`; the design uses
    `ObservationSchedule` (`design.md:171`).
  - `docs/arc42/08:174` uses `OpenUrl`; the design says "language-neutral open
    intent" (`design.md:174`, `arc42.md:12`).
  - `design.md:398` uses "clipboard transcript", which is not in the glossary,
    the use cases, the features, or the reference.
  - `docs/arc42/06:342,316` use `TestBackend` and `Clear`, which are Rust
    ecosystem names; the implementation is stdlib `curses`.
- The design's contract names (`WatchCycle`, `ClassifyActivity`,
  `InboxReadModel`, `WatchScopeEditing`, `InitialSetup`, `ConfigStore`,
  `GitHubGateway`, `SnapshotStore`, `CredentialSource`, `NotificationPort`,
  `SoundPort`, `BrowserPort`, `ObservationSchedule`, `TerminalInbox`) are
  design-level names, not glossary terms. That is acceptable for contracts;
  `TerminalInbox` is misclassified as a contract (Q8).

### Structural model

- The rebuild has no new aggregate or entity: the same Watched Pull Request,
  Activity Record, Notification, Account, Watch Scope, and read model. A no-op
  structural edit is correct in principle; the design does not record that
  assessment.
- The no-op position is not fully correct as documentation maintenance:
  `docs/domain-model.md:13-14` says the model covers the capabilities "through
  the `passuff-observation-targets` change", omits `set-up-passuff`, and
  `:141-148` says the model is complete for the approved capabilities. The
  current change is `python-rebuild`; the status line is stale. The Activity
  Record aggregate at `:119` omits the activity body (Q2). The Watched Pull
  Request aggregate at `:118` omits the presentation attributes the TUI needs
  (title, description, creator, assignees, relations), which the design's
  persisted shape must carry (Q7).
- The design must record the structural-impact assessment: no new boundaries,
  but the model's status/capability list and the Activity Record body are
  affected.

## Blockers and sign-off state

All operator decisions are recorded in the Q1-Q11 resolutions above:
Q1 Option 1, Q2 Option 1, Q3 Option 1 (via Q1), Q4 Option 1, Q10 Option 2,
Q11 Option 1. The blockers are resolved.

## Hardening applied by the author (2026-11-08)

| Question | Resolution | Applied evidence |
|---|---|---|
| Q1 | Option 1 | `givn/changes/python-rebuild/.givn-skip` removed; delta features under `givn/changes/python-rebuild/specs/`; `proposal.md` acceptance, routing, and out-of-scope amended; `design.md` objective and verification updated |
| Q2 | Option 1 | `design.md` persists the body at 4 KiB; `docs/glossary.md` activity detail corrected; `docs/arc42/06` and `docs/arc42/08` corrected; T7 scoped to the list row; T14 retained; delta modified E2E scenario adds the body assertion |
| Q3 | Option 1 via Q1 | `docs/reference.md` unknown-key rejection retained; delta configuration outline asserts it; `design.md` configuration section records it |
| Q4 | Option 1 | `design.md` names `givn/steps/`, `givn/e2e_steps/`, `givn/environment.py`, `givn/e2e_environment.py`, `givn/steps/common.py`, and `--stage e2e` |
| Q5 | Evidence-resolved | `docs/design/design-system.md` created as a thin T25 index; the design contract points to it |
| Q6 | Evidence-resolved | Matrix drivers corrected: `--scope-select` plus picker snapshots, `--scope-mode`, `--focus-set`, `--observe-for 4000`, bounded `timeout --signal=INT`, `--observe --snapshot` plus pty smoke |
| Q7 | Evidence-resolved | Data model adds title, description, creator, assignees, involvement relations, body, 500-record cap, `sort_direction`; setup preservation removed |
| Q8 | Evidence-resolved | `TerminalInbox` removed; `WatchStatus`, `ActivityInbox`, and `StackedInbox` split; `SetupSession` credential edge removed; `notify.py`/`sound.py`/`browser.py` and `cli.py`/`composition.py` split; arc42 and glossary swept |
| Q9 | Evidence-resolved | Coverage keys specified; `--name` documented; E2E CLI profiles added; `check_steps.py` rejects docstrings/constants; skip mechanism corrected; delta `@e2e` scope behavior clarified |
| Q10 | Option 2 | `design.md` packaging defers the bundle; `docs/arc42/07` dated note; `docs/arc42/11` TD-007; `RecordingDelivery` moved to `tests/support/`; 3.11/`tomli` alternative recorded |
| Q11 | Option 1 | `review-activity-inbox` goal and success guarantee reworded; duplicate `R9` renumbered `R10`; duplicate `R42`/`R43` renumbered `R45`/`R46`; `R44` ordered; design traceability cites R1-R46 |
| Q12 | Evidence-resolved | `TestBackend`, `Clear`, `OpenUrl`, `LiveObservation`, and the production `Record` removed from the docs; injected-port sentence corrected; TD-005 timing corrected; ADR-0007 status and title aligned; reference exit-code precedence corrected; glossary, domain model, and arc42 README updated |

The hardening is complete. An independent verification pass must confirm every
row and the remaining evidence-resolved findings before sign-off.

## Verification findings (2026-11-08)

Independent verification of the hardening against the artifacts, not the table.
Confirmed clean: Q1 (`.givn-skip` absent, five delta features under
`givn/changes/python-rebuild/specs/`, `givn lint --change python-rebuild` exits
2, the cited boundary cases are covered, `proposal.md` acceptance/routing/out-of-scope
and `design.md:11-13` amended); Q2's design, use-case, glossary and E2E parts
(`design.md:230`, `usecase.md:54` T7/T14, `docs/glossary.md:24`,
`specs/review-activity-inbox/activity-inbox.feature:4-19`); Q3
(`docs/reference.md:231`, `docs/arc42/04:70`, `:135` of ch5, `docs/arc42/08:5`,
`design.md:213`, delta `pr-activity-notification.feature:13`); Q4
(`design.md:88-93,367-390`); Q5 (`docs/design/design-system.md`,
`design.md:446-448`); Q6 matrix drivers against `docs/reference.md:52-67`
(`--scope-mode`, `--focus-set`, `--observe-for` milliseconds, the documented
`--keys` vocabulary, `timeout --signal=INT`, pty smoke); Q7
(`design.md:218,229-230,234,246`); Q8 (`design.md:186-205`, no `TerminalInbox`,
three separate read-model contracts, no `SetupSession` credential edge, split
adapter and CLI modules); Q9 except the item below; Q10
(`design.md:284`, `proposal.md:73,103`, `docs/arc42/07:61-63`,
`docs/arc42/11:53` TD-007, `RecordingDelivery` under `tests/support/`,
3.11/`tomli` at `design.md:33`); Q11 (`review-activity-inbox/usecase.md:13,54,89`,
`watch-pull-requests/usecase.md:48-93` R1-R46 in order, `design.md:396`); the
18-entry inventory, 18-row matrix and 18 permanent `@e2e` titles; delta feature
and scenario tags, with no permanent `@e2e` tag removed; chapter-file integrity
(all twelve present, real content, no ASCII art) and the ADR-0007
status/title, reference exit-code precedence, glossary, domain-model and arc42
README corrections.

Remaining blockers:

1. **Q2 is not applied to arc42 chapter 8, and only partially to chapter 5.**
   `docs/arc42/08-crosscutting-concepts.md:23-25` still enumerates the version-2
   activity record without the selected body or its 4 KiB bound — the exact
   contradiction Q2 required corrected — and `:134` still states "No token,
   body, command text, or enum name is rendered", contradicting T14, the
   activity-detail runtime scenario (`docs/arc42/06-runtime-view.md:319-321`),
   `docs/glossary.md:24` and `design.md:230`. `docs/arc42/05-building-block-view.md:165`
   likewise lists the state fields without the body.
2. **The chapter-6 runtime view still contradicts itself.**
   `docs/arc42/06-runtime-view.md:51-56` numbers two consecutive steps "9" and
   states both "the delivery adapter is called once for the pass" and "invokes
   the delivery adapter once per group", then "the sound plays once" and "once
   per notification". `design.md:270` commits to one digest and one sound per
   pass.
3. **The chapter-6 TUI flow still routes delivery through the CLI.**
   `docs/arc42/06-runtime-view.md:300` says "the CLI runtime delivers the
   notifications" inside the live-inbox observation scenario, whose deliverer is
   `ObservationSession` (`design.md:202,270`, `docs/arc42/08:125`).
4. **Q9's empty-delta path is undefined.** `design.md:313-318` still defines the
   `change` scope as a delta run plus a permanent run without stating that an
   empty delta run set is skipped; a change with no delta features would invoke
   `behave` with no feature path, which Q9 required the runner to define.

The hardening table overstates one row: `design-review.md:921` (Q2) claims
`docs/arc42/08` was corrected; it was not, as finding 1 shows.

## Re-verification (2026-11-08)

Previous findings 1-4 and the Q2 overstatement are resolved:

1. `docs/arc42/08-crosscutting-concepts.md:23-26` records the selected comment
   or review body bounded at 4 KiB; `:135-137` drops the no-body claim and
   scopes the 80-character excerpt to the list row while the detail overlay
   shows the body as Markdown, bounded at 4 KiB;
   `docs/arc42/05-building-block-view.md:165` records the same body and bound.
2. `docs/arc42/06-runtime-view.md:51-55` has consecutive steps 9 and 10: one
   digest per pass, one delivery-adapter invocation for that digest, one sound
   when enabled. No contradictory cardinality remains in the chapter.
3. `docs/arc42/06-runtime-view.md:299-300` names `ObservationSession` as the
   deliverer in the live-inbox observation flow.
4. `givn/changes/python-rebuild/design.md:312-314` states that an empty run set
   is skipped and the remaining run set still executes and reports its counts.
5. The Q2 hardening row (`design-review.md:921`) now matches the artifacts.

Spot-check: no new contradiction with T7/T14
(`givn/specs/review-activity-inbox/usecase.md:54,61`), `docs/glossary.md:24`,
the design's activity data model (`design.md:230`), or the delta
`activity-inbox.feature:11-14` body assertion. `givn lint --change python-rebuild`
exits 2 (WIP and shape advisories only).

DESIGN-REVIEW: PASS

## Re-review (2026-11-08)

Review kind: independent
Author: maki-author
Reviewer context: independent-reviewer-1

This section re-checks the updated design after the implementation and review
round. The original review above is unchanged.

**What changed since the signed review, and whether it is a correction.**

| Change | Evidence | Classification |
|---|---|---|
| Strictness fixtures are retained as the reproducible proof instead of being removed after setup | `design.md:330-337`, `tasks.md:5`, `scripts/verify.sh:35-47` | Correction of the retained-fixture finding (F12); no behavior change |
| The Interaction Coverage Matrix row for `Developer opens the inbox TUI` names the as-built `--observe --snapshot` mechanism and the regular pty-smoke scenario | `design.md:503`, `givn/steps/activity_inbox_steps.py:87-95` | Correction of the driving-mechanism finding (F13); no behavior change |
| The as-built module names `passuff/status.py`, `passuff/tui/session.py`, and `tests/support/recording_delivery.py` | `design.md:63,109,103` | Already the reviewed design; the tree was conformed to it (F2-F5), not the reverse |
| The 4 KiB body, the 80-character list-row excerpt, `afplay`, `--stage e2e`, and the empty-run-set skip | `design.md:230,447-469,551,368-390,312-314` | Already the reviewed decisions; the implementation now matches |

No new observable behavior is introduced: no command, flag, configuration key,
persisted field, screen, or external boundary was added after the sign-off.

**Operator decisions Q1-Q11 are honored.**

| Question | Check performed | Result |
|---|---|---|
| Q1 | `.givn-skip` absent; five delta features with 33 entries / 19 scenarios; `proposal.md:51-65,83-89` amended; the delta covers the enumerated gap list (unknown key, invalid TOML, missing config, plain HTTP, duplicate scope, CLI usage, GitHub failures, state boundaries, delivery surfaces, `gh`, list-row excerpt) | honored |
| Q2 | `design.md:230` persists the body at 4 KiB; T7 scoped to the list row; `docs/glossary.md:24`, `docs/arc42/05:165`, `06:319-321`, `08:23-26` corrected; delta `activity-inbox.feature:11-14` asserts the body | honored |
| Q3 | `docs/reference.md:230` keeps unknown-key rejection; delta `An invalid configuration is reported with exit 2` includes the unknown-key case | honored |
| Q4 | `design.md:368-390` names `givn/steps/`, `givn/e2e_steps/`, `givn/environment.py`, `givn/e2e_environment.py`, `givn/steps/common.py`, `--stage e2e`; the tree matches | honored |
| Q5 | `docs/design/design-system.md` exists; `design.md:451` points to it | honored |
| Q6 | Matrix drivers are the implemented TUI options (`passuff/tui/app.py:359-371`): `--scope-select`, `--scope-mode`, `--focus-set`, `--observe-for`, `--observe --snapshot` | honored |
| Q7 | `design.md:222-240` carries title, description, creator, assignees, involvement relations, body, 500-record cap, `sort_direction` | honored |
| Q8 | `design.md:186-205`: no `TerminalInbox`; `WatchStatus`, `ActivityInbox`, `StackedInbox` split; `SetupSession` has no `CredentialSource` edge; adapters and CLI split | honored |
| Q9 | `givn/commands.yaml` matches `design.md:299-312`; `--name`; the empty-run-set skip; `check_steps.py` in every scope | honored |
| Q10 | `pyproject.toml` `requires-python >=3.11`; bundle deferred (`design.md:276-282`); `RecordingDelivery` under `tests/support/` | honored |
| Q11 | `review-activity-inbox/usecase.md:13-25` single outcome; `watch-pull-requests` R1-R46 in order; `design.md:396` traceability | honored |

No design change reopens a grilling branch or introduces unreviewed behavior.
The residual coverage classification belongs to the final review, not to this
design change.

DESIGN-REVIEW: PASS
