# Change seed: passuff-stacked-inbox

Status: proposed by passuff-stacked-inbox

- Use-case IDs: review-pull-request-stacks
- Topic: inbox-scope-and-stacks
- Personas: multi-account-developer
- Suggested start: 1 of 2; no functional dependency.
- Handoff: ready for `/givn-propose`

## Accepted decisions

### Stack membership comes only from GitHub metadata

- Context: GitHub's stacked-PRs preview exposes a `stack` object on pull request
  resources. A ref-chain fallback could infer chains from base and head refs, but
  passuff stores neither ref and only sees authored pull requests it already
  watches. The inbox promise depends on which source is authoritative.
- Options:
  1. **Metadata only** — query the GitHub `stack` field; null or absent means
     standalone.
     In practice: a preview-enabled repository groups; a non-preview repository
     shows flat rows.
     Gain: exact membership, size, position, and ultimate base; one code path.
     Cost: no grouping where the preview is off; depends on a preview API.
  2. **Ref-chain fallback only** — infer chains from `baseRefName` and
     `headRefName` among watched pull requests.
     In practice: `feature-a` to `feature-b` groups without the preview.
     Gain: works everywhere passuff has refs.
     Cost: sees only watched authored pull requests; merged boundaries and
     retargeted bases are ambiguous; both refs must be stored.
  3. **Metadata with fallback** — metadata wins; refs fill gaps.
     In practice: preview repositories are exact; others are inferred.
     Gain: grouping everywhere available.
     Cost: two disagreeing sources, a precedence rule, and double test surface.
- Chosen: **Metadata only** because exact membership and position outweigh
  grouping in repositories where the preview is unavailable. Rejected:
  Ref-chain fallback because it is incomplete and ambiguous; metadata with
  fallback because two sources double the test surface for an approximation.

### Omit unwatched stack members; keep the stack size and positions

- Context: a stack can contain pull requests authored by other people or by an
  account passuff does not watch. The inbox must not silently expand the
  observation scope.
- Options:
  1. **Omit unwatched members** — show only watched members, keep the full size,
     and show positions that may have gaps.
     In practice: a size-3 stack with positions 1 and 3 watched shows two rows
     and notes size 3.
     Gain: the existing authored-only boundary holds.
     Cost: the group can look incomplete; the developer may need GitHub for the
     missing member.
  2. **Fetch every stack member** — query stack membership independent of
     authorship.
     In practice: all members appear, including other people's pull requests.
     Gain: complete stacks in the inbox.
     Cost: breaks the authored-only observation boundary and needs a new query
     and state records.
  3. **Show the stack only when all members are watched** — otherwise flatten.
     In practice: partial stacks appear as standalone rows.
     Gain: no partial group.
     Cost: most stacks disappear from the grouping; the feature fails its goal.
- Chosen: **Omit unwatched members** because the authored-only boundary is a
  confirmed rule. Rejected: fetch every member because it expands observation
  scope; complete-stacks-only because it defeats the grouping goal.

### Stacks are read-only

- Context: GitHub's REST Stacks API can create, extend, and dissolve stacks.
  passuff's architecture never mutates GitHub.
- Options:
  1. **Read-only** — observe metadata, group rows, never write.
     In practice: the inbox links to GitHub for stack changes.
     Gain: no new write credentials, confirmation flows, or conflict handling.
     Cost: stack management stays in GitHub.
  2. **Allow stack mutations** — create and extend stacks from the inbox.
     In practice: the developer reorders a stack in the TUI.
     Gain: one place for stack work.
     Cost: GitHub write scope, destructive operations, and a new approval model.
- Chosen: **Read-only** because the product contract excludes GitHub mutations.
  Rejected: mutations because they add write scope and destructive operations
  that the local watcher does not otherwise need.

### Activity groups under the stack, records stay per pull request

- Context: activity records are keyed by pull request; the stack is a view over
  several pull requests.
- Options:
  1. **Group activity under the stack; keep per-pull-request records** — the
     group shows one unread count and the records keep their identity.
     In practice: a comment on member #41 appears under stack #7.
     Gain: no activity schema change; read state stays per record.
     Cost: the inbox projection must map a record to its stack.
  2. **Store stack-level activity** — one record per stack event.
     In practice: a comment on #41 belongs to stack #7 only.
     Gain: simpler grouping.
     Cost: loses which pull request the activity belongs to; breaks the existing
     record contract.
  3. **Separate stack screen** — keep the inbox flat and add a stack view.
     In practice: the developer switches views to see the stack.
     Gain: no change to the inbox projection.
     Cost: two places to review the same activity.
- Chosen: **Group under the stack, keep per-pull-request records** because it
  preserves the existing activity contract and read state. Rejected:
  stack-level records because they lose pull request identity; a separate screen
  because it splits the review goal.

## Open questions

### How should the observation degrade when the preview-gated stack field is unavailable?

The stack field exists only for repositories where the preview is enabled. A
GraphQL error on that field can fail the whole account query, which would turn a
preview gap into a lost observation. The proposal must choose the mechanism.
Options: probe once per account and cache the capability, retrying without the
field; always query without the field unless a configuration flag enables it;
or send the stack query and treat a GraphQL error as "all standalone" with a
retry. The last option needs a retry to avoid losing the account's observations.
No option is chosen yet.

### Which state schema version carries stack membership?

Stack number, size, position, and base are new fields on a watched pull request.
The proposal must decide whether to bump the state version or add optional
fields with defaults to version 2. The decision affects migration, quarantine
behavior, and the snapshot fixtures.
