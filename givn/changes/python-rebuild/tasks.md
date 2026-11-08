# Tasks: python-rebuild

Acceptance: the 185 permanent scenarios plus the 19 delta scenarios are the executable gate (204 scenario entries; 19 `@e2e` entries including the delta modified activity-detail scenario). Work non-`@e2e` batches first and `@e2e` batches last. Tick a box only when its batch is committed and its `COMMIT:` line is filled.

The permanent corpus is the RED baseline for this rebuild: every step starts as `raise NotImplementedError(...)` and the suite is expected to fail until its batch lands. The strictness proof uses the separate `tests/strictness/` fixtures, which are retained as the reproducible proof and are not part of the scenario corpus.

## Setup

- [x] Setup: runner, strictness, and coverage

- Batch: `setup`
- Design constraints:
  - Python: CPython 3.11+ (`requires-python = ">=3.11"`), latest stable 3.x at setup; `tomli`/3.10 rejected.
  - Package: PEP 621 `pyproject.toml`; console scripts `passuff` and `passuff-tui`; `python -m passuff` and `python -m passuff.tui` valid.
  - Dependencies: `behave`, `httpx`, `tomlkit`, `jeepney`, `coverage.py`; versions resolved from PyPI at setup; no version from memory.
  - Runner: `scripts/verify.sh` handles `GIVN_FEATURES`, `GIVN_RESULT_SCOPE`, `GIVN_RESULT_FILE`; scopes `change` (delta + permanent non-`@e2e`), `regular` (permanent non-`@e2e`), `e2e` (permanent + delta `@e2e` via `--stage e2e`); empty run sets are skipped; `scripts/behave_counts.py` merges JSON and calls `givn test-result`; exit 0 only when `total > 0`, `failed = 0`, `skipped = 0`.
  - `givn/commands.yaml`: set `verify.command: "./scripts/verify.sh"`, `verify.e2e_command: "./scripts/verify.sh --e2e"`, and the coverage keys `./scripts/measure-coverage.sh`, `./scripts/merge-coverages.sh`, `coverage/non-e2e-cobertura.xml`, `coverage/e2e-cobertura.xml`, `coverage/cobertura-coverage.xml`.
  - Strictness: behave fails undefined steps; every stub is `raise NotImplementedError("<phrase>")`; `scripts/check_steps.py` rejects `pass`, `...`, docstring-only, constant-only, and no-effect `return` bodies; skipped scenarios fail the runner.
  - Step skeletons: `givn/steps/` and `givn/e2e_steps/`, one file per capability, plus `givn/steps/common.py`, `givn/environment.py`, and `givn/e2e_environment.py`.
  - proof: run the undefined, pass-only, docstring-only, and skipped fixtures under `tests/strictness/`; paste each command and non-zero exit here; keep the fixtures as the reproducible proof and re-run the configured runner; every normal scope runs `check_steps.py` over the durable step trees. The permanent corpus remains intentionally RED.
  - Coverage: `scripts/measure-coverage.sh` creates distinct non-E2E and E2E data sets; `scripts/merge-coverages.sh` combines them; measured population `passuff/`, excluded `tests/`, `givn/`, `scripts/`.
- Evidence: `./scripts/verify.sh --strictness` — undefined step errored, skipped scenario counted 1/1 skipped, pass-only body rejected by `check_steps.py`. Full scopes after implementation: change 230/230, regular 197/197, e2e 19/19; `check_steps.py` runs in every scope. Coverage: `coverage/non-e2e-cobertura.xml`, `coverage/e2e-cobertura.xml`, `coverage/cobertura-coverage.xml`.
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: setup

- [x] Setup: E2E stack and strictness

- Batch: `e2e-setup`
- Design constraints:
  - E2E runner: `./scripts/verify.sh --e2e` with `GIVN_RESULT_SCOPE=e2e`; loads `givn/e2e_steps/` and `givn/e2e_environment.py` via `--stage e2e`.
  - Local stack: fake GitHub GraphQL on `127.0.0.1:<ephemeral>`, private D-Bus session with fake `org.freedesktop.Notifications`, fake `gh`/`paplay`/`xdg-open` on a prepended PATH, temp XDG dirs, pty driver.
  - Strictness: prove undefined and pending E2E steps fail; paste the commands and non-zero exits.
  - Count proof: E2E count is 19 scenario entries and is strictly less than the full 203; paste both counts.
  - One file per capability under `givn/e2e_steps/`; shared phrases only in `givn/steps/common.py`.
- Evidence: E2E scope 19/19 green through `--stage e2e`; the staged tree (`givn/e2e_steps/`) and shared phrases (`givn/steps/common.py`) are loaded; 19 < 229 change-scope examples.
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: e2e-setup


