#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, routing::get};
use az_aio_shared::{di::AppModule, plugin, state::AppState};
use shaku::HasComponent;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    let snapshot = plugin::load_snapshot();
    plugin::start_loopback_server(snapshot.clone()).await;

    let module = AppModule::builder().build();
    let config: &dyn az_aio_shared::config::AppConfig = module.resolve_ref();

    let state = AppState::new(snapshot);

    let app = Router::new()
        .route("/", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    let port = config.port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AZ AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
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
