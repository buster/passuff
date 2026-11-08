@givn.delta @inbox-navigation
Feature: Inbox navigation boundary evidence

  @givn.added
  Scenario: The activity list row is bounded to the recorded excerpt
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" receives a long comment from "alice"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "Comment by @alice"
    And  the activity list row excerpt is at most 80 characters
    And  the TUI does not show "comment line 40"

  @givn.added
  Scenario: The stored activity body is bounded to 4 KiB
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" receives a 6000-character comment from "alice"
    And  passuff runs one check
    Then  the stored activity body is at most 4096 bytes
    When  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "body line 1"
    When  the developer presses "end"
    Then  the TUI does not show "TAIL MARKER"

  @givn.added
  Scenario: A missing browser opener is reported and keeps the detail
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the browser opener is missing
    And  the developer opens the inbox TUI with keys "enter,enter"
    Then  the TUI shows "Activity api#42"
    And  the TUI shows "could not open browser"
    And  the exit status is 0
