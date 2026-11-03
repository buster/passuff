@givn.delta @activity-inbox

Feature: Activity inbox

  @givn.added @e2e
  Scenario: Developer opens an activity's details
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "Activity a1"
    And  the TUI shows "New comment from @alice"
    And  the TUI shows "acme/api#42"
    And  the TUI shows "Pull request acme/api#42"
    And  the TUI shows "unread"
    And  the TUI shows "Enter open in browser"
    And  the exit status is 0

  @givn.added @e2e
  Scenario: Developer opens the pull request from an activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter,enter"
    Then  the browser opened "https://github.com/acme/api#42"
    And  the TUI does not show "Activity a1"
    And  the exit status is 0

  @givn.added
  Scenario: Escape closes the activity detail
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "enter"
    And  the developer presses "esc"
    Then  the TUI does not show "Activity a1"
    And  the TUI shows "Focus: activity"

  @givn.added
  Scenario: Enter with no activity shows no detail
    Given  a configured account "personal" with login "octo-personal"
    And  no state file
    When  the developer opens the inbox TUI
    And  the developer presses "enter"
    Then  the TUI shows "no activity yet"
    And  the TUI does not show "Enter open in browser"

  @givn.added
  Scenario: A failed browser launch keeps the activity detail open
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "enter"
    And  the browser will fail to open
    And  the developer presses "enter"
    Then  the TUI shows "Activity a1"
    And  the TUI shows "could not open browser"
