//! drive-center SSR 页面状态。

use std::sync::{OnceLock, RwLock};

use crate::backend::{
    model::DriveTaskSummary,
    routes::{DriveCenterApiState, DriveCenterStatusResponse},
};

static STATE: OnceLock<RwLock<Option<DriveCenterApiState>>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DriveCenterPageSnapshot {
    pub status: DriveCenterStatusResponse,
    pub tasks: Vec<DriveTaskSummary>,
    pub error: Option<String>,
}

pub fn install_state(state: DriveCenterApiState) {
    let lock = STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(state);
    }
}

pub fn load_snapshot_server() -> DriveCenterPageSnapshot {
    let state = STATE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()));
    let Some(state) = state else {
        return DriveCenterPageSnapshot {
            status: DriveCenterStatusResponse {
                ok: false,
                database_configured: false,
                store_connected: false,
                table_prefix: "biz_drive_center_".to_string(),
            },
            tasks: Vec::new(),
            error: Some("drive-center runtime 尚未初始化".to_string()),
        };
    };

    let status = state.status();
    let mut error = None;
    let tasks = match state.store() {
        Some(store) => match run_async(store.list_tasks()) {
            Ok(value) => value,
            Err(store_error) => {
                error = Some(store_error.to_string());
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    DriveCenterPageSnapshot {
        status,
        tasks,
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
