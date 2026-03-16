Feature: Conditional mutations

  Scenario: Mutate if condition to true
    Given a Ruby source "if x > 0 then y end"
    When I generate mutations with operator "conditional"
    Then I should see a mutation with operator "conditional/condition_to_true"

  Scenario: Mutate if condition to false
    Given a Ruby source "if x > 0 then y end"
    When I generate mutations with operator "conditional"
    Then I should see a mutation with operator "conditional/condition_to_false"

  Scenario: Remove else branch
    Given a Ruby source "if x then y else z end"
    When I generate mutations with operator "conditional"
    Then I should see a mutation with operator "conditional/remove_else"
