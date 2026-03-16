Feature: Coverage-based filtering

  Scenario: Filter out mutations on uncovered lines
    Given mutations at lines 1, 3, 5 in "app.rb"
    And coverage data covering lines 1, 5 in "app.rb"
    When I filter by coverage
    Then I should have 2 remaining mutations
    And the remaining mutations should be at lines 1, 5

  Scenario: No coverage data means no filtering
    Given mutations at lines 1, 3, 5 in "app.rb"
    And no coverage data
    When I filter by coverage
    Then I should have 3 remaining mutations
