@givn.delta @watch-scope
Feature: Watch scope boundary evidence

  @givn.added
  Scenario: Deselecting a repository in all mode is refused
    Given  the watch scope mode is "all"
    When  the developer deselects "acme/api" in the watch scope
    Then  the transcript reports "cannot deselect in all mode"
    And  the exit status is 2

  @givn.added
  Scenario Outline: Duplicate watch scope entries are rejected
    Given  a configuration with watch repositories "<repositories>"
    When  passuff runs one check
    Then  the transcript reports "configuration error"
    And  the transcript reports "duplicate repository"
    And  the exit status is 2
    Examples:
      | repositories |
      | acme/api, acme/api |
      | acme/api, Acme/Api |
