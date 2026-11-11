# Walkthrough: orders app

This chapter walks through the standalone `orders-app` workspace crate. Unlike
[withdrawing cash](walkthrough-withdrawing-cash.md), the system under test is a real
HTTP API (Axum + in-memory store). Mizeria drives **component tests**: each scenario
spawns a server on an ephemeral port and steps call it over HTTP.

Run from the repo root:

```bash
cargo test -p orders-app
```

## Layout

| Path | Role |
|---|---|
| `examples/orders-app/src/lib.rs` | Domain types, in-memory `Store`, Axum `Router`, `spawn_test_server` |
| `examples/orders-app/src/main.rs` | Optional binary that listens on `PORT` (default `3000`) |
| `examples/orders-app/features/orders.feature` | Gherkin scenarios |
| `examples/orders-app/tests/component.rs` | World, entities, step handlers, `Suite` |

The crate depends on `mizeria` via a path dependency; it is **not** a mizeria
`[[example]]` binary.

## Feature file

`examples/orders-app/features/orders.feature` (abbreviated):

```gherkin
Feature: Orders API

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
```

Highlights:

- **Background** runs before every scenario (fresh server each time).
- **Data tables** supply line items; headered rows are parsed in steps.
- **Scenario Outline** substitutes `<placeholders>` into step text **and** table
  cells before matching.

## World vs entities

Infrastructure lives in the scenario **world**. Domain facts from HTTP responses
live in the **entity bag**.

```rust,ignore
struct World {
    client: Client,
    base_url: Option<String>,
    server: Option<JoinHandle<std::io::Result<()>>>,
}

#[derive(Debug, Clone)]
struct HttpResponse {
    status: u16,
    body: Value,
}

#[derive(Debug, Clone, Copy)]
struct OrderId(u64);
```

`Drop` for `World` aborts the spawned server when the scenario ends. Steps that
need the base URL take `CtxRef<World>` or `CtxMut<World>`; assertions take
`GetRef<HttpResponse>` or `Get<OrderId>`.

Contrast: withdrawing cash uses a unit world (`|| ()`) and puts everything in
entities. Here both pieces are useful — see [Entities](entities.md).

## Macros

Composite effects for create vs plain responses:

```rust,ignore
effects! {
    struct Created {
        response: Put<HttpResponse>,
        id: Put<OrderId>,
    }

    enum HttpResult {
        Ok { response: Put<HttpResponse> },
        Created(Put<HttpResponse>, Put<OrderId>),
    }
}
```

Status strings from the feature map to a typed enum (no `Param<>` wrapper):

```rust,ignore
variants! {
    enum OrderStatusLabel = {
        Pending = "pending",
        Cancelled = "cancelled",
    }
}
```

Headered item tables use `rows!` (or `DataTable::records`):

```rust,ignore
for row in rows!(for (sku: String, quantity: u32) in &table)? {
    let row = row?;
    items.push(OrderItem {
        sku: row.sku,
        quantity: row.quantity,
    });
}
```

Failures use `fail!` / `error!` — see [Outcomes](outcomes.md).

## Step handlers

**Given** — bind `127.0.0.1:0`, spawn Axum, store the base URL:

```rust,ignore
async fn running_api(mut world: CtxMut<World>) -> Result<(), StepError> {
    let db = Arc::new(Store::new());
    let (url, handle) = spawn_test_server(db)
        .await
        .map_err(|err| error!("spawn server: {err}"))?;
    world.base_url = Some(url);
    world.server = Some(handle);
    Ok(())
}
```

**When** — create posts JSON built from the table; return a `Created` effect:

```rust,ignore
async fn create_order(
    customer: String,
    table: DataTable,
    world: CtxRef<World>,
) -> Result<Created, StepError> {
    // rows! → items → POST /orders …
    match record(response).await? {
        HttpResult::Created(response, id) => Ok(Created { response, id }),
        HttpResult::Ok { .. } => fail!("create response missing id"),
    }
}
```

Later steps load `Get<OrderId>` to fetch or cancel. Helpers that only need the last
status/body return `HttpResult` (which still `Put`s `HttpResponse`).

**Then** — assert on the bag:

```rust,ignore
fn assert_status(expected: u16, response: GetRef<HttpResponse>) -> Result<(), StepError> {
    if response.status != expected {
        fail!("expected status {expected}, got {}", response.status);
    }
    Ok(())
}

fn assert_order_status(
    expected: OrderStatusLabel,
    response: GetRef<HttpResponse>,
) -> Result<(), StepError> {
    // compare JSON "status" to expected.to_string() …
}
```

## Wiring the suite

The suite runs inside a Tokio integration test with the async runner (HTTP clients
need a real runtime):

```rust,ignore
#[tokio::test]
async fn orders_component_tests() {
    let features_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("features");
    let features = Features::builder()
        .add_path(features_dir)
        .build()
        .expect("load features");

    let mut suite = Suite::new(World::default);
    suite
        .given("a running orders API", running_api)
        .when(
            re(r#"I create an order for "([^"]+)" with items:"#),
            create_order,
        )
        .when("I fetch the created order", fetch_created_order)
        // … more when / then …
        .with_reporter(Printer::stdout().symbols(true));

    let outcome = suite.run(&features).await;
    assert!(outcome.is_ok(), "{outcome:?}");
}
```

Highlights:

- `Suite::new(World::default)` — fresh world (and thus fresh server) per scenario.
- Async step bodies + `suite.run(…).await` instead of `run_blocking`.
- `DataTable` / `DocString` are owned extractors, so they register on `Suite` like
  any other `'static` parameter — see [Extractors](extractors.md).

Full sources: `examples/orders-app/`.
