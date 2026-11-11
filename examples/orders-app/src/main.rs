//! Run the orders HTTP API with an empty in-memory store.

use std::net::SocketAddr;
use std::sync::Arc;

use orders_app::{Store, app};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("bind {addr}: {err}"));
    eprintln!("orders-app listening on http://{addr}");
    axum::serve(listener, app(Arc::new(Store::new())))
        .await
        .expect("server error");
}
