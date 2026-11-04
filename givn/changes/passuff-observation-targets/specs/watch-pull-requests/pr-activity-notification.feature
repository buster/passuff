@givn.delta @pr-activity-notification

Feature: Observation targets

  @givn.added @wip
  Scenario Outline: Activity on a merged or closed pull request does not notify
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" is "<state>"
    And  the watcher has already observed "acme/api#42"
    When  "@alice" comments "Post-terminal note" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications

    Examples:
      | state  |
      | merged |
      | closed |

  @givn.added @wip
  Scenario: A review on a merged pull request does not notify
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" is "merged"
    And  the watcher has already observed "acme/api#42"
    When  "alice" submits a "changes requested" review on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications

  @givn.added @wip
  Scenario: A failed observation keeps the change pending
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "@alice" comments "Retry me" on "acme/api#42"
    And  the next observation fails
    And  passuff runs one check
    Then  the transcript reports "account personal failed"
    When  passuff runs one check
    Then  the transcript reports "New comment"
