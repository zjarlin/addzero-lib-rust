#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    az_aio_plugin_bundled::api::ensure_linked();

    let snapshot = az_aio_plugin_host::host::load_az_aio_native_snapshot();
    let _loopback_url =
        az_aio_plugin_host::host::start_native_loopback_server(snapshot.clone())
            .await
            .ok();

    let state = AppState { snapshot };

    let app = Router::new()
        .route("/", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("AZ AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    snapshot: az_aio_plugin_host::host::HostSnapshot,
}

async fn root_page(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Html<String> {
    let html = app::render_app_html(&state.snapshot);
    axum::response::Html(html)
}

async fn health() -> &'static str {
    "ok"
}
