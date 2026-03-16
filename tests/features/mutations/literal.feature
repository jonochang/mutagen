Feature: Literal mutations

  Scenario: Mutate integer zero to one
    Given a Ruby source "x = 0"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "0" with "1"

  Scenario: Mutate non-zero integer to zero
    Given a Ruby source "x = 42"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "42" with "0"

  Scenario: Mutate empty string to non-empty
    Given a Ruby source "x = ''"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "''" with "'mutagen'"

  Scenario: Mutate non-empty string to empty
    Given a Ruby source "x = 'hello'"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "'hello'" with "''"

  Scenario: Mutate empty array to non-empty
    Given a Ruby source "x = []"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "[]" with "[nil]"
