mod auth;
mod rhai_handlers;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::{collections::BTreeMap, env, fs};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post, put},
};
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
use tower_http::cors::{AllowOrigin, CorsLayer};

use az_agent_runtime_contract::{LoginRequest, SessionUser};
use az_plugin_contract::ResolvedPage;
use az_script_engine::script::ScriptEngine;
use az_skills::{FsRepo, SkillService, SkillSource, SkillUpsert};
use serde::Deserialize;
use uuid::Uuid;

use crate::services::{
    AiProviderConfigDto, AiProviderConfigUpsertDto, AssetGraphDto, AssetSyncReportDto,
    BootstrapDatabaseSaveResultDto, BootstrapDatabaseSetupDto, BootstrapStatusDto,
    BrandingSettingsDto, BrandingSettingsUpdate, ChatRequestDto, ChatResponseDto, FilterOptions,
    KnowledgeEntryDeleteDto, KnowledgeEntryUpsertDto, KnowledgeExceptionCardDto, KnowledgeFeedDto,
    KnowledgeMaintenanceReportDto, KnowledgeNodeDetailDto, KnowledgeNodeSummaryDto,
    KnowledgeNoteDto, KnowledgeSourceRefDto, LogoUploadRequest, PluginDescriptorDto,
    PluginInstallRequestDto, ResolveKnowledgeExceptionInput, SkillDto, SkillSourceDto,
    SkillUpsertDto, StartVibeCodingRequestDto, StartVibeCodingResponseDto, StorageBrowseRequestDto,
    StorageBrowseResultDto, StorageCreateFolderDto, StorageCreateFolderResultDto,
    StorageDeleteFolderDto, StorageDeleteObjectDto, StorageDeleteResultDto, StorageShareRequestDto,
    StorageShareResultDto, StorageUploadRequestDto, StorageUploadResultDto, StoredLogoDto,
    SyncReportDto, TerminalSessionCreateDto, TerminalSessionInputDto, TerminalSessionListDto,
    TerminalSessionSnapshotDto, WasmPluginInstallRequestDto, WasmPluginInstallResultDto,
    WasmPluginRuntimeSnapshotDto, WasmPluginUploadRequestDto, WasmPluginUploadResultDto,
    download_station::{FileIndex, ScanStats, ShareLink},
    menu_system::{CreateMenuRequest, Menu, MenuTreeNode, Permission, UpdateMenuRequest},
};

use self::auth::AdminSessionService;

pub struct BackendServices {
    pub skills: SkillService,
    pub admin_auth: AdminSessionService,
    pub assets: az_assets::AssetService,
    pub cli_market: crate::services::cli_market::CliMarketService,
    pub software_catalog: Option<az_software_catalog::SoftwareCatalogService>,
    pub download_station: Option<crate::services::download_station::DownloadStationService>,
    pub menu_system: Option<crate::services::menu_system::MenuService>,
}

static SERVICES: OnceCell<BackendServices> = OnceCell::const_new();
static ADMIN_AUTH: Lazy<AdminSessionService> = Lazy::new(AdminSessionService::from_env);

pub(crate) fn admin_auth() -> &'static AdminSessionService {
    &ADMIN_AUTH
}

pub fn resolved_database_url() -> Option<String> {
    az_persistence::database_url().or_else(|| {
        let values = local_env_values();
        values
            .get("MSC_AIO_DATABASE_URL")
            .cloned()
            .or_else(|| values.get("DATABASE_URL").cloned())
            .filter(|value| !value.trim().is_empty())
    })
}

fn is_postgres_database_url(database_url: &str) -> bool {
    database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")
}

fn local_env_values() -> BTreeMap<String, String> {
    let Some(path) = az_persistence::local_env_path() else {
        return BTreeMap::new();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub async fn services() -> &'static BackendServices {
    SERVICES
        .get_or_init(|| async {
            let fs = FsRepo::default_root().unwrap_or_else(|err| {
                log::warn!("could not resolve fs root, falling back to ./skills: {err:?}");
                FsRepo::new(std::path::PathBuf::from("./skills"))
            });
            let database_url = resolved_database_url();
            let postgres_database_url = database_url
                .as_deref()
                .filter(|url| is_postgres_database_url(url));
            let skills = SkillService::try_attach(postgres_database_url, fs).await;
            if skills.is_pg_online() {
                if let Err(err) = skills.sync_now().await {
                    log::warn!("initial skill sync failed: {err:?}");
                }
            }

            let admin_auth = admin_auth().clone();
            let secret_master_key = env::var("ADDZERO_SECRET_MASTER_KEY").ok();
            if let Some(url) = postgres_database_url {
                if let Err(err) = ensure_ai_provider_schema(url).await {
                    log::warn!("ensure ai provider schema failed: {err:?}");
                }
            }
            let assets = az_assets::AssetService::try_attach(
                postgres_database_url,
                secret_master_key.as_deref(),
            )
            .await;
            let cli_market =
                crate::services::cli_market::CliMarketService::try_attach(postgres_database_url)
                    .await;
            let software_catalog = if let Some(url) = postgres_database_url {
                az_software_catalog::SoftwareCatalogService::connect(url)
                    .await
                    .ok()
            } else {
                None
            };

            let download_station = if let Some(url) = postgres_database_url {
                sqlx::PgPool::connect(url)
                    .await
                    .ok()
                    .map(|pool| crate::services::default_download_station_api(pool))
            } else {
                None
            };

            let menu_system = if let Some(url) = postgres_database_url {
                sqlx::PgPool::connect(url)
                    .await
                    .ok()
                    .map(crate::services::menu_system::MenuService::new)
            } else {
                None
            };

            BackendServices {
                skills,
                admin_auth,
                assets,
                cli_market,
                software_catalog,
                download_station,
                menu_system,
            }
        })
        .await
}

async fn ensure_ai_provider_schema(database_url: &str) -> Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ai_model_providers (
            provider TEXT PRIMARY KEY,
            base_url TEXT,
            default_model TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            key_id TEXT NOT NULL DEFAULT 'default',
            encrypted_api_key TEXT,
            api_key_configured BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_model_providers ADD COLUMN IF NOT EXISTS base_url TEXT")
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn run_migrations() -> Result<()> {
    let database_url = resolved_database_url().expect(
        "MSC_AIO_DATABASE_URL / repo .env / DATABASE_URL / ~/.config/aio/aio.env must be set",
    );

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let files = migration_files()?;

    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy();
        let sql = std::fs::read_to_string(file)?;
        println!("Running: {name}");
        for stmt in split_sql_statements(&sql) {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Err(e) = sqlx::query(trimmed).execute(&pool).await {
                eprintln!("  SKIP (may already exist): {e}");
            }
        }
    }

    println!("Migrations complete ({} files)", files.len());
    Ok(())
}

pub async fn print_migration_status() -> Result<()> {
    let files = migration_files()?;
    println!("Migration files: {}", files.len());
    for file in &files {
        println!(" - {}", file.file_name().unwrap().to_string_lossy());
    }

    let Some(database_url) = resolved_database_url() else {
        println!("Database: not configured");
        return Ok(());
    };

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM information_schema.tables WHERE table_schema = 'public'",
    )
    .fetch_one(&pool)
    .await?;

    println!("Database: configured");
    println!("Public tables: {table_count}");
    Ok(())
}

