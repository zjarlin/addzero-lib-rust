#![forbid(unsafe_code)]

use anyhow::{Context as _, Result};
use axum::{
    Json, Router,
    extract::{Query, State},
    middleware,
    response::Html,
    routing::get,
};
use az_aio_platform::{
    core::{config::AppConfig, db},
    plugin::host,
    system::{
        api_key_auth::{SystemApiKeyAuthState, optional_system_api_key_auth},
        store::{SYSTEM_ADMIN_BOOTSTRAP_SQL, SystemAdminStore},
    },
};
use rudi::Context;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
use tower_http::services::ServeDir;

mod shell;

fn main() -> Result<()> {
    enable_plugin_providers();

    let mut di = Context::auto_register();
    let config = di.resolve::<az_aio_platform::core::config::ConfigCenterConfig>();

    let port = config.port();
    let database_url = config.database_url();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("创建 AIO web runtime 失败")?;
    let bootstrap_sql = aio_bootstrap_sql();
    let toasty_models = db::collect_toasty_models(&mut di);
    let shared_db = match runtime.block_on(db::install_shared_db_singleton(
        &mut di,
        database_url.as_deref(),
        toasty_models,
        &bootstrap_sql,
    )) {
        Ok(shared_db) => shared_db,
        Err(error) => {
            eprintln!("AIO shared Toasty startup degraded: {error:#}");
            None
        }
    };

    let native_context = az_aio_platform::plugin::contract::NativePluginContext {
        api_base_url: String::new(),
        config_dir: std::path::PathBuf::from("."),
        data_dir: std::path::PathBuf::from("."),
        database_url: database_url.clone(),
        shared_db: shared_db.clone(),
    };

    let snapshot = host::load_native_snapshot(native_context, &mut di);

    runtime.block_on(run_web_server(snapshot, port, database_url, shared_db))
}

async fn run_web_server(
    snapshot: az_aio_platform::plugin::host::HostSnapshot,
    port: u16,
    database_url: Option<String>,
    shared_db: Option<db::Db>,
) -> Result<()> {
    let api_key_auth_state = if database_url
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        SystemApiKeyAuthState::from_store(shared_db.map(SystemAdminStore::from_shared))
    } else {
        SystemApiKeyAuthState::degraded()
    };
    let native_router = snapshot
        .native_router
        .clone()
        .layer(middleware::from_fn_with_state(
            api_key_auth_state,
            optional_system_api_key_auth,
        ));
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let client_dist_dir = client_dist_dir();

    let page_snapshot = snapshot.clone();
    let page_router = Router::new()
        .route("/", get(root_page))
        .route("/gateway", get(root_page))
        .route("/api/client/bootstrap", get(client_bootstrap))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new(&assets_dir))
        .nest_service("/client", ServeDir::new(&client_dist_dir))
        .nest_service("/wasm", ServeDir::new(client_dist_dir.join("wasm")))
        .with_state(page_snapshot);

    let app = page_router.merge(native_router.with_state(()));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn root_page(
    State(snapshot): State<az_aio_platform::plugin::host::HostSnapshot>,
    Query(query): Query<HashMap<String, String>>,
) -> Html<String> {
    let active_route = query
        .get("route")
        .cloned()
        .unwrap_or_else(|| "/system/account/api-keys".to_string());
    let route_for_error = active_route.clone();
    let html = tokio::task::spawn_blocking(move || {
        shell::render_workbench_page(&snapshot, &active_route, "")
    })
    .await
    .unwrap_or_else(|error| shell::render_ssr_error_page(&route_for_error, &error.to_string()));
    Html(html)
}

async fn client_bootstrap(
    State(snapshot): State<az_aio_platform::plugin::host::HostSnapshot>,
) -> Json<az_aio_platform::plugin::contract::ClientBootstrapPayload> {
    Json(host::client_bootstrap_payload(
        &snapshot,
        default_route(&snapshot),
        "",
    ))
}

fn default_route(snapshot: &az_aio_platform::plugin::host::HostSnapshot) -> String {
    snapshot
        .pages
        .first()
        .map(|page| page.route.clone())
        .unwrap_or_else(|| "/system/account/api-keys".to_string())
}

fn client_dist_dir() -> PathBuf {
    if let Ok(path) = std::env::var("AZ_AIO_CLIENT_DIST") {
        return PathBuf::from(path);
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        manifest_dir.join("../client/dist"),
        manifest_dir.join("../plugins/target/dx/az-aio-client/release/web/public"),
        manifest_dir.join("../plugins/target/dx/az-aio-client/debug/web/public"),
    ]
    .into_iter()
    .find(|path| path.join("wasm/az-aio-client.js").exists())
    .unwrap_or_else(|| manifest_dir.join("../client/dist"))
}

async fn health() -> &'static str {
    "ok"
}

