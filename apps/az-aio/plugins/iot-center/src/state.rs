//! 物联网插件 SSR 状态。

use std::sync::OnceLock;

use anyhow::{Context, anyhow};

use crate::service::IotService;

static SERVICE: OnceLock<IotService> = OnceLock::new();

/// 安装插件级物联网服务。
pub fn install_service(service: IotService) {
    let _ = SERVICE.set(service);
}

/// 读取插件级物联网服务。
pub fn service() -> anyhow::Result<IotService> {
    SERVICE
        .get()
        .cloned()
        .ok_or_else(|| anyhow!("物联网中心服务尚未初始化"))
}

/// 在 SSR 同步路径执行物联网异步查询。
pub fn run_iot_future<T, F>(future: F) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("创建物联网 SSR runtime 失败")?;
    runtime.block_on(future)
}
