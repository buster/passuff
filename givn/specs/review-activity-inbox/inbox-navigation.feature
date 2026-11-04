@inbox-navigation
Feature: Inbox navigation

  @e2e @inbox-navigation
  Scenario: Developer navigates the inbox panes
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "tab,j"
    Then  the TUI shows "Focus: PRs"
    And  the TUI shows "▶ acme/api#43"
    And  the exit status is 0

  @inbox-navigation
  Scenario: The focus cycles between watched pull requests and activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    Then  the TUI shows "Focus: PRs"
    When  the developer presses "tab"
    Then  the TUI shows "Focus: activity"

  @inbox-navigation
  Scenario: The watched pull request selection scrolls
    Given  a configured account "personal" with login "octo-personal"
    And  25 authored pull requests in "acme/api" by "octo-personal"
    And  the watcher has already observed all of them
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer moves the selection down 24 times
    Then  the TUI shows "▶ acme/api#25"
    And  the TUI shows "Focus: PRs"

  @inbox-navigation
  Scenario: A long pull request key keeps its number visible
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "very-long-organization/very-long-repository#1234" by "octo-personal"
    And  the watcher has already observed "very-long-organization/very-long-repository#1234"
    When  the developer opens the inbox TUI
    Then  the TUI shows "…repository#1234"

  @inbox-navigation
  Scenario: The watched table shows the creator and the assignee
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" is assigned to "alice"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    Then  the TUI shows "☺"
    And  the TUI shows "@octo-personal → @alice"

  @inbox-navigation
  Scenario: A stack is marked with a symbol and its position
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  the developer opens the inbox TUI
    Then  the TUI shows "☰"
    And  the TUI shows "1/2"

  @inbox-navigation
  Scenario: A standalone pull request shows no stack symbol
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    Then  the TUI does not show "☰"

  @inbox-navigation
  Scenario: The developer sorts the watched list
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "s"
    Then  the TUI shows "sort: name ↑"
    When  the developer presses "S"
    Then  the TUI shows "sort: name ↓"

  @inbox-navigation
  Scenario: The developer filters the activity to a pull request
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "f"
    Then  the TUI shows "Activity · acme/api#42"
    And  the TUI shows "1 unread of 1"
    And  the TUI shows "(filtered from 2)"
    When  the developer presses "c"
    Then  the TUI shows "2 unread of 2"
    And  the TUI does not show "(filtered from"

  @inbox-navigation
  Scenario: The developer filters the activity to a stack
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "F"
    Then  the TUI shows "Activity · acme/api stack 7"
    And  the TUI shows "2 unread of 2"

  @inbox-navigation
  Scenario: The developer filters the activity to unread records
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "r"
    And  the developer presses "u"
    Then  the TUI shows "Activity · unread"
    And  the TUI shows "1 unread of 1"
    And  the TUI shows "(filtered from 2)"
    When  the developer presses "c"
    Then  the TUI shows "1 unread of 2"
    And  the TUI does not show "(filtered from"

  @inbox-navigation
  Scenario: Marking all read applies to the filtered activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "f"
    And  the developer presses "tab"
    And  the developer presses "a"
    Then  the activity "New comment from @alice" is read
    And  the activity "New comment from @bob" is unread

  @inbox-navigation
  Scenario: The developer opens the key reference
    Given  a configured account "personal" with login "octo-personal"
    When  the developer opens the inbox TUI
    And  the developer presses "h"
    Then  the TUI shows "Activity pane"
    And  the TUI shows "mark visible read"
    When  the developer presses "esc"
    Then  the TUI does not show "Activity pane"

  @inbox-navigation
  Scenario: The pull request detail shows its metadata
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" has the description "Adds the live inbox"
    And  the pull request "acme/api#42" is assigned to "alice"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "enter"
    Then  the TUI shows "Creator: @octo-personal"
    And  the TUI shows "Assignee: @alice"
    And  the TUI shows "Adds the live inbox"
    And  the TUI shows "CI: passing"

  @inbox-navigation
  Scenario: The developer ignores and restores a pull request
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "x"
    Then  the TUI does not show "acme/api#42"
    When  the developer presses "right"
    Then  the TUI shows "acme/api#42"
    When  the developer presses "x"
    And  the developer presses "left"
    Then  the TUI shows "acme/api#42"

  @inbox-navigation
  Scenario: A comment on a merged pull request reaches the inbox
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the configured account "personal" merges "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the inbox observes the account
    And  "alice" comments "Post-merge note" on "acme/api#42"
    And  the inbox observes the account
    Then  the TUI shows "comment by @alice"

  @inbox-navigation
  Scenario: A refresh preserves the selected pull request
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "j"
    Then  the selected pull request is "acme/api#43"
    When  an authored pull request "acme/api#44" by "octo-personal" appears
    And  the developer refreshes the inbox
    Then  the selected pull request is "acme/api#43"
    And  the TUI shows "acme/api#44"

  @e2e @inbox-navigation
  Scenario: The inbox shows activity observed while it is open
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  "alice" comments "Review this" on "acme/api#42" after the first observation
    When  the developer opens the inbox TUI and lets it observe for "4" seconds
    Then  the TUI shows "comment by @alice"
    And  the TUI shows "1 unread of 1"
    And  the exit status is 0

  @inbox-navigation
  Scenario: The inbox keeps observation cheap while nothing changes
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the inbox observes the account three times with no change
    Then  the inbox has asked GitHub for the full account history "1" time
    And  the inbox has asked GitHub for a change signal "3" times

  @inbox-navigation
  Scenario: The inbox paces observation against the account allowance
    Given  a configured account "personal" with login "octo-personal"
    And  the account has "100" API points left until the allowance resets in "3600" seconds
    When  the inbox plans its next observation
    Then  the inbox waits at least "30" seconds before observing again

  @inbox-navigation
  Scenario: A failed observation keeps the last inbox view
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Review this" on "acme/api#42"
    And  the developer opens the inbox TUI
    And  the inbox observes the account
    And  GitHub becomes unavailable for the next observation
    And  the inbox observes the account
    Then  the TUI still shows "comment by @alice"
    And  the exit status is 0