fn migration_files() -> Result<Vec<PathBuf>> {
    let migration_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/server/migrations");
    let extra_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

    let mut files: Vec<_> = std::fs::read_dir(&migration_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "sql"))
        .map(|entry| entry.path())
        .collect();

    if extra_dir.exists() {
        files.extend(
            std::fs::read_dir(&extra_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "sql"))
                .map(|entry| entry.path()),
        );
    }

    files.sort();
    Ok(files)
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let bytes = sql.as_bytes();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut dollar_tag: Option<String> = None;

    while index < bytes.len() {
        let ch = bytes[index] as char;

        if let Some(tag) = dollar_tag.as_deref() {
            if ch == '$' && sql[index..].starts_with(tag) {
                index += tag.len();
                dollar_tag = None;
            } else {
                index += 1;
            }
            continue;
        }

        if in_single_quote {
            if ch == '\'' {
                if bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                    continue;
                }
                in_single_quote = false;
            }
            index += 1;
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                in_double_quote = false;
            }
            index += 1;
            continue;
        }

        match ch {
            '\'' => {
                in_single_quote = true;
                index += 1;
            }
            '"' => {
                in_double_quote = true;
                index += 1;
            }
            '$' => {
                if let Some(tag) = read_dollar_tag(&sql[index..]) {
                    index += tag.len();
                    dollar_tag = Some(tag);
                } else {
                    index += 1;
                }
            }
            ';' => {
                let statement = sql[start..index].trim();
                if !statement.is_empty() {
                    statements.push(statement.to_string());
                }
                index += 1;
                start = index;
            }
            _ => {
                index += 1;
            }
        }
    }

    let statement = sql[start..].trim();
    if !statement.is_empty() {
        statements.push(statement.to_string());
    }

    statements
}

fn read_dollar_tag(input: &str) -> Option<String> {
    let rest = input.strip_prefix('$')?;
    let end = rest.find('$')?;
    let tag = &rest[..end];
    if tag
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        Some(format!("${tag}$"))
    } else {
        None
    }
}

#[cfg(test)]
mod migration_tests {
    use super::split_sql_statements;

    #[test]
    fn split_sql_statements_keeps_dollar_quoted_blocks_together() {
        let sql = r#"
            DO $$
            BEGIN
                IF TRUE THEN
                    RAISE NOTICE 'contains; semicolon';
                END IF;
            END $$;
            CREATE TABLE IF NOT EXISTS demo (id INTEGER);
        "#;

        let statements = split_sql_statements(sql);

        assert_eq!(
            statements.len(),
            2,
            "the DO block must remain one executable statement"
        );
        assert!(statements[0].contains("RAISE NOTICE"));
        assert!(statements[1].starts_with("CREATE TABLE"));
    }
}

