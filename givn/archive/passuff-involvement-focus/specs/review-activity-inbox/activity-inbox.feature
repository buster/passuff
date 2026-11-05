@givn.delta @activity-inbox

Feature: Activity inbox

  @givn.added
  Scenario: Inbox hides activity outside the focus
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "bob"
    And  a team review is requested on "acme/web#7"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the focus relations are "authored"
    When  "bob" comments "Team queue" on "acme/web#7"
    And  "alice" comments "Mine" on "acme/api#42"
    And  passuff runs one check
    And  the developer lists all activity
    Then  the transcript does not report "New comment from @bob"
    And  the inbox shows "New comment from @alice" for "acme/api#42" as unread
    When  the developer asks for status
    Then  the transcript reports "1 unread of 1"
