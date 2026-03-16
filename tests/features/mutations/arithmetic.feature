Feature: Arithmetic mutations

  Scenario: Mutate plus to minus
    Given a Ruby source "a + b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "+" with "-"

  Scenario: Mutate minus to plus
    Given a Ruby source "a - b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "-" with "+"

  Scenario: Mutate multiply to divide
    Given a Ruby source "a * b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "*" with "/"

  Scenario: Mutate divide to multiply
    Given a Ruby source "a / b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "/" with "*"

  Scenario: Mutate modulo to multiply
    Given a Ruby source "a % b"
    When I generate mutations
    Then I should see 1 mutation
    And mutation 0 should replace "%" with "*"

  Scenario: No mutations for non-arithmetic method calls
    Given a Ruby source "a.foo(b)"
    When I generate mutations
    Then I should see 0 mutations

  Scenario: Multiple arithmetic operators
    Given a Ruby source "a + b - c"
    When I generate mutations
    Then I should see 2 mutations

  Scenario: Apply arithmetic mutation
    Given a Ruby source "a + b"
    When I generate mutations
    And I apply mutation 0
    Then the mutated source should be "a - b"
