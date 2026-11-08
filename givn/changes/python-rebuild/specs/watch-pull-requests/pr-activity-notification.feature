@givn.delta @pr-activity-notification
Feature: Pull request activity boundary evidence

  @givn.added
  Scenario Outline: An invalid configuration is reported with exit 2
    Given  a configuration with <case>
    When  passuff runs one check
    Then  the transcript reports "configuration error"
    And  the transcript reports "<message>"
    And  the exit status is 2
    Examples:
      | case | message |
      | an unknown key | unknown configuration key |
      | invalid TOML | invalid TOML |
      | a plain HTTP GitHub address outside loopback | plain HTTP |
      | a poll interval below one second | poll_interval_secs |

  @givn.added
  Scenario: A missing configuration is a configuration error
    Given  no configuration file exists
    When  passuff runs one check
    Then  the transcript reports "configuration error"
    And  the exit status is 2

  @givn.added
  Scenario Outline: A GitHub response failure is an account failure
    Given  a configured account "personal" with login "octo-personal"
    And  a configured account "work" with login "octo-work"
    And  an authored pull request "acme/web#7" by "octo-work"
    And  the GitHub response for account "personal" is <case>
    And  the watcher has already observed "acme/web#7"
    When  "alice" comments "Can you review this?" on "acme/web#7"
    And  passuff runs one check
    Then  the transcript reports "account personal failed"
    And  the transcript reports a notification for "acme/web#7"
    And  the exit status is 1
    Examples:
      | case |
      | a malformed JSON body |
      | a partial pull request payload |
      | a timeout |
      | a non-2xx response |
      | a GraphQL error payload |

  @givn.added
  Scenario: A replayed pull request node does not duplicate activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the GitHub response repeats the pull request node for "acme/api#42"
    When  "alice" comments "Only once" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"
    And  the activity list contains one record for "acme/api#42"

  @givn.added
  Scenario: An unknown state version is quarantined and replaced
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  a state file with an unknown version
    When  passuff runs one check
    Then  the transcript reports "Fresh baseline recorded"
    And  the state file version is 2
    And  the exit status is 0

  @givn.added
  Scenario: A quarantine failure is a state error
    Given  a configured account "personal" with login "octo-personal"
    And  a state file with an unknown version
    And  the state directory cannot be written
    When  passuff runs one check
    Then  the transcript reports "state error"
    And  the exit status is 3

  @givn.added
  Scenario: Duplicate activity ids are repaired on load
    Given  a configured account "personal" with login "octo-personal"
    And  a state file with duplicate activity ids
    When  the developer lists all activity
    Then  the transcript lists two activities
    And  the state file contains unique activity ids

  @givn.added
  Scenario: A reopen within the retention window is observed
    Given  a configured account "personal" with login "octo-personal"
    And  a watched pull request "acme/api#42" merged 29 days ago
    When  the pull request "acme/api#42" is reopened
    And  passuff runs one check
    Then  the transcript reports no notification for "acme/api#42"
    When  the pull request "acme/api#42" becomes "closed"
    And  passuff runs one check
    Then  the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario: A reopen after the retention window is not observed
    Given  a configured account "personal" with login "octo-personal"
    And  a watched pull request "acme/api#42" merged 31 days ago
    When  the pull request "acme/api#42" is reopened
    And  passuff runs one check
    When  the pull request "acme/api#42" becomes "closed"
    And  passuff runs one check
    Then  the transcript reports no notification for "acme/api#42"

  @givn.added
  Scenario Outline: A CLI usage error exits 2
    When  the developer runs passuff with "<arguments>"
    Then  the transcript reports "<message>"
    And  the exit status is 2
    Examples:
      | arguments | message |
      | unknown-command | invalid choice |
      | --unknown-option | unrecognized arguments |
      | --config | expected one argument |
      | --focus-set invalid | invalid focus relation |
      | --keys invalid | invalid key |

  @givn.added
  Scenario Outline: A delivery surface failure keeps the transcript
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  <surface> fails
    When  "alice" comments "Can you review this?" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript reports "<message>"
    And  the transcript reports "New comment from @alice"
    And  the exit status is 0
    Examples:
      | surface | message |
      | the desktop notification service | desktop notification failed |
      | the configured sound player | sound playback failed |

  @givn.added
  Scenario Outline: A missing or failing gh is an account failure
    Given  a configured account "work" with login "octo-work"
    And  an authored pull request "acme/web#7" by "octo-work"
    And  <case>
    When  passuff runs one check
    Then  the transcript reports "account work failed"
    And  the exit status is 1
    Examples:
      | case |
      | the gh binary is missing |
      | the gh command fails |
