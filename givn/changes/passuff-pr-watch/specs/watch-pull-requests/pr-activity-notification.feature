@givn.delta @pr-activity-notification
Feature: Pull request activity notification

  @givn.added @e2e
  Scenario: Developer starts watching and is notified of a new comment
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    When passuff starts watching
    And "alice" comments "Can you split this migration?" on "acme/api#42"
    Then the terminal shows a notification for "acme/api#42"
    And the desktop notification reports "New comment from @alice"
    And the configured sound is played once
    When passuff is stopped
    Then the exit status is 0

  @givn.added @e2e
  Scenario: Developer runs a single check and sees the current activity
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "New comment from @alice"
    And the desktop notification reports "New comment from @alice"
    And the configured sound is played once
    And the exit status is 0

  @givn.added @e2e
  Scenario: Developer asks for usage
    When the developer runs passuff with "--help"
    Then the transcript shows the available commands
    And the exit status is 0

  @givn.added @e2e
  Scenario: Developer asks for the version
    When the developer runs passuff with "--version"
    Then the transcript shows the version
    And the exit status is 0

  @givn.added
  Scenario: A review comment from another person notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" writes a review comment "This needs a test" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "Review comment from @alice"

  @givn.added
  Scenario Outline: A submitted review notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" submits a "<state>" review on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "Review <state> by @alice"

    Examples:
      | state             |
      | approved          |
      | changes requested |
      | commented         |

  @givn.added
  Scenario: A review from a configured account stays silent
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "octo-personal" submits an "approved" review on "acme/api#42"
    And passuff runs one check
    Then the transcript reports no notifications

  @givn.added
  Scenario: A commit push stays silent
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" pushes a commit to "acme/api#42"
    And passuff runs one check
    Then the transcript reports no notifications

  @givn.added
  Scenario: Activity on someone else's pull request stays silent
    Given a configured account "personal" with login "octo-personal"
    And a pull request "acme/other#9" by "someone-else"
    And the watcher has already observed "acme/other#9"
    When "alice" comments "Looks good" on "acme/other#9"
    And passuff runs one check
    Then the transcript reports no notifications

  @givn.added
  Scenario Outline: A failing CI result notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When the CI result on "acme/api#42" becomes "<state>"
    And passuff runs one check
    Then the transcript reports "CI failing"

    Examples:
      | state   |
      | failing |
      | error   |

  @givn.added
  Scenario: A CI recovery notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42" with a failing CI result
    When the CI result on "acme/api#42" becomes "successful"
    And passuff runs one check
    Then the transcript reports "CI recovered"

  @givn.added
  Scenario: A CI recovery across pending notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42" with a failing CI result
    When the CI result on "acme/api#42" becomes "pending"
    And passuff runs one check
    Then the transcript reports no notifications
    When the CI result on "acme/api#42" becomes "successful"
    And passuff runs one check
    Then the transcript reports "CI recovered"

  @givn.added
  Scenario Outline: A pull request merged or closed while the watcher was stopped is reported
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When the pull request "acme/api#42" becomes "<state>" while the watcher is stopped
    And passuff runs one check
    Then the transcript reports "<label>"

    Examples:
      | state  | label  |
      | merged | Merged |
      | closed | Closed |

  @givn.added
  Scenario Outline: A short-lived pull request merged or closed between checks notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When an authored pull request "acme/new#1" by "octo-personal" is created and "<state>" between checks
    And passuff runs one check
    Then the transcript reports a notification for "acme/new#1"
    And the transcript reports "<label>"

    Examples:
      | state  | label  |
      | merged | Merged |
      | closed | Closed |

  @givn.added
  Scenario: A merge performed by the configured account still notifies
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When the configured account "octo-personal" merges "acme/api#42"
    And passuff runs one check
    Then the transcript reports "Merged"

  @givn.added
  Scenario: A reopened pull request stays silent
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When the pull request "acme/api#42" becomes "closed"
    And passuff runs one check
    Then the transcript reports "Closed"
    When the pull request "acme/api#42" is reopened
    And passuff runs one check
    Then the transcript reports no notifications

  @givn.added
  Scenario: A reopened pull request notifies again when closed
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When the pull request "acme/api#42" becomes "closed"
    And passuff runs one check
    Then the transcript reports "Closed"
    When the pull request "acme/api#42" is reopened
    And passuff runs one check
    When the pull request "acme/api#42" becomes "closed"
    And passuff runs one check
    Then the latest transcript reports "Closed"

  @givn.added
  Scenario Outline: A bot comment is handled according to the bot setting
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And bot comments are "<bot_comments>" in configuration
    When "dependabot[bot]" comments "Bump lodash" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports <outcome>

    Examples:
      | bot_comments | outcome                            |
      | disabled     | no notifications                   |
      | enabled      | "New comment from @dependabot[bot]" |

  @givn.added
  Scenario: Historical activity is not replayed on first observation
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the pull request already has a comment by "alice"
    When passuff runs one check
    Then the transcript reports "Fresh baseline recorded"
    And the transcript reports no notifications

  @givn.added
  Scenario: A pull request first seen after the first cycle records a baseline
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When an authored pull request "acme/new#2" by "octo-personal" appears with an existing comment by "alice"
    And passuff runs one check
    Then the transcript reports "Fresh baseline recorded"
    And the report records a baseline for "acme/new#2"
    And the transcript reports no notifications

  @givn.added
  Scenario: Multiple events on one pull request produce one notification
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And "alice" submits a "changes requested" review on "acme/api#42"
    And the CI result on "acme/api#42" becomes "failing"
    And passuff runs one check
    Then the transcript reports one notification for "acme/api#42"
    And the notification mentions "New comment", "Review changes requested", and "CI failing"
    And the configured sound is played once

  @givn.added
  Scenario Outline: A disabled event family suppresses its notification
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And <setup> for "acme/api#42"
    And the "<family>" event family is disabled in configuration
    When <event> on "acme/api#42"
    And passuff runs one check
    Then the transcript reports no notifications

    Examples:
      | family        | setup                          | event                                             |
      | comments      | no prior activity exists       | "alice" comments "Can you split this?"            |
      | comments      | no prior activity exists       | "alice" writes a review comment "Needs a test"    |
      | reviews       | no prior activity exists       | "alice" submits an "approved" review              |
      | ci_failure    | the CI result is "successful"  | the CI result becomes "failing"                  |
      | ci_recovery   | the CI result is "failing"     | the CI result becomes "successful"               |
      | state_changes | the pull request is "open"      | the pull request becomes "merged"                |

  @givn.added
  Scenario: A configured sound is used
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And the sound is configured to "crates/passuff-cli/assets/notification.wav"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "New comment from @alice"
    And the configured sound file "crates/passuff-cli/assets/notification.wav" is played once

  @givn.added
  Scenario: A disabled sound produces no sound
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And sound is disabled in configuration
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "New comment from @alice"
    And the desktop notification is recorded
    And no sound is played

  @givn.added
  Scenario: A disabled desktop notification keeps the transcript
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And the desktop notification is disabled in configuration
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "New comment from @alice"
    And no desktop notification is recorded
    And the configured sound is played once
    And the exit status is 0

  @givn.added
  Scenario: An unavailable desktop surface keeps the transcript
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And the desktop notification surface is unavailable
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "New comment from @alice"
    And the transcript reports "desktop notification failed"
    And the exit status is 0

  @givn.added
  Scenario: A missing sound player still delivers the notification
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    And the sound player is unavailable
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "New comment from @alice"
    And the desktop notification is recorded
    And the transcript reports "sound unavailable"
    And the exit status is 0

  @givn.added
  Scenario: Each configured account is observed
    Given a configured account "personal" with login "octo-personal"
    And a configured account "work" with login "octo-work"
    And an authored pull request "acme/api#42" by "octo-personal"
    And an authored pull request "acme/web#7" by "octo-work"
    And the watcher has already observed "acme/api#42" and "acme/web#7"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And "bob" comments "Please update the changelog" on "acme/web#7"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"
    And the transcript reports a notification for "acme/web#7"

  @givn.added
  Scenario: A comment from another configured account stays silent
    Given a configured account "personal" with login "octo-personal"
    And a configured account "work" with login "octo-work"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "octo-work" comments "I will rerun CI" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports no notifications

  @givn.added
  Scenario: A failed account does not stop the other account
    Given a configured account "work" with an invalid credential
    And a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports "account work failed"
    And the transcript reports a notification for "acme/api#42"
    And the exit status is 1

  @givn.added
  Scenario: A restart does not replay notified activity
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"
    When passuff restarts and runs one check
    Then the transcript reports no notifications

  @givn.added
  Scenario: A corrupt state file is replaced with a baseline
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the pull request already has a comment by "alice"
    And a corrupt persisted observation
    When passuff runs one check
    Then the transcript reports "Fresh baseline recorded"
    And the transcript reports no notifications

  @givn.added
  Scenario: An invalid configuration is rejected
    Given an invalid configuration
    When passuff runs one check
    Then the transcript reports "configuration error"
    And the exit status is 2

  @givn.added
  Scenario: A state write failure is reported with a nonzero status
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the state file cannot be written
    When passuff runs one check
    Then the transcript reports "state error"
    And the exit status is 3

  @givn.added
  Scenario Outline: A non-notified change stays silent
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When a "<change>" change occurs on "acme/api#42"
    And passuff runs one check
    Then the transcript reports no notifications

    Examples:
      | change      |
      | label       |
      | draft       |
      | assignee    |
      | milestone   |
      | base branch |

  @givn.added
  Scenario: No authored pull requests produce no notifications
    Given a configured account "personal" with login "octo-personal"
    And the account has no authored pull requests
    When passuff runs one check
    Then the transcript reports no notifications
    And the exit status is 0

  @givn.added
  Scenario: A single check with no changes reports no notifications
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When passuff runs one check
    Then the transcript reports no notifications
    And no desktop notification is recorded
    And no sound is played
    And the exit status is 0
