#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, extract::Query, routing::get};
use az_aio_platform::{config::AppConfig, plugin_host};
use rudi::Context;
use serde::Deserialize;
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
    enable_plugin_providers();

    let mut di = Context::auto_register();
    let config = di.resolve::<az_aio_platform::config::ConfigCenterConfig>();

    let native_context = az_aio_platform::plugin_api::NativePluginContext {
        api_base_url: String::new(),
        config_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("."),
        database_url: config.database_url(),
    };

    let snapshot = plugin_host::load_az_aio_native_snapshot(native_context, &mut di);

    let native_router = snapshot.native_router.clone();
    let state = Arc::new(snapshot);
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let app = Router::new()
        .route("/", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new(&assets_dir))
        .merge(native_router.with_state(()))
        .with_state(state);

    let port = config.port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AZ AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_page(
    axum::extract::State(snapshot): axum::extract::State<
        Arc<az_aio_platform::plugin_host::HostSnapshot>,
    >,
    Query(params): Query<RouteQuery>,
) -> axum::response::Html<String> {
    let route = params.route.unwrap_or_else(|| "/".to_string());

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

fn enable_plugin_providers() {
    algorithm_center::enable();
    asset_hub::enable();
    config_center::enable();
    drive_center::enable();
    edge_gateway::enable();
    lowcode::enable();
    software_center::enable();
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin_api::DynNativeAzAioPlugin;

    use super::*;

    #[test]
    fn rudi_collects_all_native_plugins() {
        enable_plugin_providers();

        let mut di = Context::auto_register();
        let mut plugin_ids = di
            .resolve_by_type::<DynNativeAzAioPlugin>()
            .into_iter()
            .map(|plugin| plugin.descriptor().id)
            .collect::<Vec<_>>();
        plugin_ids.sort();

        assert_eq!(
            plugin_ids,
            [
                "algorithm-center",
                "asset-hub",
                "config-center",
                "drive-center",
                "edge-gateway",
                "lowcode",
                "software-center",
            ]
        );
    }
}
