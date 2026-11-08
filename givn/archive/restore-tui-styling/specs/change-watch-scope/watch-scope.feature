@givn.delta @watch-scope

Feature: Watch scope picker styling

  @givn.added
  Scenario: The scope picker renders the watn palette
    Given  a configured account "personal" with login "octo-personal"
    And  an authored pull request "acme/api#42" by "octo-personal"
    And  the watcher has already observed "acme/api#42"
    And  the watch scope mode is "selected"
    And  the watch scope contains "acme/api"
    When  the developer opens the scope picker
    Then  the picker shows "Watch scope"
    And  the picker border is dim
    And  the picker brand is bright and bold
    And  the picker title suffix is dim
    And  the picker mode is cyan-blue and bold
    And  the selected repository is gold
    And  the exit status is 0
