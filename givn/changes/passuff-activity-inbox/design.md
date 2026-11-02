# Design: passuff-activity-inbox

## Scope and constraints

Three capability deltas across three use cases:

- `watch-pull-requests` / `pr-activity-notification`: add the global repository
  watch scope (R39).
- `inspect-watch-status` / `watch-status` (new): the `status` read surface.
- `review-activity-inbox` / `activity-inbox` (new): inbox listing and read state.

The core persists a bounded activity history, exposes an `InboxService`, and
keeps observation unchanged apart from the scope filter. Status, inbox, and read
never contact GitHub. The TUI is the next change and consumes `InboxService`.

- State version 1 remains loadable and is rewritten as version 2 on first load.
- History is capped at 500 records; read state persists.
- No new runtime dependency.

## Technology decisions

| Decision | Choice | Rationale |
|---|---|---|
| History storage | One `ActivityRecord` per event in `WatcherState` | The existing atomic JSON store persists it; no second store. |
| Record identity | Persisted monotonic counter, `a<n>` | Stable across restarts and usable as a CLI argument. |
| Read model | `InboxService` trait over `SnapshotStore` | Status/inbox/read work without credentials or a gateway; the TUI can consume the same contract. |
| Status account merge | `InboxService::view(&CoreConfig)` | The core merges configured accounts with state entries once; no split logic in the CLI. |
| Watch scope | Global `[watch] repositories`; empty = all | One operator-level list; per-account scopes deferred. |
| Rendering | `InboxCli` + `TranscriptRenderer` | Text and human labels stay in the CLI; the TUI renders `InboxView` itself. |
| CLI commands | `status`, `inbox [--all]`, `read <id>|--all` | Three read actions sharing one service. |

Versions: Rust stable via `rust-toolchain.toml`; cucumber-rs from `Cargo.lock`;
no new dependency.

## Architecture impact

### Contracts and Components

| Contract | Consumer use cases | Consumers | Implementing component | Split trigger |
|---|---|---|---|---|
| `SnapshotStore` (load/save watch state) | watch-pull-requests, inspect-watch-status, review-activity-inbox | `PassuffCore`, `Inbox` | `JsonStateStore` | A second persistence format or remote state. |
| `InboxService` (view, mark read) | inspect-watch-status, review-activity-inbox | `InboxCli`, future `passuff-tui` | `Inbox` in `passuff-core` | A second read model with different consistency rules. |
| `InboxCli` (status/inbox/read commands) | inspect-watch-status, review-activity-inbox | `passuff` binary | `InboxCli` in `passuff-cli` | A second CLI surface with different text rules. |

`InboxService`:

```rust
#[async_trait]
pub trait InboxService: Send + Sync {
    async fn view(&self, config: &CoreConfig) -> Result<InboxView, CoreError>;
    async fn mark_read(&self, id: &str) -> Result<ReadOutcome, CoreError>;
    async fn mark_all_read(&self) -> Result<ReadOutcome, CoreError>;
}
```

`ReadOutcome { found: bool, changed: usize, unread: usize }`.

`InboxView`:

```rust
pub struct InboxView {
    pub accounts: Vec<AccountView>,   // configured labels; login/last check from state
    pub scope: Vec<String>,           // empty = all repositories
    pub pull_requests: Vec<PullRequestView>,
    pub activity: Vec<ActivityRecord>, // newest first
    pub unread: usize,
}
```

`Inbox::view` merges `config.accounts` keys with state entries: a configured
account without state has `login = None`, `last_check_at = None`; a state account
not configured is omitted. `scope` is copied from `config.watch.repositories`.
Status and inbox render the same view.

## Data model

### State version 2

```rust
pub struct WatcherState {
    pub version: u32,                    // 2
    pub accounts: BTreeMap<String, AccountState>,
    #[serde(default)] pub activity: Vec<ActivityRecord>,
    #[serde(default)] pub next_activity_id: u64,
}

pub struct ActivityRecord {
    pub id: String,                      // "a<n>"
    pub key: PullRequestKey,
    pub observed_at: String,             // AccountObservation.checked_at
    pub event: ActivityEvent,            // serde
    pub read: bool,
}
```

`ActivityEvent` gains `Serialize`/`Deserialize`. Records append in notification
order and event order; IDs increase within a cycle. After each append the oldest
records are dropped until at most 500 remain.

