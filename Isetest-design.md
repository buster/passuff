# Ise Test Report: passuff and givn

## Purpose

This report records what failed the Ise test in the passuff specification and
what the givn workflow could not handle while the rebuild was being made
executable. The goal is a fix list for givn, not a passuff feature list.

The Ise test used here is the one in the manifesto: delete the code, keep the
specification, rebuild in any language, and remain functionally identical
within the tolerance of the specification. Two things must therefore hold:

1. The specification must be sufficient: a rebuilder must not need knowledge
   that exists only in the deleted implementation.
2. The specification must be the test: the corpus must be able to fail an
   implementation that violates the spec.

Passuff failed both before the `python-rebuild` change. givn supported the
repair only after an independent design review manually found the gaps; it has
no first-class reconstruction path and a verification mode that can report
GREEN without executing a scenario.

## Method and evidence

The starting working tree contained no implementation: 0 Python files, 7
permanent feature files, 6 use cases, 185 permanent scenarios (18 `@e2e`), arc42
chapters, 7 ADRs, a glossary, a domain model, and `docs/reference.md`. `givn
status` initially reported no changes and `givn spec tree` reported the
capability inventory. `givn lint` reported advisories only.

The rebuild was planned as the change `python-rebuild`. An independent design
review (`givn/changes/python-rebuild/design-review.md`) interrogated the
proposal, design, permanent corpus, use cases, reference, arc42 chapters, and
glossary. Its Q1-Q12 findings are the primary evidence. The change added a
bounded delta of 18 scenarios, corrected the durable documents, and produced a
205-task plan.

The rebuild was executed after this assessment: the change was implemented,
reviewed, and archived on 2026-11-08. This report retains the pre-implementation
findings and adds the post-rebuild visual finding, A14.

### Identifiers used in this report

- `R<n>` — rule `n` in the `## Rules` section of a use case. `R` rules state
  general command and data behavior; `R1`-`R10` are the inbox rules in
  `review-activity-inbox`.
- `T<n>` — terminal-interface rule `n` in the same `## Rules` section. In
  `review-activity-inbox`, `T1`-`T28` govern the terminal UI. Rule `T25` is the
  palette rule; rule `T7` is the list-row excerpt and detail-body bound.
- `A<n>` — finding number in Part A of this report. Finding `A14` is the
  visual-contract finding.
- `B<n>` — fix-list item number in Part B. Item `B16` is the visual-evidence
  gate.
- `Q<n>` — question number in the independent design review (`Q1`-`Q12`).

## Verdict

Passuff was rebuild-shaped but not rebuild-proven. The feature files were
black-box and language-neutral; the architecture, reference, glossary, and
domain model were substantial. But the executable corpus was not a complete
gate, the specification contradicted itself on the activity detail, the durable
documents carried Rust-era symbols, and the runner, step definitions, verify
script, and coverage scripts were all part of the deleted code.

givn's workflow caught the defects through the independent design review. That
is a real success. The failures below are about what givn made the author and
reviewer do manually, and what it would have let through without that review.

After the rebuild, the same gap appeared in the visual layer: the text behavior
passed, but the palette and pane primitives did not, and the text-only E2E
evidence could not show it (finding A14, the visual-contract finding).

---

# Part A — What failed in passuff

## A1. The permanent corpus was not a complete gate (critical)

The proposal made the 185 permanent scenarios the acceptance gate. The
independent review found behaviors that the design itself listed but no
scenario exercised. An implementation could pass all 185 and still:

- ignore an unknown configuration key;
- accept invalid TOML, a missing configuration on `check`, a plain HTTP GitHub
  address outside loopback, or duplicate watch-scope entries;
- return a wrong exit code for an unknown command or option, a missing option
  value, an invalid `--focus-set`, or an invalid `--keys`;
- mishandle a malformed or partial GitHub response, a timeout, a non-2xx
  response, a GraphQL `errors` payload, a replayed duplicate node, or a
  rejected stack query without the one retry;
