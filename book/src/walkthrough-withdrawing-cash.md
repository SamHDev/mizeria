# Walkthrough: withdrawing cash

This chapter walks through the `withdrawing_cash` example shipped with the crate. It
shows entities, custom `Param` parsing, and a rule-scoped feature — without a world
struct.

Run it from the repo root:

```bash
cargo run --example withdrawing_cash
```

## Feature file

`examples/withdrawing_cash/features/withdrawing_cash.feature`:

```gherkin
Feature: Withdrawing cash

  Rule: Customers cannot withdraw more than their balance

    Scenario: Successful withdrawal within balance
      Given Alice has 234.56 in their account
      When Alice tries to withdraw 200.00
      Then the withdrawal is successful

    Scenario: Declined withdrawal in excess of balance
      Given Hamza has 198.76 in their account
      When Hamza tries to withdraw 200.00
      Then the withdrawal is declined
```

Two scenarios under one rule. Person names and amounts are data in the step text;
handlers will turn them into typed entities.

## Domain types

```rust,ignore
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Balance(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Withdrawal {
    Successful,
    Declined,
}

/// Amount in cents, parsed from a decimal capture such as `234.56`.
struct Currency(u64);

impl FromStr for Currency {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split on '.', interpret as dollars + cents…
        // …
    }
}
```

`Currency` is not stored in the bag; it is only a capture parser. `Balance` and
`Withdrawal` are entities.

## Step handlers

**Given** — parse person + amount, insert a named balance:

```rust,ignore
fn has_balance(person: String, Param(Currency(amount)): Param<Currency>) -> InsertNamed<Balance> {
    InsertNamed::named(person, Balance(amount))
}
```

**When** — exclusive lease on that person’s balance, return a singleton withdrawal
result:

```rust,ignore
fn try_withdraw(
    mut balance: NamedMut<Balance>,
    Param(Currency(amount)): Param<Currency>,
) -> Result<Put<Withdrawal>, StepError> {
    match balance.0.checked_sub(amount) {
        Some(next) => {
            balance.0 = next;
            Ok(Put::new(Withdrawal::Successful))
        }
        None => Ok(Put::new(Withdrawal::Declined)),
    }
}
```

Capture order for `(\w+) tries to withdraw (\d+\.\d{2})`:

1. `NamedMut<Balance>` takes the person name as its key and mutably leases the balance.
2. `Param<Currency>` takes the amount.

In-place mutation of `balance` is visible immediately. `Put<Withdrawal>` applies after
the function returns (and after the `NamedMut` lease drops).

**Then** — assert on the singleton `Withdrawal`:

```rust,ignore
fn withdrawal_successful(w: GetRef<Withdrawal>) -> Result<(), StepError> {
    match *w {
        Withdrawal::Successful => Ok(()),
        Withdrawal::Declined => Err(StepError::new("expected successful withdrawal")),
    }
}
```

Exact-string statements are enough here — no captures.

## Wiring the suite

```rust,ignore
fn main() {
    let features_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/withdrawing_cash/features");

    let features = Features::builder()
        .add_path(features_dir)
        .build()
        .expect("load features");

    let mut suite = Suite::new(|| ());
    suite
        .given(re(r"(\w+) has (\d+\.\d{2}) in their account"), has_balance)
        .when(re(r"(\w+) tries to withdraw (\d+\.\d{2})"), try_withdraw)
        .then("the withdrawal is successful", withdrawal_successful)
        .then("the withdrawal is declined", withdrawal_declined)
        .with_reporter(Printer::stdout().symbols(true));

    let outcome = suite.run_blocking(&features);
    if !outcome.is_ok() {
        std::process::exit(1);
    }
}
```

Highlights:

- Unit world (`|| ()`) — all state lives in the entity bag.
- Two `then` registrations for alternate exact phrases.
- `symbols(true)` on the coloured printer for compact step markers.
- Non-zero exit when `outcome` is not ok.

Full source: `examples/withdrawing_cash/main.rs`.