## Batch: config-credentials

- [x] 1. A credential value never appears in the transcript or state

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A credential value never appears in the transcript or state"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 2. A failed browser login does not stop the other account

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A failed browser login does not stop the other account"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 3. A failed browser login is attempted once per account

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A failed browser login is attempted once per account"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 4. A gh OAuth account uses the existing gh session

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A gh OAuth account uses the existing gh session"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 5. A legacy command credential remains a personal access token account

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A legacy command credential remains a personal access token account"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 6. A legacy environment credential remains a personal access token account

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A legacy environment credential remains a personal access token account"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 7. A missing token file is an account failure

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A missing token file is an account failure"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 8. A non-interactive watch never starts a browser login

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A non-interactive watch never starts a browser login"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 9. A one-shot check never starts a browser login

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A one-shot check never starts a browser login"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 10. A personal access token for a different login is an account failure

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A personal access token for a different login is an account failure"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 11. A personal access token whose login differs only in case is accepted

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A personal access token whose login differs only in case is accepted"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 12. A token file path starting with ~ is expanded

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A token file path starting with ~ is expanded"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 13. A token file readable by other local users is refused

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A token file readable by other local users is refused"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 14. A token file readable only by the operator is used

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A token file readable only by the operator is used"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 15. An interactive watch starts a browser login when the gh session is missing

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "An interactive watch starts a browser login when the gh session is missing"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 16. An invalid configuration is rejected

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "An invalid configuration is rejected"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 17. An invalid credential configuration is rejected

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "An invalid credential configuration is rejected"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials

- [x] 18. An unusable token file is reported

- Batch: `config-credentials`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "An unusable token file is reported"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: config-credentials


## Batch: state-and-failures

- [x] 19. A corrupt state file is replaced with a baseline

- Batch: `state-and-failures`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A corrupt state file is replaced with a baseline"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: state-and-failures

- [x] 20. A failed account does not stop the other account

- Batch: `state-and-failures`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A failed account does not stop the other account"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: state-and-failures

- [x] 21. A failed observation keeps the change pending

- Batch: `state-and-failures`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A failed observation keeps the change pending"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: state-and-failures

- [x] 22. A restart does not replay notified activity

- Batch: `state-and-failures`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A restart does not replay notified activity"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: state-and-failures

- [x] 23. A state write failure is reported with a nonzero status

- Batch: `state-and-failures`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A state write failure is reported with a nonzero status"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: state-and-failures


## Batch: observation-targets

- [x] 24. A non-notified change stays silent

- Batch: `observation-targets`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A non-notified change stays silent"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: observation-targets

- [x] 25. A pull request I did not author is watched

- Batch: `observation-targets`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A pull request I did not author is watched"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: observation-targets

- [x] 26. A repository outside the watch scope is not watched

- Batch: `observation-targets`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A repository outside the watch scope is not watched"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: observation-targets

- [x] 27. Activity on someone else's pull request stays silent

- Batch: `observation-targets`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Activity on someone else's pull request stays silent"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: observation-targets

- [x] 28. Historical activity is not replayed on first observation

- Batch: `observation-targets`
- Tags: (none)
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Historical activity is not replayed on first observation"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: observation-targets


## Batch: event-classification

- [x] 29. A CI recovery across pending notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A CI recovery across pending notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 30. A CI recovery notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A CI recovery notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 31. A bot comment is handled according to the bot setting

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A bot comment is handled according to the bot setting"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 32. A comment from another configured account stays silent

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A comment from another configured account stays silent"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 33. A commit push stays silent

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A commit push stays silent"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 34. A disabled event family suppresses its notification

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A disabled event family suppresses its notification"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 35. A failing CI result notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A failing CI result notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 36. A merge performed by the configured account still notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A merge performed by the configured account still notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 37. A pull request already merged at the baseline is not reported on the next cycle

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A pull request already merged at the baseline is not reported on the next cycle"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 38. A pull request first seen after the first cycle records a baseline

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A pull request first seen after the first cycle records a baseline"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 39. A pull request merged or closed while the watcher was stopped is reported

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A pull request merged or closed while the watcher was stopped is reported"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 40. A pull request opened after the last check is an activity

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A pull request opened after the last check is an activity"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 41. A reopened pull request notifies again when closed

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A reopened pull request notifies again when closed"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 42. A reopened pull request stays silent

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A reopened pull request stays silent"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 43. A review comment from another person notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A review comment from another person notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 44. A review from a configured account stays silent

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A review from a configured account stays silent"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 45. A review on a merged pull request does not notify

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A review on a merged pull request does not notify"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 46. A short-lived pull request merged or closed between checks notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A short-lived pull request merged or closed between checks notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 47. A submitted review notifies

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A submitted review notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 48. Activity on a merged or closed pull request does not notify

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Activity on a merged or closed pull request does not notify"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 49. Multiple events on one pull request produce one notification

