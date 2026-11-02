<!-- givn:begin:givn-project -->
This project uses **givn** for spec-driven development. This block is
auto-maintained by `givn init` / `givn skills sync` — do not edit it by hand.

- Run `givn instructions` as the first action of every session. Do not
  search, grep, or inspect files to determine what to do next — this command
  reports the current project state and the next lifecycle step, including
  `--change <CHANGE>` for active changes. A reported step applies when
  continuing that change; it does not authorize unrelated work or require every
  task to become a Givn change.
- For new or unclassified work, load `givn-start` to choose answer only, direct
  work without a Givn change, ideation, or a formal change. For a change whose
  goal and scope are already clear, run `givn new <change-id>` directly. Honor
  an explicitly requested, applicable path without another routing conversation.
- For every user-facing question, follow the effective dialog policy from
  `givn instructions dialog`; load it once per context before the first
  question, including clarification and confirmation outside a named
  workflow, and pass the obligation explicitly when work is handed to another
  context. This obligation applies with optional addons disabled and outside
  any named skill; the presenting agent remains responsible for the question.
  Before the first question of a type indexed by the policy, read that type's
  companion examples from `givn instructions dialog-examples`; retained
  examples need no reread. The policy is not loaded automatically.
- Generated guidance is skills-only: Claude Code and OpenCode use their
  native skill invocation, the neutral `.agents` target uses skill discovery,
  OMP uses `/skill:givn-<step>`, and Maki uses an explicit request or an
  agent-selected skill. No generated Givn command files exist.
- Run `givn status --change <CHANGE>` for a full artifact checklist of an
  active change (includes task progress and pending task descriptions).
- Tick off each scenario task when its batch is complete — not when the code
  merely compiles. Never batch-check boxes at the end.
- One atomic `givn commit` per coherent behavior batch, not per scenario. The
  revision message carries the release note as its message subject and names the batch
  in the body. For a single-scenario batch, the scenario title goes in the message body.
  A raw Git commit is stamped by the hook, a raw Fossil check-in is
  not.
- Each checked scenario records the batch `COMMIT:` revision. The commit must
  contain the scenario's implementation or regression test; a comment-only,
  tag-only, task-only, or feature-only change is not evidence.
- Tests may pass immediately when they protect a distinct requirement. Do not
  force a production diff per scenario. Deduplicate or parameterize genuine
  repetition.
- The full workflow and available skills are documented in the output of
  these commands — this block is intentionally just a pointer.
<!-- givn:end:givn-project -->

<!-- givn:begin:communication-policy -->
## Communication Policy

Presentation rules for all work in this project.

- Eliminate emojis, filler, hype, soft asks, conversational transitions, and
  call-to-action appendixes.
- Assume the reader retains high perception. Do not simplify for tone.
- Use blunt, directive phrasing.
- Disable engagement- and sentiment-boosting behaviors.
- Suppress satisfaction scores and emotional softening.
- Never mirror the user's diction, mood, or affect.
- No transitions. No motivational content.
- Show brief necessary context, concrete consequences, and material costs.
- Work from examples: show what the user sees, what the code does, or what
  changes observably. A label is not an example.
<!-- givn:end:communication-policy -->
