Feature: Orders API
  Component tests against the HTTP orders app with an in-memory database.

  Background:
    Given a running orders API

  Scenario: Create an order with a line-item table
    When I create an order for "Alice" with items:
      | sku    | quantity |
      | widget | 2        |
      | gizmo  | 1        |
    Then the response status is 201
    And the order status is "pending"
    When I fetch the created order
    Then the response status is 200
    And the order customer is "Alice"
    And the order has 2 items

  Scenario: List includes created orders
    When I create an order for "Bob" with items:
      | sku   | quantity |
      | gizmo | 1        |
    And I create an order for "Carol" with items:
      | sku    | quantity |
      | widget | 3        |
    When I list orders
    Then the response status is 200
    And the order list has 2 orders

  Scenario: Cancel a pending order
    When I create an order for "Dana" with items:
      | sku      | quantity |
      | sprocket | 1        |
    And I cancel the created order
    Then the response status is 200
    And the order status is "cancelled"

  Scenario Outline: Cancelling twice conflicts
    When I create an order for "<customer>" with items:
      | sku   | quantity |
      | <sku> | <qty>    |
    And I cancel the created order
    And I cancel the created order
    Then the response status is <second_status>

    Examples:
      | customer | sku      | qty | second_status |
      | Eve      | widget   | 1   | 409           |
      | Frank    | sprocket | 2   | 409           |

  Scenario: Missing order is not found
    When I get order 999999
    Then the response status is 404