fn aio_bootstrap_sql() -> Vec<&'static str> {
    let mut statements = Vec::new();
    statements.extend_from_slice(SYSTEM_ADMIN_BOOTSTRAP_SQL);
    statements.extend_from_slice(CONFIG_CENTER_BOOTSTRAP_SQL);
    statements.extend_from_slice(DRIVE_CENTER_BOOTSTRAP_SQL);
    statements.extend_from_slice(ASSET_HUB_BOOTSTRAP_SQL);
    statements.extend_from_slice(SOFTWARE_CENTER_BOOTSTRAP_SQL);
    statements.extend_from_slice(edge_gateway::backend::store::EDGE_GATEWAY_BOOTSTRAP_SQL);
    statements.extend_from_slice(az_engine::ENGINE_BOOTSTRAP_SQL);
    statements
        .extend_from_slice(az_aio_platform::system::dictionary_model::DICTIONARY_BOOTSTRAP_SQL);
    statements
}

const CONFIG_CENTER_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS biz_config_center_config_entries (id TEXT PRIMARY KEY, namespace TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_config_center_config_entries_namespace_idx ON biz_config_center_config_entries (namespace)",
    "CREATE INDEX IF NOT EXISTS biz_config_center_config_entries_key_idx ON biz_config_center_config_entries (key)",
];

const DRIVE_CENTER_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS biz_drive_center_drive_tasks (id TEXT PRIMARY KEY, drive_path TEXT NOT NULL, action TEXT NOT NULL, status TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_drive_center_drive_tasks_drive_path_idx ON biz_drive_center_drive_tasks (drive_path)",
];

const ASSET_HUB_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS biz_asset_hub_asset_records (id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, status TEXT NOT NULL, source TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_asset_hub_asset_records_kind_idx ON biz_asset_hub_asset_records (kind)",
];

const SOFTWARE_CENTER_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS biz_software_center_software_package_records (id TEXT PRIMARY KEY, name TEXT NOT NULL, source_path TEXT NOT NULL, platform TEXT NOT NULL, arch TEXT NOT NULL, status TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_software_center_software_package_records_name_idx ON biz_software_center_software_package_records (name)",
];

fn enable_plugin_providers() {
    az_aio_platform::enable();
    algorithm_center::enable();
    asset_hub::enable();
    codegen::enable();
    config_center::enable();
    drive_center::enable();
    edge_gateway::enable();
    iot_center::enable();
    lowcode::enable();
    software_center::enable();
    az_linux::enable();
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::contract::{DynAdminPluginProvider, NativePluginContext};

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
                "codegen",
                "config-center",
                "drive-center",
                "edge-gateway",
                "iot-center",
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
        let knowledge = snapshot
            .admin_menu_tree
            .sections
            .iter()
            .find(|section| section.label == "知识库");
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
        assert_eq!(
            knowledge.map(|section| section.default_href.as_str()),
            Some("/assets")
        );
        assert!(
            knowledge
                .map(|section| section
                    .menus
                    .iter()
                    .any(|node| node.href == "/assets" && node.label == "资产中心"))
                .unwrap_or(false)
        );
        // The migrated pages must keep SSR renderer IDs aligned with client route entries.
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

    #[test]
    fn rudi_collects_all_toasty_models() {
        enable_plugin_providers();

        let mut di = Context::auto_register();
        let models = db::collect_toasty_models(&mut di);
        let mut model_names = models
            .iter()
            .map(|model| model.name().upper_camel_case())
            .collect::<Vec<_>>();
        model_names.sort();

        assert_eq!(
            model_names,
            [
                "AssetRecord",
                "ConfigEntry",
                "DataRecord",
                "DictionaryItemRecord",
                "DictionaryTypeRecord",
                "DriveTask",
                "EdgeApiTokenRecord",
                "EdgeUsageRecordRow",
                "GatewayFlow",
                "GatewayRouteDefinition",
                "HookDefinition",
                "MetaField",
                "MetaModel",
                "SoftwarePackageRecord",
                "SystemApiKeyRecord",
                "SystemDataRecord",
                "SystemOperationRecord",
                "SystemPageRecord",
            ]
        );
    }

    #[test]
    fn migrated_plugins_expose_matching_client_pages() {
        enable_plugin_providers();

        let mut di = Context::auto_register();
        let snapshot = host::load_native_snapshot(NativePluginContext::default(), &mut di);
        let client_routes = snapshot
            .client_pages
            .iter()
            .map(|page| (page.route.as_str(), page.renderer_id.as_str()))
            .collect::<Vec<_>>();

        assert!(
            [
                ("/assets", "asset-hub.page"),
                ("/drive", "drive-center.page"),
                ("/software", "software-center.page"),
            ]
            .iter()
            .all(|route| client_routes.contains(route))
        );
    }

    fn menu_node_contains_href(
        node: &az_aio_platform::plugin::contract::AdminMenuNode,
        href: &str,
    ) -> bool {
        node.href == href
            || node
                .children
                .iter()
                .any(|child| menu_node_contains_href(child, href))
    }
}
