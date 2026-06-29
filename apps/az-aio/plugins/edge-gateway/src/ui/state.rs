//! edge-gateway SSR 页面状态。

use std::sync::{OnceLock, RwLock};

use crate::backend::{
    auth::EdgeUsageRecord,
    model::{GatewayFlowSummary, GatewayRouteSummary},
    routes::{EdgeGatewayApiState, EdgeGatewayStatusResponse, example_plan},
    weather_asset::{EdgeCallableAsset, weather_current_asset},
};

static STATE: OnceLock<RwLock<Option<EdgeGatewayApiState>>> = OnceLock::new();

pub struct EdgeGatewayPageSnapshot {
    pub status: EdgeGatewayStatusResponse,
    pub flows: Vec<GatewayFlowSummary>,
    pub route_definitions: Vec<GatewayRouteSummary>,
    pub callable_assets: Vec<EdgeCallableAsset>,
    pub usage_records: Vec<EdgeUsageRecord>,
    pub example_step_count: usize,
    pub error: Option<String>,
}

pub fn install_state(state: EdgeGatewayApiState) {
    let lock = STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(state);
    }
}

pub fn load_snapshot() -> EdgeGatewayPageSnapshot {
    let state = STATE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()));
    let Some(state) = state else {
        return EdgeGatewayPageSnapshot {
            status: EdgeGatewayStatusResponse {
                ok: false,
                database_configured: false,
                store_connected: false,
                table_prefix: "biz_edge_gateway_".to_string(),
            },
            flows: Vec::new(),
            route_definitions: Vec::new(),
            callable_assets: vec![weather_current_asset()],
            usage_records: Vec::new(),
            example_step_count: example_plan().steps.len(),
            error: Some("edge-gateway runtime 尚未初始化".to_string()),
        };
    };

    let status = state.status();
    let mut errors = Vec::new();
    let (flows, route_definitions) = match state.store() {
        Some(store) => {
            let flows = match run_async(store.list_flows()) {
                Ok(value) => value,
                Err(error) => {
                    errors.push(error.to_string());
                    Vec::new()
                }
            };
            let route_definitions = match run_async(store.list_route_definitions()) {
                Ok(value) => value,
                Err(error) => {
                    errors.push(error.to_string());
                    Vec::new()
                }
            };
            (flows, route_definitions)
        }
        None => (Vec::new(), Vec::new()),
    };
    let usage_records = match run_async(state.usage_records()) {
        Ok(value) => value,
        Err(error) => {
            errors.push(error.to_string());
            Vec::new()
        }
    };

    EdgeGatewayPageSnapshot {
        status,
        flows,
        route_definitions,
        callable_assets: state.callable_assets(),
        usage_records,
        example_step_count: example_plan().steps.len(),
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        },
    }
}

fn run_async<T, Fut>(future: Fut) -> anyhow::Result<T>
where
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(future)
}
