//! Orders HTTP API backed by an in-memory store.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, serve};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Shared in-memory database handle.
pub type Db = Arc<Store>;

/// Process-local order store.
#[derive(Debug, Default)]
pub struct Store {
    next_id: AtomicU64,
    orders: Mutex<HashMap<u64, Order>>,
}

impl Store {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed) + 1
    }
}

/// Order lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Cancelled,
}

/// Line item on an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderItem {
    pub sku: String,
    pub quantity: u32,
}

/// Persisted order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub customer: String,
    pub items: Vec<OrderItem>,
    pub status: OrderStatus,
}

/// Body for `POST /orders`.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub customer: String,
    pub items: Vec<OrderItem>,
}

/// Build the HTTP router for `db`.
pub fn app(db: Db) -> Router {
    Router::new()
        .route("/orders", post(create_order).get(list_orders))
        .route("/orders/{id}", get(get_order))
        .route("/orders/{id}/cancel", post(cancel_order))
        .with_state(db)
}

/// Bind `listener` and serve until the task is aborted or the listener closes.
pub async fn serve_listener(db: Db, listener: TcpListener) -> std::io::Result<()> {
    serve(listener, app(db)).await
}

/// Bind `127.0.0.1:0`, spawn the server, and return `(base_url, join handle)`.
pub async fn spawn_test_server(
    db: Db,
) -> std::io::Result<(String, JoinHandle<std::io::Result<()>>)> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(serve_listener(db, listener));
    Ok((format!("http://{addr}"), handle))
}

async fn create_order(
    State(db): State<Db>,
    Json(body): Json<CreateOrderRequest>,
) -> impl IntoResponse {
    if body.customer.trim().is_empty() || body.items.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "customer and items are required" })),
        )
            .into_response();
    }
    if body
        .items
        .iter()
        .any(|item| item.quantity == 0 || item.sku.trim().is_empty())
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "each item needs a sku and quantity > 0" })),
        )
            .into_response();
    }

    let order = Order {
        id: db.alloc_id(),
        customer: body.customer,
        items: body.items,
        status: OrderStatus::Pending,
    };
    db.orders.lock().await.insert(order.id, order.clone());
    (StatusCode::CREATED, Json(order)).into_response()
}

async fn list_orders(State(db): State<Db>) -> Json<Vec<Order>> {
    let mut orders: Vec<_> = db.orders.lock().await.values().cloned().collect();
    orders.sort_by_key(|o| o.id);
    Json(orders)
}

async fn get_order(State(db): State<Db>, Path(id): Path<u64>) -> impl IntoResponse {
    match db.orders.lock().await.get(&id).cloned() {
        Some(order) => (StatusCode::OK, Json(order)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "order not found" })),
        )
            .into_response(),
    }
}

async fn cancel_order(State(db): State<Db>, Path(id): Path<u64>) -> impl IntoResponse {
    let mut orders = db.orders.lock().await;
    let Some(order) = orders.get_mut(&id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "order not found" })),
        )
            .into_response();
    };
    if order.status == OrderStatus::Cancelled {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "order already cancelled" })),
        )
            .into_response();
    }
    order.status = OrderStatus::Cancelled;
    (StatusCode::OK, Json(order.clone())).into_response()
}
