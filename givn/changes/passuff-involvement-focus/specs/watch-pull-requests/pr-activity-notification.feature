@givn.delta @pr-activity-notification

Feature: Pull request activity notification

  @givn.added
  Scenario: A pull request outside the focus produces no notification
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/web#7" by "bob"
    And  a team review is requested on "acme/web#7"
    And  the watcher has already observed "acme/web#7"
    And  the focus relations are "authored"
    When  "alice" comments "Team queue" on "acme/web#7"
    And  passuff runs one check
    Then  the transcript does not report "New comment from @alice"
    And  the activity "New comment from @alice" is unread
    When  the developer narrows the focus to "authored,team-review-requested"
    And  the developer lists all activity
    Then  the inbox shows "New comment from @alice" for "acme/web#7" as unread