On load:

- version 1: parse with defaults, preserve accounts and watched pull requests,
  set `version = 2`, recompute
  `next_activity_id = max(parsed counter, max suffixed record number) + 1`, and
  save the rewritten state atomically. A rewrite failure is a state error.
- version 2: normalize terminal `state_reported` as today; recompute
  `next_activity_id = max(parsed counter, maximum record number)`; an append
  then uses `next_activity_id + 1`, so equality is not a collision.
- other versions: quarantine, as today.

A missing state file is an empty version-2 state; status/inbox do not create a
file. `mark_read` on a missing file reports `found = false`; `mark_all_read`
returns `changed = 0` and does not create a file.

### Configuration

```toml
[watch]
repositories = ["acme/api", "acme/web"]   # empty or absent = all
```

`WatchConfig { repositories: Vec<String> }` in `CoreConfig`. Validation: each
entry is `owner/repo`, no whitespace, exactly one slash; duplicates rejected;
matching is ASCII case-insensitive. `watch.allows(&PullRequestKey)`.

Observation applies the filter before the author filter and before
classification. After each observed account, `entry.pull_requests.retain` drops
entries the scope no longer allows, so status cannot show an out-of-scope PR.

## Decisions

- Decides D1: 500 records, oldest dropped. Realized in `record_activity`.
- Decides D2: read state per activity record. Realized in `ActivityRecord.read`.
- Decides D3: global watch scope. Realized in `CoreConfig.watch`; per-account
  scopes are deferred because one operator-level list covers the reported need
  and keeps status single-valued.

Q1 from design-review (split the compound use case) is resolved by the split
above. Q2 (transcript layout) is approved as written in the CLI contract.

## Runtime flow

### Recording

After the account loop, for every notification in the report and every event in
it: increment the counter, append an unread record with the observation time,
and drop beyond 500. The state is saved once. Recording happens before delivery,
so a delivery failure still preserves history; at-most-once delivery is
unchanged. Only accepted notifications are recorded; identity-rejected accounts
produce none.

### Status and inbox

`Inbox::view(config)` loads the state (migrating v1), merges accounts, copies
the scope, reverses activity, and counts unread. `InboxCli::status` renders the
view; `InboxCli::inbox(all)` filters or includes read records.

### Read

`mark_read(id)` / `mark_all_read()` load, mutate, save, and return counts. An
unknown id reports `no activity with id <id>` and exits 2. A save failure is a
state error (exit 3); a missing state yields `found = false` / `changed = 0`.

### Migration write

The version-1 rewrite happens in `JsonStateStore::load` after normalization:
set version 2 and call the store's save. Status, inbox, and read therefore
write only for migration or read; a read-only state filesystem exits 3.

## CLI contract

| Command | Output | Exit |
|---|---|---|
| `passuff status` | accounts, scope, watched PRs, unread/total | 0; 2 config; 3 state |
| `passuff inbox` | unread records newest first | 0; 2 config; 3 state |
| `passuff inbox --all` | unread and read records newest first | 0; 2; 3 |
| `passuff read <id>` | `passuff: 1 activity record read`, remaining unread | 0; 2 unknown/config; 3 state |
| `passuff read --all` | `passuff: <n> activity records read`, remaining unread | 0; 2; 3 |

Text rules:

```
passuff: accounts
  personal @octo-personal last check 2026-11-02T21:00:00+00:00
  work not resolved last check never
passuff: watch scope: all authored pull requests
passuff: watch scope: acme/api, acme/web
passuff: watched pull requests
  acme/api#42 open CI passing Fix the migration
passuff: no watched pull requests
passuff: activity: 2 unread of 5
passuff: inbox
  a12 2026-11-02T21:00:00+00:00 acme/api#42 unread New comment from @alice
passuff: no unread activity
passuff: no activity
passuff: 1 activity record read
passuff: 0 unread remaining
```

Human labels: `open`, `merged`, `closed`; `CI passing`, `CI failing`,
`CI pending`, `CI not started`. No enum names, tokens, bodies, or command text.
`--all` is the only inbox flag. `read` requires exactly one of an id or `--all`;
neither/both is a usage error (exit 2).

## Testing strategy

