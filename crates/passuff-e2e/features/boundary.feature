@e2e
Feature: The built binaries satisfy the acceptance boundary
  The frozen package drives the product only through its binaries: the CLI in
  a process, the TUI on a real pty, and GitHub over HTTP.

  Scenario: The CLI lists its commands
    Given the built product and a mock GitHub account
    When the developer asks for help
    Then the product exits with code 0
    And the output contains "Commands:"
    And the output contains "notify-test"

  Scenario: A first check baselines the account and status reports the scope
    Given the built product and a mock GitHub account
    When the developer runs one check
    Then the product exits with code 0
    And the output contains "Fresh baseline recorded for octo/repo#7"
    When the developer asks for status
    Then the output contains "passuff: 0 unread of 0"
    And the output contains "watched pull requests"
    And the output contains "octo/repo#7 open CI passing"

  Scenario: New activity reaches the inbox and reading all clears the unread count
    Given the built product and a mock GitHub account
    When the developer runs one check
    And the developer adds a comment on the watched pull request
    And the developer runs one check
    Then the output contains "New comment from @reviewer"
    When the developer lists the inbox
    Then the output contains "New comment from @reviewer"
    And the output contains "unread"
    When the developer marks all activity read
    Then the output contains "activity record read"
    When the developer asks for status
    Then the output contains "passuff: 0 unread of 1"

  Scenario: The inbox row is visible at the narrow terminal size
    Given the built product and a mock GitHub account
    When the developer runs one check
    And the developer adds a comment on the watched pull request
    And the developer runs one check
    And the developer opens the inbox at 80x24
    Then the screen shows "1 unread of 1"
    And the screen shows "octo/repo#7"
    When the developer quits the inbox
    Then the TUI exits with code 0

  Scenario: The activity detail opens at the wide terminal size and quits cleanly
    Given the built product and a mock GitHub account
    When the developer runs one check
    And the developer hides new activity from the authored query
    And the developer adds a comment on the watched pull request
    And the developer opens the inbox at 120x40
    Then the screen shows "comment by @reviewer"
    When the developer presses enter
    Then the screen shows "Event: comment by @reviewer"
    And the screen shows "Pull request octo/repo#7"
    When the developer quits the inbox
    Then the TUI exits with code 0

  Scenario: A notification is delivered as one JSON line
    Given a mock GitHub account with desktop notifications enabled
    When the developer runs one check
    And the developer adds a comment on the watched pull request
    And the developer runs one check
    Then the notification log holds one line titled "New activity on octo/repo#7"

  Scenario: A server error fails the check
    Given a mock GitHub account that answers every operation with a server error
    When the developer runs one check
    Then the product exits with code 1
    And the error output contains "account work failed"
    And the error output contains "GitHub returned a malformed response"

  Scenario: An unreachable endpoint fails with a diagnostic
    Given a mock GitHub account pointed at a closed port
    When the developer runs one check
    Then the product exits with code 1
    And the error output contains "account work failed"
    And the error output contains "GitHub is unavailable"
