@watch-status
Feature: Watch status

  @watch-status
  Scenario: Status lists configured accounts before the first check
    Given  a configured account "personal" with login "octo-personal"
    When  the developer asks for status
    Then  the transcript reports "personal not resolved"
    And  the transcript reports "last check never"

  @watch-status
  Scenario: Status shows the login and last check after a check
    Given  a configured account "personal" with login "octo-personal"
    When  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "personal @octo-personal"
    And  the transcript does not report "last check never"

  @watch-status
  Scenario: Status shows the watch scope when all repositories are watched
    Given  a configured account "personal" with login "octo-personal"
    When  the developer asks for status
    Then  the transcript reports "watch scope: all authored pull requests"

  @watch-status
  Scenario: Status shows the watch scope when a repository allowlist is configured
    Given  the watch scope is limited to "acme/api"
    And  a configured account "personal" with login "octo-personal"
    When  the developer asks for status
    Then  the transcript reports "watch scope: acme/api"

  @watch-status
  Scenario: Status lists watched pull requests with their state and CI
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" is "merged"
    And  the watcher has already observed "acme/api#42" with a failing CI result
    When  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "acme/api#42 merged CI failing"
    And  the transcript reports "Pull request acme/api#42"
    And  the transcript does not report "Open"
    And  the transcript does not report "Success"

  @watch-status
  Scenario: Status lists multiple accounts
    Given  a configured account "personal" with login "octo-personal"
    And  a configured account "work" with login "octo-work"
    When  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "personal @octo-personal"
    And  the transcript reports "work @octo-work"

  @watch-status
  Scenario: Status reports unread and total activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "1 unread of 1"

  @watch-status
  Scenario: Status uses human labels for closed and pending
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" is "closed"
    And  the CI result on "acme/api#42" is "pending"
    When  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "acme/api#42 closed CI pending"

  @watch-status
  Scenario Outline: A read command rejects invalid configuration
    Given  a configured account "work" with no credential source
    When  the developer <command>
    Then  the transcript reports "configuration error"
    And  the exit status is 2
    Examples:
      | command |
      | asks for status |
      | lists the inbox |
      | lists all activity |
      | marks all activity read |

  @watch-status
  Scenario: Status with no state shows empty sections
    Given  a configured account "personal" with login "octo-personal"
    And  no state file
    When  the developer asks for status
    Then  the transcript reports "no watched pull requests"
    And  the transcript reports "0 unread of 0"
    And  the exit status is 0

  @e2e @watch-status
  Scenario: Developer sees the watch state
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "personal @octo-personal"
    And  the transcript reports "acme/api#42 open CI passing"
    And  the exit status is 0
