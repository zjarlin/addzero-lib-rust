use std::collections::BTreeMap;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{EdgeGatewayError, EdgeGatewayResult},
    gateway_runtime::run_gateway_plan,
    gateway_runtime_types::{GatewayRunRequest, GatewayRunResult, GatewayRuntimeStep},
    model::{GatewayFlowSummary, TABLE_NAME_PREFIX},
    store::{EdgeGatewayStore, GatewayFlowInput},
};

#[derive(Clone)]
pub struct EdgeGatewayApiState {
    database_url: Option<String>,
    store: Option<EdgeGatewayStore>,
}

impl EdgeGatewayApiState {
    pub async fn new(database_url: Option<String>) -> EdgeGatewayResult<Self> {
        let store = match database_url.as_deref() {
            Some(value) if !value.trim().is_empty() => Some(EdgeGatewayStore::connect(value).await?),
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

pub fn edge_gateway_router(state: EdgeGatewayApiState) -> Router {
    Router::new()
        .route("/api/edge-gateway/status", get(status_handler))
        .route("/api/edge-gateway/example", get(example_handler))
        .route("/api/edge-gateway/run", post(run_handler))
        .route("/api/edge-gateway/flows", get(list_flows_handler))
        .route("/api/edge-gateway/flow", post(upsert_flow_handler))
        .with_state(state)
}

async fn status_handler(State(state): State<EdgeGatewayApiState>) -> Json<EdgeGatewayStatusResponse> {
    Json(EdgeGatewayStatusResponse {
        ok: true,
        database_configured: state.database_url.as_ref().is_some_and(|value| !value.is_empty()),
        store_connected: state.store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
    })
}

async fn example_handler() -> Json<ApiResponse<GatewayRunRequest>> {
    Json(ApiResponse::ok(example_plan()))
}

async fn run_handler(
    Json(request): Json<GatewayRunRequest>,
) -> Result<Json<ApiResponse<GatewayRunResult>>, EdgeGatewayApiError> {
    run_gateway_plan(request)
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(EdgeGatewayError::from)
        .map_err(Into::into)
}

async fn list_flows_handler(
    State(state): State<EdgeGatewayApiState>,
) -> Result<Json<ApiResponse<Vec<GatewayFlowSummary>>>, EdgeGatewayApiError> {
    let store = state.store.ok_or(EdgeGatewayError::MissingDatabaseUrl)?;
    store
        .list_flows()
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

async fn upsert_flow_handler(
    State(state): State<EdgeGatewayApiState>,
    Json(request): Json<UpsertGatewayFlowRequest>,
) -> Result<Json<ApiResponse<GatewayFlowSummary>>, EdgeGatewayApiError> {
    let store = state.store.ok_or(EdgeGatewayError::MissingDatabaseUrl)?;
    store
        .upsert_flow(GatewayFlowInput {
            id: request.id,
            route: request.route,
            name: request.name,
            status: request.status,
        })
        .await
        .map(ApiResponse::ok)
        .map(Json)
        .map_err(Into::into)
}

pub fn example_plan() -> GatewayRunRequest {
    GatewayRunRequest {
        entry_route: "/edge/session-proxy".to_string(),
        input: Value::Null,
        steps: vec![GatewayRuntimeStep {
            body_preview: String::new(),
            capture_path: "$.headers.host".to_string(),
            depends_on: Vec::new(),
            headers: BTreeMap::new(),
            id: "ping".to_string(),
            input_refs: Vec::new(),
            kind: "curl".to_string(),
            label: "GET postman echo".to_string(),
            method: "GET".to_string(),
            notes: "Reference flow".to_string(),
            url: "https://postman-echo.com/get?source=aio-desktop".to_string(),
        }],
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EdgeGatewayStatusResponse {
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
pub struct UpsertGatewayFlowRequest {
    pub id: Option<String>,
    pub route: String,
    pub name: String,
    pub status: Option<String>,
}

#[derive(Debug)]
pub struct EdgeGatewayApiError {
    source: EdgeGatewayError,
}

impl From<EdgeGatewayError> for EdgeGatewayApiError {
    fn from(source: EdgeGatewayError) -> Self {
        Self { source }
    }
}

impl IntoResponse for EdgeGatewayApiError {
    fn into_response(self) -> Response {
        let status = match self.source {
            EdgeGatewayError::MissingDatabaseUrl => StatusCode::SERVICE_UNAVAILABLE,
            EdgeGatewayError::BlankFlowName | EdgeGatewayError::BlankRoute => {
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

    #[test]
    fn example_plan_has_entry_route_and_step() {
        let plan = example_plan();
        assert_eq!(plan.entry_route, "/edge/session-proxy");
        assert_eq!(plan.steps.len(), 1);
    }
}