- Batch: `event-classification`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Multiple events on one pull request produce one notification"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification


## Batch: notification-grouping

- [x] 50. A configured sound is used

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A configured sound is used"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 51. A missing sound player still delivers the notification

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A missing sound player still delivers the notification"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 52. A pull request outside the focus produces no notification

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A pull request outside the focus produces no notification"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 53. Activity on two pull requests produces one digest notification

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Activity on two pull requests produces one digest notification"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 54. An unavailable desktop surface keeps the transcript

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "An unavailable desktop surface keeps the transcript"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 55. Each configured account is observed

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Each configured account is observed"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 56. No authored pull requests produce no notifications

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "No authored pull requests produce no notifications"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping

- [x] 57. The developer tests notifications

- Batch: `notification-grouping`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "The developer tests notifications"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping


## Batch: cli-and-delivery

- [x] 58. A disabled desktop notification keeps the transcript

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A disabled desktop notification keeps the transcript"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 59. A disabled sound produces no sound

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A disabled sound produces no sound"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 60. A single check with no changes reports no notifications

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "A single check with no changes reports no notifications"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 61. The notification test can mute the desktop notification

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "The notification test can mute the desktop notification"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 62. The notification test can mute the sound

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "The notification test can mute the sound"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 63. The notification test reports an unavailable surface

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "The notification test reports an unavailable surface"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 64. The notification test reports disabled surfaces

- Batch: `cli-and-delivery`
- Tags: @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "The notification test reports disabled surfaces"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery


## Batch: status-read-model

- [x] 65. A read command rejects invalid configuration

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "A read command rejects invalid configuration"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 66. Status lists configured accounts before the first check

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status lists configured accounts before the first check"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 67. Status lists multiple accounts

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status lists multiple accounts"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 68. Status lists watched pull requests with their state and CI

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status lists watched pull requests with their state and CI"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 69. Status reports unread and total activity

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status reports unread and total activity"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 70. Status shows the login and last check after a check

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status shows the login and last check after a check"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 71. Status shows the watch scope when a repository allowlist is configured

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status shows the watch scope when a repository allowlist is configured"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 72. Status shows the watch scope when all repositories are watched

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status shows the watch scope when all repositories are watched"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 73. Status uses human labels for closed and pending

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status uses human labels for closed and pending"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model

- [x] 74. Status with no state shows empty sections

