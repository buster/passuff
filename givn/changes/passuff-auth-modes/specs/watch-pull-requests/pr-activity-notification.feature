@givn.delta @pr-activity-notification

Feature: Pull request activity notification

  @givn.added
  Scenario: A gh OAuth account uses the existing gh session
    Given a configured account "work" authenticating through gh as "octo-work"
    And gh is authenticated as "octo-work"
    And an authored pull request "acme/api#42" by "octo-work"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario: An interactive watch starts a browser login when the gh session is missing
    Given a configured account "work" authenticating through gh as "octo-work"
    And gh is not authenticated as "octo-work"
    And the terminal is interactive
    When the watcher starts interactively
    Then gh started a browser login for "octo-work"
    And the transcript reports "passuff: watching"

  @givn.added
  Scenario: A one-shot check never starts a browser login
    Given a configured account "work" authenticating through gh as "octo-work"
    And gh is not authenticated as "octo-work"
    And the terminal is interactive
    When passuff runs one check
    Then gh did not start a browser login
    And the transcript reports "account work failed"
    And the exit status is 1

  @givn.added
  Scenario: A non-interactive watch never starts a browser login
    Given a configured account "work" authenticating through gh as "octo-work"
    And gh is not authenticated as "octo-work"
    And the terminal is not interactive
    When the watcher runs one observation cycle
    Then gh did not start a browser login
    And the transcript reports "account work failed"

  @givn.added
  Scenario: A failed browser login is attempted once per account
    Given a configured account "work" authenticating through gh as "octo-work"
    And gh is not authenticated as "octo-work"
    And a configured account "home" authenticating through gh as "octo-home"
    And gh is not authenticated as "octo-home"
    And the terminal is interactive
    And gh login always fails
    When the watcher runs two observation cycles
    Then gh started one browser login for "octo-work"
    And gh started one browser login for "octo-home"
    And the transcript reports "account work failed"
    And the transcript reports "account home failed"

  @givn.added
  Scenario: A failed browser login does not stop the other account
    Given a configured account "alpha" authenticating through gh as "octo-work"
    And gh is not authenticated as "octo-work"
    And the terminal is interactive
    And gh login always fails
    And a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And the watcher runs one observation cycle
    Then the transcript reports "account alpha failed"
    And the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario Outline: A token file readable only by the operator is used
    Given a configured account "work" with a personal access token in a file with mode "<mode>"
    And an authored pull request "acme/api#42" by "octo-work"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"

    Examples:
      | mode |
      | 600  |
      | 400  |

  @givn.added
  Scenario Outline: A token file readable by other local users is refused
    Given a configured account "work" with a personal access token in a file with mode "<mode>"
    When passuff runs one check
    Then the transcript reports "token_file is not private"
    And the exit status is 2

    Examples:
      | mode |
      | 640  |
      | 604  |
      | 660  |

  @givn.added
  Scenario: A missing token file is an account failure
    Given a configured account "work" with a personal access token file that does not exist
    When passuff runs one check
    Then the transcript reports "account work failed"
    And the exit status is 1

  @givn.added
  Scenario: A token file path starting with ~ is expanded
    Given a configured account "work" with a personal access token in "~/work.token"
    And an authored pull request "acme/api#42" by "octo-work"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario: A personal access token for a different login is an account failure
    Given a configured account "personal" with login "octo-personal"
    And a configured account "work" with an expected login "octo-work" and a personal access token belonging to "octo-work"
    And an authored pull request "acme/api#42" by "octo-personal"
    And an authored pull request "acme/work#7" by "octo-work"
    And the watcher has already observed "acme/api#42" and "acme/work#7"
    When "octo-other" comments "I am someone else" on "acme/api#42"
    And "alice" comments "Please rebase" on "acme/work#7"
    And the credential for "work" becomes a token belonging to "octo-other"
    And passuff runs one check
    Then the transcript reports "credential belongs to @octo-other, expected @octo-work"
    And the transcript reports a notification for "acme/api#42"
    And the transcript reports no notification for "acme/work#7"
    And the exit status is 1

  @givn.added
  Scenario: A personal access token whose login differs only in case is accepted
    Given a configured account "work" with an expected login "octo-work" and a personal access token belonging to "octo-WORK"
    And an authored pull request "acme/api#42" by "octo-WORK"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario: A legacy environment credential remains a personal access token account
    Given a configured account "work" with an environment credential
    And an authored pull request "acme/api#42" by "octo-work"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario: A legacy command credential remains a personal access token account
    Given a configured account "work" with a command credential
    And an authored pull request "acme/api#42" by "octo-work"
    And the watcher has already observed "acme/api#42"
    When "alice" comments "Can you split this migration?" on "acme/api#42"
    And passuff runs one check
    Then the transcript reports a notification for "acme/api#42"

  @givn.added
  Scenario Outline: An invalid credential configuration is rejected
    Given a configured account "work" with <invalid>
    When passuff runs one check
    Then the transcript reports <message>
    And the exit status is 2

    Examples:
      | invalid                                     | message                                              |
      | no credential source                        | "must select exactly one credential source"          |
      | an unknown auth mode                        | "unknown auth mode"                                  |
      | two personal access token sources           | "must select exactly one credential source"          |
      | gh OAuth but no expected login               | "requires github_user"                               |
      | gh OAuth and a personal access token source  | "must not set a personal access token source"        |
      | a literal personal access token              | "must not contain a literal token"                   |
