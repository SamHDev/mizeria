# Entities

Each scenario gets an `EntityBag`: a typed map from keys to values. Handlers interact
with it through extractors (read) and **effects** (write after the body succeeds).

This is the right tool when steps talk about named domain objects — accounts, users,
orders — instead of stuffing everything into one world struct.

## Keys

- Unit key `()` — one singleton of type `T` per scenario (`Get<T>`, `Put<T>`, …).
- `String` — named entities via the `Named*` aliases (`NamedMut<Balance>`,
  `InsertNamed<Balance>`, …).
- Any `EntityKey` — typically a type that implements `FromCapture` (parsed from the
  next capture group).

Named extractors consume a key capture **before** later `Param`s:

```rust,ignore
// Step: "Alice tries to withdraw 200.00"
// Regex: r"(\w+) tries to withdraw (\d+\.\d{2})"
fn try_withdraw(
    mut balance: NamedMut<Balance>,           // key from "Alice"
    Param(Currency(amount)): Param<Currency>, // "200.00"
) -> Result<Put<Withdrawal>, StepError> {
    // …
}
```

## Reading

| Type | Behaviour |
|---|---|
| `Get<T>` / `Named<T>` | Clone snapshot; no lease held |
| `GetRef<T>` / `NamedRef<T>` | Shared lease |
| `GetMut<T>` / `NamedMut<T>` | Exclusive lease; mutate in place |

Missing entities fail the step (`EntityRejection::Missing` → `StepError`).

## Writing via effects

Return values that implement `Effect` are applied to the bag **after** the body
returns successfully (and after extractor leases are dropped):

| Effect | Behaviour |
|---|---|
| `Put<T>` / `PutNamed<T>` | Insert or replace |
| `Insert<T>` / `InsertNamed<T>` | Insert only; fails if the key exists |
| `Remove<T>` / `RemoveNamed<T>` | Remove (idempotent if missing) |

```rust,ignore
fn has_balance(person: String, Param(Currency(amount)): Param<Currency>) -> InsertNamed<Balance> {
    InsertNamed::named(person, Balance(amount))
}

fn record_result(status: Withdrawal) -> Put<Withdrawal> {
    Put::new(status) // unit-keyed singleton
}
```

Tuples of effects apply left-to-right. `Result<Effect, StepError>` applies the effect
only on `Ok`.

## World vs entities

| Prefer a **world** when… | Prefer **entities** when… |
|---|---|
| State is one cohesive struct | Steps name distinct objects |
| Most steps mutate the same fields | You insert / look up / remove by key |
| You want simple `CtxMut` handlers | You want effect-returning handlers |

You can use both in the same suite. The [withdrawing cash](walkthrough-withdrawing-cash.md)
example uses a unit world (`|| ()`) and entities only.

## Custom keys

Implement `FromCapture` (and thus `EntityKey`) for your key type, or use `String` /
integers that already parse from captures. Key extraction failures surface as
`EntityRejection::MissingKey` or `Parse`.
