@givn.delta @inbox-navigation

Feature: Inbox navigation

  @givn.added @e2e
  Scenario: The inbox shows activity observed while it is open
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  "alice" comments "Review this" on "acme/api#42" after the first observation
    When  the developer opens the inbox TUI and lets it observe for "4" seconds
    Then  the TUI shows "New comment from @alice"
    And  the TUI shows "1 unread of 1"
    And  the exit status is 0

  @givn.added
  Scenario: The inbox keeps observation cheap while nothing changes
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the inbox observes the account three times with no change
    Then  the inbox has asked GitHub for the full account history "1" time
    And  the inbox has asked GitHub for a change signal "3" times

  @givn.added
  Scenario: The inbox paces observation against the account allowance
    Given  a configured account "personal" with login "octo-personal"
    And  the account has "100" API points left until the allowance resets in "3600" seconds
    When  the inbox plans its next observation
    Then  the inbox waits at least "30" seconds before observing again

  @givn.added
  Scenario: A failed observation keeps the last inbox view
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Review this" on "acme/api#42"
    And  the developer opens the inbox TUI
    And  the inbox observes the account
    And  GitHub becomes unavailable for the next observation
    And  the inbox observes the account
    Then  the TUI still shows "New comment from @alice"
    And  the exit status is 0