- fail an unknown state version, quarantine, duplicate activity ids, the
  30-day retirement boundary, or `poll_interval_secs` below 1 incorrectly;
- swallow a D-Bus `Notify` error, a non-zero sound player, a missing `gh`, or a
  missing browser opener;
- exceed the 80-character list-row excerpt.

Evidence: the review's Q1, the design's original external-boundary table, and
`docs/reference.md`'s validation list. The permanent features covered the
happy paths and many failures, but not these boundaries.

Consequence: the corpus measured scenario count, not specification coverage.
That is the Ise test failing in the most direct way: the spec was not the test.

Resolved in the change: 18 delta scenarios under
`givn/changes/python-rebuild/specs/`, 17 regular and 1 modified `@e2e`; the
`specs` artifact was unskipped, and the proposal's acceptance clause now names
the permanent corpus plus the delta.

## A2. The activity-detail body contradicted the specification (critical)

The use case and the executable feature required the full selected comment or
review body:

- Terminal-interface rule T14 of `givn/specs/review-activity-inbox/usecase.md`
  required "the selected comment or review body in full, rendered as Markdown";
- the scenario `The activity detail shows and scrolls a long comment` asserts
  `comment line 40` after `End`.

Three other authorities denied it:

- `docs/glossary.md` said the activity detail "never contains a comment or
  review body";
- `docs/arc42/06-runtime-view.md` said "No comment or review body is shown";
- the design's persisted state had only a "bounded summary".

Terminal-interface rule T7 (the list-row excerpt and detail-body bound) also
capped the screen at the 80-character excerpt, which is incompatible with a long
body in the detail.

Consequence: a rebuilder following the glossary or the runtime view would omit
the body and pass the E2E detail scenario, which asserted metadata only. A
rebuilder following rule T14 would persist a body the state model did not carry.
The specification was not sufficient; it was self-contradictory.

Resolved in the change design: the activity record persists the body with a
4 KiB bound; the list row keeps the 80-character excerpt; the glossary, arc42
chapters 5/6/8, rule T7, and the design data model are corrected; the delta adds
a `@givn.modified @e2e` assertion for the body.

## A3. Specified behavior was not executable (critical)

The unknown-key rejection existed in the reference, arc42 chapter 4, and
chapter 8's `deny_unknown_fields`, but not in any use-case rule or scenario.
The same held for parts of the reference's validation list and the delivery
failure paths. The rule was real in the documents and invisible to the test
suite.

Consequence: the rebuild could choose the opposite behavior and still report
GREEN. A configuration typo such as `poll_interval_sec` could be silently
ignored.

Resolved in the change: the rule is kept, the reference names it, and the delta
configuration outline asserts it. The same pattern is now closed for the other
boundary cases in A1.

## A4. The use-case rule set was internally broken (high)

- `givn/specs/review-activity-inbox/usecase.md` had two `R9` rules: focus
  filtering and "recording history never replays a notification".
- `givn/specs/watch-pull-requests/usecase.md` had two `R42`s and two `R43`s,
  with `R44` placed between `R41` and `R42`.
- The design's traceability cited `R1-R38` while the use case had rules through
  `R44`; `R39-R44` were unmapped.

Consequence: rule references in the design, review, and tests could point at
the wrong rule. The specification was not a stable contract.

Resolved in the change: the duplicate `R9` became `R10`; the duplicate `R42`
and `R43` became `R45` and `R46`; `R44` is ordered; the design cites `R1-R46`.

## A5. The use-case goal and success guarantee were compound (high)

`review-activity-inbox` said "The developer reviews the activity passuff
recorded and marks it read" and "The developer can review the retained activity
and mark it read; the history and read state survive restarts..." The review
instruction's scope criteria reject a `## Goal` with two verb phrases and a
`## Success guarantee` with two outcomes.

Consequence: the use-case boundary was ambiguous; a rebuilder could not tell
whether mark-read was a separate goal or part of review.