pub async fn run_api_server() -> Result<()> {
    let bind = std::env::var("AIO_API_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let address: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let router = Router::new()
        .route("/api/bootstrap/status", get(get_bootstrap_status))
        .route("/api/bootstrap/database", post(save_bootstrap_database))
        .route(
            "/api/bootstrap/database/sqlite-local",
            post(save_bootstrap_local_sqlite),
        )
        .route("/api/admin/session", get(get_session))
        .route("/api/admin/session/login", post(login))
        .route("/api/admin/session/logout", post(logout))
        .route(
            "/api/admin/session/permissions",
            get(get_session_permissions),
        )
        .route("/api/admin/storage/logo", post(upload_logo))
        .route("/api/admin/storage/files/browse", post(browse_files))
        .route("/api/admin/storage/files/upload", post(upload_files))
        .route("/api/admin/storage/files/folders", post(create_folder))
        .route(
            "/api/admin/storage/files/folders/delete",
            post(delete_folder),
        )
        .route("/api/admin/storage/files/share", post(share_file))
        .route("/api/admin/storage/files/delete", post(delete_file))
        .route(
            "/api/admin/storage/files/download/{token}",
            get(download_file),
        )
        .route("/api/admin/assets/sync", post(sync_assets))
        .route("/api/admin/assets/graph", get(load_asset_graph))
        .route(
            "/api/admin/settings/branding",
            get(get_branding_settings).post(save_branding_settings),
        )
        .route("/api/skills", get(list_skills))
        .route("/api/skills/status", get(skill_status))
        .route("/api/skills/sync", post(sync_skills))
        .route("/api/skills/upsert", post(upsert_skill))
        .route("/api/skills/{name}", get(get_skill).delete(delete_skill))
        .route(
            "/api/knowledge/entries",
            get(list_knowledge_entries).post(save_knowledge_entry),
        )
        .route(
            "/api/knowledge/entries/delete",
            post(delete_knowledge_entry),
        )
        .route(
            "/api/ai/providers",
            get(list_ai_providers).post(save_ai_provider),
        )
        .route("/api/ai/chat", post(run_ai_chat))
        .route("/api/plugins/builtin", get(list_builtin_plugins))
        .route("/api/plugins", get(list_loaded_plugins))
        .route("/api/plugins/install", post(install_plugin))
        .route("/api/plugins/{id}/enable", post(enable_plugin))
        .route("/api/plugins/{id}/disable", post(disable_plugin))
        .route("/api/plugins/{id}", axum::routing::delete(uninstall_plugin))
        .route("/api/wasm/plugins/overview", get(wasm_plugin_overview))
        .route("/api/wasm/plugins/upload", post(upload_wasm_plugin))
        .route(
            "/api/wasm/plugins/install-catalog",
            post(install_catalog_wasm_plugin),
        )
        .route(
            "/api/wasm/plugins/system/{plugin_id}/{page_id}",
            get(resolve_system_wasm_plugin_page),
        )
        .route(
            "/api/wasm/plugins/apps/{instance_slug}/{page_id}",
            get(resolve_instance_wasm_plugin_page),
        )
        .route("/api/vibe-coding/start", post(start_vibe_coding))
        .route(
            "/api/admin/terminal/sessions",
            get(list_terminal_sessions).post(create_terminal_session),
        )
        .route(
            "/api/admin/terminal/sessions/{id}",
            get(get_terminal_session_snapshot).delete(close_terminal_session),
        )
        .route(
            "/api/admin/terminal/sessions/{id}/input",
            post(send_terminal_session_input),
        )
        .route("/api/admin/menus/tree", get(get_menu_tree))
        .route("/api/admin/menus", post(create_menu).put(update_menu))
        .route("/api/admin/menus/{id}", get(get_menu).delete(delete_menu))
        .route("/api/admin/menus/sync", post(sync_file_routes))
        .route("/api/admin/permissions", get(get_permissions))
        .route("/api/admin/knowledge/feed", get(knowledge_feed))
        .route(
            "/api/admin/knowledge/nodes/{id}",
            get(knowledge_node_detail),
        )
        .route(
            "/api/admin/knowledge/nodes/{id}/sources",
            get(knowledge_node_sources),
        )
        .route("/api/admin/knowledge/exceptions", get(knowledge_exceptions))
        .route("/api/admin/knowledge/raw-items", post(knowledge_ingest_raw))
        .route(
            "/api/admin/knowledge/exceptions/{id}/resolve",
            post(knowledge_resolve_exception),
        )
        .route(
            "/api/admin/knowledge/maintenance/run",
            post(knowledge_run_maintenance),
        )
        // ─── System Management ──────────────────────────────────────
        .route(
            "/api/admin/system/menus",
            get(sys_list_menus).post(sys_create_menu),
        )
        .route(
            "/api/admin/system/menus/{id}",
            put(sys_update_menu).delete(sys_delete_menu),
        )
        .route(
            "/api/admin/system/roles",
            get(sys_list_roles).post(sys_create_role),
        )
        .route(
            "/api/admin/system/roles/{id}",
            get(sys_get_role)
                .put(sys_update_role)
                .delete(sys_delete_role),
        )
        .route(
            "/api/admin/system/roles/{id}/menus",
            put(sys_authorize_role_menus),
        )
        .route(
            "/api/admin/system/users",
            get(sys_list_users).post(sys_create_user),
        )
        .route(
            "/api/admin/system/users/{id}",
            get(sys_get_user)
                .put(sys_update_user)
                .delete(sys_delete_user),
        )
        .route(
            "/api/admin/system/users/{id}/roles",
            put(sys_authorize_user_roles),
        )
        .route(
            "/api/admin/system/users/{id}/menus",
            get(sys_get_user_effective_menus),
        )
        // ─── Departments ──────────────────────────────────────────
        .route(
            "/api/admin/system/departments",
            get(sys_list_departments).post(sys_create_department),
        )
        .route(
            "/api/admin/system/departments/{id}",
            put(sys_update_department).delete(sys_delete_department),
        )
        // ─── Dictionary Groups ────────────────────────────────────
        .route(
            "/api/admin/system/dict-groups",
            get(sys_list_dict_groups).post(sys_create_dict_group),
        )
        .route(
            "/api/admin/system/dict-groups/{id}",
            put(sys_update_dict_group).delete(sys_delete_dict_group),
        )
        // ─── Dictionary Items ─────────────────────────────────────
        .route(
            "/api/admin/system/dict-items",
            get(sys_list_dict_items).post(sys_create_dict_item),
        )
        .route(
            "/api/admin/system/dict-items/{id}",
            put(sys_update_dict_item).delete(sys_delete_dict_item),
        )
        // ─── Download Station ───────────────────────────────────────
        .route(
            "/api/admin/download-station/scan",
            post(ds_scan_directories),
        )
        .route("/api/admin/download-station/files", post(ds_list_files))
        .route("/api/admin/download-station/stats", get(ds_get_stats))
        .route("/api/admin/download-station/share", post(ds_create_share))
        .route(
            "/api/admin/download-station/share/{token}",
            get(ds_get_share),
        )
        .route(
            "/api/admin/download-station/download/{source}/{path:path}",
            get(ds_download_file),
        )
        .route("/api/engine/rhai/run", post(run_rhai))
        .route(
            "/api/scripts",
            get(rhai_handlers::list_scripts).post(rhai_handlers::save_script),
        )
        .route(
            "/api/scripts/{name}",
            get(rhai_handlers::get_script).delete(rhai_handlers::delete_script),
        )
        .route(
            "/api/engine/rhai/eval-env",
            post(rhai_handlers::eval_rhai_env),
        )
        .layer(cors_layer());

    axum::serve(listener, router).await?;
    Ok(())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            is_allowed_admin_origin(origin)
        }))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true)
}

