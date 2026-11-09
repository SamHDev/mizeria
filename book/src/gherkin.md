# Gherkin support

Cutecumber parses `.feature` files with a built-in Gherkin AST. The runner understands
the structures you typically use in Cucumber suites.

## Feature layout

```gherkin
Feature: Withdrawing cash

  Background:
    Given the bank is open

  Rule: Customers cannot withdraw more than their balance

    Scenario: Successful withdrawal within balance
      Given Alice has 234.56 in their account
      When Alice tries to withdraw 200.00
      Then the withdrawal is successful
```

Supported building blocks:

- **Feature** with children
- **Background** at feature or rule level (run before each scenario in that scope)
- **Rule** grouping scenarios
- **Scenario** with steps
- **Scenario Outline** / **Examples** (see below)
- Step **doc strings** and **data tables**
- **`@tags`** on Feature, Rule, Scenario, and Examples (used for
  [tag filtering](features.md) on loaded `Features`)

Feature-level background runs first, then rule background (if any), then scenario
steps.

## Scenario Outline

Placeholders like `<amount>` are substituted from each Examples row before matching
and before handing doc strings / tables to extractors:

```gherkin
Scenario Outline: withdraw
  Given Alice has <balance> in their account
  When Alice tries to withdraw <amount>
  Then the withdrawal is <result>

  Examples:
    | balance | amount | result       |
    | 234.56  | 200.00 | successful   |
    | 198.76  | 200.00 | declined     |
```

Each Examples body row becomes its own scenario for outcome counting and listeners.

## Dialects

With the default `dialects` feature, the official Gherkin language catalog is embedded
(French `Fonctionnalité`, etc.). Disable default features and omit `dialects` to keep
only English keywords and a smaller binary — see [Crate features](crate-features.md).

## Keywords vs matching

Gherkin keywords (`Given`, `When`, `Then`, `And`, `But`, …) appear in listener events
and feature source. Step **definition** matching uses the step text after the keyword.
`And` / `But` steps match the same registered statements as `Given` / `When` / `Then`
based on text alone.
