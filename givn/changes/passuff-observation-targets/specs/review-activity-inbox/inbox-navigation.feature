@givn.delta @inbox-navigation

Feature: Observation targets in the live inbox

  @givn.removed @inbox-navigation
  Scenario: A comment on a merged pull request reaches the inbox
    Then  the scenario is removed

  @givn.added @wip
  Scenario: The inbox observes only the open pull request that changed
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42" and "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42" after the first observation
    And  the developer opens the inbox TUI and lets it observe for "4" seconds
    Then  the TUI shows "comment by @alice"
    And  the inbox has observed only "acme/api#42"

  @givn.added @wip
  Scenario: The inbox reconciles open pull requests when the interval elapses
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "alice"
    And  the pull request "acme/api#42" is assigned to me
    And  the watcher has already observed "acme/api#42"
    And  the reconciliation interval is "2" seconds
    When  "bob" comments "Can you review this?" on "acme/api#42" after the first observation
    And  the developer opens the inbox TUI and lets it observe for "6" seconds
    Then  the TUI shows "comment by @bob"