fn is_allowed_admin_origin(origin: &HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };

    ["http://localhost:", "http://127.0.0.1:", "http://[::1]:"]
        .iter()
        .any(|prefix| {
            origin
                .strip_prefix(prefix)
                .is_some_and(is_valid_local_dev_port)
        })
}

fn is_valid_local_dev_port(port: &str) -> bool {
    !port.is_empty() && port.parse::<u16>().is_ok()
}

async fn get_session(headers: HeaderMap) -> ApiResult<Json<SessionUser>> {
    Ok(Json(admin_auth().session_user(&headers)))
}

async fn get_bootstrap_status() -> ApiResult<Json<BootstrapStatusDto>> {
    let status = crate::services::desktop_bootstrap::bootstrap_status_on_server()
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(status))
}

async fn save_bootstrap_database(
    Json(input): Json<BootstrapDatabaseSetupDto>,
) -> ApiResult<Json<BootstrapDatabaseSaveResultDto>> {
    let result = crate::services::desktop_bootstrap::save_database_url_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(result))
}

async fn save_bootstrap_local_sqlite() -> ApiResult<Json<BootstrapDatabaseSaveResultDto>> {
    let result = crate::services::desktop_bootstrap::save_local_sqlite_on_server()
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(result))
}

async fn login(Json(input): Json<LoginRequest>) -> ApiResult<Response> {
    let auth = admin_auth();
    let cookie = auth
        .authenticate(&input)
        .map_err(|err| ApiError::unauthorized(err.message()))?;
    let mut response = Json(SessionUser {
        authenticated: true,
        username: Some(input.username.trim().to_string()),
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&auth.set_cookie_header(&cookie))
            .map_err(|_| ApiError::internal("failed to encode session cookie"))?,
    );
    Ok(response)
}

async fn logout() -> ApiResult<Response> {
    let auth = admin_auth();
    let mut response = Json(SessionUser {
        authenticated: false,
        username: None,
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&auth.clear_cookie_header())
            .map_err(|_| ApiError::internal("failed to encode logout cookie"))?,
    );
    Ok(response)
}

async fn get_session_permissions(headers: HeaderMap) -> ApiResult<Json<Vec<String>>> {
    let username = admin_auth()
        .current_user(&headers)
        .ok_or_else(|| ApiError::unauthorized("未登录"))?;
    let codes =
        crate::services::system_management::get_effective_permission_codes_on_server(&username)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    match codes {
        None => Ok(Json(Vec::new())), // admin: empty vec = no restriction (frontend treats empty=all)
        Some(codes) => Ok(Json(codes)),
    }
}

async fn get_branding_settings(headers: HeaderMap) -> ApiResult<Json<BrandingSettingsDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let settings = crate::services::branding_settings::load_branding_settings_on_server()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(settings))
}

async fn sync_assets(headers: HeaderMap) -> ApiResult<Json<AssetSyncReportDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let report = crate::services::asset_graph::sync_assets_on_server()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(report))
}

async fn load_asset_graph(headers: HeaderMap) -> ApiResult<Json<AssetGraphDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let graph = crate::services::asset_graph::load_asset_graph_on_server()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(graph))
}

async fn save_branding_settings(
    headers: HeaderMap,
    Json(input): Json<BrandingSettingsUpdate>,
) -> ApiResult<Json<BrandingSettingsDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let settings = crate::services::branding_settings::save_branding_settings_on_server(input)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(settings))
}

async fn upload_logo(
    headers: HeaderMap,
    Json(input): Json<LogoUploadRequest>,
) -> ApiResult<Json<StoredLogoDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let stored = tokio::task::spawn_blocking(move || {
        crate::services::logo_storage::upload_logo_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("logo 上传任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(stored))
}

async fn browse_files(
    headers: HeaderMap,
    Json(input): Json<StorageBrowseRequestDto>,
) -> ApiResult<Json<StorageBrowseResultDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let explorer = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::browse_files_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("文件浏览任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(explorer))
}

async fn upload_files(
    headers: HeaderMap,
    Json(input): Json<StorageUploadRequestDto>,
) -> ApiResult<Json<StorageUploadResultDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let report = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::upload_files_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("文件上传任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(report))
}

async fn create_folder(
    headers: HeaderMap,
    Json(input): Json<StorageCreateFolderDto>,
) -> ApiResult<Json<StorageCreateFolderResultDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let result = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::create_folder_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("创建目录任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(result))
}

async fn share_file(
    headers: HeaderMap,
    Json(input): Json<StorageShareRequestDto>,
) -> ApiResult<Json<StorageShareResultDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let result = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::share_file_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("分享链接生成任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(result))
}

async fn delete_file(
    headers: HeaderMap,
    Json(input): Json<StorageDeleteObjectDto>,
) -> ApiResult<Json<StorageDeleteResultDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let result = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::delete_file_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("删除文件任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(result))
}

async fn delete_folder(
    headers: HeaderMap,
    Json(input): Json<StorageDeleteFolderDto>,
) -> ApiResult<Json<StorageDeleteResultDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let result = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::delete_folder_on_server(input)
    })
    .await
    .map_err(|err| ApiError::internal(format!("删除目录任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(result))
}

async fn download_file(headers: HeaderMap, Path(token): Path<String>) -> ApiResult<Redirect> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let url = tokio::task::spawn_blocking(move || {
        crate::services::minio_files::presign_download_url_on_server(&token)
    })
    .await
    .map_err(|err| ApiError::internal(format!("生成下载链接任务失败：{err}")))?
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Redirect::temporary(&url))
}

