#![forbid(unsafe_code)]

use anyhow::{Context as _, Result};
use axum::{Router, middleware, response::Html, routing::get};
use az_aio_platform::{
    core::{config::AppConfig, db},
    plugin::host,
    system::{
        api_key_auth::{SystemApiKeyAuthState, optional_system_api_key_auth},
        store::{SYSTEM_ADMIN_BOOTSTRAP_SQL, SystemAdminStore},
    },
};
use rudi::Context;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;

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
    let shared_db = match runtime.block_on(db::install_shared_db_singleton(
        &mut di,
        database_url.as_deref(),
        aio_toasty_models(),
        &bootstrap_sql,
    )) {
        Ok(shared_db) => shared_db,
        Err(error) => {
            eprintln!("AIO shared Toasty startup degraded: {error:#}");
            None
        }
    };

    let native_context = az_aio_platform::plugin::api::NativePluginContext {
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

    let app = Router::new()
        .route("/", get(root_page))
        .route("/gateway", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new(&assets_dir))
        .merge(native_router.with_state(()));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_page() -> Html<&'static str> {
    Html(include_str!("../assets/react/index.html"))
}

async fn health() -> &'static str {
    "ok"
}

fn aio_toasty_models() -> toasty::ModelSet {
    toasty::models!(
        az_aio_platform::system::model::SystemPageRecord,
        az_aio_platform::system::model::SystemOperationRecord,
        az_aio_platform::system::model::SystemDataRecord,
        az_aio_platform::system::model::SystemApiKeyRecord,
        config_center::backend::model::ConfigEntry,
        drive_center::backend::model::DriveTask,
        asset_hub::backend::model::AssetRecord,
        software_center::backend::model::SoftwarePackageRecord,
        edge_gateway::backend::model::GatewayFlow,
        edge_gateway::backend::model::GatewayRouteDefinition,
        edge_gateway::backend::model::EdgeApiTokenRecord,
        edge_gateway::backend::model::EdgeUsageRecordRow,
        az_engine::MetaModel,
        az_engine::MetaField,
        az_engine::HookDefinition,
        az_engine::DataRecord
    )
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
    config_center::enable();
    drive_center::enable();
    edge_gateway::enable();
    lowcode::enable();
    software_center::enable();
    az_linux::enable();
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
}
