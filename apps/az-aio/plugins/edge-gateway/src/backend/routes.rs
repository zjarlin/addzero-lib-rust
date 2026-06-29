use std::{collections::BTreeMap, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use anyhow::anyhow;
use az_aio_platform::{
    core::api_error::{ApiError, ApiForm, ApiJson, ApiResponse, ok_json},
    system::model::SystemApiKeySummary,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    backend::{
        auth::{
            DEMO_WEATHER_TOKEN, EdgeApiToken, EdgeAuthError, EdgeTokenStore, EdgeUsageRecord,
            bearer_token, now_epoch_secs,
        },
        gateway_runtime::run_gateway_plan,
        gateway_runtime_types::{GatewayRunRequest, GatewayRunResult, GatewayRuntimeStep},
        model::{GatewayFlowSummary, GatewayRouteSummary, TABLE_NAME_PREFIX},
        store::{EdgeGatewayStore, GatewayFlowInput, GatewayRouteInput},
        weather_asset::{
            EdgeCallableAsset, WEATHER_CURRENT_ASSET_ID, WEATHER_CURRENT_ROUTE,
            WeatherCurrentRequest, WeatherCurrentResponse, query_current_weather,
            weather_current_asset,
        },
    },
};

#[derive(Clone)]
pub struct EdgeGatewayApiState {
    database_url: Option<String>,
    store: Option<EdgeGatewayStore>,
    http: Client,
    tokens: Arc<Mutex<EdgeTokenStore>>,
}

impl EdgeGatewayApiState {
    pub fn degraded(database_url: Option<String>) -> Self {
        Self {
            database_url,
            store: None,
            http: Client::new(),
            tokens: Arc::new(Mutex::new(default_token_store())),
        }
    }

    pub fn from_store(database_url: Option<String>, store: Option<EdgeGatewayStore>) -> Self {
        Self {
            database_url,
            store,
            http: Client::new(),
            tokens: Arc::new(Mutex::new(default_token_store())),
        }
    }

    pub fn status(&self) -> EdgeGatewayStatusResponse {
        EdgeGatewayStatusResponse {
            ok: true,
            database_configured: self.database_url.as_ref().is_some_and(|value| !value.is_empty()),
            store_connected: self.store.is_some(),
            table_prefix: TABLE_NAME_PREFIX.to_string(),
        }
    }

    pub fn store(&self) -> Option<EdgeGatewayStore> {
        self.store.clone()
    }

    pub fn callable_assets(&self) -> Vec<EdgeCallableAsset> {
        vec![weather_current_asset()]
    }

    pub async fn usage_records(&self) -> anyhow::Result<Vec<EdgeUsageRecord>> {
        if let Some(store) = &self.store {
            return store.list_usage_records().await;
        }
        Ok(self.tokens.lock().await.usage_records().to_vec())
    }
}

fn default_token_store() -> EdgeTokenStore {
    let mut store = EdgeTokenStore::default();
    store.upsert_token(EdgeApiToken::active(
        "tok_weather_demo",
        "demo weather",
        DEMO_WEATHER_TOKEN,
        [WEATHER_CURRENT_ROUTE],
    ));
    store
}


pub fn edge_gateway_router(state: EdgeGatewayApiState) -> Router {
    Router::new()
        .route("/api/edge-gateway/status", get(status_handler))
        .route("/api/edge-gateway/example", get(example_handler))
        .route("/api/edge-gateway/run", post(run_handler))
        .route("/api/edge-gateway/assets", get(list_assets_handler))
        .route("/api/edge-gateway/assets/usage", get(usage_handler))
        .route("/api/edge-gateway/routes", get(list_routes_handler))
        .route("/api/edge-gateway/route", post(upsert_route_handler))
        .route("/api/edge-gateway/ui-route", post(ui_route_action_handler))
        .route(WEATHER_CURRENT_ROUTE, post(weather_current_handler))
        .route("/api/edge-gateway/flows", get(list_flows_handler))
        .route("/api/edge-gateway/flow", post(upsert_flow_handler))
        .with_state(state)
}

async fn status_handler(State(state): State<EdgeGatewayApiState>) -> Json<EdgeGatewayStatusResponse> {
    Json(state.status())
}

async fn example_handler() -> Json<ApiResponse<GatewayRunRequest>> {
    Json(ApiResponse::ok(example_plan()))
}

async fn run_handler(
    ApiJson(request): ApiJson<GatewayRunRequest>,
) -> Result<Json<ApiResponse<GatewayRunResult>>, Response> {
    run_gateway_plan(request)
        .await
        .map(ok_json)
        .map_err(edge_gateway_error_response)
}

async fn list_assets_handler(
    State(state): State<EdgeGatewayApiState>,
) -> Json<ApiResponse<Vec<EdgeCallableAsset>>> {
    ok_json(state.callable_assets())
}

async fn usage_handler(
    State(state): State<EdgeGatewayApiState>,
) -> Result<Json<ApiResponse<Vec<EdgeUsageRecord>>>, Response> {
    state
        .usage_records()
        .await
        .map(ok_json)
        .map_err(edge_gateway_error_response)
}


async fn list_routes_handler(
    State(state): State<EdgeGatewayApiState>,
) -> Result<Json<ApiResponse<Vec<GatewayRouteSummary>>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing edge-gateway database url"))
        .map_err(edge_gateway_error_response)?;
    store
        .list_route_definitions()
        .await
        .map(ok_json)
        .map_err(edge_gateway_error_response)
}