async fn list_skills(headers: HeaderMap) -> ApiResult<Json<Vec<SkillDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let skills = backend
        .skills
        .list()
        .await
        .map_err(ApiError::internal_from)?;
    Ok(Json(skills.into_iter().map(skill_to_dto).collect()))
}

async fn get_skill(
    headers: HeaderMap,
    Path(name): Path<String>,
) -> ApiResult<Json<Option<SkillDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let skill = backend
        .skills
        .get(name.as_str())
        .await
        .map_err(ApiError::internal_from)?;
    Ok(Json(skill.map(skill_to_dto)))
}

async fn upsert_skill(
    headers: HeaderMap,
    Json(input): Json<SkillUpsertDto>,
) -> ApiResult<Json<SkillDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let saved = backend
        .skills
        .upsert(SkillUpsert {
            name: input.name,
            keywords: input.keywords,
            description: input.description,
            body: input.body,
        })
        .await
        .map_err(ApiError::internal_from)?;
    Ok(Json(skill_to_dto(saved)))
}

async fn delete_skill(headers: HeaderMap, Path(name): Path<String>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    backend
        .skills
        .delete(name.as_str())
        .await
        .map_err(ApiError::internal_from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sync_skills(headers: HeaderMap) -> ApiResult<Json<SyncReportDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let report = backend
        .skills
        .sync_now()
        .await
        .map_err(ApiError::internal_from)?;
    Ok(Json(sync_report_to_dto(
        report,
        backend.skills.is_pg_online(),
        backend.skills.fs_root_display(),
    )))
}

async fn skill_status(headers: HeaderMap) -> ApiResult<Json<SyncReportDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let report = backend.skills.last_report().await.unwrap_or_default();
    Ok(Json(sync_report_to_dto(
        report,
        backend.skills.is_pg_online(),
        backend.skills.fs_root_display(),
    )))
}

async fn list_knowledge_entries(headers: HeaderMap) -> ApiResult<Json<Vec<KnowledgeNoteDto>>> {
    ensure_auth(admin_auth(), &headers)?;
    let notes = crate::services::knowledge_entries::list_knowledge_entries_on_server()
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(notes))
}

async fn save_knowledge_entry(
    headers: HeaderMap,
    Json(input): Json<KnowledgeEntryUpsertDto>,
) -> ApiResult<Json<KnowledgeNoteDto>> {
    ensure_auth(admin_auth(), &headers)?;
    let saved = crate::services::knowledge_entries::save_knowledge_entry_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(saved))
}

async fn delete_knowledge_entry(
    headers: HeaderMap,
    Json(input): Json<KnowledgeEntryDeleteDto>,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_auth(admin_auth(), &headers)?;
    crate::services::knowledge_entries::delete_knowledge_entry_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn list_ai_providers(headers: HeaderMap) -> ApiResult<Json<Vec<AiProviderConfigDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let providers = crate::services::ai_chat::list_provider_configs_on_server()
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(providers))
}

async fn save_ai_provider(
    headers: HeaderMap,
    Json(input): Json<AiProviderConfigUpsertDto>,
) -> ApiResult<Json<AiProviderConfigDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let provider = crate::services::ai_chat::upsert_provider_config_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(provider))
}

async fn run_ai_chat(
    headers: HeaderMap,
    Json(input): Json<ChatRequestDto>,
) -> ApiResult<Json<ChatResponseDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let response = crate::services::ai_chat::chat_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(response))
}

async fn list_builtin_plugins(headers: HeaderMap) -> ApiResult<Json<Vec<PluginDescriptorDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    Ok(Json(
        crate::services::plugins::list_builtin_plugins_on_server().await,
    ))
}

async fn list_loaded_plugins(headers: HeaderMap) -> ApiResult<Json<Vec<PluginDescriptorDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    Ok(Json(
        crate::services::plugins::list_loaded_plugins_on_server().await,
    ))
}

async fn install_plugin(
    headers: HeaderMap,
    Json(input): Json<PluginInstallRequestDto>,
) -> ApiResult<Json<PluginDescriptorDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let plugin = crate::services::plugins::install_plugin_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(plugin))
}

async fn enable_plugin(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<PluginDescriptorDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let plugin = crate::services::plugins::enable_plugin_on_server(id)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(plugin))
}

async fn disable_plugin(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<PluginDescriptorDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let plugin = crate::services::plugins::disable_plugin_on_server(id)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(plugin))
}

async fn uninstall_plugin(headers: HeaderMap, Path(id): Path<String>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::plugins::uninstall_plugin_on_server(id)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn wasm_plugin_overview(headers: HeaderMap) -> ApiResult<Json<WasmPluginRuntimeSnapshotDto>> {
    ensure_auth(admin_auth(), &headers)?;
    let snapshot = crate::services::wasm_plugins::wasm_plugin_runtime_snapshot_on_server()
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(snapshot))
}

async fn install_catalog_wasm_plugin(
    headers: HeaderMap,
    Json(input): Json<WasmPluginInstallRequestDto>,
) -> ApiResult<Json<WasmPluginInstallResultDto>> {
    ensure_auth(admin_auth(), &headers)?;
    let result = crate::services::wasm_plugins::install_catalog_wasm_plugin_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(result))
}

