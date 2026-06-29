#![forbid(unsafe_code)]

mod app;

use anyhow::{Context as _, Result};
use axum::{Router, extract::RawQuery, middleware, routing::get};
use az_aio_platform::{
    core::config::AppConfig,
    plugin::host,
    system::api_key_auth::{SystemApiKeyAuthState, optional_system_api_key_auth},
};
use rudi::Context;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

fn main() -> Result<()> {
    enable_plugin_providers();

    let mut di = Context::auto_register();
    let config = di.resolve::<az_aio_platform::core::config::ConfigCenterConfig>();

    let native_context = az_aio_platform::plugin::api::NativePluginContext {
        api_base_url: String::new(),
        config_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("."),
        database_url: config.database_url(),
    };

    let snapshot = host::load_native_snapshot(native_context, &mut di);
    let port = config.port();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("创建 AIO web runtime 失败")?;

    runtime.block_on(run_web_server(snapshot, port, config.database_url()))
}

async fn run_web_server(
    snapshot: az_aio_platform::plugin::host::HostSnapshot,
    port: u16,
    database_url: Option<String>,
) -> Result<()> {
    let api_key_auth_state = SystemApiKeyAuthState::new(database_url)
        .await
        .unwrap_or_else(|_| SystemApiKeyAuthState::degraded());
    let native_router = snapshot
        .native_router
        .clone()
        .layer(middleware::from_fn_with_state(
            api_key_auth_state,
            optional_system_api_key_auth,
        ));
    let state = Arc::new(snapshot);
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let app = Router::new()
        .route("/", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new(&assets_dir))
        .merge(native_router.with_state(()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AIO web workbench listening on http://{addr}");

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
    let html = match tokio::task::spawn_blocking(move || {
        app::render_app_html(&snapshot, &route, &query)
    })
    .await
    {
        Ok(html) => html,
        Err(error) => format!(
            r#"<!doctype html><meta charset="utf-8"><title>AIO</title><main>SSR 渲染失败：{error}</main>"#
        ),
    };
    axum::response::Html(html)
}

async fn health() -> &'static str {
    "ok"
}

fn enable_plugin_providers() {
    az_aio_platform::enable();
    algorithm_center::enable();
    asset_hub::enable();
    config_center::enable();
    drive_center::enable();
    edge_gateway::enable();
    lowcode::enable();
    software_center::enable();
    az_linux::enable();
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
    use az_aio_platform::plugin::api::{DynAdminPluginProvider, NativePluginContext};

    use super::*;

    #[test]
    fn rudi_collects_all_admin_plugin_providers() {
        enable_plugin_providers();

        let mut di = Context::auto_register();
        let mut plugin_ids = di
            .resolve_by_type::<DynAdminPluginProvider>()
            .into_iter()
            .map(|plugin| plugin.admin_descriptor().id)
            .collect::<Vec<_>>();
        plugin_ids.sort();

        assert_eq!(
            plugin_ids,
            [
                "admin-scenes",
                "algorithm-center",
                "asset-hub",
                "config-center",
                "drive-center",
                "edge-gateway",
                "linux",
                "lowcode",
                "software-center",
                "system",
            ]
        );
    }

    #[test]
    fn rudi_menu_reserves_admin_knowledge_base_and_gateway_scenes() {
        enable_plugin_providers();

        let mut di = Context::auto_register();
        let snapshot = host::load_native_snapshot(NativePluginContext::default(), &mut di);
        let labels = snapshot
            .admin_menu_tree
            .sections
            .iter()
            .map(|section| section.label.as_str())
            .collect::<Vec<_>>();
        let gateway = snapshot
            .admin_menu_tree
            .sections
            .iter()
            .find(|section| section.label == "智能网关");
        let system = snapshot
            .admin_menu_tree
            .sections
            .iter()
            .find(|section| section.label == "管理后台");

        assert!(labels.contains(&"管理后台"));
        assert!(labels.contains(&"知识库"));
        assert!(labels.contains(&"智能网关"));
        assert!(
            system
                .map(|section| section
                    .menus
                    .iter()
                    .any(|node| menu_node_contains_href(node, "/config")))
                .unwrap_or(false)
        );
        assert!(
            gateway
                .map(|section| section.menus.iter().any(|node| node.label == "算法中心"))
                .unwrap_or(false)
        );
    }

    fn menu_node_contains_href(
        node: &az_aio_platform::plugin::api::AdminMenuNode,
        href: &str,
    ) -> bool {
        node.href == href
            || node
                .children
                .iter()
                .any(|child| menu_node_contains_href(child, href))
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
