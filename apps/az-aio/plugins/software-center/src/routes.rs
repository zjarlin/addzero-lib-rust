use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{SoftwareCenterError, SoftwareCenterResult},
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
    pub async fn new(database_url: Option<String>) -> SoftwareCenterResult<Self> {
        let store = match database_url.as_deref() {
            Some(value) if !value.trim().is_empty() => {
                Some(SoftwareCenterStore::connect(value).await?)
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

pub fn software_center_router(state: SoftwareCenterApiState) -> Router {
    Router::new()
        .route("/api/software-center/status", get(status_handler))
        .route("/api/software-center/installers", get(scan_installers_handler))
        .route("/api/software-center/organize", post(organize_installers_handler))
        .route("/api/software-center/packages", get(list_packages_handler))
        .route("/api/software-center/package", post(upsert_package_handler))
        .with_state(state)
}

async fn status_handler(
    State(state): State<SoftwareCenterApiState>,
) -> Json<SoftwareCenterStatusResponse> {
    Json(SoftwareCenterStatusResponse {
        ok: true,
        database_configured: state.database_url.as_ref().is_some_and(|value| !value.is_empty()),
        store_connected: state.store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
    })
}

async fn scan_installers_handler(
) -> Result<Json<ApiResponse<Vec<InstallerPackage>>>, SoftwareCenterApiError> {
    scan_installers()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(SoftwareCenterError::from)
        .map_err(Into::into)
}

async fn organize_installers_handler(
) -> Result<Json<ApiResponse<Vec<InstallerPackage>>>, SoftwareCenterApiError> {
    organize_installers()
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(SoftwareCenterError::from)
        .map_err(Into::into)
}

async fn list_packages_handler(
    State(state): State<SoftwareCenterApiState>,
) -> Result<Json<ApiResponse<Vec<SoftwarePackageSummary>>>, SoftwareCenterApiError> {
    let store = state.store.ok_or(SoftwareCenterError::MissingDatabaseUrl)?;
    store
        .list_packages()
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

async fn upsert_package_handler(
    State(state): State<SoftwareCenterApiState>,
    Json(request): Json<UpsertSoftwarePackageRequest>,
) -> Result<Json<ApiResponse<SoftwarePackageSummary>>, SoftwareCenterApiError> {
    let store = state.store.ok_or(SoftwareCenterError::MissingDatabaseUrl)?;
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
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

#[derive(Clone, Debug, Serialize)]
pub struct SoftwareCenterStatusResponse {
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
pub struct UpsertSoftwarePackageRequest {
    pub id: Option<String>,
    pub name: String,
    pub source_path: String,
    pub platform: String,
    pub arch: String,
    pub status: Option<String>,
}

#[derive(Debug)]
pub struct SoftwareCenterApiError {
    source: SoftwareCenterError,
}

impl From<SoftwareCenterError> for SoftwareCenterApiError {
    fn from(source: SoftwareCenterError) -> Self {
        Self { source }
    }
}

impl IntoResponse for SoftwareCenterApiError {
    fn into_response(self) -> Response {
        let status = match self.source {
            SoftwareCenterError::MissingDatabaseUrl => StatusCode::SERVICE_UNAVAILABLE,
            SoftwareCenterError::BlankPackageName | SoftwareCenterError::BlankSourcePath => {
                StatusCode::BAD_REQUEST
            }
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
        let state = SoftwareCenterApiState::degraded(None);
        assert!(state.store.is_none());
        assert!(state.database_url.is_none());
    }
}