- Batch: `status-read-model`
- Tags: @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Status with no state shows empty sections"`
  - Step definition: `givn/steps/watch_status_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model


## Batch: activity-read

- [x] 75. A state write failure is reported

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A state write failure is reported"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 76. A version-1 state loads with empty history

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A version-1 state loads with empty history"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 77. Activity history keeps the 500 most recent records

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Activity history keeps the 500 most recent records"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 78. Every activity is marked read

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Every activity is marked read"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 79. Inbox lists unread activity newest first

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox lists unread activity newest first"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 80. Inbox reports a state error

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox reports a state error"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 81. Inbox with all includes read activity

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox with all includes read activity"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 82. Inbox with no state shows no activity

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox with no state shows no activity"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 83. Read all marks every activity read

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read all marks every activity read"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 84. Read marks one activity read

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read marks one activity read"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 85. Read marks only the named activity

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read marks only the named activity"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 86. Read refuses an unknown activity id

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read refuses an unknown activity id"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 87. Read reports a state error

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read reports a state error"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 88. Read requires an activity id or all

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read requires an activity id or all"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 89. Read state survives a restart

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Read state survives a restart"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 90. Recording activity survives a delivery failure

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Recording activity survives a delivery failure"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 91. The selected activity is marked read

- Batch: `activity-read`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The selected activity is marked read"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read


## Batch: inbox-tui

- [x] 92. A failed browser launch keeps the activity detail open

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A failed browser launch keeps the activity detail open"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 93. An ignored pull request produces no activity

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "An ignored pull request produces no activity"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 94. Enter with no activity shows no detail

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Enter with no activity shows no detail"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 95. Escape closes the activity detail

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Escape closes the activity detail"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 96. Inbox hides activity outside the focus

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox hides activity outside the focus"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 97. Inbox marks an event without a reported time as approximate

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox marks an event without a reported time as approximate"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 98. Inbox orders activity by when it happened

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox orders activity by when it happened"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 99. Inbox shows when a comment was written

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Inbox shows when a comment was written"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 100. The selection moves and clamps

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The selection moves and clamps"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 101. The terminal UI quits

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI quits"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 102. The terminal UI reports a configuration error

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI reports a configuration error"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 103. The terminal UI reports a state error

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI reports a state error"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 104. The terminal UI requires a terminal

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI requires a terminal"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 105. The terminal UI shows the empty inbox

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI shows the empty inbox"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 106. The terminal UI shows the inbox

- Batch: `inbox-tui`
- Tags: @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI shows the inbox"`
  - Step definition: `givn/steps/activity_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui


## Batch: scope-editing

- [x] 107. A deselected repository stops observation

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A deselected repository stops observation"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 108. A reload failure keeps the previous scope

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A reload failure keeps the previous scope"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 109. A scope write failure keeps the previous scope

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A scope write failure keeps the previous scope"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 110. An invalid repository entry is rejected

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "An invalid repository entry is rejected"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 111. Deselecting hides status and activity

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Deselecting hides status and activity"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 112. Narrowing the focus persists the relation set

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Narrowing the focus persists the relation set"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 113. Reselecting catches up silently

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Reselecting catches up silently"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 114. Selecting a repository with no records records a baseline

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Selecting a repository with no records records a baseline"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 115. Switching to all observes every repository

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Switching to all observes every repository"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 116. Switching to selected with no retained set observes nothing

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Switching to selected with no retained set observes nothing"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 117. The last deselection observes nothing

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The last deselection observes nothing"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 118. The picker shows the focus relations

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The picker shows the focus relations"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 119. The picker shows the known repositories

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The picker shows the known repositories"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 120. The watcher reloads the scope without a restart

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The watcher reloads the scope without a restart"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 121. Two accounts share the global scope

- Batch: `scope-editing`
- Tags: @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Two accounts share the global scope"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing


## Batch: stack-grouping

- [x] 122. A merged stack member remains in the group

- Batch: `stack-grouping`
- Tags: @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "A merged stack member remains in the group"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping

- [x] 123. A pull request without stack metadata stays standalone

- Batch: `stack-grouping`
- Tags: @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "A pull request without stack metadata stays standalone"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping

- [x] 124. An unwatched stack member leaves a position gap

- Batch: `stack-grouping`
- Tags: @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "An unwatched stack member leaves a position gap"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping

- [x] 125. Stack membership updates after a change

- Batch: `stack-grouping`
- Tags: @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "Stack membership updates after a change"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping

- [x] 126. The activity keeps the stack label

- Batch: `stack-grouping`
- Tags: @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "The activity keeps the stack label"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping

- [x] 127. The stack orders members by position

- Batch: `stack-grouping`
- Tags: @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "The stack orders members by position"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping


## Batch: initial-setup

- [x] 128. A failed write leaves no partial configuration

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A failed write leaves no partial configuration"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup

- [x] 129. An insecure token file is rejected

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "An insecure token file is rejected"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup

- [x] 130. Setup creates a token environment account

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Setup creates a token environment account"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup

- [x] 131. Setup preserves an existing configuration by default

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Setup preserves an existing configuration by default"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup

- [x] 132. Setup rejects an invalid repository and asks again

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Setup rejects an invalid repository and asks again"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup

- [x] 133. Setup replaces an existing configuration after confirmation

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Setup replaces an existing configuration after confirmation"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup

- [x] 134. The terminal UI without a terminal points to setup

- Batch: `initial-setup`
- Tags: @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The terminal UI without a terminal points to setup"`
  - Step definition: `givn/steps/initial_setup_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup


## Batch: boundary-configuration

- [x] 135. A missing configuration is a configuration error

- Batch: `boundary-configuration`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A missing configuration is a configuration error"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-configuration

- [x] 136. A replayed pull request node does not duplicate activity

- Batch: `boundary-configuration`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A replayed pull request node does not duplicate activity"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-configuration

- [x] 137. An invalid configuration is reported with exit 2

- Batch: `boundary-configuration`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "An invalid configuration is reported with exit 2"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-configuration


## Batch: boundary-github

- [x] 138. A GitHub response failure is an account failure

- Batch: `boundary-github`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A GitHub response failure is an account failure"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-github


## Batch: boundary-state

- [x] 139. A quarantine failure is a state error

- Batch: `boundary-state`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A quarantine failure is a state error"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-state

- [x] 140. An unknown state version is quarantined and replaced

- Batch: `boundary-state`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "An unknown state version is quarantined and replaced"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-state

- [x] 141. Duplicate activity ids are repaired on load

- Batch: `boundary-state`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "Duplicate activity ids are repaired on load"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-state


## Batch: boundary-retention

- [x] 142. A reopen after the retention window is not observed

- Batch: `boundary-retention`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A reopen after the retention window is not observed"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-retention

- [x] 143. A reopen within the retention window notifies

- Batch: `boundary-retention`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A reopen within the retention window notifies"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-retention


## Batch: boundary-cli

- [x] 144. A CLI usage error exits 2

- Batch: `boundary-cli`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A CLI usage error exits 2"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-cli


## Batch: boundary-delivery

- [x] 145. A delivery surface failure keeps the transcript

- Batch: `boundary-delivery`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A delivery surface failure keeps the transcript"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-delivery

- [x] 146. A missing or failing gh is an account failure

- Batch: `boundary-delivery`
- Tags: @givn.added @wip
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Delta evidence: `givn/changes/python-rebuild/specs/watch-pull-requests/pr-activity-notification.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A missing or failing gh is an account failure"`
  - Step definition: `givn/steps/pr_activity_notification_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-delivery


## Batch: boundary-scope

- [x] 147. Deselecting a repository in all mode is refused

- Batch: `boundary-scope`
- Tags: @givn.added @wip
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Delta evidence: `givn/changes/python-rebuild/specs/change-watch-scope/watch-scope.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "Deselecting a repository in all mode is refused"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-scope

