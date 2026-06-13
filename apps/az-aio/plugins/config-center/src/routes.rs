use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    dotfiles_monitor::scan_dotfiles_status,
    dotfiles_monitor_types::DotfilesMonitorStatus,
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
    pub async fn new(database_url: Option<String>) -> anyhow::Result<Self> {
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
) -> Result<Json<ConfigCenterStatusResponse>, Response> {
    let paths = resolve_config_center_paths().map_err(config_center_error_response)?;
    Ok(Json(ConfigCenterStatusResponse {
        ok: true,
        database_configured: state.database_url.as_ref().is_some_and(|value| !value.is_empty()),
        store_connected: state.store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
        paths,
    }))
}

async fn dotfiles_handler(
) -> Result<Json<ApiResponse<DotfilesMonitorStatus>>, Response> {
    scan_dotfiles_status()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(config_center_error_response)
}

async fn pairing_handler() -> Result<Json<ApiResponse<PairingLocalInfo>>, Response> {
    ensure_local_pairing_device_info().map_err(config_center_error_response)?;
    local_pairing_info()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(config_center_error_response)
}

async fn list_entries_handler(
    State(state): State<ConfigCenterApiState>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<ApiResponse<Vec<ConfigEntrySummary>>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing config-center database url"))
        .map_err(config_center_error_response)?;
    store
        .list_entries(query.namespace.as_deref().unwrap_or("az-aio.dev"))
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(config_center_error_response)
}

async fn upsert_entry_handler(
    State(state): State<ConfigCenterApiState>,
    Json(request): Json<UpsertConfigEntryRequest>,
) -> Result<Json<ApiResponse<ConfigEntrySummary>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing config-center database url"))
        .map_err(config_center_error_response)?;
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
        .map_err(config_center_error_response)
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

fn config_center_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    let status = config_center_error_status(&message);
    let body = ApiResponse::<()> {
        success: false,
        message,
        data: None,
    };
    (status, Json(body)).into_response()
}

fn config_center_error_status(message: &str) -> StatusCode {
    match message {
        "missing config-center database url" => StatusCode::SERVICE_UNAVAILABLE,
        "config key must not be blank" | "config value must not be blank" => {
            StatusCode::BAD_REQUEST
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
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
