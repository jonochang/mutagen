Feature: Sharding

  Scenario: Shard distributes mutations across shards
    Given 100 mutations
    When I shard into 4 with index 1
    Then I should have between 20 and 30 remaining mutations

  Scenario: All shards together cover all mutations
    Given 50 mutations
    When I collect all 4 shards
    Then all 50 mutations should be covered

  Scenario: Single shard returns all
    Given 10 mutations
    When I shard into 1 with index 1
    Then I should have 10 remaining mutations