async fn upsert_route_handler(
    State(state): State<EdgeGatewayApiState>,
    ApiJson(request): ApiJson<UpsertGatewayRouteRequest>,
) -> Result<Json<ApiResponse<GatewayRouteSummary>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing edge-gateway database url"))
        .map_err(edge_gateway_error_response)?;
    store
        .upsert_route_definition(route_input_from_request(request))
        .await
        .map(ok_json)
        .map_err(edge_gateway_error_response)
}

async fn ui_route_action_handler(
    State(state): State<EdgeGatewayApiState>,
    ApiForm(form): ApiForm<UpsertGatewayRouteRequest>,
) -> Response {
    let redirect = match apply_ui_route_action(state, form).await {
        Ok(route_id) => format!("/?route=/gateway&routeId={route_id}&saved=route"),
        Err(error) => format!(
            "/?route=/gateway&error={}",
            urlencoding::encode(&error.to_string())
        ),
    };
    Redirect::to(&redirect).into_response()
}

async fn apply_ui_route_action(
    state: EdgeGatewayApiState,
    request: UpsertGatewayRouteRequest,
) -> anyhow::Result<String> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing edge-gateway database url"))?;
    let route = store
        .upsert_route_definition(route_input_from_request(request))
        .await?;
    Ok(route.id)
}

async fn weather_current_handler(
    State(state): State<EdgeGatewayApiState>,
    system_api_key: Option<Extension<SystemApiKeySummary>>,
    headers: HeaderMap,
    ApiJson(request): ApiJson<WeatherCurrentRequest>,
) -> Result<Json<ApiResponse<WeatherCurrentResponse>>, Response> {
    let started = Instant::now();
    let authorized = authorize_asset_call(
        &state,
        WEATHER_CURRENT_ROUTE,
        &headers,
        system_api_key.map(|Extension(value)| value),
    )
    .await?;
    match query_current_weather(&state.http, request).await {
        Ok(response) => {
            record_asset_usage(&state, &authorized.token_id, WEATHER_CURRENT_ROUTE, 200, started).await;
            Ok(ok_json(response))
        }
        Err(error) => {
            record_asset_usage(&state, &authorized.token_id, WEATHER_CURRENT_ROUTE, 502, started).await;
            Err(ApiError::new(StatusCode::BAD_GATEWAY, error.to_string()).into_response())
        }
    }
}

async fn authorize_asset_call(
    state: &EdgeGatewayApiState,
    route: &str,
    headers: &HeaderMap,
    system_api_key: Option<SystemApiKeySummary>,
) -> Result<crate::backend::auth::EdgeAuthorizedToken, Response> {
    if let Some(api_key) = system_api_key {
        return Ok(crate::backend::auth::EdgeAuthorizedToken {
            token_id: format!("system:{}", api_key.id),
            token_name: api_key.name,
        });
    }

    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    let Some(token) = bearer_token(authorization) else {
        return Err(edge_auth_error_response(EdgeAuthError::MissingToken));
    };
    if let Some(store) = &state.store {
        return store
            .authorize_token(route, token)
            .await
            .map_err(edge_gateway_error_response);
    }
    state
        .tokens
        .lock()
        .await
        .authorize(route, token)
        .map_err(edge_auth_error_response)
}

