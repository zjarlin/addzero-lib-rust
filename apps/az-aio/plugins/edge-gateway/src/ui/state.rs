//! edge-gateway SSR 页面状态。

use std::sync::{OnceLock, RwLock};

use crate::backend::{
    model::GatewayFlowSummary,
    routes::{EdgeGatewayApiState, EdgeGatewayStatusResponse, example_plan},
};

static STATE: OnceLock<RwLock<Option<EdgeGatewayApiState>>> = OnceLock::new();

pub struct EdgeGatewayPageSnapshot {
    pub status: EdgeGatewayStatusResponse,
    pub flows: Vec<GatewayFlowSummary>,
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
            example_step_count: example_plan().steps.len(),
            error: Some("edge-gateway runtime 尚未初始化".to_string()),
        };
    };

    let status = state.status();
    let mut error = None;
    let flows = match state.store() {
        Some(store) => match run_async(store.list_flows()) {
            Ok(value) => value,
            Err(store_error) => {
                error = Some(store_error.to_string());
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    EdgeGatewayPageSnapshot {
        status,
        flows,
        example_step_count: example_plan().steps.len(),
        error,
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
