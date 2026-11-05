@givn.delta @watch-scope

Feature: Watch scope editing

  @givn.modified @e2e
  Scenario: Developer toggles a repository in the watch scope
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer selects "acme/api" in the watch scope
    Then  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    And  the picker shows "Watch scope"
    And  the picker shows "[x] acme/api"
    When  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    When  the developer deselects "acme/api" in the watch scope
    Then  the watch scope does not contain "acme/api"
    When  the developer opens the inbox TUI
    Then  the TUI does not show "acme/api#42"
    And  the state retains "acme/api#42"

  @givn.removed @watch-scope
  Scenario: Adding a repository by owner/name
    Given the manual repository entry no longer exists

  @givn.added
  Scenario: The picker shows the focus relations
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the scope picker
    Then  the picker shows "Focus"
    And  the picker shows "[x] authored"
    And  the picker shows "[x] assigned"
    And  the picker shows "[x] review requested"
    And  the picker shows "[x] team review"
    And  the picker does not show "Add:"
    And  the picker does not show "owner/name"

  @givn.added
  Scenario: Narrowing the focus persists the relation set
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer narrows the focus to "authored,assigned"
    Then  the focus relations are "authored,assigned"

  @givn.added @e2e
  Scenario: Developer narrows the focus in the watch scope
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer narrows the focus to "authored,assigned"
    Then  the focus relations are "authored,assigned"
    And  the picker shows "[x] authored"
    And  the picker shows "[ ] team review"
