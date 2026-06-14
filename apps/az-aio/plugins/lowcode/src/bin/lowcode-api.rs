//! Standalone lowcode API server.
//! Run: cargo run -p lowcode --bin lowcode-api

use axum::Router;
use lowcode::routes::{LowcodeApiState, lowcode_router};
use lowcode::store::LowcodeStore;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = LowcodeStore::in_memory();
    store.seed_demo();
    lowcode::record::RecordStore::global().seed_demo();

    let state = LowcodeApiState { store };
    let app = Router::new()
        .merge(lowcode_router(state))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8081".into()).parse()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Lowcode API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
