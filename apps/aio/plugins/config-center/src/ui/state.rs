//! config-center SSR 页面状态。

use std::sync::{OnceLock, RwLock};

use crate::backend::{
    dotfiles_monitor::scan_dotfiles_status,
    dotfiles_monitor_types::DotfilesMonitorStatus,
    model::ConfigEntrySummary,
    pairing::{PairingLocalInfo, local_pairing_info},
    routes::{ConfigCenterApiState, ConfigCenterStatusResponse},
};

pub const DEFAULT_NAMESPACE: &str = "az-aio";

static STATE: OnceLock<RwLock<Option<ConfigCenterApiState>>> = OnceLock::new();

pub struct ConfigCenterPageSnapshot {
    pub status: Option<ConfigCenterStatusResponse>,
    pub dotfiles: Option<DotfilesMonitorStatus>,
    pub pairing: Option<PairingLocalInfo>,
    pub entries: Vec<ConfigEntrySummary>,
    pub errors: Vec<String>,
}

pub fn install_state(state: ConfigCenterApiState) {
    let lock = STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(state);
    }
}

pub fn load_snapshot() -> ConfigCenterPageSnapshot {
    let mut errors = Vec::new();
    let state = STATE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()));

    let Some(state) = state else {
        errors.push("config-center runtime 尚未初始化".to_string());
        return ConfigCenterPageSnapshot {
            status: None,
            dotfiles: load_dotfiles(&mut errors),
            pairing: load_pairing(&mut errors),
            entries: Vec::new(),
            errors,
        };
    };

    let status = match state.status() {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error.to_string());
            None
        }
    };
    let entries = match state.store() {
        Some(store) => match run_async(store.list_entries(DEFAULT_NAMESPACE)) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error.to_string());
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    ConfigCenterPageSnapshot {
        status,
        dotfiles: load_dotfiles(&mut errors),
        pairing: load_pairing(&mut errors),
        entries,
        errors,
    }
}

fn load_dotfiles(errors: &mut Vec<String>) -> Option<DotfilesMonitorStatus> {
    match scan_dotfiles_status() {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error.to_string());
            None
        }
    }
}

fn load_pairing(errors: &mut Vec<String>) -> Option<PairingLocalInfo> {
    match local_pairing_info() {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error.to_string());
            None
        }
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
