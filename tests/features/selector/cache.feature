Feature: Incremental cache

  Scenario: Cache hit skips unchanged mutation
    Given a cached result for mutation "m1" with status "killed" and source hash "abc123"
    When I check cache for mutation "m1" with source hash "abc123"
    Then the cache should return status "killed"

  Scenario: Cache miss for changed source
    Given a cached result for mutation "m1" with status "killed" and source hash "abc123"
    When I check cache for mutation "m1" with source hash "def456"
    Then the cache should return no result

  Scenario: Cache miss for unknown mutation
    Given an empty cache
    When I check cache for mutation "m1" with source hash "abc123"
    Then the cache should return no result

  Scenario: Round-trip save and load
    Given a cached result for mutation "m1" with status "survived" and source hash "abc123"
    When I save the cache to a file
    And I load the cache from the file
    And I check cache for mutation "m1" with source hash "abc123"
    Then the cache should return status "survived"
