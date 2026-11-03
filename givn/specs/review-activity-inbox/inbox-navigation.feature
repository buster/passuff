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
