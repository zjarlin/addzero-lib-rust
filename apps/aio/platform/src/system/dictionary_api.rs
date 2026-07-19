//! 系统字典 REST API。

use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};

use crate::{
    core::{
        api_error::{ApiError, ApiJson, ApiPath, ApiQuery, ApiResponse, ok_json},
    },
    system::{
        api::SystemAdminApiState,
        dictionary_model::{
            DictionaryItemInput, DictionaryItemPage, DictionaryItemQuery, DictionaryItemSummary,
            DictionaryTypeInput, DictionaryTypeSummary,
        },
        store::SystemAdminStore,
    },
};

/// 返回系统字典资源路由。
pub fn dictionary_routes() -> Router<SystemAdminApiState> {
    Router::new()
        .route(
            "/api/system/dictionary-types",
            get(list_dictionary_types_handler).post(create_dictionary_type_handler),
        )
        .route(
            "/api/system/dictionary-types/{id}",
            put(update_dictionary_type_handler).delete(delete_dictionary_type_handler),
        )
        .route(
            "/api/system/dictionary-items",
            get(list_dictionary_items_handler).post(create_dictionary_item_handler),
        )
        .route(
            "/api/system/dictionary-items/{id}",
            put(update_dictionary_item_handler).delete(delete_dictionary_item_handler),
        )
}

/// 判断操作契约是否由字典专用 API 处理。
pub fn is_dictionary_api_path(path: &str) -> bool {
    path.starts_with("/api/system/dictionary-types")
        || path.starts_with("/api/system/dictionary-items")
}

async fn list_dictionary_types_handler(
    State(state): State<SystemAdminApiState>,
) -> Result<Json<ApiResponse<Vec<DictionaryTypeSummary>>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .list_dictionary_types()
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn create_dictionary_type_handler(
    State(state): State<SystemAdminApiState>,
    ApiJson(input): ApiJson<DictionaryTypeInput>,
) -> Result<Json<ApiResponse<DictionaryTypeSummary>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .create_dictionary_type(input)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn update_dictionary_type_handler(
    State(state): State<SystemAdminApiState>,
    ApiPath(id): ApiPath<String>,
    ApiJson(input): ApiJson<DictionaryTypeInput>,
) -> Result<Json<ApiResponse<DictionaryTypeSummary>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .update_dictionary_type(&id, input)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn delete_dictionary_type_handler(
    State(state): State<SystemAdminApiState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .delete_dictionary_type(&id)
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(ApiError::from)
}

async fn list_dictionary_items_handler(
    State(state): State<SystemAdminApiState>,
    ApiQuery(query): ApiQuery<DictionaryItemQuery>,
) -> Result<Json<ApiResponse<DictionaryItemPage>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .list_dictionary_items(
            &query.dictionary_type_id,
            query.q.as_deref(),
            query.o.unwrap_or(0),
            query.s.unwrap_or(50),
        )
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn create_dictionary_item_handler(
    State(state): State<SystemAdminApiState>,
    ApiJson(input): ApiJson<DictionaryItemInput>,
) -> Result<Json<ApiResponse<DictionaryItemSummary>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .create_dictionary_item(input)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn update_dictionary_item_handler(
    State(state): State<SystemAdminApiState>,
    ApiPath(id): ApiPath<String>,
    ApiJson(input): ApiJson<DictionaryItemInput>,
) -> Result<Json<ApiResponse<DictionaryItemSummary>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .update_dictionary_item(&id, input)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn delete_dictionary_item_handler(
    State(state): State<SystemAdminApiState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let store = require_dictionary_store(&state)?;
    store
        .delete_dictionary_item(&id)
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(ApiError::from)
}

fn require_dictionary_store(state: &SystemAdminApiState) -> Result<SystemAdminStore, ApiError> {
    state
        .store()
        .ok_or_else(|| ApiError::service_unavailable("missing system-admin database url"))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::StatusCode};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn dictionary_api_requires_postgresql_store() -> anyhow::Result<()> {
        let app = dictionary_routes().with_state(SystemAdminApiState::degraded(None));
        let request = axum::http::Request::builder()
            .uri("/api/system/dictionary-types")
            .body(Body::empty())?;
        let response = match app.oneshot(request).await {
            Ok(response) => response,
            Err(error) => match error {},
        };

        // 正式字典数据禁止降级为内存列表。
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        Ok(())
    }

    #[test]
    fn operation_paths_use_dictionary_resources() {
        // 字典 CRUD 必须绕过只记录审计、不执行业务写入的通用占位处理器。
        assert!(is_dictionary_api_path("/api/system/dictionary-types"));
        assert!(is_dictionary_api_path(
            "/api/system/dictionary-items/{id}"
        ));
        assert!(!is_dictionary_api_path("/api/system/users"));
    }
}