async fn upload_wasm_plugin(
    headers: HeaderMap,
    Json(input): Json<WasmPluginUploadRequestDto>,
) -> ApiResult<Json<WasmPluginUploadResultDto>> {
    ensure_auth(admin_auth(), &headers)?;
    let result = crate::services::wasm_plugins::upload_wasm_plugin_on_server(input)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(result))
}

async fn resolve_system_wasm_plugin_page(
    headers: HeaderMap,
    Path((plugin_id, page_id)): Path<(String, String)>,
) -> ApiResult<Json<ResolvedPage>> {
    ensure_auth(admin_auth(), &headers)?;
    let page = crate::services::wasm_plugins::resolve_system_wasm_plugin_page_on_server(
        plugin_id, page_id,
    )
    .await
    .map_err(ApiError::bad_request)?;
    Ok(Json(page))
}

async fn resolve_instance_wasm_plugin_page(
    headers: HeaderMap,
    Path((instance_slug, page_id)): Path<(String, String)>,
) -> ApiResult<Json<ResolvedPage>> {
    ensure_auth(admin_auth(), &headers)?;
    let page = crate::services::wasm_plugins::resolve_instance_wasm_plugin_page_on_server(
        instance_slug,
        page_id,
    )
    .await
    .map_err(ApiError::bad_request)?;
    Ok(Json(page))
}

async fn start_vibe_coding(
    headers: HeaderMap,
    Json(input): Json<StartVibeCodingRequestDto>,
) -> ApiResult<Json<StartVibeCodingResponseDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let response = crate::services::vibe_coding::start_vibe_coding_on_server(input)
        .map_err(ApiError::bad_request)?;
    Ok(Json(response))
}

async fn list_terminal_sessions(headers: HeaderMap) -> ApiResult<Json<TerminalSessionListDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let sessions = crate::services::terminal_sessions::list_terminal_sessions_on_server()
        .map_err(ApiError::bad_request)?;
    Ok(Json(sessions))
}

async fn create_terminal_session(
    headers: HeaderMap,
    Json(input): Json<TerminalSessionCreateDto>,
) -> ApiResult<Json<TerminalSessionSnapshotDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let session = crate::services::terminal_sessions::create_terminal_session_on_server(input)
        .map_err(ApiError::bad_request)?;
    Ok(Json(session))
}

async fn get_terminal_session_snapshot(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<TerminalSessionSnapshotDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let session = crate::services::terminal_sessions::get_terminal_session_snapshot_on_server(&id)
        .map_err(ApiError::bad_request)?;
    Ok(Json(session))
}

async fn send_terminal_session_input(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<TerminalSessionInputDto>,
) -> ApiResult<Json<TerminalSessionSnapshotDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let session = crate::services::terminal_sessions::send_terminal_input_on_server(&id, input)
        .map_err(ApiError::bad_request)?;
    Ok(Json(session))
}

async fn close_terminal_session(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::terminal_sessions::close_terminal_session_on_server(&id)
        .map_err(ApiError::bad_request)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn knowledge_feed(headers: HeaderMap) -> ApiResult<Json<KnowledgeFeedDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let feed = crate::services::knowledge_graph::load_knowledge_feed_on_server()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(feed))
}

async fn knowledge_node_detail(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<KnowledgeNodeDetailDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let detail = crate::services::knowledge_graph::load_knowledge_node_detail_on_server(&id)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(detail))
}

async fn knowledge_node_sources(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<KnowledgeSourceRefDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let sources = crate::services::knowledge_graph::load_knowledge_node_sources_on_server(&id)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(sources))
}

async fn knowledge_exceptions(
    headers: HeaderMap,
) -> ApiResult<Json<Vec<KnowledgeExceptionCardDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let items = crate::services::knowledge_graph::load_knowledge_exceptions_on_server()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(items))
}

async fn knowledge_ingest_raw(
    headers: HeaderMap,
    Json(input): Json<crate::services::IngestKnowledgeRawInput>,
) -> ApiResult<Json<KnowledgeNodeSummaryDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let node = crate::services::knowledge_graph::ingest_knowledge_raw_on_server(input)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(node))
}

async fn knowledge_resolve_exception(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ResolveKnowledgeExceptionInput>,
) -> ApiResult<Json<KnowledgeExceptionCardDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let item = crate::services::knowledge_graph::resolve_knowledge_exception_on_server(&id, input)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(item))
}

async fn knowledge_run_maintenance(
    headers: HeaderMap,
) -> ApiResult<Json<KnowledgeMaintenanceReportDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let report = crate::services::knowledge_graph::run_knowledge_maintenance_on_server()
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(report))
}

// ─── System Management Handlers ─────────────────────────────────────────────

use crate::services::system_management::{
    AuthorizeRoleMenusDto, AuthorizeUserRolesDto, DepartmentDto, DepartmentUpsertDto, DictGroupDto,
    DictGroupUpsertDto, DictItemDto, DictItemUpsertDto, MenuDto, MenuUpsertDto, RoleDto,
    RoleUpsertDto, RoleWithMenusDto, UserDto, UserUpsertDto, UserWithRolesDto,
};

async fn sys_list_menus(headers: HeaderMap) -> ApiResult<Json<Vec<MenuDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::list_menus_on_server()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

