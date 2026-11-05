@givn.delta @activity-inbox

Feature: Activity inbox

  @givn.modified @wip
  Scenario: Inbox lists unread activity newest first
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer lists the inbox
    Then  the inbox shows "CI failing" before "New comment from @alice" for "acme/api#42"
    And  every inbox record is unread
    And  the inbox shows an activity id and occurrence time
    And  the transcript does not report "Open"
    And  the transcript does not report "Success"
    And  the transcript does not report "Can you split this migration?"

  @givn.added @wip
  Scenario: Inbox shows when a comment was written
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42" at "2026-11-02T09:15:00Z"
    And  passuff runs one check
    And  the developer lists the inbox
    Then  the inbox shows the occurrence time "2026-11-02T09:15:00Z" for "acme/api#42"

  @givn.added @wip
  Scenario: Inbox marks an event without a reported time as approximate
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer lists the inbox
    Then  the inbox marks "CI failing" on "acme/api#42" as approximate

  @givn.added @wip
  Scenario: Inbox orders activity by when it happened
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "First" on "acme/api#42" at "2026-11-02T09:15:00Z"
    And  "bob" comments "Second" on "acme/api#42" at "2026-11-02T09:45:00Z"
    And  passuff runs one check
    And  the developer lists the inbox
    Then  the inbox shows "New comment from @bob" before "New comment from @alice" for "acme/api#42"
