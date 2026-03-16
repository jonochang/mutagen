Feature: Boolean mutations

  Scenario: Mutate logical and to logical or
    Given a Ruby source "a && b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "&&" with "||"

  Scenario: Mutate logical or to logical and
    Given a Ruby source "a || b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "||" with "&&"

  Scenario: Mutate true to false
    Given a Ruby source "x = true"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "true" with "false"

  Scenario: Mutate false to true
    Given a Ruby source "x = false"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "false" with "true"