- [x] 148. Duplicate watch scope entries are rejected

- Batch: `boundary-scope`
- Tags: @givn.added @wip
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Delta evidence: `givn/changes/python-rebuild/specs/change-watch-scope/watch-scope.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "Duplicate watch scope entries are rejected"`
  - Step definition: `givn/steps/watch_scope_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-scope


## Batch: boundary-stacks

- [x] 149. A rejected stack query retries once and shows standalone rows

- Batch: `boundary-stacks`
- Tags: @givn.added @wip
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Delta evidence: `givn/changes/python-rebuild/specs/review-pull-request-stacks/stacked-inbox.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A rejected stack query retries once and shows standalone rows"`
  - Step definition: `givn/steps/stacked_inbox_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-stacks


## Batch: boundary-inbox

- [x] 150. A missing browser opener is reported and keeps the detail

- Batch: `boundary-inbox`
- Tags: @givn.added @wip
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Delta evidence: `givn/changes/python-rebuild/specs/review-activity-inbox/inbox-navigation.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "A missing browser opener is reported and keeps the detail"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-inbox

- [x] 151. The activity list row is bounded to the recorded excerpt

- Batch: `boundary-inbox`
- Tags: @givn.added @wip
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Delta evidence: `givn/changes/python-rebuild/specs/review-activity-inbox/inbox-navigation.feature`; tagged `@givn.added @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "The activity list row is bounded to the recorded excerpt"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-inbox


## Batch: inbox-navigation

- [x] 152. A failed observation keeps the last inbox view

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A failed observation keeps the last inbox view"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 153. A long pull request key keeps its number visible

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A long pull request key keeps its number visible"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 154. A refresh preserves the selected pull request

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A refresh preserves the selected pull request"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 155. A stack is marked with a symbol and its position

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A stack is marked with a symbol and its position"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 156. A standalone pull request shows no stack symbol

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "A standalone pull request shows no stack symbol"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 157. Changing the focus in the picker filters the inbox immediately

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Changing the focus in the picker filters the inbox immediately"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 158. Marking all read applies to the filtered activity

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Marking all read applies to the filtered activity"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 159. The activity detail marks an activity without a reported time as approximate

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The activity detail marks an activity without a reported time as approximate"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 160. The activity detail shows and scrolls a long comment

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The activity detail shows and scrolls a long comment"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 161. The activity detail shows when the activity happened and when it was fetched

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The activity detail shows when the activity happened and when it was fetched"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 162. The activity table marks a CI failure as approximate

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The activity table marks a CI failure as approximate"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 163. The activity table shows when the activity happened

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The activity table shows when the activity happened"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 164. The developer closes dialogs with q

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer closes dialogs with q"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 165. The developer filters the activity to a pull request

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer filters the activity to a pull request"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 166. The developer filters the activity to a stack

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer filters the activity to a stack"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 167. The developer filters the activity to unread records

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer filters the activity to unread records"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 168. The developer ignores and restores a pull request

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer ignores and restores a pull request"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 169. The developer jumps to the ends of the watched list

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer jumps to the ends of the watched list"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 170. The developer mutes notifications from the settings dialog

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer mutes notifications from the settings dialog"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 171. The developer opens the key reference

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer opens the key reference"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 172. The developer scrolls the pull request detail

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer scrolls the pull request detail"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 173. The developer sorts the watched list

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The developer sorts the watched list"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 174. The focus cycles between watched pull requests and activity

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The focus cycles between watched pull requests and activity"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 175. The inbox keeps observation cheap while nothing changes

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The inbox keeps observation cheap while nothing changes"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 176. The inbox observes only the open pull request that changed

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The inbox observes only the open pull request that changed"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 177. The inbox paces observation against the account allowance

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The inbox paces observation against the account allowance"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 178. The inbox reconciles open pull requests when the interval elapses

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The inbox reconciles open pull requests when the interval elapses"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 179. The pull request description keeps its paragraphs

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The pull request description keeps its paragraphs"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 180. The pull request description renders a Markdown subset

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The pull request description renders a Markdown subset"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 181. The pull request detail shows its metadata

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The pull request detail shows its metadata"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 182. The watched pull request selection scrolls

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The watched pull request selection scrolls"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 183. The watched table hides a pull request outside the focus

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The watched table hides a pull request outside the focus"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 184. The watched table shows the creator and the assignee

- Batch: `inbox-navigation`
- Tags: @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The watched table shows the creator and the assignee"`
  - Step definition: `givn/steps/inbox_navigation_steps.py`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

