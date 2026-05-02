mod auth;
mod runtime;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header, header::SET_COOKIE},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use tokio::sync::OnceCell;
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

use addzero_agent_runtime_contract::{
    AgentHeartbeat, AgentRuntimeOverview, LoginRequest, PairingCreateResponse,
    PairingExchangeRequest, PairingSessionSummary, ResolveConflictRequest, SessionUser,
    SkillSyncRequest, SkillSyncResponse,
};
use addzero_skills::{FsRepo, SkillService, SkillSource, SkillUpsert};

use crate::services::{
    BrandingSettingsDto, BrandingSettingsUpdate, KnowledgeExceptionCardDto, KnowledgeFeedDto,
    KnowledgeMaintenanceReportDto, KnowledgeNodeDetailDto, KnowledgeNodeSummaryDto,
    KnowledgeSourceRefDto, LogoUploadRequest, ResolveKnowledgeExceptionInput, SkillDto,
    SkillSourceDto, SkillUpsertDto, StoredLogoDto, SyncReportDto,
};

use self::auth::AdminSessionService;
use self::runtime::AgentRuntimeService;

pub struct BackendServices {
    pub skills: SkillService,
    pub runtime: AgentRuntimeService,
    pub admin_auth: AdminSessionService,
    pub cli_market: crate::services::cli_market::CliMarketService,
    pub software_catalog: Option<addzero_software_catalog::SoftwareCatalogService>,
}

static SERVICES: OnceCell<BackendServices> = OnceCell::const_new();

pub async fn services() -> &'static BackendServices {
    SERVICES
        .get_or_init(|| async {
            let fs = FsRepo::default_root().unwrap_or_else(|err| {
                log::warn!("could not resolve fs root, falling back to ./skills: {err:?}");
                FsRepo::new(std::path::PathBuf::from("./skills"))
            });
            let database_url = std::env::var("DATABASE_URL").ok();
            let skills = SkillService::try_attach(database_url.as_deref(), fs).await;
            if skills.is_pg_online() {
                if let Err(err) = skills.sync_now().await {
                    log::warn!("initial skill sync failed: {err:?}");
                }
            }

            let base_url = std::env::var("ADDZERO_ADMIN_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8787".into());
            let runtime = AgentRuntimeService::try_attach(database_url.as_deref(), base_url).await;
            let admin_auth = AdminSessionService::from_env();
            let cli_market =
                crate::services::cli_market::CliMarketService::try_attach(database_url.as_deref())
                    .await;
            let software_catalog =
                if let Some(url) = database_url.as_deref().filter(|url| !url.trim().is_empty()) {
                    addzero_software_catalog::SoftwareCatalogService::connect(url)
                        .await
                        .ok()
                } else {
                    None
                };

            BackendServices {
                skills,
                runtime,
                admin_auth,
                cli_market,
                software_catalog,
            }
        })
        .await
}

