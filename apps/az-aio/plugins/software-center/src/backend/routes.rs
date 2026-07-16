use anyhow::anyhow;
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use az_aio_platform::core::api_error::{ApiError, ApiJson, ApiResponse, ok_json};
use serde::{Deserialize, Serialize};

use crate::backend::{
    installer_scanner::{InstallerPackage, organize_installers, scan_installers},
    model::{SoftwarePackageSummary, TABLE_NAME_PREFIX},
    store::{SoftwareCenterStore, SoftwarePackageInput},
};

#[derive(Clone)]
pub struct SoftwareCenterApiState {
    database_url: Option<String>,
    store: Option<SoftwareCenterStore>,
}

impl SoftwareCenterApiState {
    pub fn degraded(database_url: Option<String>) -> Self {
        Self {
            database_url,
            store: None,
        }
    }

    pub fn from_store(database_url: Option<String>, store: Option<SoftwareCenterStore>) -> Self {
        Self {
            database_url,
            store,
        }
    }

    pub fn status(&self) -> SoftwareCenterStatusResponse {
        SoftwareCenterStatusResponse {
            ok: true,
            database_configured: self
                .database_url
                .as_ref()
                .is_some_and(|value| !value.is_empty()),
            store_connected: self.store.is_some(),
            table_prefix: TABLE_NAME_PREFIX.to_string(),
        }
    }

    pub fn store(&self) -> Option<SoftwareCenterStore> {
        self.store.clone()
    }
}

pub fn software_center_router(state: SoftwareCenterApiState) -> Router {
    Router::new()
        .route("/api/software-center/status", get(status_handler))
        .route(
            "/api/software-center/installers",
            get(scan_installers_handler),
        )
        .route(
            "/api/software-center/organize",
            post(organize_installers_handler),
        )
        .route("/api/software-center/packages", get(list_packages_handler))
        .route("/api/software-center/package", post(upsert_package_handler))
        .with_state(state)
}

async fn status_handler(
    State(state): State<SoftwareCenterApiState>,
) -> Json<SoftwareCenterStatusResponse> {
    Json(state.status())
}

async fn scan_installers_handler() -> Result<Json<ApiResponse<Vec<InstallerPackage>>>, Response> {
    scan_installers()
        .map(ok_json)
        .map_err(software_center_error_response)
}

async fn organize_installers_handler() -> Result<Json<ApiResponse<Vec<InstallerPackage>>>, Response>
{
    organize_installers()
        .map(ok_json)
        .map_err(software_center_error_response)
}

async fn list_packages_handler(
    State(state): State<SoftwareCenterApiState>,
) -> Result<Json<ApiResponse<Vec<SoftwarePackageSummary>>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing software-center database url"))
        .map_err(software_center_error_response)?;
    store
        .list_packages()
        .await
        .map(ok_json)
        .map_err(software_center_error_response)
}

async fn upsert_package_handler(
    State(state): State<SoftwareCenterApiState>,
    ApiJson(request): ApiJson<UpsertSoftwarePackageRequest>,
) -> Result<Json<ApiResponse<SoftwarePackageSummary>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing software-center database url"))
        .map_err(software_center_error_response)?;
    store
        .upsert_package(SoftwarePackageInput {
            id: request.id,
            name: request.name,
            source_path: request.source_path,
            platform: request.platform,
            arch: request.arch,
            status: request.status,
        })
        .await
        .map(ok_json)
        .map_err(software_center_error_response)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SoftwareCenterStatusResponse {
    pub ok: bool,
    pub database_configured: bool,
    pub store_connected: bool,
    pub table_prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSoftwarePackageRequest {
    pub id: Option<String>,
    pub name: String,
    pub source_path: String,
    pub platform: String,
    pub arch: String,
    pub status: Option<String>,
}

fn software_center_error_response(error: anyhow::Error) -> Response {
    ApiError::from(error).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn degraded_state_reports_disconnected_store() {
        let state = SoftwareCenterApiState::degraded(None);
        assert!(state.store.is_none());
        assert!(state.database_url.is_none());
    }
}
