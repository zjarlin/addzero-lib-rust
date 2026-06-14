//! 插件快照加载与 loopback 服务。
//!
//! 桌面和 web 平台共用：编译时 inventory 发现所有插件，
//! 构建 `HostSnapshot` 并启动原生插件 loopback 服务器。

use az_aio_plugin_host::host::HostSnapshot;

/// 加载原生插件快照。
///
/// 通过 `inventory` 发现所有 `register_native_plugin!` 注册的插件，
/// 依 enablement 状态过滤后构建快照。
pub fn load_snapshot() -> HostSnapshot {
    az_aio_plugin_bundled::api::ensure_linked();
    az_aio_plugin_host::host::load_az_aio_native_snapshot()
}

/// 启动原生插件 loopback 服务器。
///
/// 返回本地监听地址（如 `http://127.0.0.1:xxxxx`）。
/// 失败时仅打印警告，不影响主流程。
pub async fn start_loopback_server(snapshot: HostSnapshot) {
    match az_aio_plugin_host::host::start_native_loopback_server(snapshot).await {
        Ok(addr) => println!("plugin loopback server listening on {addr}"),
        Err(e) => eprintln!("plugin loopback server failed to start: {e}"),
    }
}
