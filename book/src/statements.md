# Statements

A **statement** decides whether a registered step matches a Gherkin step’s text, and
what capture groups the handler’s extractors will see.

## Exact match

Bare `&'static str` values match the entire step text with no captures:

```rust,ignore
suite.then("the withdrawal is successful", withdrawal_successful);
```

Equivalent to `Const::new("…")`.

## Regex match

Use `re(...)` (panics on a bad pattern at registration time) or `Regex::new` when you
want a `Result`:

```rust,ignore
suite.given(re(r"I have (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
    world.cukes = n;
});
```

Important details:

- The regex must match the **whole** step text (anchored start and end). Partial
  matches are rejected.
- Positional groups (`(\d+)`, `(\w+)`, …) become captures in order.
- Named groups (`(?P<name>…)`) are available via `StepContext::named_capture` for
  custom extractors; positional extractors still consume groups left-to-right.

## Captures feed extractors

Capture order is the contract between the statement and the handler signature:

```gherkin
Given Alice has 234.56 in their account
```

```rust,ignore
suite.given(
    re(r"(\w+) has (\d+\.\d{2}) in their account"),
    |person: String, Param(Currency(amount)): Param<Currency>| {
        InsertNamed::named(person, Balance(amount))
    },
);
```

1. First group → `person: String`
2. Second group → `Param<Currency>`

Entity extractors like `NamedMut<T>` also consume a key capture — see
[Entities](entities.md).

## Custom statements

Anything implementing `Statement` works. The built-in `Const` and `Regex` types cover
almost all suite registration; custom matchers are an advanced escape hatch if you need
something other than string equality or a regex.
