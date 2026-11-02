@givn.delta @activity-inbox

Feature: Activity inbox terminal UI

  @givn.added
  Scenario: The terminal UI shows the inbox
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    And the developer opens the inbox TUI
    Then the TUI shows "1 unread of 1"
    And the TUI shows "personal @octo-personal"
    And the TUI shows "watch scope: all authored pull requests"
    And the TUI shows "Pull request acme/api#42"
    And the TUI shows "New comment from @alice"
    And the TUI does not show "Open"
    And the TUI does not show "Success"
    And the TUI does not show "Can you split this migration?"
    And the TUI does not show "token-TOKEN_PERSONAL"

  @givn.added
  Scenario: The terminal UI shows the empty inbox
    Given a configured account "personal" with login "octo-personal"
    And no state file
    When the developer opens the inbox TUI
    Then the TUI shows "no activity yet"
    And the TUI shows "0 unread of 0"

  @givn.added
  Scenario: The selection moves and clamps
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And the CI result on "acme/api#42" becomes "failing"
    And passuff runs one check
    And the developer opens the inbox TUI
    And the developer presses "j"
    Then the selected activity is "New comment from @alice"
    When the developer presses "j"
    Then the selected activity is "New comment from @alice"
    When the developer presses "k"
    Then the selected activity is "CI failing: build"

  @givn.added
  Scenario: The selected activity is marked read
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And the CI result on "acme/api#42" becomes "failing"
    And passuff runs one check
    And the developer opens the inbox TUI
    And the developer presses "r"
    Then the TUI reports "1 unread of 2"
    And the activity "CI failing: build" is read
    And the activity "New comment from @alice" is unread

  @givn.added
  Scenario: Every activity is marked read
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And the CI result on "acme/api#42" becomes "failing"
    And passuff runs one check
    And the developer opens the inbox TUI
    And the developer presses "a"
    Then the TUI reports "0 unread of 2"

  @givn.added
  Scenario Outline: The terminal UI quits
    Given a configured account "personal" with login "octo-personal"
    And no state file
    When the developer opens the inbox TUI
    And the developer presses "<key>"
    Then the terminal UI is closed

    Examples:
      | key |
      | q   |
      | esc |

  @givn.added
  Scenario: The terminal UI reports a configuration error
    Given a configured account "work" with no credential source
    When the developer starts the inbox TUI binary
    Then the TUI reports "configuration error"
    And the exit status is 2

  @givn.added
  Scenario: The terminal UI reports a state error
    Given a configured account "personal" with login "octo-personal"
    And the state path is a directory
    When the developer starts the inbox TUI binary
    Then the TUI reports "state error"
    And the exit status is 3

  @givn.added @e2e
  Scenario: Developer opens the inbox TUI
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    And the developer opens the inbox TUI
    Then the TUI shows "1 unread of 1"
    And the TUI shows "New comment from @alice"
    And the exit status is 0
