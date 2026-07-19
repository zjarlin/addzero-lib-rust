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
    model::{DriveTaskSummary, TABLE_NAME_PREFIX},
    store::{DriveCenterStore, DriveTaskInput},
};

#[derive(Clone)]
pub struct DriveCenterApiState {
    database_url: Option<String>,
    store: Option<DriveCenterStore>,
}

impl DriveCenterApiState {
    pub fn degraded(database_url: Option<String>) -> Self {
        Self {
            database_url,
            store: None,
        }
    }

    pub fn from_store(database_url: Option<String>, store: Option<DriveCenterStore>) -> Self {
        Self {
            database_url,
            store,
        }
    }

    pub fn status(&self) -> DriveCenterStatusResponse {
        DriveCenterStatusResponse {
            ok: true,
            database_configured: self
                .database_url
                .as_ref()
                .is_some_and(|value| !value.is_empty()),
            store_connected: self.store.is_some(),
            table_prefix: TABLE_NAME_PREFIX.to_string(),
        }
    }

    pub fn store(&self) -> Option<DriveCenterStore> {
        self.store.clone()
    }
}

pub fn drive_center_router(state: DriveCenterApiState) -> Router {
    Router::new()
        .route("/api/drive-center/status", get(status_handler))
        .route("/api/drive-center/tasks", get(list_tasks_handler))
        .route("/api/drive-center/task", post(enqueue_task_handler))
        .with_state(state)
}

async fn status_handler(
    State(state): State<DriveCenterApiState>,
) -> Json<DriveCenterStatusResponse> {
    Json(state.status())
}

async fn list_tasks_handler(
    State(state): State<DriveCenterApiState>,
) -> Result<Json<ApiResponse<Vec<DriveTaskSummary>>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing drive-center database url"))
        .map_err(drive_center_error_response)?;
    store
        .list_tasks()
        .await
        .map(ok_json)
        .map_err(drive_center_error_response)
}

async fn enqueue_task_handler(
    State(state): State<DriveCenterApiState>,
    ApiJson(request): ApiJson<EnqueueDriveTaskRequest>,
) -> Result<Json<ApiResponse<DriveTaskSummary>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing drive-center database url"))
        .map_err(drive_center_error_response)?;
    store
        .enqueue_task(DriveTaskInput {
            id: request.id,
            path: request.path,
            action: request.action,
            status: request.status,
        })
        .await
        .map(ok_json)
        .map_err(drive_center_error_response)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DriveCenterStatusResponse {
    pub ok: bool,
    pub database_configured: bool,
    pub store_connected: bool,
    pub table_prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct EnqueueDriveTaskRequest {
    pub id: Option<String>,
    pub path: String,
    pub action: String,
    pub status: Option<String>,
}

fn drive_center_error_response(error: anyhow::Error) -> Response {
    ApiError::from(error).into_response()
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
