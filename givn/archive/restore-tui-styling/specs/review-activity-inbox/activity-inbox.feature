@givn.delta @activity-inbox

Feature: Terminal inbox styling

  @givn.modified @e2e
  Scenario: Developer opens the inbox TUI
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  the developer opens the inbox TUI observing the account
    Then  the TUI shows "1 unread of 1"
    And  the unread count is gold and bold
    And  the focus label is dim
    And  the focused pane name is cyan-blue and bold
    And  the TUI shows "comment by @alice"
    And  the TUI shows "passuff — Inbox"
    And  the outer frame border is dim
    And  the frame title is bright and bold
    And  the frame title suffix is dim
    And  the account login is cyan-blue
    And  the watched table headers are cyan-blue and bold
    And  the watched table shows the people column
    And  the activity table headers are cyan-blue and bold
    And  the focused pane title is green and bold
    And  the selected row is gold and bold
    And  the activity body is bright gray
    And  the controls bar title is cyan-blue
    And  the key hints are cyan-blue and bold
    And  the exit status is 0

  @givn.modified @e2e
  Scenario: Developer opens an activity's details
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "Activity api#42"
    And  the TUI shows "comment by @alice"
    And  the TUI shows "Can you split this migration?"
    And  the TUI shows "acme/api#42"
    And  the TUI shows "Pull request acme/api#42"
    And  the TUI shows "unread"
    And  the TUI shows "Enter open in browser"
    And  the detail overlay border is dim
    And  the detail overlay title is cyan-blue and bold
    And  the metadata labels are cyan-blue
    And  the metadata values are dim
    And  the detail separator uses the rule color
    And  the activity body is bright gray
    And  the exit status is 0

  @givn.added
  Scenario: An ignored row is dim and marked
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the pull request "acme/api#42" is ignored
    When  the developer opens the inbox TUI
    Then  the TUI shows "⊘"
    And  the ignored row is dim
    And  the exit status is 0

  @givn.added
  Scenario: The empty inbox uses the primary text
    Given  a configured account "personal" with login "octo-personal"
    When  the developer opens the inbox TUI
    Then  the TUI shows "no watched pull requests"
    And  the TUI shows "no activity yet"
    And  the empty state text is bright
    And  the exit status is 0
