#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, extract::RawQuery, routing::get};
use az_aio_platform::{core::config::AppConfig, plugin::host};
use rudi::Context;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    enable_plugin_providers();

    let mut di = Context::auto_register();
    let config = di.resolve::<az_aio_platform::core::config::ConfigCenterConfig>();

    let native_context = az_aio_platform::plugin::api::NativePluginContext {
        api_base_url: String::new(),
        config_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("."),
        database_url: config.database_url(),
    };

    let snapshot = host::load_az_aio_native_snapshot(native_context, &mut di);

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
        Arc<az_aio_platform::plugin::host::HostSnapshot>,
    >,
    RawQuery(raw_query): RawQuery,
) -> axum::response::Html<String> {
    let (route, query) = split_route_query(raw_query.as_deref());

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

fn split_route_query(raw_query: Option<&str>) -> (String, String) {
    let mut route = "/".to_string();
    let mut query_parts = Vec::new();

    for pair in raw_query.unwrap_or_default().split('&') {
        if pair.is_empty() {
            continue;
        }
        let key = pair.split_once('=').map(|(key, _)| key).unwrap_or(pair);
        if key == "route" {
            let raw_value = pair
                .split_once('=')
                .map(|(_, value)| value)
                .unwrap_or_default();
            route = urlencoding::decode(raw_value)
                .unwrap_or_else(|_| raw_value.into())
                .into_owned();
        } else {
            query_parts.push(pair.to_string());
        }
    }

    let query = if query_parts.is_empty() {
        String::new()
    } else {
        format!("?{}", query_parts.join("&"))
    };

    (route, query)
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::api::DynNativeAzAioPlugin;

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

    #[test]
    fn split_route_query_preserves_repeated_plugin_params() {
        let (route, query) = split_route_query(Some(
            "route=%2Falgorithms&algorithm=flame_detection&algorithm=face_detection&active=flame_detection",
        ));

        assert_eq!(route, "/algorithms");
        assert_eq!(
            query,
            "?algorithm=flame_detection&algorithm=face_detection&active=flame_detection"
        );
    }
}
