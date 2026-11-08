@inbox-navigation
Feature: Inbox navigation

  @e2e @inbox-navigation
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
    And  the exit status is 0

  @inbox-navigation
  Scenario: The focus cycles between watched pull requests and activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    Then  the TUI shows "Focus: PRs"
    When  the developer presses "tab"
    Then  the TUI shows "Focus: activity"

  @inbox-navigation
  Scenario: The watched pull request selection scrolls
    Given  a configured account "personal" with login "octo-personal"
    And  25 authored pull requests in "acme/api" by "octo-personal"
    And  the watcher has already observed all of them
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer moves the selection down 24 times
    Then  the TUI shows "▶ acme/api#25"
    And  the TUI shows "Focus: PRs"

  @inbox-navigation
  Scenario: A long pull request key keeps its number visible
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "very-long-organization/very-long-repository#1234" by "octo-personal"
    And  the watcher has already observed "very-long-organization/very-long-repository#1234"
    When  the developer opens the inbox TUI
    Then  the TUI shows "…repository#1234"

  @inbox-navigation
  Scenario: The watched table shows the creator and the assignee
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" is assigned to "alice"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    Then  the TUI shows "☺"
    And  the TUI shows "@octo-personal → @alice"

  @inbox-navigation
  Scenario: A stack is marked with a symbol and its position
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  the developer opens the inbox TUI
    Then  the TUI shows "☰"
    And  the TUI shows "1/2"

  @inbox-navigation
  Scenario: A standalone pull request shows no stack symbol
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    Then  the TUI does not show "☰"

  @inbox-navigation
  Scenario: The developer jumps to the ends of the watched list
    Given  a configured account "personal" with login "octo-personal"
    And  5 authored pull requests in "acme/api" by "octo-personal"
    And  the watcher has already observed all of them
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "end"
    Then  the selected pull request is "acme/api#5"
    When  the developer presses "home"
    Then  the selected pull request is "acme/api#1"

  @inbox-navigation
  Scenario: The developer sorts the watched list
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "s"
    Then  the TUI shows "sort: name ↑"
    When  the developer presses "S"
    Then  the TUI shows "sort: name ↓"

  @inbox-navigation
  Scenario: The developer filters the activity to a pull request
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "f"
    Then  the TUI shows "Activity · acme/api#42"
    And  the TUI shows "1 unread of 1"
    And  the TUI shows "(filtered from 2)"
    When  the developer presses "c"
    Then  the TUI shows "2 unread of 2"
    And  the TUI does not show "(filtered from"

  @inbox-navigation
  Scenario: The developer filters the activity to a stack
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    And  GitHub reports stack 7 for "acme/api#42" at position 1 of 2 with base "main"
    And  GitHub reports stack 7 for "acme/api#43" at position 2 of 2 with base "main"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "F"
    Then  the TUI shows "Activity · acme/api stack 7"
    And  the TUI shows "2 unread of 2"

  @inbox-navigation
  Scenario: The developer filters the activity to unread records
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "r"
    And  the developer presses "u"
    Then  the TUI shows "Activity · unread"
    And  the TUI shows "1 unread of 1"
    And  the TUI shows "(filtered from 2)"
    When  the developer presses "c"
    Then  the TUI shows "1 unread of 2"
    And  the TUI does not show "(filtered from"

  @inbox-navigation
  Scenario: Marking all read applies to the filtered activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  "alice" comments "Review this" on "acme/api#42"
    And  "bob" comments "Review that" on "acme/api#43"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "f"
    And  the developer presses "tab"
    And  the developer presses "a"
    Then  the activity "New comment from @alice" is read
    And  the activity "New comment from @bob" is unread

  @inbox-navigation
  Scenario: The developer closes dialogs with q
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Review this" on "acme/api#42"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "enter"
    Then  the TUI shows "open in browser"
    When  the developer presses "q"
    Then  the TUI does not show "open in browser"
    When  the developer presses "tab"
    And  the developer presses "enter"
    Then  the TUI shows "Creator:"
    When  the developer presses "q"
    Then  the TUI does not show "Creator:"
    When  the developer presses "h"
    Then  the TUI shows "Pull request pane"
    When  the developer presses "q"
    Then  the TUI does not show "Pull request pane"
    When  the developer presses "comma"
    Then  the TUI shows "Desktop notifications"
    When  the developer presses "q"
    Then  the TUI does not show "Desktop notifications"

  @inbox-navigation
  Scenario: The developer mutes notifications from the settings dialog
    Given  a configured account "personal" with login "octo-personal"
    When  the developer opens the inbox TUI
    And  the developer presses "comma"
    Then  the TUI shows "Settings"
    And  the TUI shows "Desktop notifications"
    And  the TUI shows "Sound"
    When  the developer presses "j"
    And  the developer presses "space"
    Then  the notification sound is muted
    And  the desktop notification is enabled
    When  the developer presses "esc"
    Then  the TUI does not show "Desktop notifications"

  @inbox-navigation
  Scenario: The developer opens the key reference
    Given  a configured account "personal" with login "octo-personal"
    When  the developer opens the inbox TUI
    And  the developer presses "h"
    Then  the TUI shows "Activity pane"
    And  the TUI shows "mark visible read"
    When  the developer presses "esc"
    Then  the TUI does not show "Activity pane"

  @inbox-navigation
  Scenario: The pull request detail shows its metadata
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" has the description "Adds the live inbox"
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

  @inbox-navigation
  Scenario: The activity detail shows and scrolls a long comment
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" receives a long comment from "alice"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    And  the developer presses "enter"
    Then  the TUI shows "Comment by @alice"
    And  the TUI shows "comment line 1"
    And  the TUI does not show "comment line 40"
    When  the developer presses "end"
    Then  the TUI shows "comment line 40"
    When  the developer presses "home"
    Then  the TUI shows "comment line 1"
    And  the TUI does not show "comment line 40"

  @inbox-navigation
  Scenario: The developer scrolls the pull request detail
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" has a long description
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "enter"
    Then  the TUI shows "line 1"
    And  the TUI does not show "line 40"
    When  the developer presses "end"
    Then  the TUI shows "line 40"
    When  the developer presses "home"
    Then  the TUI shows "line 1"
    And  the TUI does not show "line 40"

  @inbox-navigation
  Scenario: The pull request description keeps its paragraphs
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" has a multi-line description
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "enter"
    Then  the TUI shows "Description:"
    And  the TUI shows "Summary"
    And  the TUI shows "Second paragraph"
    And  the TUI does not show "Summary First paragraph"

  @inbox-navigation
  Scenario: The pull request description renders a Markdown subset
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the pull request "acme/api#42" has a markdown description
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "enter"
    Then  the TUI shows "Summary"
    And  the TUI shows "• first item"
    And  the TUI shows "bold text"
    And  the TUI does not show "## Summary"
    And  the TUI does not show "**bold**"

  @inbox-navigation
  Scenario: The developer ignores and restores a pull request
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "x"
    Then  the TUI shows "acme/api#42"
    And  the TUI shows "⊘"
    And  the TUI shows "x unignore"
    When  the developer presses "right"
    Then  the TUI shows "acme/api#42"
    When  the developer presses "x"
    Then  the TUI shows "x ignore"
    And  the TUI does not show "⊘"
    When  the developer presses "left"
    Then  the TUI shows "acme/api#42"

  @inbox-navigation
  Scenario: A refresh preserves the selected pull request
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/api#43"
    When  the developer opens the inbox TUI
    And  the developer presses "tab"
    And  the developer presses "j"
    Then  the selected pull request is "acme/api#43"
    When  an authored pull request "acme/api#44" by "octo-personal" appears
    And  the developer refreshes the inbox
    Then  the selected pull request is "acme/api#43"
    And  the TUI shows "acme/api#44"

  @e2e @inbox-navigation
  Scenario: The inbox shows activity observed while it is open
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  "alice" comments "Review this" on "acme/api#42" after the first observation
    When  the developer opens the inbox TUI and lets it observe for "4" seconds
    Then  the TUI shows "comment by @alice"
    And  the TUI shows "1 unread of 1"
    And  the exit status is 0

  @inbox-navigation
  Scenario: The inbox keeps observation cheap while nothing changes
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer opens the inbox TUI
    And  the inbox observes the account three times with no change
    Then  the inbox has asked GitHub for the full account history "1" time
    And  the inbox has asked GitHub for a change signal "3" times

  @inbox-navigation
  Scenario: The inbox paces observation against the account allowance
    Given  a configured account "personal" with login "octo-personal"
    And  the account has "100" API points left until the allowance resets in "3600" seconds
    When  the inbox plans its next observation
    Then  the inbox waits at least "30" seconds before observing again

  @inbox-navigation
  Scenario: A failed observation keeps the last inbox view
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Review this" on "acme/api#42"
    And  the developer opens the inbox TUI
    And  the inbox observes the account
    And  GitHub becomes unavailable for the next observation
    And  the inbox observes the account
    Then  the TUI still shows "comment by @alice"
    And  the exit status is 0

  @inbox-navigation
  Scenario: The inbox observes only the open pull request that changed
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/api#43" by "octo-personal"
    And  the watcher has already observed "acme/api#42" and "acme/api#43"
    When  the developer opens the inbox TUI
    And  the inbox observes the account
    And  "alice" comments "Review this" on "acme/api#42"
    And  the inbox observes the account
    Then  the TUI shows "comment by @alice"
    And  the inbox has observed only "acme/api#42"

  @inbox-navigation
  Scenario: The inbox reconciles open pull requests when the interval elapses
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "alice"
    And  the pull request "acme/api#42" is assigned to me
    And  the watcher has already observed "acme/api#42"
    And  the reconciliation interval is "1" seconds
    When  the developer opens the inbox TUI
    And  the inbox observes the account
    And  "bob" comments "Can you review this?" on "acme/api#42"
    And  the inbox waits "2" seconds
    And  the inbox observes the account
    Then  the TUI shows "comment by @bob"

  @inbox-navigation
  Scenario: The activity table shows when the activity happened
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42" at "2026-11-02T09:15:00Z"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "11-02 09:15"

  @inbox-navigation
  Scenario: The activity table marks a CI failure as approximate
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI marks "CI failing" on "acme/api#42" as approximate

  @inbox-navigation
  Scenario: The activity detail shows when the activity happened and when it was fetched
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  "alice" comments "Can you split this migration?" on "acme/api#42" at "2026-11-02T09:15:00Z"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "When: 11-02 09:15"
    And  the TUI shows "Fetched: 11-02 12:00"

  @inbox-navigation
  Scenario: The activity detail marks an activity without a reported time as approximate
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the CI result on "acme/api#42" becomes "failing"
    And  passuff runs one check
    And  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "When: ~11-02 12:00"
    And  the TUI shows "(fetch time)"

  @inbox-navigation
  Scenario: The watched table hides a pull request outside the focus
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "bob"
    And  a team review is requested on "acme/web#7"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the focus relations are "authored"
    When  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI does not show "acme/web#7"

  @inbox-navigation
  Scenario: Changing the focus in the picker filters the inbox immediately
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "bob"
    And  a team review is requested on "acme/web#7"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the involvement relations are unknown
    When  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI shows "acme/web#7"
    When  the developer opens the live scope picker
    And  the developer presses "space" in the scope picker
    And  the developer presses "esc" in the scope picker
    Then  the TUI does not show "acme/api#42"
    And  the TUI shows "acme/web#7"

  @inbox-navigation
  Scenario: The activity list row is bounded to the recorded excerpt
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" receives a long comment from "alice"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "Comment by @alice"
    And  the activity list row excerpt is at most 80 characters
    And  the TUI does not show "comment line 40"

  @inbox-navigation
  Scenario: The stored activity body is bounded to 4 KiB
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the pull request "acme/api#42" receives a 6000-character comment from "alice"
    And  passuff runs one check
    Then  the stored activity body is at most 4096 bytes
    When  the developer opens the inbox TUI with keys "enter"
    Then  the TUI shows "body line 1"
    When  the developer presses "end"
    Then  the TUI does not show "TAIL MARKER"

  @inbox-navigation
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
    And  the exit status is 0
