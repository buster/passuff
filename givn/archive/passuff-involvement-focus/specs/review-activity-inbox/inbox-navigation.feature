@givn.delta @inbox-navigation

Feature: Inbox navigation

  @givn.added
  Scenario: The watched table hides a pull request outside the focus
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "bob"
    And  a team review is requested on "acme/web#7"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the focus relations are "authored"
    When  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI does not show "acme/web#7"
