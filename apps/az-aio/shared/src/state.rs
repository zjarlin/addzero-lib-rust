//! 共享应用状态。

use az_aio_plugin_host::host::HostSnapshot;

pub use az_aio_plugin_host::host::HostSnapshot;

/// 平台无关的应用状态。
///
/// 持有插件快照，由各平台的路由处理器使用。
#[derive(Clone)]
pub struct AppState {
    pub snapshot: HostSnapshot,
}

impl AppState {
    pub fn new(snapshot: HostSnapshot) -> Self {
        Self { snapshot }
    }
}
