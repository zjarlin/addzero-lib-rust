//! 系统后台管理 API。
//!
//! 这些接口暴露 admin shell 所需的页面契约、导航快照、PG 边界摘要与统一操作入口。
//! 操作路径来自同一套 [`SystemOperation`](crate::system::catalog::SystemOperation)
//! 定义，CLI 与 REST 不再维护两份漂移的接口面。

use anyhow::anyhow;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{OriginalUri, RawQuery, State},
    http::{HeaderMap, Method, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::core::api_error::{ApiError, ApiQuery, ApiResponse, ok_json};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::system::{
    catalog::{
        SystemDashboardView, SystemPageView, system_dashboard_view, system_page_for_route,
        system_pages,
    },
    dictionary_api::{dictionary_routes, is_dictionary_api_path},
    model::{
        CreatedSystemApiKey, SystemApiKeySummary, SystemOperationRecordSummary,
        SystemPageDataResponse, SystemPageRecordSummary, SystemStoreStatus,
    },
    navigation::{AdminSectionSnapshot, system_admin_sections},
    store::{CreateSystemApiKeyInput, SystemAdminStore, SystemOperationInput, system_store_status},
};

#[derive(Clone)]
pub struct SystemAdminApiState {
    database_url: Option<String>,
    store: Option<SystemAdminStore>,
}

impl SystemAdminApiState {
    pub fn degraded(database_url: Option<String>) -> Self {
        Self {
            database_url,
            store: None,
        }
    }

    pub fn from_store(database_url: Option<String>, store: Option<SystemAdminStore>) -> Self {
        Self {
            database_url,
            store,
        }
    }

    pub(crate) fn store(&self) -> Option<SystemAdminStore> {
        self.store.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemAdminStatus {
    pub domain_id: String,
    pub label: String,
    pub default_route: String,
    pub implemented_pages: usize,
    pub reference_pages: usize,
    pub pg_tables: usize,
    pub api_surface: Vec<String>,
    pub store: SystemStoreStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct PageQuery {
    pub route: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct OperationRecordsQuery {
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct DataRecordsQuery {
    pub page_id: Option<String>,
    pub o: Option<usize>,
    pub s: Option<usize>,
}

pub fn system_admin_router(state: SystemAdminApiState) -> Router {
    let mut router = Router::new()
        .route("/api/system/status", get(status_handler))
        .route("/api/system/dashboard", get(dashboard_handler))
        .route("/api/system/pages", get(pages_handler))
        .route("/api/system/page", get(page_handler))
        .route("/api/system/navigation", get(navigation_handler))
        .route("/api/system/store/pages", get(store_pages_handler))
        .route("/api/system/store/records", get(data_records_handler))
        .route(
            "/api/system/store/operations",
            get(operation_records_handler),
        )
        .route("/api/system/api-keys", get(api_keys_handler))
        .route("/api/system/api-key", post(create_api_key_handler))
        .route("/api/system/api-key/revoke", post(revoke_api_key_handler))
        .route(
            "/admin-api/system/ui-api-key",
            post(ui_create_api_key_handler),
        )
        .route(
            "/admin-api/system/ui-api-key/revoke",
            post(ui_revoke_api_key_handler),
        )
        .merge(dictionary_routes());

    for page in system_pages() {
        for operation in page.operations {
            if is_dictionary_api_path(operation.path) {
                continue;
            }
            router = route_operation(router, operation.method, operation.path);
        }
    }

    router.with_state(state)
}

async fn status_handler(State(state): State<SystemAdminApiState>) -> Json<SystemAdminStatus> {
    let dashboard = system_dashboard_view();
    let api_surface = dashboard
        .pages
        .iter()
        .flat_map(|page| page.operations.iter())
        .map(|operation| format!("{} {}", operation.method, operation.path))
        .collect();

    Json(SystemAdminStatus {
        domain_id: dashboard.domain_id,
        label: dashboard.label,
        default_route: dashboard.default_route,
        implemented_pages: dashboard.implemented_count,
        reference_pages: dashboard.reference_count,
        pg_tables: dashboard.pg_table_count,
        api_surface,
        store: system_store_status(&state.database_url, &state.store),
    })
}

async fn dashboard_handler() -> Json<SystemDashboardView> {
    Json(system_dashboard_view())
}

async fn pages_handler() -> Json<Vec<SystemPageView>> {
    Json(system_dashboard_view().pages)
}

async fn page_handler(ApiQuery(query): ApiQuery<PageQuery>) -> Json<Option<SystemPageView>> {
    let route = query
        .route
        .unwrap_or_else(|| crate::system::catalog::SYSTEM_DEFAULT_ROUTE.to_string());
    let page = system_page_for_route(&route).map(|page| page.view());

    Json(page)
}

async fn navigation_handler() -> Json<Vec<AdminSectionSnapshot>> {
    Json(system_admin_sections())
}

async fn store_pages_handler(
    State(state): State<SystemAdminApiState>,
) -> Result<Json<ApiResponse<Vec<SystemPageRecordSummary>>>, Response> {
    let store = require_store(state.store)?;
    store
        .list_page_snapshots()
        .await
        .map(ok_json)
        .map_err(system_error_response)
}

async fn operation_records_handler(
    State(state): State<SystemAdminApiState>,
    ApiQuery(query): ApiQuery<OperationRecordsQuery>,
) -> Result<Json<ApiResponse<Vec<SystemOperationRecordSummary>>>, Response> {
    let store = require_store(state.store)?;
    store
        .list_operation_records(query.limit.unwrap_or(20))
        .await
        .map(ok_json)
        .map_err(system_error_response)
}

async fn data_records_handler(
    State(state): State<SystemAdminApiState>,
    ApiQuery(query): ApiQuery<DataRecordsQuery>,
) -> Result<Json<ApiResponse<SystemPageDataResponse>>, Response> {
    let store = require_store(state.store)?;
    store
        .list_page_data_records(
            query.page_id.as_deref(),
            query.o.unwrap_or(0),
            query.s.unwrap_or(20),
        )
        .await
        .map(ok_json)
        .map_err(system_error_response)
}

async fn api_keys_handler(
    State(state): State<SystemAdminApiState>,
) -> Result<Json<ApiResponse<Vec<SystemApiKeySummary>>>, Response> {
    let store = require_store(state.store)?;
    store
        .list_api_keys()
        .await
        .map(ok_json)
        .map_err(system_error_response)
}

async fn create_api_key_handler(
    State(state): State<SystemAdminApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ApiResponse<CreatedSystemApiKey>>, Response> {
    let payload = payload_from_body(&headers, &body)?;
    let input = api_key_input_from_payload(&payload)?;
    let store = require_store(state.store)?;
    store
        .create_api_key(input)
        .await
        .map(ok_json)
        .map_err(system_error_response)
}

async fn revoke_api_key_handler(
    State(state): State<SystemAdminApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ApiResponse<SystemApiKeySummary>>, Response> {
    let payload = payload_from_body(&headers, &body)?;
    let id = required_payload_string(&payload, "id")?;
    let store = require_store(state.store)?;
    store
        .revoke_api_key(&id)
        .await
        .map(ok_json)
        .map_err(system_error_response)
}

async fn ui_create_api_key_handler(
    State(state): State<SystemAdminApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let redirect = match create_api_key_for_ui(state, headers, body).await {
        Ok(created) => format!(
            "/?route=/system/account/api-keys&created=1&prefix={}",
            urlencoding::encode(&created.summary.prefix)
        ),
        Err(error) => format!(
            "/?route=/system/account/api-keys&error={}",
            urlencoding::encode(&error.to_string())
        ),
    };
    axum::response::Redirect::to(&redirect).into_response()
}

async fn ui_revoke_api_key_handler(
    State(state): State<SystemAdminApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let redirect = match revoke_api_key_for_ui(state, headers, body).await {
        Ok(_) => "/?route=/system/account/api-keys&revoked=1".to_string(),
        Err(error) => format!(
            "/?route=/system/account/api-keys&error={}",
            urlencoding::encode(&error.to_string())
        ),
    };
    axum::response::Redirect::to(&redirect).into_response()
}

async fn create_api_key_for_ui(
    state: SystemAdminApiState,
    headers: HeaderMap,
    body: Bytes,
) -> anyhow::Result<CreatedSystemApiKey> {
    let payload = payload_from_body(&headers, &body)
        .map_err(|response| anyhow!("invalid api_key form: {}", response.status()))?;
    let input = api_key_input_from_payload(&payload)
        .map_err(|response| anyhow!("invalid api_key form: {}", response.status()))?;
    state
        .store
        .ok_or_else(|| anyhow!("missing system-admin database url"))?
        .create_api_key(input)
        .await
}

async fn revoke_api_key_for_ui(
    state: SystemAdminApiState,
    headers: HeaderMap,
    body: Bytes,
) -> anyhow::Result<SystemApiKeySummary> {
    let payload = payload_from_body(&headers, &body)
        .map_err(|response| anyhow!("invalid api_key form: {}", response.status()))?;
    let id = required_payload_string(&payload, "id")
        .map_err(|response| anyhow!("invalid api_key form: {}", response.status()))?;
    state
        .store
        .ok_or_else(|| anyhow!("missing system-admin database url"))?
        .revoke_api_key(&id)
        .await
}

async fn get_operation_handler(
    State(state): State<SystemAdminApiState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ApiResponse<SystemOperationRecordSummary>>, Response> {
    let mut payload = json!({
        "query": raw_query.unwrap_or_default(),
    });
    let path = uri.path();
    inject_contract_payload(&mut payload, method.as_str(), path)?;
    execute_operation(state, payload).await.map(Json)
}

async fn post_operation_handler(
    State(state): State<SystemAdminApiState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ApiResponse<SystemOperationRecordSummary>>, Response> {
    let mut payload = payload_from_body(&headers, &body)?;
    let path = uri.path();
    inject_contract_payload(&mut payload, method.as_str(), path)?;
    execute_operation(state, payload).await.map(Json)
}

async fn execute_operation(
    state: SystemAdminApiState,
    payload: Value,
) -> Result<ApiResponse<SystemOperationRecordSummary>, Response> {
    let operation_id = payload
        .get("operation_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("system operation_id is required"))
        .map_err(system_error_response)?;
    let page_id = payload
        .get("page_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("system page_id is required"))
        .map_err(system_error_response)?;
    let method = payload
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("system method is required"))
        .map_err(system_error_response)?;
    let path = payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("system path is required"))
        .map_err(system_error_response)?;
    let store = require_store(state.store)?;
    store
        .execute_operation(SystemOperationInput {
            page_id: page_id.to_string(),
            operation_id: operation_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            payload,
        })
        .await
        .map(ApiResponse::ok)
        .map_err(system_error_response)
}

fn route_operation(
    router: Router<SystemAdminApiState>,
    method: &str,
    path: &str,
) -> Router<SystemAdminApiState> {
    if method == "GET" {
        router.route(path, get(get_operation_handler))
    } else {
        router.route(path, post(post_operation_handler))
    }
}

fn inject_contract_payload(payload: &mut Value, method: &str, path: &str) -> Result<(), Response> {
    let (page, operation) = operation_for_http(method, path)?;
    if let Some(object) = payload.as_object_mut() {
        object.insert("page_id".to_string(), json!(page.id));
        object.insert("operation_id".to_string(), json!(operation.id));
        object.insert("method".to_string(), json!(operation.method));
        object.insert("path".to_string(), json!(operation.path));
        return Ok(());
    }
    Err(system_error_response(anyhow!(
        "system operation payload must be object"
    )))
}

fn payload_from_body(headers: &HeaderMap, body: &Bytes) -> Result<Value, Response> {
    if body.is_empty() {
        return Ok(json!({}));
    }

    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if content_type.contains("application/json") {
        return serde_json::from_slice(body).map_err(|error| {
            system_error_response(anyhow!("invalid system operation json: {error}"))
        });
    }

    parse_form_body(body)
}

fn parse_form_body(body: &Bytes) -> Result<Value, Response> {
    let body = std::str::from_utf8(body).map_err(|error| {
        system_error_response(anyhow!("invalid system operation form: {error}"))
    })?;
    let mut object = serde_json::Map::new();

    for pair in body.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = urlencoding::decode(raw_key).map_err(|error| {
            system_error_response(anyhow!("invalid system operation form key: {error}"))
        })?;
        let value = urlencoding::decode(raw_value).map_err(|error| {
            system_error_response(anyhow!("invalid system operation form value: {error}"))
        })?;
        object.insert(key.into_owned(), json!(value.into_owned()));
    }

    Ok(Value::Object(object))
}

fn api_key_input_from_payload(payload: &Value) -> Result<CreateSystemApiKeyInput, Response> {
    Ok(CreateSystemApiKeyInput {
        name: required_payload_string(payload, "name")?,
        scope: optional_payload_string(payload, "scope"),
    })
}

fn required_payload_string(payload: &Value, key: &str) -> Result<String, Response> {
    optional_payload_string(payload, key)
        .ok_or_else(|| system_error_response(anyhow!("api_key {key} is required")))
}

fn optional_payload_string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn operation_for_http(
    method: &str,
    path: &str,
) -> Result<
    (
        crate::system::catalog::SystemPage,
        crate::system::catalog::SystemOperation,
    ),
    Response,
> {
    for page in system_pages().iter().copied() {
        if let Some(operation) = page
            .operations
            .iter()
            .copied()
            .find(|operation| operation.method == method && operation.path == path)
        {
            return Ok((page, operation));
        }
    }
    Err(system_error_response(anyhow!(
        "system operation route not found: {method} {path}"
    )))
}

fn require_store(store: Option<SystemAdminStore>) -> Result<SystemAdminStore, Response> {
    store
        .ok_or_else(|| anyhow!("missing system-admin database url"))
        .map_err(system_error_response)
}

fn system_error_response(error: anyhow::Error) -> Response {
    ApiError::from(error).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn status_reports_shared_api_surface() {
        let state = SystemAdminApiState::degraded(None);
        let Json(status) = status_handler(State(state)).await;

        assert_eq!(status.domain_id, "system");
        assert!(status.implemented_pages >= 5);

        // 关键断言：状态接口暴露的 API 面来自页面操作契约。
        assert!(
            status
                .api_surface
                .iter()
                .any(|route| route == "POST /api/system/users")
        );
    }

    #[tokio::test]
    async fn page_handler_resolves_default_route() {
        let Json(page) = page_handler(ApiQuery(PageQuery { route: None })).await;

        assert_eq!(page.as_ref().map(|page| page.id.as_str()), Some("identity"));
    }

    #[test]
    fn operation_contract_injects_page_and_operation() {
        let page = system_pages()
            .iter()
            .copied()
            .find(|page| !page.operations.is_empty())
            .unwrap();
        let operation = page.operations[0];
        let mut payload = json!({});

        let result = inject_contract_payload(&mut payload, operation.method, operation.path);

        assert!(result.is_ok());
        assert_eq!(payload["page_id"], page.id);
        assert_eq!(payload["operation_id"], operation.id);
    }

    #[test]
    fn form_payload_becomes_operation_object() {
        let headers = HeaderMap::new();
        let body = Bytes::from("note=%E6%89%A7%E8%A1%8C");

        let payload = payload_from_body(&headers, &body);

        assert_eq!(
            payload
                .ok()
                .and_then(|value| value["note"].as_str().map(str::to_string)),
            Some("执行".to_string())
        );
    }

    #[test]
    fn data_records_query_uses_offset_size_names() {
        let query = DataRecordsQuery {
            page_id: Some("identity".to_string()),
            o: Some(10),
            s: Some(20),
        };

        assert_eq!(query.o, Some(10));
        assert_eq!(query.s, Some(20));
    }

    #[test]
    fn api_key_payload_accepts_form_body() {
        let headers = HeaderMap::new();
        let body = Bytes::from("name=%E5%A4%A9%E6%B0%94%E8%B0%83%E7%94%A8&scope=all-services");
        let payload = payload_from_body(&headers, &body).unwrap();

        let input = api_key_input_from_payload(&payload).unwrap();

        assert_eq!(input.name, "天气调用");
        assert_eq!(input.scope.as_deref(), Some("all-services"));
    }
}
