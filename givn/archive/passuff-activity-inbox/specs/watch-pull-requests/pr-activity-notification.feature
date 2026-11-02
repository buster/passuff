@givn.delta @pr-activity-notification

Feature: Pull request activity notification

  @givn.added
  Scenario: A repository outside the watch scope is not watched
    Given the watch scope is limited to "acme/api"
    And a configured account "personal" with login "octo-personal"
    And an authored pull request "acme/api#42" by "octo-personal"
    And an authored pull request "acme/web#7" by "octo-personal"
    And a state already watches "acme/web#7"
    When "alice" comments "Please update the changelog" on "acme/web#7"
    And passuff runs one check
    Then the transcript reports no notifications
    When the developer asks for status
    Then the transcript reports "acme/api#42 open CI passing"
    And the transcript does not report "acme/web#7"