pub async fn run_api_server() -> Result<()> {
    let bind = std::env::var("DIOXUS_ADMIN_API_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let address: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let router = Router::new()
        .route("/api/admin/session", get(get_session))
        .route("/api/admin/session/login", post(login))
        .route("/api/admin/session/logout", post(logout))
        .route("/api/admin/session/permissions", get(get_session_permissions))
        .route("/api/admin/storage/logo", post(upload_logo))
        .route(
            "/api/admin/settings/branding",
            get(get_branding_settings).post(save_branding_settings),
        )
        .route("/api/skills", get(list_skills))
        .route("/api/skills/status", get(skill_status))
        .route("/api/skills/sync", post(sync_skills))
        .route("/api/skills/upsert", post(upsert_skill))
        .route("/api/skills/{name}", get(get_skill).delete(delete_skill))
        .route("/api/runtime/overview", get(runtime_overview))
        .route("/api/runtime/pairings", post(create_pairing))
        .route("/api/runtime/pairings/{id}", get(get_pairing))
        .route("/api/runtime/pairings/{id}/approve", post(approve_pairing))
        .route(
            "/api/runtime/pairings/{id}/exchange",
            post(exchange_pairing),
        )
        .route("/api/runtime/heartbeat", post(heartbeat))
        .route("/api/runtime/skills/sync", post(runtime_skill_sync))
        .route(
            "/api/runtime/conflicts/{id}/resolve",
            post(resolve_conflict),
        )
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
        .route("/api/admin/system/menus", get(sys_list_menus).post(sys_create_menu))
        .route(
            "/api/admin/system/menus/{id}",
            put(sys_update_menu).delete(sys_delete_menu),
        )
        .route("/api/admin/system/roles", get(sys_list_roles).post(sys_create_role))
        .route(
            "/api/admin/system/roles/{id}",
            get(sys_get_role).put(sys_update_role).delete(sys_delete_role),
        )
        .route(
            "/api/admin/system/roles/{id}/menus",
            put(sys_authorize_role_menus),
        )
        .route("/api/admin/system/users", get(sys_list_users).post(sys_create_user))
        .route(
            "/api/admin/system/users/{id}",
            get(sys_get_user).put(sys_update_user).delete(sys_delete_user),
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
        .route("/api/admin/system/departments", get(sys_list_departments).post(sys_create_department))
        .route(
            "/api/admin/system/departments/{id}",
            put(sys_update_department).delete(sys_delete_department),
        )
        // ─── Dictionary Groups ────────────────────────────────────
        .route("/api/admin/system/dict-groups", get(sys_list_dict_groups).post(sys_create_dict_group))
        .route(
            "/api/admin/system/dict-groups/{id}",
            put(sys_update_dict_group).delete(sys_delete_dict_group),
        )
        // ─── Dictionary Items ─────────────────────────────────────
        .route("/api/admin/system/dict-items", get(sys_list_dict_items).post(sys_create_dict_item))
        .route(
            "/api/admin/system/dict-items/{id}",
            put(sys_update_dict_item).delete(sys_delete_dict_item),
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
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
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
    let backend = services().await;
    Ok(Json(backend.admin_auth.session_user(&headers)))
}

async fn login(Json(input): Json<LoginRequest>) -> ApiResult<Response> {
    let backend = services().await;
    let cookie = backend
        .admin_auth
        .authenticate(&input)
        .map_err(|err| ApiError::unauthorized(err.message()))?;
    let mut response = Json(SessionUser {
        authenticated: true,
        username: Some(input.username.trim().to_string()),
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&backend.admin_auth.set_cookie_header(&cookie))
            .map_err(|_| ApiError::internal("failed to encode session cookie"))?,
    );
    Ok(response)
}

async fn logout() -> ApiResult<Response> {
    let backend = services().await;
    let mut response = Json(SessionUser {
        authenticated: false,
        username: None,
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&backend.admin_auth.clear_cookie_header())
            .map_err(|_| ApiError::internal("failed to encode logout cookie"))?,
    );
    Ok(response)
}

async fn get_session_permissions(headers: HeaderMap) -> ApiResult<Json<Vec<String>>> {
    let backend = services().await;
    let username = backend
        .admin_auth
        .current_user(&headers)
        .ok_or_else(|| ApiError::unauthorized("未登录"))?;
    let codes = crate::services::system_management::get_effective_permission_codes_on_server(&username)
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

async fn runtime_overview(headers: HeaderMap) -> ApiResult<Json<AgentRuntimeOverview>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let overview = backend
        .runtime
        .overview(
            backend.skills.fs_root_display(),
            backend.skills.is_pg_online(),
        )
        .await
        .map_err(ApiError::internal_from)?;
    Ok(Json(overview))
}

async fn create_pairing(
    Json(input): Json<addzero_agent_runtime_contract::PairingRequest>,
) -> ApiResult<Json<PairingCreateResponse>> {
    let backend = services().await;
    let response = backend
        .runtime
        .create_pairing(input)
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(response))
}

#[derive(serde::Deserialize)]
struct PollQuery {
    poll_token: Option<String>,
}

async fn get_pairing(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(query): Query<PollQuery>,
) -> ApiResult<Json<PairingSessionSummary>> {
    let backend = services().await;
    if query.poll_token.is_none() {
        ensure_auth(&backend.admin_auth, &headers)?;
    }
    let pairing = backend
        .runtime
        .get_pairing(id, query.poll_token.as_deref())
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(pairing))
}

async fn approve_pairing(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<PairingSessionSummary>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let pairing = backend
        .runtime
        .approve_pairing(id)
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(pairing))
}

async fn exchange_pairing(
    Path(id): Path<Uuid>,
    Json(input): Json<PairingExchangeRequest>,
) -> ApiResult<Json<addzero_agent_runtime_contract::PairingExchangeResponse>> {
    let backend = services().await;
    let response = backend
        .runtime
        .exchange_pairing(id, input)
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(response))
}

async fn heartbeat(
    Json(input): Json<AgentHeartbeat>,
) -> ApiResult<Json<addzero_agent_runtime_contract::AgentNode>> {
    let backend = services().await;
    let node = backend
        .runtime
        .heartbeat(input)
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(node))
}

async fn runtime_skill_sync(
    Json(input): Json<SkillSyncRequest>,
) -> ApiResult<Json<SkillSyncResponse>> {
    let backend = services().await;
    let response = backend
        .runtime
        .sync_skills(input, &backend.skills)
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(response))
}

async fn resolve_conflict(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<ResolveConflictRequest>,
) -> ApiResult<Json<addzero_agent_runtime_contract::SkillConflict>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let conflict = backend
        .runtime
        .resolve_conflict(id, input, &backend.skills)
        .await
        .map_err(ApiError::bad_request_from)?;
    Ok(Json(conflict))
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
    AuthorizeRoleMenusDto, AuthorizeUserRolesDto, MenuDto, MenuUpsertDto, RoleDto,
    RoleUpsertDto, RoleWithMenusDto, UserDto, UserUpsertDto, UserWithRolesDto,
    DepartmentDto, DepartmentUpsertDto, DictGroupDto, DictGroupUpsertDto,
    DictItemDto, DictItemUpsertDto,
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

fn ensure_auth(auth: &AdminSessionService, headers: &HeaderMap) -> ApiResult<()> {
    if auth.current_user(headers).is_none() {
        return Err(ApiError::unauthorized("需要先登录后台"));
    }
    Ok(())
}

fn skill_to_dto(skill: addzero_skills::Skill) -> SkillDto {
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
    report: addzero_skills::SyncReport,
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

    fn bad_request_from(err: anyhow::Error) -> Self {
        Self::bad_request(err.to_string())
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