async fn sys_create_menu(
    headers: HeaderMap,
    Json(input): Json<MenuUpsertDto>,
) -> ApiResult<Json<MenuDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::create_menu_on_server(input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_update_menu(
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(input): Json<MenuUpsertDto>,
) -> ApiResult<Json<MenuDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::update_menu_on_server(id, input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_delete_menu(headers: HeaderMap, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::delete_menu_on_server(id)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_list_roles(headers: HeaderMap) -> ApiResult<Json<Vec<RoleDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::list_roles_on_server()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

async fn sys_get_role(
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> ApiResult<Json<RoleWithMenusDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::get_role_on_server(id)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_create_role(
    headers: HeaderMap,
    Json(input): Json<RoleUpsertDto>,
) -> ApiResult<Json<RoleDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::create_role_on_server(input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_update_role(
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(input): Json<RoleUpsertDto>,
) -> ApiResult<Json<RoleDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::update_role_on_server(id, input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_delete_role(headers: HeaderMap, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::delete_role_on_server(id)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_authorize_role_menus(
    headers: HeaderMap,
    Path(role_id): Path<i32>,
    Json(input): Json<AuthorizeRoleMenusDto>,
) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::authorize_role_menus_on_server(role_id, input.menu_ids)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_list_users(headers: HeaderMap) -> ApiResult<Json<Vec<UserWithRolesDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::list_users_on_server()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

async fn sys_get_user(
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> ApiResult<Json<UserWithRolesDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::get_user_on_server(id)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_create_user(
    headers: HeaderMap,
    Json(input): Json<UserUpsertDto>,
) -> ApiResult<Json<UserDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::create_user_on_server(input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_update_user(
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(input): Json<UserUpsertDto>,
) -> ApiResult<Json<UserDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::update_user_on_server(id, input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_delete_user(headers: HeaderMap, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::delete_user_on_server(id)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_authorize_user_roles(
    headers: HeaderMap,
    Path(user_id): Path<i32>,
    Json(input): Json<AuthorizeUserRolesDto>,
) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::authorize_user_roles_on_server(user_id, input.role_ids)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sys_get_user_effective_menus(
    headers: HeaderMap,
    Path(user_id): Path<i32>,
) -> ApiResult<Json<Vec<i32>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::get_user_effective_menu_ids_on_server(user_id)
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

// ─── Department Handlers ────────────────────────────────────────────────────

async fn sys_list_departments(headers: HeaderMap) -> ApiResult<Json<Vec<DepartmentDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::list_departments_on_server()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

async fn sys_create_department(
    headers: HeaderMap,
    Json(input): Json<DepartmentUpsertDto>,
) -> ApiResult<Json<DepartmentDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::create_department_on_server(input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_update_department(
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(input): Json<DepartmentUpsertDto>,
) -> ApiResult<Json<DepartmentDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::update_department_on_server(id, input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_delete_department(headers: HeaderMap, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::delete_department_on_server(id)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Dict Group Handlers ────────────────────────────────────────────────────

async fn sys_list_dict_groups(headers: HeaderMap) -> ApiResult<Json<Vec<DictGroupDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::list_dict_groups_on_server()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

async fn sys_create_dict_group(
    headers: HeaderMap,
    Json(input): Json<DictGroupUpsertDto>,
) -> ApiResult<Json<DictGroupDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::create_dict_group_on_server(input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_update_dict_group(
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(input): Json<DictGroupUpsertDto>,
) -> ApiResult<Json<DictGroupDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::update_dict_group_on_server(id, input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_delete_dict_group(headers: HeaderMap, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::delete_dict_group_on_server(id)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Dict Item Handlers ─────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct GroupIdQuery {
    group_id: i32,
}

async fn sys_list_dict_items(
    headers: HeaderMap,
    Query(q): Query<GroupIdQuery>,
) -> ApiResult<Json<Vec<DictItemDto>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::list_dict_items_on_server(q.group_id)
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

async fn sys_create_dict_item(
    headers: HeaderMap,
    Json(input): Json<DictItemUpsertDto>,
) -> ApiResult<Json<DictItemDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::create_dict_item_on_server(input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_update_dict_item(
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(input): Json<DictItemUpsertDto>,
) -> ApiResult<Json<DictItemDto>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::update_dict_item_on_server(id, input)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e.to_string()))
}

async fn sys_delete_dict_item(headers: HeaderMap, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    crate::services::system_management::delete_dict_item_on_server(id)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Download Station Handlers ─────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct ScanRequest {
    directories: Vec<String>,
}

async fn ds_scan_directories(
    headers: HeaderMap,
    Json(req): Json<ScanRequest>,
) -> ApiResult<Json<ScanStats>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ds = backend
        .download_station
        .as_ref()
        .ok_or_else(|| ApiError::internal("Download Station service not available"))?;
    let result: ScanStats = ds
        .scan_directories(req.directories)
        .await
        .map_err(|e: anyhow::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(result))
}

async fn ds_list_files(
    headers: HeaderMap,
    Json(filter): Json<FilterOptions>,
) -> ApiResult<Json<Vec<FileIndex>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ds = backend
        .download_station
        .as_ref()
        .ok_or_else(|| ApiError::internal("Download Station service not available"))?;
    let result: Vec<FileIndex> = ds
        .list_files(filter)
        .await
        .map_err(|e: anyhow::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(result))
}

async fn ds_get_stats(headers: HeaderMap) -> ApiResult<Json<ScanStats>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ds = backend
        .download_station
        .as_ref()
        .ok_or_else(|| ApiError::internal("Download Station service not available"))?;
    let result: ScanStats = ds
        .get_stats()
        .await
        .map_err(|e: anyhow::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(result))
}

#[derive(serde::Deserialize)]
struct CreateShareRequest {
    source: String,
    path: String,
    file_name: String,
    hours: i64,
}

async fn ds_create_share(
    headers: HeaderMap,
    Json(req): Json<CreateShareRequest>,
) -> ApiResult<Json<ShareLink>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ds = backend
        .download_station
        .as_ref()
        .ok_or_else(|| ApiError::internal("Download Station service not available"))?;
    let user = backend.admin_auth.current_user(&headers);
    let created_by = user.as_deref();
    let share_link = ds
        .create_share(
            &req.source,
            &req.path,
            &req.file_name,
            req.hours,
            created_by,
        )
        .await
        .map_err(|e: anyhow::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(share_link))
}

async fn ds_get_share(
    headers: HeaderMap,
    Path(token): Path<String>,
) -> ApiResult<Json<Option<ShareLink>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ds = backend
        .download_station
        .as_ref()
        .ok_or_else(|| ApiError::internal("Download Station service not available"))?;
    let result: Option<ShareLink> = ds
        .get_share(&token)
        .await
        .map_err(|e: anyhow::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(result))
}

async fn ds_download_file(
    headers: HeaderMap,
    Path((source, path)): Path<(String, String)>,
) -> ApiResult<Response> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;

    // 从数据库获取文件完整路径
    let ds = backend
        .download_station
        .as_ref()
        .ok_or_else(|| ApiError::internal("Download Station service not available"))?;

    let files: Vec<FileIndex> = ds
        .list_files(FilterOptions {
            source: Some(source.clone()),
            category: None,
            query: Some(path.clone()),
            offset: 0,
            limit: 1,
        })
        .await
        .map_err(|e: anyhow::Error| ApiError::internal(e.to_string()))?;

    let file = files
        .first()
        .ok_or_else(|| ApiError::bad_request("File not found"))?;

    let file_path = PathBuf::from(&file.full_path);
    if !file_path.exists() {
        return Err(ApiError::bad_request("File not found on disk"));
    }

    let mime_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    let contents = tokio::fs::read(&file_path)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to read file: {}", e)))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.name),
        )
        .body(axum::body::Body::from(contents))
        .map_err(|e| ApiError::internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

fn ensure_auth(auth: &AdminSessionService, headers: &HeaderMap) -> ApiResult<()> {
    if auth.current_user(headers).is_none() {
        return Err(ApiError::unauthorized("需要先登录后台"));
    }
    Ok(())
}

fn skill_to_dto(skill: az_skills::Skill) -> SkillDto {
    SkillDto {
        name: skill.name,
        keywords: skill.keywords,
        description: skill.description,
        body: skill.body,
        content_hash: skill.content_hash,
        updated_at: skill.updated_at,
        source: match skill.source {
            SkillSource::Postgres => SkillSourceDto::Postgres,
            SkillSource::FileSystem => SkillSourceDto::FileSystem,
            SkillSource::Both => SkillSourceDto::Both,
        },
    }
}

fn sync_report_to_dto(
    report: az_skills::SyncReport,
    pg_online: bool,
    fs_root: String,
) -> SyncReportDto {
    SyncReportDto {
        added_to_fs: report.added_to_fs,
        added_to_pg: report.added_to_pg,
        updated_in_fs: report.updated_in_fs,
        updated_in_pg: report.updated_in_pg,
        conflicts: report.conflicts,
        finished_at: report.finished_at,
        pg_online,
        fs_root,
    }
}

type ApiResult<T> = Result<T, ApiError>;

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn internal_from(err: anyhow::Error) -> Self {
        Self::internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

// ─── Script Engine Handlers ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RunRhaiRequest {
    source: String,
    #[serde(default)]
    vars: std::collections::BTreeMap<String, serde_json::Value>,
}

async fn run_rhai(
    headers: HeaderMap,
    Json(body): Json<RunRhaiRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;

    let engine = az_script_engine_rhai::RhaiEngine::new();
    let input = az_script_engine::script::ScriptInput {
        source: body.source,
        lang: az_script_engine::script::ScriptLang::Rhai,
        vars: body.vars,
        policy: az_sandbox::sandbox::SandboxPolicy::permissive(),
        timeout_secs: 30,
    };

    let output = tokio::task::spawn_blocking(move || engine.run(input))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "exit_code": output.exit_code,
        "stdout": output.stdout,
        "stderr": output.stderr,
        "vars": output.vars,
        "duration_ms": output.duration_ms,
    })))
}

// ─── Menu Management Handlers ─────────────────────────────────────────────────────

async fn get_menu_tree(headers: HeaderMap) -> ApiResult<Json<Vec<MenuTreeNode>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    let tree = ms
        .get_menu_tree()
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(tree))
}

async fn get_menu(Path(id): Path<Uuid>, headers: HeaderMap) -> ApiResult<Json<Menu>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    let menu = ms
        .get_menu_by_id(id)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(
        menu.ok_or_else(|| ApiError::not_found("Menu not found"))?,
    ))
}

async fn create_menu(
    headers: HeaderMap,
    Json(req): Json<CreateMenuRequest>,
) -> ApiResult<Json<Menu>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    let menu = ms
        .create_menu(req)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(menu))
}

async fn update_menu(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateMenuRequest>,
) -> ApiResult<Json<Menu>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    let menu = ms
        .update_menu(id, req)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(
        menu.ok_or_else(|| ApiError::not_found("Menu not found"))?,
    ))
}

async fn delete_menu(Path(id): Path<Uuid>, headers: HeaderMap) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    ms.delete_menu(id)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sync_file_routes(
    headers: HeaderMap,
    Json(routes): Json<Vec<String>>,
) -> ApiResult<Json<u64>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    let count = ms
        .sync_file_routes(routes)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(count))
}

async fn get_permissions(headers: HeaderMap) -> ApiResult<Json<Vec<Permission>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let ms = backend
        .menu_system
        .as_ref()
        .ok_or_else(|| ApiError::internal("Menu system not available"))?;
    let permissions = ms
        .get_all_permissions()
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e.to_string()))?;
    Ok(Json(permissions))
}
