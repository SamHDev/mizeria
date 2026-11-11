# Extractors

Handlers declare their inputs as typed **extractors**, similar to Axum request
extractors. Before the function body runs, Mizeria walks the parameter list
left-to-right and builds each value from a `StepContext`.

## `FromStep`

Every extractor implements `FromStep`. Rejections become `StepError` and fail the step.

Built-in extractors:

| Extractor | Source |
|---|---|
| `u32`, `String`, `bool`, … | Next regex capture, parsed with `FromStr` |
| `Param<T>` | Next capture into a custom `FromStr` type |
| `&str` | Next capture, borrowed (see note below) |
| `CtxMut<W>` / `Ctx<W>` | Exclusive lease on the scenario world |
| `CtxRef<W>` | Shared lease on the scenario world |
| `DocString` | Step doc string argument |
| `DataTable` | Step data table argument |
| Entity getters (`Get`, `GetOrDefault`, `NamedMut`, …) | Entity bag (+ key capture) — [Entities](entities.md) |

> **Registration note:** suite `given` / `when` / `then` require owned (`'static`)
> extractors. Prefer `String` / `Param<T>` / `CtxMut` / `DataTable` / `DocString`
> at registration time. Borrowed extractors such as `Param<&str>` are supported via
> the lower-level `call` helper for advanced use.

## Capture parameters

Std types that implement `FromStr` extract directly:

```rust,ignore
suite.when(re(r"I eat (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
    world.cukes -= n;
});
```

Wrap custom types in `Param<T>`:

```rust,ignore
struct Currency(u64);

impl FromStr for Currency {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { /* … */ }
}

fn step(Param(Currency(amount)): Param<Currency>) { /* … */ }
```

### Composite params — `params!`

One capture per field, as a named (or tuple) struct:

```rust,ignore
params! {
    struct Transfer {
        from: String,
        amount: u32,
    }
}

suite.when(re(r"(\w+) sends (\d+)"), |t: Transfer| {
    assert!(t.amount > 0);
});
```

Field types must implement `FromCapture` (the same cell/capture parse trait used by tables).

### String enums — `variants!`

Map unit variants to exact literals; the type extracts like `u32` (no `Param<>` needed):

```rust,ignore
variants! {
    enum Status = {
        Open = "open",
        Closed = "closed",
    }
}

suite.then(re(r"status is (\w+)"), |status: Status| {
    assert_eq!(status, Status::Open);
});
```

Unknown strings fail with `ParamRejection::Parse` listing the allowed literals.

## World access

```rust,ignore
fn bump(mut world: CtxMut<World>) {
    world.cukes += 1;
}

fn peek(world: CtxRef<World>) {
    assert!(world.cukes > 0);
}
```

`CtxMut` and `CtxRef` hold leases (`WriteGuard` / `ReadGuard`). Drop them before the
handler returns if you need another exclusive lease in a later step — leases do not
cross steps, only within a single invocation.

## Doc strings and tables

```gherkin
When I submit the payload
  """
  { "ok": true }
  """

When I import rows
  | name  | qty |
  | apple | 3   |
```

```rust,ignore
suite.when("I submit the payload", |DocString(body): DocString| {
    assert!(body.contains("ok"));
});

suite.when("I import rows", |table: DataTable| -> Result<(), StepError> {
    // Headered table: field names become expected headers (case-insensitive).
    for row in rows!(for (name: &str, qty: u32) in &table)? {
        let row = row?;
        assert!(row.qty > 0);
        let _ = row.name;
    }
    Ok(())
});
```

`DocString` and `DataTable` clone into owned values on extraction, so they are
`'static` and work with `Suite` registration.

For tables **with** a header row, prefer `rows!` or `DataTable::records`:

```rust,ignore
for row in table.records::<(String, u32)>(&["sku", "quantity"])? {
    let (sku, qty) = row?;
}
```

`records` checks the header case-insensitively, then yields body rows parsed with
`FromRow` (tuples of `FromCapture` types). Header mismatches fail eagerly;
cell parse failures are `Err` items (`TableRejection`).

`DataTable::rows` still iterates every row as data when there is no header.

## Handler return values

After extractors succeed, the body may return:

- `()` — pass
- `Result<T, E>` where `T: IntoStepResult` and `E: Into<StepError>`
- An entity [`Effect`](entities.md) (`Put`, `Insert`, …) or a tuple of effects
- `StepError` / `StepOutcome` directly
- An `async` future yielding any of the above

Entity leases from extractors are dropped **before** return effects apply, so a
handler can read with `NamedMut` and still return a `Put` for a different type.
