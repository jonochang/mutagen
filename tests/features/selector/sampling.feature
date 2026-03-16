Feature: Random sampling

  Scenario: Sample by fixed count
    Given 100 mutations
    When I sample 25 mutations
    Then I should have 25 remaining mutations

  Scenario: Sample by percentage
    Given 100 mutations
    When I sample 50 percent
    Then I should have 50 remaining mutations

  Scenario: Sample 100 percent keeps all
    Given 100 mutations
    When I sample 100 percent
    Then I should have 100 remaining mutations
