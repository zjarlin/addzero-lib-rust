use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AssetHubError, AssetHubResult},
    model::{AssetSummary, TABLE_NAME_PREFIX},
    skill_scanner::{ScannedSkillAsset, scan_skill_assets},
    store::{AssetHubStore, AssetInput},
};

#[derive(Clone)]
pub struct AssetHubApiState {
    database_url: Option<String>,
    store: Option<AssetHubStore>,
}

impl AssetHubApiState {
    pub async fn new(database_url: Option<String>) -> AssetHubResult<Self> {
        let store = match database_url.as_deref() {
            Some(value) if !value.trim().is_empty() => Some(AssetHubStore::connect(value).await?),
            _ => None,
        };
        Ok(Self {
            database_url,
            store,
        })
    }

    pub fn degraded(database_url: Option<String>) -> Self {
        Self {
            database_url,
            store: None,
        }
    }
}

pub fn asset_hub_router(state: AssetHubApiState) -> Router {
    Router::new()
        .route("/api/asset-hub/status", get(status_handler))
        .route("/api/asset-hub/skills", get(scan_skills_handler))
        .route("/api/asset-hub/assets", get(list_assets_handler))
        .route("/api/asset-hub/asset", post(upsert_asset_handler))
        .with_state(state)
}

async fn status_handler(State(state): State<AssetHubApiState>) -> Json<AssetHubStatusResponse> {
    Json(AssetHubStatusResponse {
        ok: true,
        database_configured: state.database_url.as_ref().is_some_and(|value| !value.is_empty()),
        store_connected: state.store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
    })
}

async fn scan_skills_handler() -> Result<Json<ApiResponse<Vec<ScannedSkillAsset>>>, AssetHubApiError>
{
    scan_skill_assets()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(AssetHubError::from)
        .map_err(Into::into)
}

async fn list_assets_handler(
    State(state): State<AssetHubApiState>,
) -> Result<Json<ApiResponse<Vec<AssetSummary>>>, AssetHubApiError> {
    let store = state.store.ok_or(AssetHubError::MissingDatabaseUrl)?;
    store
        .list_assets()
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

async fn upsert_asset_handler(
    State(state): State<AssetHubApiState>,
    Json(request): Json<UpsertAssetRequest>,
) -> Result<Json<ApiResponse<AssetSummary>>, AssetHubApiError> {
    let store = state.store.ok_or(AssetHubError::MissingDatabaseUrl)?;
    store
        .upsert_asset(AssetInput {
            id: request.id,
            kind: request.kind,
            title: request.title,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            source: request.source.unwrap_or_else(|| "asset-hub".to_string()),
        })
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

#[derive(Clone, Debug, Serialize)]
pub struct AssetHubStatusResponse {
    pub ok: bool,
    pub database_configured: bool,
    pub store_connected: bool,
    pub table_prefix: String,
}

#[derive(Clone, Debug, Serialize)]
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
pub struct UpsertAssetRequest {
    pub id: Option<String>,
    pub kind: String,
    pub title: String,
    pub status: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug)]
pub struct AssetHubApiError {
    source: AssetHubError,
}

impl From<AssetHubError> for AssetHubApiError {
    fn from(source: AssetHubError) -> Self {
        Self { source }
    }
}

impl IntoResponse for AssetHubApiError {
    fn into_response(self) -> Response {
        let status = match self.source {
            AssetHubError::MissingDatabaseUrl => StatusCode::SERVICE_UNAVAILABLE,
            AssetHubError::BlankTitle | AssetHubError::BlankStatus => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = ApiResponse::<()> {
            success: false,
            message: self.source.to_string(),
            data: None,
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn degraded_state_reports_disconnected_store() {
        let state = AssetHubApiState::degraded(None);
        assert!(state.store.is_none());
        assert!(state.database_url.is_none());
    }
}
