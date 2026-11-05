@givn.delta @inbox-navigation

Feature: Inbox navigation

  @givn.added
  Scenario: The activity table shows when the activity happened
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42" at "2026-11-02T09:15:00Z"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "11-02 09:15"

  @givn.added
  Scenario: The activity table marks a CI failure as approximate
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI marks "CI failing" on "acme/api#42" as approximate

  @givn.added
  Scenario: The activity detail shows when the activity happened and when it was fetched
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42" at "2026-11-02T09:15:00Z"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "When: 11-02 09:15"
    And  the TUI shows "Fetched: 11-02 12:00"

  @givn.added
  Scenario: The activity detail marks an activity without a reported time as approximate
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "When: ~11-02 12:00"
    And  the TUI shows "(fetch time)"
