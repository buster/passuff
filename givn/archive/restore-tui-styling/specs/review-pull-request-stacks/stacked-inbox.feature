@givn.delta @stacked-inbox

Feature: Stacked inbox styling

  @givn.modified @e2e
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
    Then  the TUI shows "1/2"
    And  the TUI shows "2/2"
    And  the TUI shows "comment by @alice"
    And  the stack column header is cyan-blue and bold
    And  the stack positions are bright
    And  the selected row is gold and bold
    And  the exit status is 0
