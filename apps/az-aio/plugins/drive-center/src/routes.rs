use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{DriveCenterError, DriveCenterResult},
    model::{DriveTaskSummary, TABLE_NAME_PREFIX},
    store::{DriveCenterStore, DriveTaskInput},
};

#[derive(Clone)]
pub struct DriveCenterApiState {
    database_url: Option<String>,
    store: Option<DriveCenterStore>,
}

impl DriveCenterApiState {
    pub async fn new(database_url: Option<String>) -> DriveCenterResult<Self> {
        let store = match database_url.as_deref() {
            Some(value) if !value.trim().is_empty() => Some(DriveCenterStore::connect(value).await?),
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

pub fn drive_center_router(state: DriveCenterApiState) -> Router {
    Router::new()
        .route("/api/drive-center/status", get(status_handler))
        .route("/api/drive-center/tasks", get(list_tasks_handler))
        .route("/api/drive-center/task", post(enqueue_task_handler))
        .with_state(state)
}

async fn status_handler(State(state): State<DriveCenterApiState>) -> Json<DriveCenterStatusResponse> {
    Json(DriveCenterStatusResponse {
        ok: true,
        database_configured: state.database_url.as_ref().is_some_and(|value| !value.is_empty()),
        store_connected: state.store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
    })
}

async fn list_tasks_handler(
    State(state): State<DriveCenterApiState>,
) -> Result<Json<ApiResponse<Vec<DriveTaskSummary>>>, DriveCenterApiError> {
    let store = state.store.ok_or(DriveCenterError::MissingDatabaseUrl)?;
    store
        .list_tasks()
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

async fn enqueue_task_handler(
    State(state): State<DriveCenterApiState>,
    Json(request): Json<EnqueueDriveTaskRequest>,
) -> Result<Json<ApiResponse<DriveTaskSummary>>, DriveCenterApiError> {
    let store = state.store.ok_or(DriveCenterError::MissingDatabaseUrl)?;
    store
        .enqueue_task(DriveTaskInput {
            id: request.id,
            path: request.path,
            action: request.action,
            status: request.status,
        })
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

#[derive(Clone, Debug, Serialize)]
pub struct DriveCenterStatusResponse {
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
pub struct EnqueueDriveTaskRequest {
    pub id: Option<String>,
    pub path: String,
    pub action: String,
    pub status: Option<String>,
}

#[derive(Debug)]
pub struct DriveCenterApiError {
    source: DriveCenterError,
}

impl From<DriveCenterError> for DriveCenterApiError {
    fn from(source: DriveCenterError) -> Self {
        Self { source }
    }
}

impl IntoResponse for DriveCenterApiError {
    fn into_response(self) -> Response {
        let status = match self.source {
            DriveCenterError::MissingDatabaseUrl => StatusCode::SERVICE_UNAVAILABLE,
            DriveCenterError::BlankPath | DriveCenterError::BlankAction => StatusCode::BAD_REQUEST,
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
        let state = DriveCenterApiState::degraded(None);
        assert!(state.store.is_none());
        assert!(state.database_url.is_none());
    }
}
