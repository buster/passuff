@stacked-inbox
Feature: Stacked inbox

  @e2e @stacked-inbox
  Scenario: Developer reviews a stacked inbox
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    When  "alice" comments "Can you split this migration?" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "stack 7"
    And  the TUI shows "base main"
    And  the TUI shows "size 2"
    And  the TUI shows "position 1"
    And  the TUI shows "position 2"
    And  the TUI shows "New comment from @alice"
    And  the exit status is 0

  @stacked-inbox
  Scenario: The stack orders members by position
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#43"
    And  the watcher has already observed "acme/api#42"
    And  GitHub reports stack 7 for "acme/api#43" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#42" at position 2 of 2 with base "main"
    When  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "position 1" before "position 2"

  @stacked-inbox
  Scenario: An unwatched stack member leaves a position gap
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#44" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#44"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 3 with base "main"
    And  GitHub reports stack 7 for "acme/api#44" at position 3 of 3 with base "main"
    When  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "size 3"
    And  the TUI shows "position 1"
    And  the TUI shows "position 3"
    And  the TUI does not show "position 2"

  @stacked-inbox
  Scenario: A merged stack member remains in the group
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the pull request "acme/api#42" is "merged"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    When  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "stack 7"
    And  the TUI shows "merged"
    And  the TUI shows "position 1"

  @stacked-inbox
  Scenario: A pull request without stack metadata stays standalone
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI does not show "stack 7"
    And  the TUI does not show "base main"

  @stacked-inbox
  Scenario: Stack membership updates after a change
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 1 with base "main"
    When  passuff runs one check
    And  GitHub reports stack 8 for "acme/api#42" at position 1 of 1 with base "release"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "stack 8"
    And  the TUI shows "base release"
    And  the TUI does not show "stack 7"

  @stacked-inbox
  Scenario: The stack unread count aggregates its members
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  an authored pull request "acme/api#40" by "octo-personal"
    And  the watcher has already observed "acme/api#40"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    When  "carol" comments "Standalone" on "acme/api#40"
    And  "bob" comments "Review this" on "acme/api#42"
    And  "alice" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer marks the newest activity read
    And  the developer opens the inbox TUI
    Then  the TUI shows "2 unread of 3"
    And  the TUI shows "1 unread in stack"
