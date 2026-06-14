//! Standalone lowcode server: Axum API + wasm frontend.
//! Run with: cargo run --bin lowcode-server --target x86_64-apple-darwin

use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize lowcode store (in-memory for now)
    let store = lowcode::store::LowcodeStore::in_memory();
    store.seed_demo();

    let api_state = lowcode::routes::LowcodeApiState { store };
    let api_router = lowcode::routes::lowcode_router(api_state);

    let dist_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .nest_service("/", ServeDir::new(&dist_dir).fallback(
            ServeDir::new(&dist_dir).not_found_service(
                axum::routing::get(|| async { "Not Found" })
            )
        ))
        .merge(api_router);

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8081);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Lowcode standalone server listening on http://{addr}");
    println!("Open http://localhost:{port} in your browser", port=port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
