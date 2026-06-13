#![cfg(not(target_arch = "wasm32"))]

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    config::{LowcodeConfig, LowcodeConfigSource, MISSING_DATABASE_URL_MESSAGE},
    model::{LowcodeAppSummary, LowcodePageSummary},
    store::{INVALID_APP_ID_MESSAGE, INVALID_PAGE_ID_MESSAGE, LowcodeAppInput, LowcodePageInput, LowcodeStore},
};

#[derive(Clone)]
pub struct LowcodeApiState {
    store: LowcodeStore,
    config: LowcodeConfig,
}

impl LowcodeApiState {
    pub fn new(store: LowcodeStore, config: LowcodeConfig) -> Self {
        Self { store, config }
    }

    pub async fn connect(config: LowcodeConfig) -> anyhow::Result<Self> {
        let store = LowcodeStore::connect(&config.database_url).await?;
        Ok(Self::new(store, config))
    }
}

pub fn lowcode_api_router(state: LowcodeApiState) -> Router {
    Router::new()
        .route("/api/lowcode/status", get(status_handler))
        .route("/api/lowcode/apps", get(list_apps_handler))
        .route("/api/lowcode/app", post(upsert_app_handler))
        .route("/api/lowcode/pages", get(list_pages_handler))
        .route("/api/lowcode/page", post(upsert_page_handler))
        .route("/api/lowcode/page/delete", post(delete_page_handler))
        .with_state(state)
}

async fn status_handler(State(state): State<LowcodeApiState>) -> Json<LowcodeStatusResponse> {
    Json(LowcodeStatusResponse {
        ok: true,
        database_config_namespace: "az-aio.dev".to_string(),
        database_config_key: "lowcode.database_url".to_string(),
        database_source: match state.config.source {
            LowcodeConfigSource::ConfigCenter => "config-center",
            LowcodeConfigSource::Environment => "environment",
        }
        .to_string(),
    })
}

async fn list_apps_handler(
    State(state): State<LowcodeApiState>,
) -> Result<Json<ApiResponse<Vec<LowcodeAppSummary>>>, Response> {
    state
        .store
        .list_apps()
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(lowcode_error_response)
}

async fn upsert_app_handler(
    State(state): State<LowcodeApiState>,
    Json(request): Json<UpsertLowcodeAppRequest>,
) -> Result<Json<ApiResponse<LowcodeAppSummary>>, Response> {
    let input = LowcodeAppInput {
        id: request.id,
        slug: request.slug,
        name: request.name,
        description: request.description.unwrap_or_default(),
        enabled: request.enabled.unwrap_or(true),
    };
    state
        .store
        .upsert_app(input)
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(lowcode_error_response)
}

async fn list_pages_handler(
    State(state): State<LowcodeApiState>,
    Query(query): Query<ListPagesQuery>,
) -> Result<Json<ApiResponse<Vec<LowcodePageSummary>>>, Response> {
    state
        .store
        .list_pages(&query.app_id)
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(lowcode_error_response)
}

async fn upsert_page_handler(
    State(state): State<LowcodeApiState>,
    Json(request): Json<UpsertLowcodePageRequest>,
) -> Result<Json<ApiResponse<LowcodePageSummary>>, Response> {
    let input = LowcodePageInput {
        id: request.id,
        app_id: request.app_id,
        route: request.route,
        title: request.title,
        schema_json: request.schema_json,
        enabled: request.enabled.unwrap_or(true),
    };
    state
        .store
        .upsert_page(input)
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(lowcode_error_response)
}

async fn delete_page_handler(
    State(state): State<LowcodeApiState>,
    Json(request): Json<DeleteLowcodePageRequest>,
) -> Result<Json<ApiResponse<DeleteLowcodePageResponse>>, Response> {
    state
        .store
        .delete_page(&request.page_id)
        .await
        .map_err(lowcode_error_response)?;
    Ok(Json(ApiResponse::ok(DeleteLowcodePageResponse {
        deleted: true,
    })))
}

#[derive(Debug, Serialize)]
pub struct LowcodeStatusResponse {
    pub ok: bool,
    pub database_config_namespace: String,
    pub database_config_key: String,
    pub database_source: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            message: "ok".to_string(),
            data: Some(data),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPagesQuery {
    #[serde(rename = "appId")]
    pub app_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertLowcodeAppRequest {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertLowcodePageRequest {
    pub id: Option<String>,
    #[serde(rename = "appId")]
    pub app_id: String,
    pub route: String,
    pub title: String,
    #[serde(rename = "schemaJson")]
    pub schema_json: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteLowcodePageRequest {
    #[serde(rename = "pageId")]
    pub page_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteLowcodePageResponse {
    pub deleted: bool,
}

fn lowcode_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    let status = lowcode_error_status(&message);
    let body = ApiResponse::<()> {
        success: false,
        message,
        data: None,
    };
    (status, Json(body)).into_response()
}

fn lowcode_error_status(message: &str) -> StatusCode {
    match message {
        INVALID_APP_ID_MESSAGE | INVALID_PAGE_ID_MESSAGE => StatusCode::BAD_REQUEST,
        MISSING_DATABASE_URL_MESSAGE => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn status_reports_az_aio_dev_namespace() {
        let config = LowcodeConfig {
            database_url: "postgresql://postgres:postgres@127.0.0.1/lowcode".to_string(),
            source: LowcodeConfigSource::Environment,
        };
        let response = LowcodeStatusResponse {
            ok: true,
            database_config_namespace: "az-aio.dev".to_string(),
            database_config_key: "lowcode.database_url".to_string(),
            database_source: match config.source {
                LowcodeConfigSource::ConfigCenter => "config-center",
                LowcodeConfigSource::Environment => "environment",
            }
            .to_string(),
        };

        assert_eq!(response.database_config_namespace, "az-aio.dev");
        assert_eq!(response.database_config_key, "lowcode.database_url");
    }
}