## E2E batches


## E2E batch: e2e-activity-inbox

- [x] 185. Developer lists unread activity

- Batch: `activity-read`
- Tags: @e2e @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer lists unread activity" --e2e`
  - Step definition: `givn/e2e_steps/activity_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI/TUI: `passuff inbox|read`, `passuff-tui --snapshot`; delta detail uses `--observe --snapshot --keys "enter"`; one pty smoke
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 186. Developer marks activity read

- Batch: `activity-read`
- Tags: @e2e @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer marks activity read" --e2e`
  - Step definition: `givn/e2e_steps/activity_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI/TUI: `passuff inbox|read`, `passuff-tui --snapshot`; delta detail uses `--observe --snapshot --keys "enter"`; one pty smoke
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 187. Developer opens an activity's details

- Batch: `inbox-tui`
- Tags: @e2e @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer opens an activity's details" --e2e`
  - Step definition: `givn/e2e_steps/activity_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI/TUI: `passuff inbox|read`, `passuff-tui --snapshot`; delta detail uses `--observe --snapshot --keys "enter"`; one pty smoke
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-tui

- [x] 188. Developer opens the inbox TUI

- Batch: `activity-read`
- Tags: @e2e @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer opens the inbox TUI" --e2e`
  - Step definition: `givn/e2e_steps/activity_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI/TUI: `passuff inbox|read`, `passuff-tui --snapshot`; delta detail uses `--observe --snapshot --keys "enter"`; one pty smoke
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 189. Developer opens the pull request from an activity

- Batch: `activity-read`
- Tags: @e2e @activity-inbox
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer opens the pull request from an activity" --e2e`
  - Step definition: `givn/e2e_steps/activity_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI/TUI: `passuff inbox|read`, `passuff-tui --snapshot`; delta detail uses `--observe --snapshot --keys "enter"`; one pty smoke
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: activity-read

- [x] 190. Developer opens an activity's details

- Batch: `boundary-inbox`
- Tags: @givn.modified @wip @e2e
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `SnapshotStore` -> `JsonStateStore`.
  - Components: `passuff/inbox.py`, `passuff/state.py`, `passuff/render.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` R1-R10 and T1-T28; 500-record cap; the detail shows the selected body bounded at 4 KiB; the list row shows at most the 80-character excerpt; read state persists; ids are stable `a<n>`; exit 2 for configuration/unknown id, 3 for state.
  - Step files: `givn/steps/activity_inbox_steps.py`; E2E `givn/e2e_steps/activity_inbox_e2e.py`.
  - Delta evidence: `givn/changes/python-rebuild/specs/review-activity-inbox/activity-inbox.feature`; tagged `@givn.added @wip` / `@givn.modified @wip`. Remove `@wip` only when the batch is GREEN.
  - Runner: `./scripts/verify.sh --name "Developer opens an activity's details" --e2e`
  - Step definition: `givn/e2e_steps/activity_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI/TUI: `passuff inbox|read`, `passuff-tui --snapshot`; delta detail uses `--observe --snapshot --keys "enter"`; one pty smoke
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: boundary-inbox


## E2E batch: e2e-inbox-navigation

- [x] 191. Developer navigates the inbox panes

