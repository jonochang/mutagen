Feature: Comparison mutations

  Scenario: Mutate greater-than to greater-than-or-equal
    Given a Ruby source "a > b"
    When I generate mutations
    Then I should see 2 mutations
    And mutation 0 should replace ">" with ">="
    And mutation 1 should replace ">" with "<"

  Scenario: Mutate less-than to less-than-or-equal
    Given a Ruby source "a < b"
    When I generate mutations
    Then I should see 2 mutations
    And mutation 0 should replace "<" with "<="
    And mutation 1 should replace "<" with ">"

  Scenario: Mutate equal-equal to not-equal
    Given a Ruby source "a == b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "==" with "!="

  Scenario: Mutate not-equal to equal-equal
    Given a Ruby source "a != b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "!=" with "=="

  Scenario: Mutate greater-than-or-equal to greater-than
    Given a Ruby source "a >= b"
    When I generate mutations
    Then I should see 2 mutations
    And mutation 0 should replace ">=" with ">"
    And mutation 1 should replace ">=" with "<="

  Scenario: Mutate less-than-or-equal to less-than
    Given a Ruby source "a <= b"
    When I generate mutations
    Then I should see 2 mutations
    And mutation 0 should replace "<=" with "<"
    And mutation 1 should replace "<=" with ">="
