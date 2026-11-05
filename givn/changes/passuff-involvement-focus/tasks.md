# Tasks: passuff-involvement-focus

### Setup: runner, strict mode, and the failing baseline

- [x] Setup: confirm the runner and strict mode.
  - `verify.command` and `verify.e2e_command` are unchanged
    (`./scripts/verify.sh`, `./scripts/verify.sh --e2e`).
  - Strict mode is `.fail_on_skipped()` plus the runner's `total == 0` failure.
  - Step files per capability: `tests/steps/watch_scope.rs`,
    `tests/steps/activity_inbox.rs`, `tests/steps/activity_inbox_tui.rs`,
    `tests/steps/pr_activity_notification.rs`.
  - Proof before implementation:
    `GIVN_FEATURES=$PWD/givn/changes/passuff-involvement-focus/specs GIVN_RESULT_SCOPE=change ./scripts/verify.sh`
    exited 1 with `givn test-result: total 7, passed 1, failed 6, skipped 0`;
    the undefined phrases are `the developer focuses on "..."`,
    `the focus relations are "..."`, `the picker does not show "..."`, and
    `a team review is requested on "..."`.
  - Domain constraints carried from `change-watch-scope` R12–R15,
    `review-activity-inbox` R9, and `pr-activity-notification` R11: the focus
    filters the read model and notifications, never observation; unknown
    relations stay visible; the manual `owner/name` entry is gone.

### Batch: `involvement-focus`

Contracts: `InboxService`, `GitHubGateway`, `WatchScope`. No consumer imports
an implementing component.

- [x] Core: record involvement relations.
  - [x] `Involvement` (authored, assigned, review requested, team review
        requested) with `all()` and `label()`.
  - [x] `DiscoveredPullRequest { node_id, involvements }` replaces
        `AccountObservation.involved_node_ids`.
  - [x] Discovery queries: `assignee:@me` → assigned,
        `user-review-requested:@me` → review requested,
        `team-review-requested-user:@me` → team review requested; merge by
        node id; a failed search warns and continues.
  - [x] `WatchedPullRequest.involvements` with `#[serde(default)]`;
        `PullRequestView.involvements`.
  - [x] Cycle: replace when discovery reports the node, keep otherwise, always
        union `Authored` for the account login.

- [x] Config: persist the focus.
  - [x] `FocusConfig { relations }`, default all four, with `includes`
        (empty relations match every focus).
  - [x] `CoreConfig.focus`; top-level `[focus]` parsing; unknown values exit 2.
  - [x] `ScopeEditor::set_focus` writes `[focus] relations` atomically.

- [x] Gates.
  - [x] Cycle: push a notification only when the focus includes the stored
        relations; activity records are appended regardless.
  - [x] `Inbox::view_from_state`: filter pull requests and activity by the
        focus, recompute `unread`; an unknown pull request key keeps its
        activity.

- [x] Watch-scope picker.
  - [x] Focus section with four rows above the repository list; one cursor over
        both; `space` toggles the focused row; `m` keeps the mode switch.
  - [x] Remove the `Add:` buffer, insert mode, `i`, `enter`-to-add, the
        `type owner/name` hint, and `ScopeEditor::add`.
  - [x] Remove `--scope-add`; add `--focus-set <relations>` with the accepted
        values in the error.
  - [x] Footer: `↑↓ move · space toggle · m mode · esc/q close`.

- [x] Tests and support.
  - [x] World: involvement map keyed by pull request; gateway reports the
        relations; steps for the focus and the team review request.
  - [x] Unit tests: `FocusConfig` default/includes, config parsing, picker
        snapshot, `set_focus` persistence and write failure, discovery tags and
        deduplication, the notification gate, the read-model filter.
  - [x] README: picker keys, removed `--scope-add`, new `--focus-set`, `[focus]`
        table.

- [x] Scenario: `Developer toggles a repository in the watch scope` (modified)
  - [x] **Design constraints:** the e2e scenario keeps its title and `@e2e` tag;
        the manual-entry steps are removed; the toggle, picker, TUI, and
        retention assertions stay.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `Adding a repository by owner/name` (removed)
  - [x] **Design constraints:** `@givn.removed` with one placeholder step; the
        permission and format paths used by `select` remain.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `The picker shows the focus relations`
  - [x] **Design constraints:** the snapshot shows `Focus`, `[x] authored`,
        `[x] assigned`, `[x] review requested`, `[x] team review`; it does not
        show `Add:` or `owner/name`.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `Narrowing the focus persists the relation set`
  - [x] **Design constraints:** `set_focus(["authored","assigned"])` writes the
        `[focus] relations` array; the configuration keeps the order given.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `Developer narrows the focus in the watch scope` (e2e)
  - [x] **Design constraints:** spawn `passuff-tui --snapshot --picker
        --focus-set authored,assigned`; assert the config and the picker frame
        with `[x] authored` and `[ ] team review`; exit 0.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `Inbox hides activity outside the focus`
  - [x] **Design constraints:** a team-review pull request and an authored pull
        request are both watched; the focus is `authored`; the team comment is
        absent from `inbox --all`, the authored comment is unread, and status
        reports `1 unread of 1`.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `The watched table hides a pull request outside the focus`
  - [x] **Design constraints:** the TUI frame shows `acme/api#42` and hides
        `acme/web#7` with focus `authored`.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

- [x] Scenario: `A pull request outside the focus produces no notification`
  - [x] **Design constraints:** the cycle records the comment as unread but
        delivers no notification; after focusing on
        `authored,team-review-requested`, `inbox --all` shows it.
  - [x] COMMIT: `8359900` -- Batch: `involvement-focus`

### Closeout

- [x] Full suite: `./scripts/verify.sh` and `./scripts/verify.sh --e2e` exit 0
      with no failed or skipped scenarios.
- [x] Delta step phrases are bound in the permanent step files; the pre-merge corpus keeps the `enters` phrase bound through the selecting shim until archive removes its scenario.
