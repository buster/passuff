# Tasks: passuff-stacked-inbox

Behavior batch: stack model, projection, activity grouping, degradation, and
the end-to-end proof.

Legend: an unchecked box means pending; a checked box is done only when the commit is filled in.

---

## Strict-mode proof

- [x] **Proof-of-strictness**: the delta runner rejects an undefined step with `failed > 0`.

      ```
      GIVN_FEATURES=/tmp/stacked-strict.feature ./scripts/verify.sh
      FAILED: Strict probe :: Undefined stacked inbox step fails
        undefined And  -> an undefined stacked inbox step
          Step doesn't match any function
      givn test-result: total 1, passed 0, failed 1, skipped 0
      cargo exit status: 1
      ```

---

### Batch: `stack-grouping`

- [x] Setup: `PullRequestStack` in the domain, the GraphQL `stack`/`stackEntry`
      mapping with the retry, the optional state field, `PullRequestView`, the
      stack fixtures, and the step file.
      Files: `domain.rs`, `github.rs`, `cycle.rs`, `inbox.rs`, `lib.rs`,
      `tui.rs`, `tests/support/mod.rs`, `tests/support/e2e.rs`,
      `tests/steps/stacked_inbox.rs`, `tests/features_runner.rs`.
      COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `The stack orders members by position`
  - [x] **Design constraints:** R2. Group by `(repository, stack.number)`;
        members sort by `position` ascending. `#43` has position 1 and `#42`
        position 2, so key order must not decide. Files: `tui.rs`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `An unwatched stack member leaves a position gap`
  - [x] **Design constraints:** R4. Show only watched members, keep the stack
        `size`, and show positions 1 and 3 with no invented position 2.
        Files: `tui.rs`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `A pull request without stack metadata stays standalone`
  - [x] **Design constraints:** R1/R6. No summary line, blank stack cells,
        no `base main` for a standalone row. Files: `tui.rs`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `A merged stack member remains in the group`
  - [x] **Design constraints:** Extension 3b. A merged member keeps its
        position, state label, and group membership. Files: `tui.rs`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `The stack unread count aggregates its members`
  - [x] **Design constraints:** R5. The group count is the number of unread
        records whose key belongs to the group; the standalone `#40` unread
        record stays outside. `2 unread of 3` and `1 unread in stack`.
        Files: `tui.rs`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `Stack membership updates after a change`
  - [x] **Design constraints:** R1/R3. The next observation replaces the
        recorded membership; `stack 8`/`base release` replace `stack 7`/`main`.
        Files: `cycle.rs`, `domain.rs`, `tui.rs`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Adapter retry and read-only regression tests.
  - [x] **Design constraints:** D1/R7/R8. A wiremock first response names
        `stack` and fails; the retry removes both stack field lines from
        `MAIN_QUERY` and `NODES_QUERY` and returns an observation with
        `stack: None`. `queries_are_read_only` asserts no `mutation` in either
        query. Files: `github.rs`.
  - [x] Unit tests: `maps_stack_membership`,
        `incomplete_stack_membership_is_standalone`,
        `rejects_stack_fields_and_retries_without_them`,
        `nodes_stack_error_triggers_retry`, `queries_are_read_only`;
        `tui.rs` projection tests.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] E2e setup: the mock GitHub response branch for `stack`/`stackEntry`
      exists in `tests/support/e2e.rs`; the delta e2e scope command is
      `GIVN_FEATURES=$PWD/givn/changes/passuff-stacked-inbox/specs ./scripts/verify.sh --e2e`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

- [x] Scenario: `Developer reviews a stacked inbox` (end-to-end)
  - [x] **Design constraints:** T6/R3/R5. Spawn the real
        `CARGO_BIN_EXE_passuff-tui --snapshot` against the mock GitHub server;
        assert `stack 7`, `base main`, `size 2`, `position 1`, `position 2`,
        the activity summary, and exit 0. Transcript evidence under
        `evidence/visual/developer-reviews-a-stacked-inbox/transcript.txt`.
  - [x] COMMIT: `57c4652` -- Batch: `stack-grouping`

---

**Done when:** every box checked with the commit, the regular and end-to-end
suites exit 0, and the e2e count (1) is less than the full delta count (7).

Evidence:
- Delta: 7/7 passed.
- Delta e2e: 1/1 passed.
- Permanent regular: 123/123 passed.
- Permanent e2e: 8/8 passed.
- Unit: `passuff-core` 28 + `passuff-cli` 20.