async fn record_asset_usage(
    state: &EdgeGatewayApiState,
    token_id: &str,
    route: &str,
    status_code: u16,
    started: Instant,
) {
    let record = EdgeUsageRecord {
        token_id: token_id.to_string(),
        route: route.to_string(),
        asset_id: WEATHER_CURRENT_ASSET_ID.to_string(),
        status_code,
        request_units: 1,
        duration_ms: started.elapsed().as_millis(),
        created_at_epoch_secs: now_epoch_secs(),
    };
    if let Some(store) = &state.store {
        let _ = store.record_usage(record).await;
    } else {
        state.tokens.lock().await.record_usage(record);
    }
}

async fn list_flows_handler(
    State(state): State<EdgeGatewayApiState>,
) -> Result<Json<ApiResponse<Vec<GatewayFlowSummary>>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing edge-gateway database url"))
        .map_err(edge_gateway_error_response)?;
    store
        .list_flows()
        .await
        .map(ok_json)
        .map_err(edge_gateway_error_response)
}

async fn upsert_flow_handler(
    State(state): State<EdgeGatewayApiState>,
    ApiJson(request): ApiJson<UpsertGatewayFlowRequest>,
) -> Result<Json<ApiResponse<GatewayFlowSummary>>, Response> {
    let store = state
        .store
        .ok_or_else(|| anyhow!("missing edge-gateway database url"))
        .map_err(edge_gateway_error_response)?;
    store
        .upsert_flow(GatewayFlowInput {
            id: request.id,
            route: request.route,
            name: request.name,
            status: request.status,
        })
        .await
        .map(ok_json)
        .map_err(edge_gateway_error_response)
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


#[derive(Debug, Deserialize)]
pub struct UpsertGatewayRouteRequest {
    pub id: Option<String>,
    pub route: String,
    pub method: String,
    pub name: String,
    pub status: Option<String>,
    #[serde(alias = "authRequired")]
    pub auth_required: Option<String>,
    #[serde(alias = "scriptLanguage")]
    pub script_language: Option<String>,
    #[serde(alias = "scriptCode")]
    pub script_code: String,
    #[serde(alias = "requestExample")]
    pub request_example: String,
    #[serde(alias = "responseTemplate")]
    pub response_template: String,
    pub notes: String,
}

fn route_input_from_request(request: UpsertGatewayRouteRequest) -> GatewayRouteInput {
    GatewayRouteInput {
        id: request.id.filter(|value| !value.trim().is_empty()),
        route: request.route,
        method: request.method,
        name: request.name,
        status: request.status.filter(|value| !value.trim().is_empty()),
        auth_required: request
            .auth_required
            .as_deref()
            .map(|value| matches!(value, "true" | "on" | "1" | "yes")),
        script_language: request.script_language,
        script_code: request.script_code,
        request_example: request.request_example,
        response_template: request.response_template,
        notes: request.notes,
    }
}

#[derive(Debug, Deserialize)]
pub struct UpsertGatewayFlowRequest {
    pub id: Option<String>,
    pub route: String,
    pub name: String,
    pub status: Option<String>,
}

fn edge_gateway_error_response(error: anyhow::Error) -> Response {
    ApiError::from(error).into_response()
}

fn edge_auth_error_response(error: EdgeAuthError) -> Response {
    let status = StatusCode::from_u16(error.status_code()).unwrap_or(StatusCode::UNAUTHORIZED);
    ApiError::new(
        status,
        format!("unauthorized edge asset call: {}", error.code()),
    )
    .into_response()
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

    #[tokio::test]
    async fn degraded_state_exposes_weather_asset_and_demo_token() {
        let state = EdgeGatewayApiState::degraded(None);

        assert_eq!(state.callable_assets()[0].route, WEATHER_CURRENT_ROUTE);
        assert!(
            state
                .tokens
                .lock()
                .await
                .authorize(WEATHER_CURRENT_ROUTE, DEMO_WEATHER_TOKEN)
                .is_ok()
        );
    }

    #[tokio::test]
    async fn weather_asset_route_requires_bearer_token() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let app = edge_gateway_router(EdgeGatewayApiState::degraded(None));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(WEATHER_CURRENT_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"location":"Shanghai"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
