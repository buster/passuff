@pr-activity-notification
Feature: Pull request activity notification

  @e2e @pr-activity-notification
  Scenario: Developer starts watching and is notified of a new comment
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    When  passuff starts watching
    And  "alice" comments "Can you split this migration?" on "acme/api#42"
    Then  the terminal shows a notification for "acme/api#42"
    And  the desktop notification reports "New comment from @alice"
    And  the configured sound is played once

  @pr-activity-notification
  Scenario: A pull request I did not author is watched
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "alice"
    And  the pull request "acme/api#42" is assigned to me
    And  the watcher has already observed "acme/api#42"
    When  "bob" comments "Can you review this?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario: A pull request opened after the last check is an activity
    Given  a configured account "personal" with login "octo-personal"
    When  passuff runs one check
    And  an authored pull request "acme/api#42" by "octo-personal" is created after the last check
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"
    And  the transcript reports "New pull request by @octo-personal"

  @e2e @pr-activity-notification
  Scenario: Developer runs a single check and sees the current activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "New comment from @alice"
    And  the desktop notification reports "New comment from @alice"
    And  the configured sound is played once
    And  the exit status is 0

  @e2e @pr-activity-notification
  Scenario: Developer asks for usage
    When  the developer runs passuff with "--help"
    Then  the transcript shows the available commands
    And  the exit status is 0

  @e2e @pr-activity-notification
  Scenario: Developer asks for the version
    When  the developer runs passuff with "--version"
    Then  the transcript shows the version
    And  the exit status is 0

  @pr-activity-notification
  Scenario: A review comment from another person notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" writes a review comment "This needs a test" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "Review comment from @alice"

  @pr-activity-notification
  Scenario Outline: A submitted review notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" submits a "<state>" review on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "Review <state> by @alice"
    Examples:
      | state |
      | approved |
      | changes requested |
      | commented |

  @pr-activity-notification
  Scenario: A review from a configured account stays silent
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "octo-personal" submits an "approved" review on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications
    When  the developer lists all activity
    Then  the transcript reports "Review approved by @octo-personal"
    And  the activity "Review approved by @octo-personal" is read

  @pr-activity-notification
  Scenario: A commit push stays silent
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" pushes a commit to "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications

  @pr-activity-notification
  Scenario: Activity on someone else's pull request stays silent
    Given  a configured account "personal" with login "octo-personal"
    And  a pull request "acme/other#9" by "someone-else"
    And  the watcher has already observed "acme/other#9"
    When  "alice" comments "Looks good" on "acme/other#9"
    And  passuff runs one check
    Then  the transcript reports no notifications

  @pr-activity-notification
  Scenario Outline: A failing CI result notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the CI result on "acme/api#42" becomes "<state>"
    And  passuff runs one check
    Then  the transcript reports "CI failing"
    Examples:
      | state |
      | failing |
      | error |

  @pr-activity-notification
  Scenario: A CI recovery notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42" with a failing CI result
    When  the CI result on "acme/api#42" becomes "successful"
    And  passuff runs one check
    Then  the transcript reports "CI recovered"

  @pr-activity-notification
  Scenario: A CI recovery across pending notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42" with a failing CI result
    When  the CI result on "acme/api#42" becomes "pending"
    And  passuff runs one check
    Then  the transcript reports no notifications
    When  the CI result on "acme/api#42" becomes "successful"
    And  passuff runs one check
    Then  the transcript reports "CI recovered"

  @pr-activity-notification
  Scenario Outline: A pull request merged or closed while the watcher was stopped is reported
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" becomes "<state>" while the watcher is stopped
    And  passuff runs one check
    Then  the transcript reports "<label>"
    Examples:
      | state | label |
      | merged | Merged |
      | closed | Closed |

  @pr-activity-notification
  Scenario Outline: A short-lived pull request merged or closed between checks notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  an authored pull request "acme/new#1" by "octo-personal" is created and "<state>" between checks
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/new#1"
    And  the transcript reports "<label>"
    Examples:
      | state | label |
      | merged | Merged |
      | closed | Closed |

  @pr-activity-notification
  Scenario: A merge performed by the configured account still notifies
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the configured account "octo-personal" merges "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "Merged"

  @pr-activity-notification
  Scenario: A reopened pull request stays silent
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" becomes "closed"
    And  passuff runs one check
    Then  the transcript reports "Closed"
    When  the pull request "acme/api#42" is reopened
    And  passuff runs one check
    Then  the transcript reports no notifications

  @pr-activity-notification
  Scenario: A reopened pull request notifies again when closed
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" becomes "closed"
    And  passuff runs one check
    Then  the transcript reports "Closed"
    When  the pull request "acme/api#42" is reopened
    And  passuff runs one check
    When  the pull request "acme/api#42" becomes "closed"
    And  passuff runs one check
    Then  the latest transcript reports "Closed"

  @pr-activity-notification
  Scenario Outline: A bot comment is handled according to the bot setting
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  bot comments are "<bot_comments>" in configuration
    When  "dependabot[bot]" comments "Bump lodash" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports <outcome>
    Examples:
      | bot_comments | outcome |
      | disabled | no notifications |
      | enabled | "New comment from @dependabot[bot]" |

  @pr-activity-notification
  Scenario: A pull request already merged at the baseline is not reported on the next cycle
    Given a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the pull request "acme/api#42" is "merged"
    When passuff runs one check
    Then the transcript reports no notifications
    When passuff runs one check
    Then the transcript reports no notifications

  Scenario: Historical activity is not replayed on first observation
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request already has a comment by "alice"
    When  passuff runs one check
    Then  the transcript reports "Fresh baseline recorded"
    And  the transcript reports no notifications

  @pr-activity-notification
  Scenario: A pull request first seen after the first cycle records a baseline
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  an authored pull request "acme/new#2" by "octo-personal" appears with an existing comment by "alice"
    And  passuff runs one check
    Then  the transcript reports "Fresh baseline recorded"
    And  the report records a baseline for "acme/new#2"
    And  the transcript reports no notifications

  @pr-activity-notification
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
    And  the configured sound is played once

  @pr-activity-notification
  Scenario Outline: A disabled event family suppresses its notification
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  <setup> for "acme/api#42"
    And  the "<family>" event family is disabled in configuration
    When  <event> on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications
    Examples:
      | family | setup | event |
      | comments | no prior activity exists | "alice" comments "Can you split this?" |
      | comments | no prior activity exists | "alice" writes a review comment "Needs a test" |
      | reviews | no prior activity exists | "alice" submits an "approved" review |
      | ci_failure | the CI result is "successful" | the CI result becomes "failing" |
      | ci_recovery | the CI result is "failing" | the CI result becomes "successful" |
      | state_changes | the pull request is "open" | the pull request becomes "merged" |

  @pr-activity-notification
  Scenario: A configured sound is used
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the sound is configured to "crates/passuff-cli/assets/notification.wav"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "New comment from @alice"
    And  the configured sound file "crates/passuff-cli/assets/notification.wav" is played once

  @pr-activity-notification
  Scenario: A disabled sound produces no sound
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  sound is disabled in configuration
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "New comment from @alice"
    And  the desktop notification is recorded
    And  no sound is played

  @pr-activity-notification
  Scenario: A disabled desktop notification keeps the transcript
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the desktop notification is disabled in configuration
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "New comment from @alice"
    And  no desktop notification is recorded
    And  the configured sound is played once
    And  the exit status is 0

  @pr-activity-notification
  Scenario: An unavailable desktop surface keeps the transcript
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the desktop notification surface is unavailable
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "New comment from @alice"
    And  the transcript reports "desktop notification failed"
    And  the exit status is 0

  @pr-activity-notification
  Scenario: A missing sound player still delivers the notification
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the sound player is unavailable
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "New comment from @alice"
    And  the desktop notification is recorded
    And  the transcript reports "sound unavailable"
    And  the exit status is 0

  @pr-activity-notification
  Scenario: Each configured account is observed
    Given  a configured account "personal" with login "octo-personal"
    And  a configured account "work" with login "octo-work"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "octo-work"
    And  the watcher has already observed "acme/api#42" and "acme/web#7"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  "bob" comments "Please update the changelog" on "acme/web#7"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"
    And  the transcript reports a notification for "acme/web#7"

  @pr-activity-notification
  Scenario: A comment from another configured account stays silent
    Given  a configured account "personal" with login "octo-personal"
    And  a configured account "work" with login "octo-work"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "octo-work" comments "I will rerun CI" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications
    When  the developer lists all activity
    Then  the transcript reports "New comment from @octo-work"
    And  the activity "New comment from @octo-work" is read

  @pr-activity-notification
  Scenario: A failed account does not stop the other account
    Given  a configured account "work" with an invalid credential
    And  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "account work failed"
    And  the transcript reports a notification for "acme/api#42"
    And  the exit status is 1

  @pr-activity-notification
  Scenario: A restart does not replay notified activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"
    When  passuff restarts and runs one check
    Then  the transcript reports no notifications

  @pr-activity-notification
  Scenario: A corrupt state file is replaced with a baseline
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request already has a comment by "alice"
    And  a corrupt persisted observation
    When  passuff runs one check
    Then  the transcript reports "Fresh baseline recorded"
    And  the transcript reports no notifications

  @pr-activity-notification
  Scenario: An invalid configuration is rejected
    Given  an invalid configuration
    When  passuff runs one check
    Then  the transcript reports "configuration error"
    And  the exit status is 2

  @pr-activity-notification
  Scenario: A state write failure is reported with a nonzero status
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the state file cannot be written
    When  passuff runs one check
    Then  the transcript reports "state error"
    And  the exit status is 3

  @pr-activity-notification
  Scenario Outline: A non-notified change stays silent
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  a "<change>" change occurs on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports no notifications
    Examples:
      | change |
      | label |
      | draft |
      | assignee |
      | milestone |
      | base branch |

  @pr-activity-notification
  Scenario: No authored pull requests produce no notifications
    Given  a configured account "personal" with login "octo-personal"
    And  the account has no authored pull requests
    When  passuff runs one check
    Then  the transcript reports no notifications
    And  the exit status is 0

  @pr-activity-notification
  Scenario: A single check with no changes reports no notifications
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  passuff runs one check
    Then  the transcript reports no notifications
    And  no desktop notification is recorded
    And  no sound is played
    And  the exit status is 0

  @pr-activity-notification
  Scenario: A gh OAuth account uses the existing gh session
    Given  a configured account "work" authenticating through gh as "octo-work"
    And  gh is authenticated as "octo-work"
    And  an authored pull request "acme/api#42" by "octo-work"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario: An interactive watch starts a browser login when the gh session is missing
    Given  a configured account "work" authenticating through gh as "octo-work"
    And  gh is not authenticated as "octo-work"
    And  the terminal is interactive
    When  the watcher starts interactively
    Then  gh started a browser login for "octo-work"
    And  the transcript reports "passuff: watching"

  @pr-activity-notification
  Scenario: A one-shot check never starts a browser login
    Given  a configured account "work" authenticating through gh as "octo-work"
    And  gh is not authenticated as "octo-work"
    And  the terminal is interactive
    When  passuff runs one check
    Then  gh did not start a browser login
    And  the transcript reports "account work failed"
    And  the exit status is 1

  @pr-activity-notification
  Scenario: A non-interactive watch never starts a browser login
    Given  a configured account "work" authenticating through gh as "octo-work"
    And  gh is not authenticated as "octo-work"
    And  the terminal is not interactive
    When  the watcher runs one observation cycle
    Then  gh did not start a browser login
    And  the transcript reports "account work failed"

  @pr-activity-notification
  Scenario: A failed browser login is attempted once per account
    Given  a configured account "work" authenticating through gh as "octo-work"
    And  gh is not authenticated as "octo-work"
    And  a configured account "home" authenticating through gh as "octo-home"
    And  gh is not authenticated as "octo-home"
    And  the terminal is interactive
    And  gh login always fails
    When  the watcher runs two observation cycles
    Then  gh started one browser login for "octo-work"
    And  gh started one browser login for "octo-home"
    And  the transcript reports "account work failed"
    And  the transcript reports "account home failed"
    And  the transcript reports "gh auth login failed"

  @pr-activity-notification
  Scenario: A failed browser login does not stop the other account
    Given  a configured account "alpha" authenticating through gh as "octo-work"
    And  gh is not authenticated as "octo-work"
    And  the terminal is interactive
    And  gh login always fails
    And  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the watcher runs one observation cycle
    Then  the transcript reports "account alpha failed"
    And  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario Outline: A token file readable only by the operator is used
    Given  a configured account "work" with a personal access token in a file with mode "<mode>"
    And  an authored pull request "acme/api#42" by "octo-work"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"
    Examples:
      | mode |
      | 600 |
      | 400 |

  @pr-activity-notification
  Scenario Outline: A token file readable by other local users is refused
    Given  a configured account "work" with a personal access token in a file with mode "<mode>"
    When  passuff runs one check
    Then  the transcript reports "token_file is not private"
    And  the exit status is 2
    Examples:
      | mode |
      | 640 |
      | 604 |
      | 660 |

  @pr-activity-notification
  Scenario: A missing token file is an account failure
    Given  a configured account "work" with a personal access token file that does not exist
    When  passuff runs one check
    Then  the transcript reports "account work failed"
    And  the transcript reports "token file is missing: "
    And  the exit status is 1

  @pr-activity-notification
  Scenario Outline: An unusable token file is reported
    Given  a configured account "work" with <case>
    When  passuff runs one check
    Then  the transcript reports <message>
    And  the exit status is <status>
    Examples:
      | case | message | status |
      | a relative token file path | "must be absolute or start with ~" | 2 |
      | a token file path that is a directory | "not a regular file" | 2 |
      | an empty token file | "token file is empty" | 1 |

  @pr-activity-notification
  Scenario: A token file path starting with ~ is expanded
    Given  a configured account "work" with a personal access token in "~/work.token"
    And  an authored pull request "acme/api#42" by "octo-work"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario: A personal access token for a different login is an account failure
    Given  a configured account "personal" with login "octo-personal"
    And  a configured account "work" with an expected login "octo-work" and a personal access token belonging to "octo-work"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/work#7" by "octo-work"
    And  the watcher has already observed "acme/api#42" and "acme/work#7"
    When  "octo-other" comments "I am someone else" on "acme/api#42"
    And  "alice" comments "Please rebase" on "acme/work#7"
    And  the credential for "work" becomes a token belonging to "octo-other"
    And  passuff runs one check
    Then  the transcript reports "credential belongs to @octo-other, expected @octo-work"
    And  the transcript reports a notification for "acme/api#42"
    And  the transcript reports no notification for "acme/work#7"
    And  the exit status is 1

  @pr-activity-notification
  Scenario: A personal access token whose login differs only in case is accepted
    Given  a configured account "work" with an expected login "octo-work" and a personal access token belonging to "octo-WORK"
    And  an authored pull request "acme/api#42" by "octo-WORK"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario: A credential value never appears in the transcript or state
    Given  a configured account "work" with login "octo-work"
    And  an authored pull request "acme/api#42" by "octo-work"
    When  passuff runs one check
    Then  the transcript reports "Fresh baseline recorded"
    And  the credential value "token-TOKEN_WORK" appears in neither the transcript nor the state file

  @pr-activity-notification
  Scenario: A legacy environment credential remains a personal access token account
    Given  a configured account "work" with an environment credential
    And  an authored pull request "acme/api#42" by "octo-work"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario: A legacy command credential remains a personal access token account
    Given  a configured account "work" with a command credential
    And  an authored pull request "acme/api#42" by "octo-work"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @pr-activity-notification
  Scenario Outline: An invalid credential configuration is rejected
    Given  a configured account "work" with <invalid>
    When  passuff runs one check
    Then  the transcript reports <message>
    And  the exit status is 2
    Examples:
      | invalid | message |
      | no credential source | "must select exactly one credential source" |
      | an unknown auth mode | "unknown auth mode" |
      | two personal access token sources | "must select exactly one credential source" |
      | gh OAuth but no expected login | "requires github_user" |
      | gh OAuth and a personal access token source | "must not set a personal access token source" |
      | a literal personal access token | "must not contain a literal token" |

  @pr-activity-notification
  Scenario: A repository outside the watch scope is not watched
    Given  the watch scope is limited to "acme/api"
    And  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "octo-personal"
    And  a state already watches "acme/web#7"
    When  "alice" comments "Please update the changelog" on "acme/web#7"
    And  passuff runs one check
    Then  the transcript reports no notification for "acme/web#7"
    When  the developer asks for status
    Then  the transcript reports "acme/api#42 open CI passing"
    And  the transcript does not report "acme/web#7"
