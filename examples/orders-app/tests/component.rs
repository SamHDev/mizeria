//! Component tests: real HTTP against a per-scenario in-memory orders API.
//!
//! World holds HTTP infra; scenario facts live in entities (`HttpResponse`, `OrderId`).

use std::path::PathBuf;
use std::sync::Arc;

use mizeria::prelude::*;
use orders_app::{OrderItem, Store, spawn_test_server};
use reqwest::Client;
use serde_json::Value;
use tokio::task::JoinHandle;

struct World {
    client: Client,
    base_url: Option<String>,
    server: Option<JoinHandle<std::io::Result<()>>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            client: Client::new(),
            base_url: None,
            server: None,
        }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if let Some(handle) = self.server.take() {
            handle.abort();
        }
    }
}

impl World {
    fn base_url(&self) -> Result<&str, StepError> {
        self.base_url
            .as_deref()
            .ok_or_else(|| error!("orders API is not running"))
    }
}

/// Last HTTP response for the scenario.
#[derive(Debug, Clone)]
struct HttpResponse {
    status: u16,
    body: Value,
}

/// Id of the most recently created order.
#[derive(Debug, Clone, Copy)]
struct OrderId(u64);

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

variants! {
    enum OrderStatusLabel = {
        Pending = "pending",
        Cancelled = "cancelled",
    }
}

async fn record(response: reqwest::Response) -> Result<HttpResult, StepError> {
    let status = response.status().as_u16();
    let body = response
        .json::<Value>()
        .await
        .map_err(|err| error!("response json: {err}"))?;
    let response = Put::new(HttpResponse { status, body: body.clone() });
    Ok(match body.get("id").and_then(Value::as_u64) {
        Some(id) => HttpResult::Created(response, Put::new(OrderId(id))),
        None => HttpResult::Ok { response },
    })
}

async fn running_api(mut world: CtxMut<World>) -> Result<(), StepError> {
    let db = Arc::new(Store::new());
    let (url, handle) = spawn_test_server(db)
        .await
        .map_err(|err| error!("spawn server: {err}"))?;
    world.base_url = Some(url);
    world.server = Some(handle);
    Ok(())
}

async fn create_order(
    customer: String,
    table: DataTable,
    world: CtxRef<World>,
) -> Result<Created, StepError> {
    let mut items = Vec::new();
    for row in rows!(for (sku: String, quantity: u32) in &table)? {
        let row = row?;
        items.push(OrderItem {
            sku: row.sku,
            quantity: row.quantity,
        });
    }
    if items.is_empty() {
        fail!("items table has no data rows");
    }
    let url = format!("{}/orders", world.base_url()?);
    let response = world
        .client
        .post(url)
        .json(&serde_json::json!({
            "customer": customer,
            "items": items,
        }))
        .send()
        .await
        .map_err(|err| error!("create order: {err}"))?;
    match record(response).await? {
        HttpResult::Created(response, id) => Ok(Created { response, id }),
        HttpResult::Ok { .. } => fail!("create response missing id"),
    }
}

async fn fetch_created_order(
    id: Get<OrderId>,
    world: CtxRef<World>,
) -> Result<HttpResult, StepError> {
    let url = format!("{}/orders/{}", world.base_url()?, id.0);
    let response = world
        .client
        .get(url)
        .send()
        .await
        .map_err(|err| error!("get order: {err}"))?;
    record(response).await
}

async fn get_order(id: u64, world: CtxRef<World>) -> Result<HttpResult, StepError> {
    let url = format!("{}/orders/{id}", world.base_url()?);
    let response = world
        .client
        .get(url)
        .send()
        .await
        .map_err(|err| error!("get order: {err}"))?;
    record(response).await
}

async fn list_orders(world: CtxRef<World>) -> Result<HttpResult, StepError> {
    let url = format!("{}/orders", world.base_url()?);
    let response = world
        .client
        .get(url)
        .send()
        .await
        .map_err(|err| error!("list orders: {err}"))?;
    record(response).await
}

async fn cancel_created_order(
    id: Get<OrderId>,
    world: CtxRef<World>,
) -> Result<HttpResult, StepError> {
    let url = format!("{}/orders/{}/cancel", world.base_url()?, id.0);
    let response = world
        .client
        .post(url)
        .send()
        .await
        .map_err(|err| error!("cancel order: {err}"))?;
    record(response).await
}

fn assert_status(expected: u16, response: GetRef<HttpResponse>) -> Result<(), StepError> {
    if response.status != expected {
        fail!(
            "expected status {expected}, got {}",
            response.status
        );
    }
    Ok(())
}

fn assert_customer(expected: String, response: GetRef<HttpResponse>) -> Result<(), StepError> {
    let actual = response
        .body
        .get("customer")
        .and_then(Value::as_str)
        .ok_or_else(|| error!("response missing customer"))?;
    if actual != expected {
        fail!("expected customer {expected:?}, got {actual:?}");
    }
    Ok(())
}

fn assert_order_status(
    expected: OrderStatusLabel,
    response: GetRef<HttpResponse>,
) -> Result<(), StepError> {
    let actual = response
        .body
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| error!("response missing status"))?;
    let expected = expected.to_string();
    if actual != expected {
        fail!("expected status {expected:?}, got {actual:?}");
    }
    Ok(())
}

fn assert_item_count(expected: usize, response: GetRef<HttpResponse>) -> Result<(), StepError> {
    let items = response
        .body
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| error!("response missing items array"))?;
    if items.len() != expected {
        fail!("expected {expected} items, got {}", items.len());
    }
    Ok(())
}

fn assert_list_len(expected: usize, response: GetRef<HttpResponse>) -> Result<(), StepError> {
    let list = response
        .body
        .as_array()
        .ok_or_else(|| error!("expected a JSON array body"))?;
    if list.len() != expected {
        fail!("expected {expected} orders, got {}", list.len());
    }
    Ok(())
}

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
        .when(re(r"I get order (\d+)"), get_order)
        .when("I list orders", list_orders)
        .when("I cancel the created order", cancel_created_order)
        .then(re(r"the response status is (\d+)"), assert_status)
        .then(re(r#"the order customer is "([^"]+)""#), assert_customer)
        .then(re(r#"the order status is "([^"]+)""#), assert_order_status)
        .then(re(r"the order has (\d+) items"), assert_item_count)
        .then(re(r"the order list has (\d+) orders"), assert_list_len)
        .with_reporter(Printer::stdout().symbols(true));

    let outcome = suite.run(&features).await;
    assert!(outcome.is_ok(), "{outcome:?}");
}
