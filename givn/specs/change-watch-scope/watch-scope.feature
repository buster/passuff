@watch-scope
Feature: Watch scope editing

  @e2e @watch-scope
  Scenario: Developer toggles a repository in the watch scope
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    When  the developer selects "acme/api" in the watch scope
    Then  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    And  the picker shows "Watch scope"
    And  the picker shows "[x] acme/api"
    When  the developer enters "acme/web" as a repository
    Then  the watch scope contains "acme/web"
    When  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    When  the developer deselects "acme/api" in the watch scope
    Then  the watch scope does not contain "acme/api"
    When  the developer opens the inbox TUI
    Then  the TUI does not show "acme/api#42"
    And  the state retains "acme/api#42"

  @watch-scope
  Scenario: The picker shows the known repositories
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    When  the developer opens the scope picker
    Then  the picker shows "Watch scope"
    And  the picker shows "[x] acme/api"
    And  the picker shows "selected"

  @watch-scope
  Scenario: A deselected repository stops observation
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "selected"
    And  the watch scope does not contain "acme/api"
    When  "alice" comments "While away" on "acme/api#42"
    And  passuff runs one check
    Then  the transcript does not report "New comment from @alice"
    And  the state retains "acme/api#42"
    When  the developer opens the inbox TUI
    Then  the TUI does not show "acme/api#42"

  @watch-scope
  Scenario: Adding a repository by owner/name
    Given  a configured account "personal" with login "octo-personal"
    And  the watch scope mode is "selected"
    When  the developer enters "acme/new" as a repository
    Then  the watch scope contains "acme/new"
    And  the watch scope mode is "selected"

  @watch-scope
  Scenario: The watcher reloads the scope without a restart
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    And  a watcher is running with a configuration path
    When  the developer selects "acme/web" in the watch scope
    And  "bob" comments "After reload" on "acme/web#7"
    And  the watcher runs another cycle
    Then  the activity "New comment from @bob" is unread

  @e2e @watch-scope
  Scenario: Developer switches the watch scope mode
    Given  a configured account "personal" with login "octo-personal"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    When  the developer switches the watch scope to "all"
    Then  the watch scope mode is "all"
    When  the developer opens the scope picker
    Then  the picker shows "Mode: all"
    When  the developer opens the inbox TUI
    Then  the TUI shows "watch scope: all authored pull requests"
    When  the developer switches the watch scope to "selected"
    Then  the watch scope contains "acme/api"
    When  the developer opens the scope picker
    Then  the picker shows "Mode: selected"

  @watch-scope
  Scenario: Switching to selected with no retained set observes nothing
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "all"
    When  the developer switches the watch scope to "selected"
    Then  the watch scope mode is "selected"
    And  the watch scope is empty
    And  the watch scope does not contain "acme/api"
    When  the developer opens the inbox TUI
    Then  the TUI shows "watching nothing"

  @watch-scope
  Scenario: The last deselection observes nothing
    Given  a configured account "personal" with login "octo-personal"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    When  the developer deselects "acme/api" in the watch scope
    Then  the watch scope mode is "selected"
    When  the developer asks for status
    Then  the transcript reports "watching nothing"

  @watch-scope
  Scenario: Switching to all observes every repository
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    When  the developer switches the watch scope to "all"
    And  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI shows "acme/web#7"

  @watch-scope
  Scenario: Selecting a repository with no records records a baseline
    Given  a configured account "personal" with login "octo-personal"
    And  the watch scope mode is "selected"
    And  the watch scope does not contain "acme/api"
    And  an authored pull request "acme/api#42" by "octo-personal"
    When  the developer selects "acme/api" in the watch scope
    And  passuff runs one check
    Then  the transcript reports "Fresh baseline recorded for acme/api#42"

  @watch-scope
  Scenario: An invalid repository entry is rejected
    Given  a configured account "personal" with login "octo-personal"
    And  the watch scope mode is "selected"
    When  the developer enters "acme" as a repository
    Then  the TUI reports "owner/repo"
    And  the watch scope does not contain "acme"

  @watch-scope
  Scenario: Reselecting catches up silently
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "selected"
    And  the watch scope does not contain "acme/api"
    And  passuff runs one check
    When  "alice" comments "While you were away" on "acme/api#42"
    And  the developer selects "acme/api" in the watch scope
    And  passuff runs one check
    Then  the activity "New comment from @alice" is unread
    And  the transcript does not report "New comment from @alice"
    When  the developer opens the inbox TUI
    Then  the TUI shows "comment by @alice"

  @watch-scope
  Scenario: A scope write failure keeps the previous scope
    Given  a configured account "personal" with login "octo-personal"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    And  the configuration directory cannot be written
    When  the developer selects "acme/web" in the watch scope
    Then  the TUI reports "cannot write configuration"
    And  the watch scope contains "acme/api"
    And  the watch scope does not contain "acme/web"

  @watch-scope
  Scenario: Deselecting hides status and activity
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "selected"
    And  the watch scope does not contain "acme/api"
    When  "alice" comments "Before deselect" on "acme/api#42"
    And  passuff runs one check
    And  the developer asks for status
    Then  the transcript does not report "acme/api#42"
    When  the developer opens the inbox TUI
    Then  the TUI does not show "comment by @alice"
    And  the state retains "acme/api#42"

  @watch-scope
  Scenario: Two accounts share the global scope
    Given  a configured account "personal" with login "octo-personal"
    And  a configured account "work" with login "octo-work"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  an authored pull request "acme/web#7" by "octo-work"
    And  the watcher has already observed "acme/api#42"
    And  the watcher has already observed "acme/web#7"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    When  passuff runs one check
    And  the developer opens the inbox TUI
    Then  the TUI shows "acme/api#42"
    And  the TUI does not show "acme/web#7"

  @watch-scope
  Scenario: A reload failure keeps the previous scope
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    And  a watcher is running with a configuration path
    When  the configuration file is corrupted
    And  the watcher runs another cycle
    Then  the transcript reports "configuration error"
