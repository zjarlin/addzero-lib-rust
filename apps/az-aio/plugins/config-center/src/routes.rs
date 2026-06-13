use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    dotfiles_monitor::scan_dotfiles_status,
    dotfiles_monitor_types::DotfilesMonitorStatus,
    error::{ConfigCenterError, ConfigCenterResult},
    model::{ConfigEntrySummary, TABLE_NAME_PREFIX},
    pairing::{PairingLocalInfo, ensure_local_pairing_device_info, local_pairing_info},
    paths::{ConfigCenterPaths, resolve_config_center_paths},
    store::{ConfigCenterStore, ConfigEntryInput},
};

#[derive(Clone)]
pub struct ConfigCenterApiState {
    database_url: Option<String>,
    store: Option<ConfigCenterStore>,
}

impl ConfigCenterApiState {
    pub async fn new(database_url: Option<String>) -> ConfigCenterResult<Self> {
        let store = match database_url.as_deref() {
            Some(value) if !value.trim().is_empty() => {
                Some(ConfigCenterStore::connect(value).await?)
            }
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

pub fn config_center_router(state: ConfigCenterApiState) -> Router {
    Router::new()
        .route("/api/config-center/status", get(status_handler))
        .route("/api/config-center/dotfiles", get(dotfiles_handler))
        .route("/api/config-center/pairing", get(pairing_handler))
        .route("/api/config-center/entries", get(list_entries_handler))
        .route("/api/config-center/entry", post(upsert_entry_handler))
        .with_state(state)
}

async fn status_handler(
    State(state): State<ConfigCenterApiState>,
) -> Result<Json<ConfigCenterStatusResponse>, ConfigCenterApiError> {
    let paths = resolve_config_center_paths().map_err(ConfigCenterError::from)?;
    Ok(Json(ConfigCenterStatusResponse {
        ok: true,
        database_configured: state.database_url.as_ref().is_some_and(|value| !value.is_empty()),
        store_connected: state.store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
        paths,
    }))
}

async fn dotfiles_handler(
) -> Result<Json<ApiResponse<DotfilesMonitorStatus>>, ConfigCenterApiError> {
    scan_dotfiles_status()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(ConfigCenterError::from)
        .map_err(Into::into)
}

async fn pairing_handler() -> Result<Json<ApiResponse<PairingLocalInfo>>, ConfigCenterApiError> {
    ensure_local_pairing_device_info().map_err(ConfigCenterError::from)?;
    local_pairing_info()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(ConfigCenterError::from)
        .map_err(Into::into)
}

async fn list_entries_handler(
    State(state): State<ConfigCenterApiState>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<ApiResponse<Vec<ConfigEntrySummary>>>, ConfigCenterApiError> {
    let store = state.store.ok_or(ConfigCenterError::MissingDatabaseUrl)?;
    store
        .list_entries(query.namespace.as_deref().unwrap_or("az-aio.dev"))
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

async fn upsert_entry_handler(
    State(state): State<ConfigCenterApiState>,
    Json(request): Json<UpsertConfigEntryRequest>,
) -> Result<Json<ApiResponse<ConfigEntrySummary>>, ConfigCenterApiError> {
    let store = state.store.ok_or(ConfigCenterError::MissingDatabaseUrl)?;
    store
        .upsert_entry(ConfigEntryInput {
            id: request.id,
            namespace: request.namespace.unwrap_or_else(|| "az-aio.dev".to_string()),
            key: request.key,
            value: request.value,
        })
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

#[derive(Clone, Debug, Serialize)]
pub struct ConfigCenterStatusResponse {
    pub ok: bool,
    pub database_configured: bool,
    pub store_connected: bool,
    pub table_prefix: String,
    pub paths: ConfigCenterPaths,
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
pub struct ListEntriesQuery {
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertConfigEntryRequest {
    pub id: Option<String>,
    pub namespace: Option<String>,
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct ConfigCenterApiError {
    source: ConfigCenterError,
}

impl From<ConfigCenterError> for ConfigCenterApiError {
    fn from(source: ConfigCenterError) -> Self {
        Self { source }
    }
}

impl IntoResponse for ConfigCenterApiError {
    fn into_response(self) -> Response {
        let status = match self.source {
            ConfigCenterError::MissingDatabaseUrl => StatusCode::SERVICE_UNAVAILABLE,
            ConfigCenterError::BlankKey | ConfigCenterError::BlankValue => StatusCode::BAD_REQUEST,
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
        let state = ConfigCenterApiState::degraded(None);
        assert!(state.store.is_none());
        assert!(state.database_url.is_none());
    }
}
