@givn.delta @inbox-navigation

Feature: Inbox pane styling

  @givn.modified @e2e
  Scenario: Developer navigates the inbox panes
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "tab,j"
    Then  the TUI shows "Focus: PRs"
    And  the TUI shows "▶ acme/api#43"
    And  the focused pane border is green
    And  the focused pane title is green and bold
    And  the unfocused pane border is dim
    And  the unfocused pane title is dim
    And  the selected row is gold and bold
    And  the exit status is 0

  @givn.added
  Scenario: The settings overlay follows the palette
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI with keys "comma,j,space"
    Then  the TUI shows "Settings"
    And  the settings overlay border is dim
    And  the settings title is cyan-blue and bold
    And  the settings selected row is gold and bold
    And  the enabled settings values are green
    And  the disabled settings values are dim
    And  the exit status is 0

  @givn.modified
  Scenario: The pull request detail shows its metadata
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" has the description "# Adds the live inbox"
    And  the pull request "acme/api#42" is assigned to "alice"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "enter"
    Then  the TUI shows "Status: open   CI: passing"
    And  the TUI shows "Creator: @octo-personal"
    And  the TUI shows "Assignee: @alice"
    And  the TUI shows "Adds the live inbox"
    And  the TUI shows "CI: passing"
    And  the TUI shows "Pull request acme/api#42"
    And  the pull request detail border is dim
    And  the pull request detail title is cyan-blue and bold
    And  the pull request detail heading is green and bold
    And  the metadata labels are cyan-blue
    And  the metadata values are dim
    And  the exit status is 0

  @givn.modified
  Scenario: The developer opens the key reference
    Given  a configured account "personal" with login "octo-personal"
    When  the developer opens the inbox TUI
    And  the developer presses "h"
    Then  the TUI shows "Activity pane"
    And  the TUI shows "mark visible read"
    And  the help heading is cyan-blue and bold
    And  the help keys are cyan-blue and bold
    When  the developer presses "esc"
    Then  the TUI does not show "Activity pane"
    And  the exit status is 0

  @givn.modified
  Scenario: A missing browser opener is reported and keeps the detail
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42"
    And  passuff runs one check
    And  the browser opener is missing
    And  the developer opens the inbox TUI with keys "enter,enter"
    Then  the TUI shows "Activity api#42"
    And  the TUI shows "could not open browser"
    And  the status line is gold
    And  the exit status is 0

  @givn.added
  Scenario: The 80x24 frame keeps the composition
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI at 80x24
    Then  the TUI shows "passuff — Inbox"
    And  the watched table headers are cyan-blue and bold
    And  the activity table headers are cyan-blue and bold
    And  the focused pane title is green and bold
    And  the exit status is 0

  @givn.added
  Scenario: The developer confirms saving settings on quit
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI with keys "tab,s,q"
    Then  the TUI shows "Save sort and filter settings?"
    And  the save prompt border is dim
    And  the save prompt title is cyan-blue
    And  the save prompt question is bright
    And  the save prompt keys are cyan-blue and bold
    When  the developer presses "y"
    Then  the TUI does not show "Save sort and filter settings?"
    And  the saved sort is "name"
    And  the exit status is 0

  @givn.added
  Scenario: The developer discards settings on quit
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI with keys "tab,s,q"
    Then  the TUI shows "Save sort and filter settings?"
    When  the developer presses "n"
    Then  the TUI does not show "Save sort and filter settings?"
    And  the saved sort is "updated"
    And  the exit status is 0

  @givn.added
  Scenario: The developer cancels the quit from the save prompt
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI with keys "tab,s,q"
    Then  the TUI shows "Save sort and filter settings?"
    When  the developer presses "esc"
    Then  the TUI does not show "Save sort and filter settings?"
    And  the TUI shows "sort: name ↑"
    And  the saved sort is "updated"
    And  the exit status is 0

  @givn.added
  Scenario: The developer quits without a save prompt when settings are unchanged
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI with keys "q"
    Then  the TUI does not show "Save sort and filter settings?"
    And  the exit status is 0
