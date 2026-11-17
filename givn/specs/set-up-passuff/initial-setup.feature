@initial-setup
Feature: Initial setup

  @component @initial-setup
  Scenario: Developer sets up passuff on first run
    Given  no configuration file exists
    When  the developer runs setup with:
      | label | personal |
      | auth | gh_oauth |
      | login | octo-personal |
      | watch | all |
    Then  the configuration is created
    And  the configuration contains account "personal"
    And  the configuration contains auth "gh_oauth"
    And  the configuration does not contain "token"
    When  the developer opens the inbox TUI
    Then  the TUI shows "personal"
    And  the exit status is 0

  @initial-setup
  Scenario: Setup creates a token environment account
    Given  no configuration file exists
    When  the developer runs setup with:
      | label | work |
      | auth | token_env |
      | variable | WORK_TOKEN |
      | login | octo-work |
      | watch | all |
    Then  the configuration contains token env "WORK_TOKEN"
    And  the configuration does not contain "token-value"

  @initial-setup
  Scenario: Setup rejects an invalid repository and asks again
    Given  no configuration file exists
    When  the developer runs setup with:
      | label | personal |
      | auth | gh_oauth |
      | login | octo-personal |
      | watch | selected |
      | repos | acme |
    Then  setup reports "must be owner/repo"
    And  the configuration is not created

  @initial-setup
  Scenario: Setup preserves an existing configuration by default
    Given  a configuration file exists
    When  the developer runs setup and answers "no" to replacement
    Then  the configuration is unchanged
    And  the exit status is 0

  @initial-setup
  Scenario: Setup replaces an existing configuration after confirmation
    Given  a configuration file exists
    When  the developer runs setup with:
      | label | personal |
      | auth | gh_oauth |
      | login | octo-new |
      | watch | all |
      | replace | yes |
    Then  the configuration contains login "octo-new"

  @initial-setup
  Scenario: An insecure token file is rejected
    Given  no configuration file exists
    And  a world-readable token file
    When  the developer runs setup with:
      | label | personal |
      | auth | token_file |
      | file | TOKEN_FILE |
      | login | octo-personal |
      | watch | all |
    Then  setup reports "insecure"
    And  the configuration is not created

  @initial-setup
  Scenario: A failed write leaves no partial configuration
    Given  no configuration file exists
    And  the setup configuration directory cannot be written
    When  the developer runs setup with:
      | label | personal |
      | auth | gh_oauth |
      | login | octo-personal |
      | watch | all |
    Then  setup reports "cannot write configuration"
    And  the configuration is not created

  @initial-setup
  Scenario: The terminal UI without a terminal points to setup
    Given  no configuration file exists
    When  the developer opens the inbox TUI without a terminal
    Then  the TUI reports "passuff setup"
    And  the exit status is 2

  @e2e
  Scenario: Setup writes a usable configuration
    Given the built product and a mock GitHub account
    And no configuration file exists
    When the developer runs setup with:
      """
      work
      token_env
      tester
      PASSUFF_E2E_TOKEN
      n
      all
      """
    Then the product exits with code 0
    And the configuration file is created
    And the configuration file contains "[accounts.work]"
    And the configuration file contains "github_user"
    And the configuration file contains "PASSUFF_E2E_TOKEN"
    When the developer asks for status
    Then the product exits with code 0
    And the output contains "  work not resolved"
    And the output contains "all authored pull requests"

  @e2e
  Scenario: Setup leaves an existing configuration alone without confirmation
    Given the built product and a mock GitHub account
    And an existing configuration file
    When the developer runs setup and declines replacement
    Then the product exits with code 0
    And the output contains "Configuration exists. Replace? [y/N]"
    And the configuration file is unchanged

  @e2e
  Scenario: Setup rejects an invalid watch repository and asks again
    Given the built product and a mock GitHub account
    And no configuration file exists
    When the developer runs setup with:
      """
      work
      token_env
      tester
      PASSUFF_E2E_TOKEN
      n
      selected
      acme
      octo/repo
      """
    Then the product exits with code 0
    And the output contains "must be owner/repo"
    When the developer asks for status
    Then the output contains "passuff: watch scope: octo/repo"