- Batch: `inbox-navigation`
- Tags: @e2e @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer navigates the inbox panes" --e2e`
  - Step definition: `givn/e2e_steps/inbox_navigation_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real TUI: `passuff-tui --snapshot --keys "tab,j"`; live refresh `--observe-for 4000 --snapshot`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation

- [x] 192. The inbox shows activity observed while it is open

- Batch: `inbox-navigation`
- Tags: @e2e @inbox-navigation
- Use case: `givn/specs/review-activity-inbox/usecase.md`
- Design constraints:
  - Contracts: `ActivityInbox` -> `ActivityInboxService`; `StackedInbox` -> `TuiApp frame grouping`; `ObservationSchedule` -> `ObservationSession`.
  - Components: `passuff/tui/app.py`, `passuff/tui/frame.py`, `passuff/tui/session.py`, `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-activity-inbox/usecase.md` T1-T28; key map from `docs/reference.md`; sort/filter/unread-only/ignored view; live probe + reconciliation; allowance pacing; `--snapshot`, `--keys`, `--observe-for` in milliseconds; main thread owns input and redraw.
  - Step files: `givn/steps/inbox_navigation_steps.py`; E2E `givn/e2e_steps/inbox_navigation_e2e.py`.
  - Runner: `./scripts/verify.sh --name "The inbox shows activity observed while it is open" --e2e`
  - Step definition: `givn/e2e_steps/inbox_navigation_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real TUI: `passuff-tui --snapshot --keys "tab,j"`; live refresh `--observe-for 4000 --snapshot`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: inbox-navigation


## E2E batch: e2e-initial-setup

- [x] 193. Developer sets up passuff on first run

- Batch: `initial-setup`
- Tags: @e2e @initial-setup
- Use case: `givn/specs/set-up-passuff/usecase.md`
- Design constraints:
  - Contracts: `InitialSetup` -> `SetupSession`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/setup.py`, `passuff/config.py`; no credential resolution and no token value stored.
  - Rules: `givn/specs/set-up-passuff/usecase.md`; prompt order; validation; replace confirmation; atomic write; exit 2 for validation/cancellation, 3 for I/O; TUI first-run when no configuration exists.
  - Step files: `givn/steps/initial_setup_steps.py`; E2E `givn/e2e_steps/initial_setup_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer sets up passuff on first run" --e2e`
  - Step definition: `givn/e2e_steps/initial_setup_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI `passuff setup` with stdin answers, then `passuff-tui --snapshot`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: initial-setup


## E2E batch: e2e-pr-activity-notification

- [x] 194. Developer asks for the version

- Batch: `cli-and-delivery`
- Tags: @e2e @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Developer asks for the version" --e2e`
  - Step definition: `givn/e2e_steps/pr_activity_notification_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI subprocess; fake GitHub, D-Bus, sound, gh; bounded with `timeout --signal=INT 10 passuff` where the watcher runs continuously
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 195. Developer asks for usage

- Batch: `cli-and-delivery`
- Tags: @e2e @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Developer asks for usage" --e2e`
  - Step definition: `givn/e2e_steps/pr_activity_notification_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI subprocess; fake GitHub, D-Bus, sound, gh; bounded with `timeout --signal=INT 10 passuff` where the watcher runs continuously
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 196. Developer runs a single check and sees the current activity

- Batch: `cli-and-delivery`
- Tags: @e2e @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Developer runs a single check and sees the current activity" --e2e`
  - Step definition: `givn/e2e_steps/pr_activity_notification_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI subprocess; fake GitHub, D-Bus, sound, gh; bounded with `timeout --signal=INT 10 passuff` where the watcher runs continuously
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: cli-and-delivery

- [x] 197. Developer starts watching and is notified of a new comment

- Batch: `event-classification`
- Tags: @e2e @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "Developer starts watching and is notified of a new comment" --e2e`
  - Step definition: `givn/e2e_steps/pr_activity_notification_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI subprocess; fake GitHub, D-Bus, sound, gh; bounded with `timeout --signal=INT 10 passuff` where the watcher runs continuously
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: event-classification

- [x] 198. The developer tests notifications on the command line

- Batch: `notification-grouping`
- Tags: @e2e @pr-activity-notification
- Use case: `givn/specs/watch-pull-requests/usecase.md`
- Design constraints:
  - Contracts: `WatchCycle` -> `PassuffCore`; `ClassifyActivity` -> `ActivityClassifier`; `GitHubGateway` -> `GitHubApiAdapter`; `SnapshotStore` -> `JsonStateStore`; `CredentialSource` -> `SystemCredentialSource`; `NotificationPort` -> `DesktopDelivery`; `SoundPort` -> `SystemSoundPlayer`.
  - Components: `passuff/core.py`, `passuff/classify.py`, `passuff/github.py`, `passuff/state.py`, `passuff/credentials.py`, `passuff/notify.py`, `passuff/sound.py`, `passuff/cli.py`.
  - Rules: `givn/specs/watch-pull-requests/usecase.md` R1-R46; one digest and one sound per check pass; baselines suppress historical activity; no replay after restart; focus filters inbox, counts, and notifications without changing the fetch set; exit 0/1/2/3 as specified; no token in transcript or state.
  - Tech: Python 3.11+, `httpx` GraphQL with explicit timeouts, stdlib `tomllib`, `tomlkit` writes, `coverage.py`; no live GitHub in tests.
  - Step file: `givn/steps/pr_activity_notification_steps.py` (shared phrases in `givn/steps/common.py`).
  - Runner: `./scripts/verify.sh --name "The developer tests notifications on the command line" --e2e`
  - Step definition: `givn/e2e_steps/pr_activity_notification_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI subprocess; fake GitHub, D-Bus, sound, gh; bounded with `timeout --signal=INT 10 passuff` where the watcher runs continuously
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: notification-grouping