Resolved in the change: the goal is one outcome ("keeps the retained
activity's read state current through review") and the guarantee is one
outcome. No behavior or scenario changed.

## A6. The capability route was incomplete (high)

`docs/arc42/01-introduction-and-goals.md` said "five use cases" and omitted
`set-up-passuff`. The glossary's capability list omitted `inbox-navigation` and
`initial-setup`. The domain model said the model covered the capabilities
"through the `passuff-observation-targets` change" and omitted initial setup.

Consequence: a rebuilder could not reach the setup capability from the goals,
the vocabulary, or the structural model.

Resolved in the change: six use cases are routed; the glossary list and domain
model are corrected; `arc42.md` records the correction.

## A7. The durable documents carried Rust-era implementation residue (high)

The specification contained names and mechanisms from the deleted Rust
implementation:

- `docs/glossary.md` used `CredentialSpec::GhOAuth`,
  `CredentialSpec::{Env,Command,File}`, and `CredentialSpec::File`;
- `docs/arc42/06-runtime-view.md` used `TuiAction::OpenUrl` and `toml_edit`;
- `docs/arc42/08-crosscutting-concepts.md` used `deny_unknown_fields`,
  `toml_edit`, and `RecordingDelivery` as macOS E2E evidence;
- `docs/arc42/05-building-block-view.md` had `LiveObservation`, `Record` in
  the production diagram, and the read model routed through `Core`;
- `docs/arc42/06-runtime-view.md` used `Clear` and `TestBackend`;
- `docs/arc42/05` used `InboxService` after the read model was split.

Consequence: a rebuilder in Python would either import concepts that no longer
exist or preserve stale names. This is exactly the manifesto's Law 2: knowledge
that lives only in the code, or in the code's vocabulary, is a spec bug.

Resolved in the change: the glossary, arc42 chapters 4/5/6/8/10/11, and the
design use language-neutral or Python names; `RecordingDelivery` moved to
`tests/support/`; the production diagram no longer contains the test double.

## A8. The persisted data model was incomplete for the specified screens (high)

The original design persisted a bounded summary but the TUI reads a pull
request title, description, creator, assignees, involvement relations, stack
membership, CI result, and state; the activity detail needs the selected body.
The design also omitted the 500-record cap and the sort direction in `ui.json`.

Consequence: an implementer following the design would not have the data the
specified screens require, and the E2E detail scenario would fail only if the
body assertion existed.

Resolved in the change design: the design data model carries those fields, the
body bound, the cap, and `sort_direction`. The implementation is still pending.

## A9. The reference and the TUI exit codes disagreed (medium)

`docs/reference.md` said `passuff-tui` returns 3 when the terminal is missing
or state loading fails. The setup feature requires exit 2 when no
configuration exists, even without a terminal.

Consequence: the same invocation had two specified exit codes.

Resolved in the change: exit 2 for a missing or invalid configuration; exit 3
when a valid configuration is present but the terminal is missing or state
loading fails.

## A10. The macOS bundle was documented but unbuildable in the corpus (medium)

arc42 chapter 7 and risk R-001 documented `Passuff.app` as the supported macOS
notification-attribution path. The corpus had no scenario for it, and the
original design added a build script with only a manual smoke and no owner,
date, or condition.

Consequence: a specified deployment artifact had no executable evidence.

Resolved in the change: the bundle is deferred to a separate packaging change;
chapter 7 carries a dated note; TD-007 records the debt; the design no longer
claims it.

## A11. The executable half of the specification was gone (structural)

The deleted implementation also contained the Gherkin runner, the step
definitions, `scripts/verify.sh`, and the coverage scripts. At the start,
`givn/commands.yaml` had `verify` and `coverage` commented out, and
`givn check` could not go GREEN. The corpus was a document, not a test.

Consequence: "the spec is the test" was false. The Ise test could not be run.

Resolved in the design and tasks: `behave`, `givn/steps/`,
`givn/e2e_steps/`, `scripts/verify.sh`, `scripts/behave_counts.py`,
`scripts/check_steps.py`, the coverage scripts, and the strictness proof are
the first setup batch.

## A12. The "no delta" premise was not honest (high)

The first proposal claimed a pure non-delta rebuild: skip the `specs` artifact
and accept the 185 permanent scenarios. After A1 and A2, that claim was false.
The change is a bounded spec-completion delta, not a behavior change.

Consequence: the original change would have archived with an incomplete gate
and a false completeness claim.

Resolved in the change: the proposal, routing table, acceptance clause, and
verification design were amended; the change is recorded as a spec-completion
delta with no user-visible behavior change.

## A13. The reference remains a second normative document (remaining risk)

`docs/reference.md` presents itself as "Exhaustive commands, flags,
keybindings, configuration keys, exit codes, and observation rules." Chapter 1
names the use cases authoritative. The delta now asserts many reference rules,
but not every bullet (for example sound-file existence and some stack and
rate-limit details) has a scenario. The hierarchy is stated, but the reference
has not been reduced to a derived index.

Consequence: future drift is still possible between the reference and the
executable corpus.

Suggested passuff follow-up: either declare `docs/reference.md` derived in its
first paragraph and point to the owning use cases, or add scenarios for every
normative bullet. Do not leave two unqualified authorities.

## A14. The visual contract was not executable (high)

The spec does contain visual guidance. Rule T25 — terminal-interface rule 25
in the `Rules` section of `givn/specs/review-activity-inbox/usecase.md` — names
watn's palette: dim borders; cyan-blue labels, key hints, and table headers;
bold key hints; gold selected rows and attention text; green focus borders and
Markdown headings; muted metadata; bright-gray description and activity body.
`docs/design/design-system.md` names the pane, table row, overlay, footer, and
status-line primitives. None of it was executable.

The rebuilt TUI is monochrome and flat. `passuff/tui/app.py` paints every frame
line with `stdscr.addnstr` and never calls `init_pair`, `color_pair`, `A_BOLD`,
or `A_DIM`; `passuff/tui/frame.py` returns plain strings, so no color role,
border, pane, table header, or overlay frame exists in the rendered output. The
captured Alacritty screenshot at `docs/design/passuff-tui-alacritty.png` shows
the header, accounts, scope, PR rows with the stack and people markers, activity
rows, and footer as an unstyled vertical list separated by a 60-character rule.

The E2E evidence could not have caught this. The `@e2e` transcripts under
`givn/archive/python-rebuild/evidence/visual/` are plain text; the `@e2e`
assertions are text-substring assertions; the independent review signed
`VISUAL-REVIEW: PASS` by reading those transcripts. Text transcripts preserve
content and ordering but not color, attributes, borders, or pane geometry.

Consequence: the rebuild is functionally GREEN and visually divergent. The
"spec is the test" claim is false for the visual layer: the specification had
visual guidance, but the executable corpus and its evidence did not.

Suggested passuff/givn follow-up: make the visual contract executable. At
minimum, add a color- and attribute-aware frame dump (for example an ANSI
snapshot asserted role by role) and a rendered visual artifact (screenshot or
terminal-state capture) per user-facing screen; require the visual review to
compare against rule T25 and the design-system primitives, not only text
substrings.

---

# Part B — What failed in givn (fix list for the Ise principle)

## P0 — Blocking

### B1. No first-class reconstruction change

givn's lifecycle is delta-shaped: proposal -> specs -> design -> arc42-docs ->
design-review -> tasks -> review. `givn graph` has no reconstruction artifact
and `givn new --skip specs` only suppresses the artifact; it does not replace
the behavior delta with the permanent corpus.

The author had to choose between a change with a vacuous delta (and a verify
gate that runs nothing) and direct work without tasks or review. Both are wrong
for a rebuild.

Fix: add a reconstruction change kind or a `--from-permanent` mode whose
acceptance source is the permanent corpus, whose design/tasks/review still
apply, and whose status explicitly says "no behavior delta; permanent corpus
gate".

### B2. A change with zero delta scenarios verifies ZERO and can report GREEN

The runner contract says `change` executes delta scenarios; `regular` and
`e2e` execute the permanent corpus. A non-delta change therefore runs zero
scenarios in `change` scope. Nothing in givn requires `total > 0`; the design
had to invent the permanent-corpus union and the `total > 0` exit condition.

Evidence: `givn/commands.yaml` runner comment and the original design. The
independent review called this the "zero delta scenarios report GREEN" trap.

Fix: givn itself must refuse a change-scope verification that selects zero
scenarios, or must automatically include the permanent corpus when a change has
no delta. Do not leave this to each project's runner script.

### B3. No rule-to-scenario completeness check

givn counts scenarios (`spec coverage`, `spec tree`) and reports structural
scenario similarity (`spec duplicates`). It does not map use-case rules
(`R1..`, `T1..`, `S1..`) or reference validation bullets to scenarios. A rule
without a scenario is invisible. The design review found A1 and A3 by reading
every document; that is not repeatable.

Fix: a `givn spec trace`/completeness command that:
- extracts rule IDs and requirement bullets from use cases/reference;
- maps each to the scenario titles/steps that assert it;
- reports unbound rules, orphan scenarios, duplicate rule IDs, and rule
  references in the design that do not resolve;
- fails or warns at design-review and review for a reconstruction change.

## P1 — High

### B4. No mechanical stale-language or stale-symbol scan

`givn lint` checks `.feature` syntax, tags, and duplicates. It does not scan
`docs/`, `givn/specs/*/usecase.md`, or the glossary for code syntax or symbols
that no longer exist. It would not have caught `CredentialSpec::GhOAuth`,
`TuiAction::OpenUrl`, `toml_edit`, `deny_unknown_fields`, `TestBackend`, or
`Clear`.

Fix: a `givn lint --docs`/`givn check residue` that flags:
- language syntax in prose (`::`, crate/module paths, framework attributes,
  test-backend names);
- symbols in the glossary's code-symbol column that the design does not name;
- design contracts that no longer match the glossary;
- implementation names in a spec-layer chapter (arc42 1-3).

### B5. No spec-integrity gate for use cases and traceability

givn serves the use-case scope criteria in `givn instructions review`, but no
command applies them. Duplicate rule IDs, compound goals, a design citing a
rule range that does not exist, and a glossary whose capability list omits a
capability are all manual findings.

Fix: `givn check spec-integrity` that runs the served criteria mechanically:
unique rule IDs per use case; one outcome per goal and success guarantee;
design rule references resolve; every capability appears in the glossary and
arc42 route; every use case has a feature file.

### B6. The runner contract is underspecified

The contract says the adapter must execute `GIVN_FEATURES` and handle
`GIVN_RESULT_SCOPE` and `GIVN_RESULT_FILE`. It does not say how
`GIVN_FEATURES` is encoded (whitespace, newline, colon?), what happens when it
is empty, or what counts as a valid result for a scope with no features. The
design had to normalize all three and skip empty run sets.

Fix: document the exact encoding and the empty-set rule; make `givn check`
refuse an empty scope for a change that claims behavioral work; require
`total > 0` in the result contract.

### B7. Delta `@givn.modified`/`@givn.removed` are not deduplicated in the E2E scope

A delta `@givn.modified` scenario with the same title as a permanent `@e2e`
scenario runs twice in the E2E scope (permanent + delta) until archive. The
specs instruction does not define the dedupe. The design had to state "the
delta version is the stronger assertion; the permanent counterpart remains
until archive replaces it."

Fix: either givn excludes the permanent scenario when a delta
`@givn.modified`/`@givn.removed` with the same capability and title exists, or
document the double-run explicitly in the runner contract. `givn lint` should
report the collision.

## P2 — Medium

### B8. E2E step-definition policy collides with Gherkin phrase binding

The served specs/design instructions require separate E2E step definitions.
Gherkin runners bind a phrase once; a second tree with shared phrases creates
an ambiguous-step error. The design had to invent a staged layout
(`--stage e2e`, `givn/e2e_steps/`, `givn/e2e_environment.py`) and a shared
`givn/steps/common.py` for phrase ownership.

Fix: document a canonical staged layout per supported runner, including where
shared phrases live and how the runner selects one tree without loading both.
Alternatively, state when a single tree is allowed because every scenario
drives the real interface.

### B9. The task template assumes a delta with `@wip` scenarios

The setup task says "back to a passing baseline with the real scenarios still
`@wip`." For a reconstruction, the real scenarios are the permanent corpus and
are not `@wip`; the permanent corpus is intentionally RED. The design had to
write that deviation. The task list also repeats a full design-constraint block
for every one of 203 scenarios.

Fix: a reconstruction task template whose RED baseline is the permanent corpus,
with per-batch constraint blocks instead of per-scenario copies, and a status
view that summarizes by batch.

### B10. `givn new --skip` has no inverse and no warning

Skipping `specs` writes a 6-byte `.givn-skip` file. There is no `givn unskip`
command; the author deleted the file manually after the review. The status
shows `⊘ specs` but does not say how to restore it or what gate consequence the
skip has.

Fix: `givn skip`/`givn unskip` with status hints; skipping `specs` should mark
the change as "no executable delta" and require an explicit acceptance source.

### B11. No first-class deferred-capability record

The macOS bundle was specified in arc42 chapter 7 and risk R-001 but absent
from the corpus. The change had to add a dated chapter note and a TD row. givn
has no "specified but not built" state in the proposal or design, and no archive
gate for an unowned deferral.

Fix: a `deferred` list in the proposal/design with the owning durable document,
a follow-up change ID, and an archive gate that refuses an unowned deferral.

### B12. The major-change review classification mixes behavior and implementation

`givn instructions review` defines major change as including "multiple
capabilities or high blast radius." A behavior-frozen rebuild touches all seven
capabilities but changes no contract, schema, or behavior. The proposal first
recorded self-review; the design-review had to ask the operator and switch to
independent after the fact.

Fix: separate behavioral blast radius from implementation blast radius; add a
reconstruction category whose default review kind is independent; gate the
proposal so a self-review cannot be recorded when the major criteria match.

### B13. `givn spec route` gives no signal for a rebuild

Route reported a 12.00 tie and no ranked list. Every capability is a candidate
in a rebuild, so the recommendation is meaningless. The proposal recorded
`REUSE`/`EXTEND` by hand.

Fix: a reconstruction route mode that lists every permanent capability, its
scenario count, and whether the design carries a delta or relies on the
permanent corpus.

### B14. `givn spec duplicates` is noise for a sufficiency audit

It reported 2,450 structural matches without behavioral equivalence. It does
not answer the Ise question ("is every specified behavior gated?"). The rule
trace in B3 is the useful signal.

Fix: keep duplicates as an advisory cleanup tool; do not treat its output as
coverage or sufficiency evidence; add the rule trace instead.

### B15. `givn lint` does not apply the use-case criteria

`givn lint` checks `.feature` syntax and delta tags. It does not run the
use-case scope rejection criteria, duplicate-rule detection, or goal/success
guarantee shape. Those were manual design-review findings.

Fix: a `givn lint --specs` mode that includes the spec-integrity checks in B5.

### B16. The E2E visual gate accepts text transcripts as visual evidence

The canonical E2E policy requires a real-interface primary assertion. For a
terminal product, the real interface is the rendered terminal state. The
reviewed evidence is `evidence/visual/<slug>/transcript.txt`, which contains
only the text written to the terminal. It cannot show color pairs, attributes,
borders, pane geometry, overlays, or cursor state. The review can therefore
sign `VISUAL-REVIEW: PASS` while the product renders a monochrome flat list
that contradicts the spec's palette and primitives (A14).

Fix: add a visual-evidence mode that captures the rendered terminal state with
attributes (ANSI, HTML, or a terminal-state snapshot), and require at least one
such artifact per user-facing screen. Keep the text transcript as secondary
evidence. A visual review must be able to fail on a palette or layout
divergence without a human eye.

---

# Part C — What givn did right

- The permanent corpus was language-neutral and black-box. The feature steps did
  not name Rust types or module paths.
- `givn spec tree` gave an exact capability inventory and counts.
- `givn lint` accepted the delta structure and reported only advisories; it did
  not silently accept malformed tags.
- The artifact separation (proposal, specs, design, arc42, design-review, tasks)
  gave the review a place to find the gaps and the author a place to correct
  them.
- The independent design review found the two blockers (Q1 corpus completeness,
  Q2 activity body), the rule-numbering defects, the residue, the contract
  splits, and the bundle ambiguity before any implementation started.
- The delta mechanism (`@givn.added`, `@givn.modified`) made a spec-completion
  change expressible without pretending the behavior changed.
- The design-review-before-tasks order prevented 203 scenarios being built
  against an incomplete gate.

The framework did not pass the Ise test by itself. It made the failure visible
and repairable. That is the difference between a delta workflow and a
reconstruction workflow, and it is the gap this report asks givn to close.

---

# Part D — Remaining risks after the change

1. The rebuild is functionally GREEN (248/248 at `959cec4`, archived) but not
   fully proven: the visual contract is unverified (finding A14, the
   visual-contract finding) and the evidence is text-only. The 203-entry plan is
   no longer the claim; the archived receipt is.
2. `docs/reference.md` remains a second normative document (A13).
3. The fake GitHub GraphQL twin defines the external boundary for tests. A
   rebuild can pass the twin and still mismatch the live API; the spec tolerates
   only the twin's shape.
4. The step definitions are transient timber. `check_steps.py` and the
   strictness proof reduce empty-step GREEN, but the acceptance suite is only as
   good as the steps' assertions.
5. The macOS bundle is deferred (A10). macOS notification attribution is
   unverified until the packaging change.
6. The change receipt covers the change scope (248/248 at `959cec4`); the\n   measured coverage boundary still excludes delta-only paths, so those paths are\n   verified by the change scenarios, not by the Cobertura reports.
7. The visual layer is unverified (finding A14). The archived change signed
   `VISUAL-REVIEW: PASS` from text transcripts; the rebuilt TUI is monochrome
   and flat against terminal-interface rule T25 and `docs/design/design-system.md`.

---

# Appendix — Evidence index

- `givn/archive/python-rebuild/` — the archived change (2026-11-08). The
  `givn/changes/python-rebuild/...` paths below are pre-archive locations.
- `givn/changes/python-rebuild/design-review.md` — Q1-Q12, resolutions, and
  `DESIGN-REVIEW: PASS`.
- `docs/design/passuff-tui-alacritty.png` — captured screenshot of the rebuilt
  TUI under Alacritty; shows the monochrome flat frame (A14).
- `givn/archive/python-rebuild/evidence/visual/` — the 18 text transcripts used
  as the archived change's visual evidence.
- `givn/specs/review-activity-inbox/usecase.md` terminal-interface rule T25 and
  `docs/design/design-system.md` — the visual contract that was not executable.
- `givn/changes/python-rebuild/proposal.md` — acceptance delta, routing,
  out-of-scope, review context.
- `givn/changes/python-rebuild/design.md` — Python target, runner union, step
  trees, data model, coverage, E2E matrix.
- `givn/changes/python-rebuild/specs/` — 19 delta scenarios.
- `givn/changes/python-rebuild/arc42.md` — 12-row assessment.
- `givn/archive/python-rebuild/tasks.md` — 207 tasks, all checked; the change
  receipt is 248/248.
- `givn/specs/review-activity-inbox/usecase.md` — terminal-interface rules T7
  and T14 and inbox rules R1-R10.
- `givn/specs/watch-pull-requests/usecase.md` — R1-R46.
- `docs/reference.md` — validation list and exit codes.
- `docs/arc42/` — chapters 1, 4, 5, 6, 7, 8, 9, 10, 11.
- `docs/glossary.md`, `docs/domain-model.md`, `docs/design/design-system.md`.
- `givn/commands.yaml` — verify and coverage keys.
