@activity-inbox
Feature: Activity inbox

  @activity-inbox
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
    And  the inbox shows an activity id and observation time
    And  the transcript does not report "Open"
    And  the transcript does not report "Success"
    And  the transcript does not report "Can you split this migration?"

  @activity-inbox
  Scenario: Inbox with all includes read activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer marks the newest activity read
    And  the developer lists all activity
    Then  the inbox shows "New comment from @alice" for "acme/api#42" as read

  @activity-inbox
  Scenario: Read marks one activity read
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer marks the newest activity read
    Then  the transcript reports "1 activity record read"
    When  the developer asks for status
    Then  the transcript reports "0 unread of 1"

  @activity-inbox
  Scenario: Read all marks every activity read
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer marks all activity read
    Then  the transcript reports "2 activity records read"
    When  the developer asks for status
    Then  the transcript reports "0 unread of 2"

  @activity-inbox
  Scenario: Read marks only the named activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer marks the newest activity read
    Then  the transcript reports "1 activity record read"
    When  the developer asks for status
    Then  the transcript reports "1 unread of 2"
    When  the developer lists all activity
    Then  the inbox shows "CI failing" for "acme/api#42" as read
    And  the inbox shows "New comment from @alice" for "acme/api#42" as unread

  @activity-inbox
  Scenario: Read refuses an unknown activity id
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer marks activity "a999" read
    Then  the transcript reports "no activity with id a999"
    And  the exit status is 2

  @activity-inbox
  Scenario: Read state survives a restart
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer marks the newest activity read
    And  the developer reopens passuff
    And  the developer asks for status
    Then  the transcript reports "0 unread of 1"

  @activity-inbox
  Scenario: Activity history keeps the 500 most recent records
    Given  a configured account "personal" with login "octo-personal"
    And  a state with 500 activity records
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "One more" on "acme/api#42"
    And  passuff runs one check
    Then  the state retains 500 activity records
    And  the oldest activity record is gone

  @activity-inbox
  Scenario: Recording activity survives a delivery failure
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the desktop notification surface is unavailable
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer asks for status
    Then  the transcript reports "1 unread of 1"

  @activity-inbox
  Scenario: Inbox with no state shows no activity
    Given  a configured account "personal" with login "octo-personal"
    And  no state file
    When  the developer lists all activity
    Then  the transcript reports "no activity"
    And  no state file was created
    And  the exit status is 0

  @activity-inbox
  Scenario: A state write failure is reported
    Given  a configured account "personal" with login "octo-personal"
    And  a version-1 state file with one watched pull request
    And  the state directory cannot be written
    When  the developer asks for status
    Then  the transcript reports "state error"
    And  the exit status is 3

  @activity-inbox
  Scenario: Inbox reports a state error
    Given  a configured account "personal" with login "octo-personal"
    And  a version-1 state file with one watched pull request
    And  the state directory cannot be written
    When  the developer lists all activity
    Then  the transcript reports "state error"
    And  the exit status is 3

  @activity-inbox
  Scenario: Read reports a state error
    Given  a configured account "personal" with login "octo-personal"
    And  a state with one activity record
    And  the state directory cannot be written
    When  the developer marks the newest activity read
    Then  the transcript reports "state error"
    And  the exit status is 3

  @activity-inbox
  Scenario: Read requires an activity id or all
    Given  a configured account "personal" with login "octo-personal"
    When  the developer runs read with no target
    Then  the transcript reports "read requires exactly one"
    And  the exit status is 2

  @activity-inbox
  Scenario: A version-1 state loads with empty history
    Given  a configured account "personal" with login "octo-personal"
    And  a version-1 state file with one watched pull request
    When  the developer lists all activity
    Then  the transcript reports "no activity"
    And  the state file version is 2
    When  the developer asks for status
    Then  the transcript reports "acme/api#42 open CI passing"

  @e2e @activity-inbox
  Scenario: Developer lists unread activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer lists the inbox
    Then  the transcript reports "New comment from @alice"
    And  the transcript reports "acme/api#42"
    And  the exit status is 0

  @e2e @activity-inbox
  Scenario: Developer marks activity read
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer marks all activity read
    Then  the transcript reports "1 activity record read"
    When  the developer lists the inbox
    Then  the transcript reports "no unread activity"
    And  the exit status is 0

  @activity-inbox
  Scenario: The terminal UI shows the inbox
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "1 unread of 1"
    And  the TUI shows "personal @octo-personal"
    And  the TUI shows "watch scope: all authored pull requests"
    And  the TUI shows "Pull request acme/api#42"
    And  the TUI shows "New comment from @alice"
    And  the TUI does not show "Open"
    And  the TUI does not show "Success"
    And  the TUI does not show "Can you split this migration?"
    And  the TUI does not show "token-TOKEN_PERSONAL"

  @activity-inbox
  Scenario: The terminal UI shows the empty inbox
    Given  a configured account "personal" with login "octo-personal"
    And  no state file
    When  the developer opens the inbox TUI
    And  the developer presses "r"
    Then  the TUI shows "no activity yet"
    And  the TUI shows "0 unread of 0"

  @activity-inbox
  Scenario: The selection moves and clamps
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "down"
    Then  the selected activity is "New comment from @alice"
    When  the developer presses "up"
    Then  the selected activity is "CI failing: build"
    When  the developer presses "j"
    Then  the selected activity is "New comment from @alice"
    When  the developer presses "j"
    Then  the selected activity is "New comment from @alice"
    When  the developer presses "k"
    Then  the selected activity is "CI failing: build"

  @activity-inbox
  Scenario: The selected activity is marked read
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "r"
    Then  the TUI reports "1 unread of 2"
    And  the activity "CI failing: build" is read
    And  the activity "New comment from @alice" is unread

  @activity-inbox
  Scenario: Every activity is marked read
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "a"
    Then  the TUI reports "0 unread of 2"

  @activity-inbox
  Scenario Outline: The terminal UI quits
    Given  a configured account "personal" with login "octo-personal"
    And  no state file
    When  the developer opens the inbox TUI
    And  the developer presses "<key>"
    Then  the terminal UI is closed
    Examples:
      | key |
      | q |
      | esc |

  @activity-inbox
  Scenario: The terminal UI reports a configuration error
    Given  a configured account "work" with no credential source
    When  the developer starts the inbox TUI binary
    Then  the TUI reports "configuration error"
    And  the exit status is 2

  @activity-inbox
  Scenario: The terminal UI reports a state error
    Given  a configured account "personal" with login "octo-personal"
    And  the state path is a directory
    When  the developer starts the inbox TUI binary
    Then  the TUI reports "state error"
    And  the exit status is 3

  @activity-inbox
  Scenario: The terminal UI requires a terminal
    Given  a configured account "personal" with login "octo-personal"
    And  no state file
    When  the developer starts the inbox TUI without a snapshot
    Then  the TUI reports "requires a terminal"
    And  the exit status is 3

  @e2e @activity-inbox
  Scenario: Developer opens the inbox TUI
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the developer opens the inbox TUI observing the account
    Then  the TUI shows "1 unread of 1"
    And  the TUI shows "New comment from @alice"
    And  the exit status is 0
