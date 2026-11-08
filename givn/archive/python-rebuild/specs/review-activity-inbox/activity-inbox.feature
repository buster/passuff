@givn.delta @activity-inbox
Feature: Activity inbox boundary evidence

  @givn.modified @e2e
  Scenario: Developer opens an activity's details
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "Activity api#42"
    And  the TUI shows "comment by @alice"
    And  the TUI shows "Can you split this migration?"
    And  the TUI shows "acme/api#42"
    And  the TUI shows "Pull request acme/api#42"
    And  the TUI shows "unread"
    And  the TUI shows "Enter open in browser"
    And  the exit status is 0