- Runner: existing `./scripts/verify.sh` / `--e2e`; strict
  `.fail_on_skipped()`; stub pattern `unimplemented!()`; single scenario
  `GIVN_SCENARIO="<title>" GIVN_FEATURES=<change specs> ./scripts/verify.sh`.
- Step files, one per capability, each handling regular and e2e via
  `world.e2e`:
  - `crates/passuff-cli/tests/steps/watch_status.rs` (status + shared scope
    Given);
  - `crates/passuff-cli/tests/steps/activity_inbox.rs` (inbox/read);
  - watch scope assertions reuse the status step; the watch scenario lives in
    `pr_activity_notification.rs` for its Given/When steps.
  - The runner includes both new modules; no duplicate phrases.
- Harness: the regular World uses a temp `JsonStateStore`; `run_status`,
  `run_inbox`, `run_read` build `InboxCli` over the store. E2E steps spawn the
  real binary with `--config` and `--state`; stdout is the transcript.
- Evidence: `e2e_write_evidence` resolves the active change directory
  (`givn/changes/*/specs` parent) and writes `evidence/visual/<slug>/transcript.txt`;
  after archive it falls back to `target/visual`.
- State fixtures: v1 JSON with one watched PR; 500 records; a seeded watched PR
  for the scope-prune scenario; a `SeededFailingStore` that loads the v1 state and
  fails `save`, for the migration write-failure scenario. The no-state inbox
  scenario asserts the state path was not created.
- Unit tests: v1 migration and rewrite, ID collision, cap insertion, missing
  file read semantics, scope validation/case-insensitivity, only accepted
  notifications recorded, delivery failure still records.
- E2E actions: status, inbox, read --all (three distinct consumer actions).

## Interaction coverage

| Use case | Capability | Inventory entry | @e2e scenario | Interface |
|---|---|---|---|---|
| inspect-watch-status | watch-status | show the watch state | Developer sees the watch state | real `passuff status` |
| review-activity-inbox | activity-inbox | list unread activity | Developer lists unread activity | real `passuff inbox` |
| review-activity-inbox | activity-inbox | mark activity read | Developer marks activity read | real `passuff read --all` |
| watch-pull-requests | pr-activity-notification | start continuous watching (existing) | unchanged | real `passuff` |

The watch scope adds no consumer action; it changes which PRs observation sees.

## Visual Design Contract

- Interface: terminal
- Viewports: not applicable; transcripts at 100x30
- Design system: none; text rules above

Persona: `multi-account-developer` (confirmed). Screens: status (accounts,
scope, PRs, counts), inbox (unread/all), read result, empty states. Capture: one
transcript per @e2e scenario under the change evidence path. Motion: none.

## Use-case traceability

| Rule | Decision | Evidence |
|---|---|---|
| S1 | `Inbox::view` merges config accounts + state | status accounts scenarios |
| S2 | scope copied from config | scope scenarios |
| S3 | `PullRequestView` + human labels | PR label scenario |
| S4 | `unread`/total | count scenario |
| S5 | view uses the store only | E2E status without GitHub |
| S6 | missing state = empty view | no-state scenario |
| S7 | error mapping | config/state error tests |
| R1-R2 | newest-first list, `--all` | inbox scenarios |
| R3-R4 | mark one/all | read scenarios |
| R5 | append + cap 500 | cap scenario, unit test |
| R6 | persisted read | restart scenario |
| R7 | v1 migration + rewrite | v1 scenario, unit test |
| R8 | missing state | no-state scenario |
| R9 | classification unchanged | existing no-replay scenarios |
| R39 (watch) | scope filter + prune | scope scenario |

## Risks

- Version-1 rewrite on load can fail on a read-only state filesystem; this is a
  state error (exit 3), not silent data loss.
- `next_activity_id` collisions are prevented by recomputing it from the maximum
  record number on load.
- 500 records enlarge the state file but remain bounded.
- A global scope cannot express per-account exceptions; deferred.
- Status/inbox write only for migration or read; a write failure is visible.

## Arc42 impact and ADR

Chapters 03, 04, 05, 06, 08, 09, 10, 11, 12 are affected. The state-v2
compatibility decision qualifies as a new ADR: it fixes a persisted-data
boundary that future changes must migrate against; alternatives are keeping v1
with defaulted fields, a v2 migration, or a new file. Route:
`ADR-0005-state-version-2-migration`.
