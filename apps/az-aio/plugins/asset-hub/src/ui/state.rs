//! asset-hub SSR 页面状态。

use std::sync::{OnceLock, RwLock};

use crate::backend::{
    model::AssetSummary,
    routes::{AssetHubApiState, AssetHubStatusResponse},
    skill_scanner::{ScannedSkillAsset, scan_skill_assets},
};

static STATE: OnceLock<RwLock<Option<AssetHubApiState>>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetHubPageSnapshot {
    pub status: AssetHubStatusResponse,
    pub assets: Vec<AssetSummary>,
    pub scanned_skills: Vec<ScannedSkillAsset>,
    pub error: Option<String>,
}

pub fn install_state(state: AssetHubApiState) {
    let lock = STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(state);
    }
}

pub fn load_snapshot_server() -> AssetHubPageSnapshot {
    let state = STATE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()));
    let Some(state) = state else {
        return AssetHubPageSnapshot {
            status: AssetHubStatusResponse {
                ok: false,
                database_configured: false,
                store_connected: false,
                table_prefix: "biz_asset_hub_".to_string(),
            },
            assets: Vec::new(),
            scanned_skills: Vec::new(),
            error: Some("asset-hub runtime 尚未初始化".to_string()),
        };
    };

    let status = state.status();
    let mut error = None;
    let scanned_skills = match scan_skill_assets() {
        Ok(value) => value,
        Err(scan_error) => {
            error = Some(scan_error.to_string());
            Vec::new()
        }
    };
    let assets = match state.store() {
        Some(store) => match run_async(store.list_assets()) {
            Ok(value) => value,
            Err(store_error) => {
                error = Some(store_error.to_string());
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    AssetHubPageSnapshot {
        status,
        assets,
        scanned_skills,
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