## E2E batch: e2e-stacked-inbox

- [x] 199. Developer reviews a stacked inbox

- Batch: `stack-grouping`
- Tags: @e2e @stacked-inbox
- Use case: `givn/specs/review-pull-request-stacks/usecase.md`
- Design constraints:
  - Contract: `StackedInbox` -> `TuiApp frame grouping`.
  - Components: `passuff/tui/frame.py`, `passuff/github.py` (read-only stack fields, one retry without them), `passuff/tui/frame.py`.
  - Rules: `givn/specs/review-pull-request-stacks/usecase.md`; stack number, size, 1-based position, ultimate base; standalone rows when absent; unwatched members leave a position gap; merged members remain; the activity keeps the stack label.
  - Step file: `givn/steps/stacked_inbox_steps.py`.
  - Runner: `./scripts/verify.sh --name "Developer reviews a stacked inbox" --e2e`
  - Step definition: `givn/e2e_steps/stacked_inbox_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real TUI: `passuff-tui --snapshot` with fake stack metadata
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: stack-grouping


## E2E batch: e2e-watch-scope

- [x] 200. Developer narrows the focus in the watch scope

- Batch: `scope-editing`
- Tags: @e2e @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer narrows the focus in the watch scope" --e2e`
  - Step definition: `givn/e2e_steps/watch_scope_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real TUI/flags: `--scope-select/--scope-deselect`, `--scope-mode`, `--focus-set`, then `--picker --snapshot`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 201. Developer switches the watch scope mode

- Batch: `scope-editing`
- Tags: @e2e @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer switches the watch scope mode" --e2e`
  - Step definition: `givn/e2e_steps/watch_scope_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real TUI/flags: `--scope-select/--scope-deselect`, `--scope-mode`, `--focus-set`, then `--picker --snapshot`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing

- [x] 202. Developer toggles a repository in the watch scope

- Batch: `scope-editing`
- Tags: @e2e @watch-scope
- Use case: `givn/specs/change-watch-scope/usecase.md`
- Design constraints:
  - Contracts: `WatchScopeEditing` -> `ScopeEditor`; `ConfigStore` -> `TomlConfigStore`.
  - Components: `passuff/scope.py`, `passuff/config.py` (`tomlkit` write, atomic replace, comments preserved), TUI picker in `passuff/tui/`.
  - Rules: `givn/specs/change-watch-scope/usecase.md` R1-R11; explicit all/selected mode; empty selected set observes none; reload before each cycle; catch-up on reselect is silent; focus persists and filters the inbox; `--scope-deselect` is refused in all mode.
  - Step files: `givn/steps/watch_scope_steps.py`; E2E `givn/e2e_steps/watch_scope_e2e.py`.
  - Runner: `./scripts/verify.sh --name "Developer toggles a repository in the watch scope" --e2e`
  - Step definition: `givn/e2e_steps/watch_scope_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real TUI/flags: `--scope-select/--scope-deselect`, `--scope-mode`, `--focus-set`, then `--picker --snapshot`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: scope-editing


## E2E batch: e2e-watch-status

- [x] 203. Developer sees the watch state

- Batch: `status-read-model`
- Tags: @e2e @watch-status
- Use case: `givn/specs/inspect-watch-status/usecase.md`
- Design constraints:
  - Contract: `WatchStatus` -> `WatchStatusReader`.
  - Components: `passuff/status.py`, `passuff/state.py` (`JsonStateStore`), `passuff/render.py`.
  - Rules: `givn/specs/inspect-watch-status/usecase.md`; status reads persisted state only, never GitHub; sections, human labels, unread/total counts, state errors exit 3, configuration errors exit 2.
  - Step file: `givn/steps/watch_status_steps.py`.
  - Runner: `./scripts/verify.sh --name "Developer sees the watch state" --e2e`
  - Step definition: `givn/e2e_steps/watch_status_e2e.py` (E2E; shared phrases via `givn/steps/common.py`)
  - E2E driver: real CLI subprocess `passuff status`
- [x] COMMIT: 14e0b99b19b4851e203c78ac375f3ce69a863da9 -- Batch: status-read-model
