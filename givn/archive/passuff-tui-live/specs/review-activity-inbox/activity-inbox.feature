@givn.delta @activity-inbox

Feature: Activity inbox

  @givn.modified @e2e
  Scenario: Developer opens the inbox TUI
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the developer opens the inbox TUI observing the account
    Then  the TUI shows "1 unread of 1"
    And  the TUI shows "New comment from @alice"
    And  the exit status is 0
