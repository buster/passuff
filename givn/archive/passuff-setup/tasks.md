# Tasks: passuff-setup

### Batch: `initial-setup`

- [x] Setup: `SetupSession`, `passuff setup`, first-run TUI fallback, step file,
      unit tests, and README.
      COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `Developer sets up passuff on first run` (e2e)
  - [x] **Design constraints:** real `passuff --config PATH setup` with scripted
        stdin; config created; TUI snapshot shows the account; exit 0.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `Setup creates a token environment account`
  - [x] **Design constraints:** variable name only; no token value.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `Setup rejects an invalid repository and asks again`
  - [x] **Design constraints:** `must be owner/repo`; no config.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `Setup preserves an existing configuration by default`
  - [x] **Design constraints:** default no; unchanged; exit 0.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `Setup replaces an existing configuration after confirmation`
  - [x] **Design constraints:** yes; new login written.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `An insecure token file is rejected`
  - [x] **Design constraints:** `0o077` check; no config.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `A failed write leaves no partial configuration`
  - [x] **Design constraints:** `cannot write configuration`; no file.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Scenario: `The terminal UI without a terminal points to setup`
  - [x] **Design constraints:** `passuff setup` hint; exit 2.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

- [x] Unit tests: GH/token accounts, invalid repository, preserve/replace,
      insecure token file, failed write.
  - [x] COMMIT: `e8d8113` -- Batch: `initial-setup`

**Done when:** every box checked with the commit; delta 8/8, delta e2e 1/1,
permanent 146/146, permanent e2e 11/11; e2e count 1 < delta count 8.
