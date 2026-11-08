@givn.delta @stacked-inbox
Feature: Stacked inbox boundary evidence

  @givn.added
  Scenario: A rejected stack query retries once and shows standalone rows
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the GitHub stack fields are rejected for "acme/api"
    And  the watcher has already observed "acme/api#42"
    When  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI does not show "☰"
    And  the inbox has asked GitHub for the stack fields "2" times
    And  the exit status is 0
