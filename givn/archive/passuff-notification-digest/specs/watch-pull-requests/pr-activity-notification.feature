@givn.delta @pr-activity-notification

Feature: Notification digest

  @givn.modified @pr-activity-notification
  Scenario: Multiple events on one pull request produce one notification
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  "alice" submits a "changes requested" review on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    Then  the transcript reports one notification for "acme/api#42"
    And  the notification mentions "New comment", "Review changes requested", and "CI failing"
    And  the desktop notification reports "3 activities on acme/api#42"
    And  the configured sound is played once

  @givn.added @pr-activity-notification
  Scenario: Activity on two pull requests produces one digest notification
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "octo-personal"
    And  the watcher has already observed "acme/api#42" and "acme/web#7"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Ship it" on "acme/web#7"
    And  passuff runs one check
    Then  the transcript reports "2 activities in 2 pull requests"
    And  the desktop notification reports "2 activities in 2 pull requests"
    And  the desktop notification reports "acme/api#42: 1 activity"
    And  the desktop notification reports "acme/web#7: 1 activity"
    And  the transcript reports "New comment from @alice"
    And  the transcript reports "New comment from @bob"
    And  the configured sound is played once
