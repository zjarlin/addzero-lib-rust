#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, extract::Query, routing::get};
use az_aio_platform::{di::AppModule, plugin_host};
use serde::Deserialize;
use shaku::HasComponent;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Deserialize, Default)]
struct RouteQuery {
    #[serde(default)]
    route: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let module = AppModule::builder().build();
    let config: &dyn az_aio_platform::config::AppConfig = module.resolve_ref();

    let context = az_aio_platform::plugin_api::NativePluginContext {
        api_base_url: String::new(),
        config_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("."),
        database_url: config.database_url(),
    };

    az_aio_platform::link_plugins();

    let snapshot = plugin_host::load_az_aio_native_snapshot(context);

    let state = Arc::new(snapshot);
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let app = Router::new()
        .route("/", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new(&assets_dir))
        .with_state(state);

    let port = config.port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AZ AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_page(
    axum::extract::State(snapshot): axum::extract::State<Arc<az_aio_platform::plugin_host::HostSnapshot>>,
    Query(params): Query<RouteQuery>,
) -> axum::response::Html<String> {
    let route = params.route.unwrap_or_else(|| "/assets".to_string());

    let mut query_parts: Vec<String> = Vec::new();
    for (k, v) in &params.extra {
        query_parts.push(format!("{k}={v}"));
    }
    let query = if query_parts.is_empty() {
        String::new()
    } else {
        format!("?{}", query_parts.join("&"))
    };

    let html = app::render_app_html(&snapshot, &route, &query);
    axum::response::Html(html)
}

async fn health() -> &'static str {
    "ok"
}
// Force-link plugin crates so inventory sections are preserved.
use algorithm_center as _;
use asset_hub as _;
use config_center as _;
use drive_center as _;
use edge_gateway as _;
use lowcode as _;
use software_center as _;
