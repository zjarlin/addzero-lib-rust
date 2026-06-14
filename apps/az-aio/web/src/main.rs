#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, routing::get};
use az_aio_shared::{di::AppModule, state::AppState};
use shaku::HasComponent;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    let module = AppModule::builder().build();
    let config: &dyn az_aio_shared::config::AppConfig = module.resolve_ref();

    let context = az_aio_plugin_api::api::NativePluginContext {
        api_base_url: "http://127.0.0.1:0".to_string(),
        config_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("."),
        database_url: config.database_url(),
    };

    az_aio_plugin_bundled::api::ensure_linked();
    let snapshot = az_aio_plugin_host::host::load_az_aio_native_snapshot(context);
    start_loopback_server(snapshot.clone()).await;

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

/// 启动原生插件 loopback 服务器。
async fn start_loopback_server(snapshot: az_aio_plugin_host::host::HostSnapshot) {
    match az_aio_plugin_host::host::start_native_loopback_server(snapshot).await {
        Ok(addr) => println!("plugin loopback server listening on {addr}"),
        Err(e) => eprintln!("plugin loopback server failed to start: {e}"),
    }
}
