Feature: Eating Cucumbers (cukes)

  Scenario: Eating 5 Cucumbers
    Given I have 20 cukes
    When I eat 5 cukes
    Then I have 15 cukes left

  @greedy
  Scenario: Eating all the Cucumbers
    Given I have 25 cukes
    When I eat 25 cukes
    Then I have 0 cukes left  
