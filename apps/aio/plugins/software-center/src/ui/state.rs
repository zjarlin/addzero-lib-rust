//! software-center SSR 页面状态。

use std::sync::{OnceLock, RwLock};

use crate::backend::{
    installer_scanner::{InstallerPackage, scan_installers},
    model::SoftwarePackageSummary,
    routes::{SoftwareCenterApiState, SoftwareCenterStatusResponse},
};

static STATE: OnceLock<RwLock<Option<SoftwareCenterApiState>>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareCenterPageSnapshot {
    pub status: SoftwareCenterStatusResponse,
    pub installers: Vec<InstallerPackage>,
    pub packages: Vec<SoftwarePackageSummary>,
    pub error: Option<String>,
}

pub fn install_state(state: SoftwareCenterApiState) {
    let lock = STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(state);
    }
}

pub fn load_snapshot_server() -> SoftwareCenterPageSnapshot {
    let state = STATE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()));
    let Some(state) = state else {
        return SoftwareCenterPageSnapshot {
            status: SoftwareCenterStatusResponse {
                ok: false,
                database_configured: false,
                store_connected: false,
                table_prefix: "biz_software_center_".to_string(),
            },
            installers: Vec::new(),
            packages: Vec::new(),
            error: Some("software-center runtime 尚未初始化".to_string()),
        };
    };

    let status = state.status();
    let mut error = None;
    let installers = match scan_installers() {
        Ok(value) => value,
        Err(scan_error) => {
            error = Some(scan_error.to_string());
            Vec::new()
        }
    };
    let packages = match state.store() {
        Some(store) => match run_async(store.list_packages()) {
            Ok(value) => value,
            Err(store_error) => {
                error = Some(store_error.to_string());
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    SoftwareCenterPageSnapshot {
        status,
        installers,
        packages,
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
