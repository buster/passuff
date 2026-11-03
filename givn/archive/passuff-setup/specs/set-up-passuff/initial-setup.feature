@givn.delta @initial-setup

Feature: Initial setup

  @givn.added @e2e
  Scenario: Developer sets up passuff on first run
    Given no configuration file exists
    When the developer runs setup with:
      | label     | personal      |
      | auth      | gh_oauth      |
      | login     | octo-personal |
      | watch     | all           |
    Then the configuration is created
    And the configuration contains account "personal"
    And the configuration contains auth "gh_oauth"
    And the configuration does not contain "token"
    When the developer opens the inbox TUI
    Then the TUI shows "personal"
    And the exit status is 0

  @givn.added
  Scenario: Setup creates a token environment account
    Given no configuration file exists
    When the developer runs setup with:
      | label     | work       |
      | auth      | token_env  |
      | variable  | WORK_TOKEN |
      | login     | octo-work  |
      | watch     | all        |
    Then the configuration contains token env "WORK_TOKEN"
    And the configuration does not contain "token-value"

  @givn.added
  Scenario: Setup rejects an invalid repository and asks again
    Given no configuration file exists
    When the developer runs setup with:
      | label     | personal      |
      | auth      | gh_oauth      |
      | login     | octo-personal |
      | watch     | selected      |
      | repos     | acme          |
    Then setup reports "must be owner/repo"
    And the configuration is not created

  @givn.added
  Scenario: Setup preserves an existing configuration by default
    Given a configuration file exists
    When the developer runs setup and answers "no" to replacement
    Then the configuration is unchanged
    And the exit status is 0

  @givn.added
  Scenario: Setup replaces an existing configuration after confirmation
    Given a configuration file exists
    When the developer runs setup with:
      | label     | personal      |
      | auth      | gh_oauth      |
      | login     | octo-new      |
      | watch     | all           |
      | replace   | yes           |
    Then the configuration contains login "octo-new"

  @givn.added
  Scenario: An insecure token file is rejected
    Given no configuration file exists
    And a world-readable token file
    When the developer runs setup with:
      | label     | personal      |
      | auth      | token_file    |
      | file      | TOKEN_FILE    |
      | login     | octo-personal |
      | watch     | all           |
    Then setup reports "insecure"
    And the configuration is not created

  @givn.added
  Scenario: A failed write leaves no partial configuration
    Given no configuration file exists
    And the setup configuration directory cannot be written
    When the developer runs setup with:
      | label     | personal      |
      | auth      | gh_oauth      |
      | login     | octo-personal |
      | watch     | all           |
    Then setup reports "cannot write configuration"
    And the configuration is not created

  @givn.added
  Scenario: The terminal UI without a terminal points to setup
    Given no configuration file exists
    When the developer opens the inbox TUI without a terminal
    Then the TUI reports "passuff setup"
    And the exit status is 2
