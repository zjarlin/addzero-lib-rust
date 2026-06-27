//! lowcode 插件的 engine 运行时状态。

use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use az_engine::EngineStore;

static STORE: OnceLock<EngineStore> = OnceLock::new();

/// 使用 PostgreSQL 连接串初始化全局 engine store。
pub fn connect_store_sync(database_url: &str) -> anyhow::Result<EngineStore> {
    build_runtime()?
        .block_on(EngineStore::connect(database_url))
        .context("初始化 lowcode engine store 失败")
}

/// 安装插件级全局 store，供 SSR renderer 同步读取。
pub fn install_store(store: EngineStore) {
    let _ = STORE.set(store);
}

/// 读取插件级全局 store。
pub fn store() -> anyhow::Result<EngineStore> {
    STORE
        .get()
        .cloned()
        .ok_or_else(|| anyhow!("lowcode engine store 尚未初始化"))
}

/// 在 SSR 同步渲染路径里执行 engine 异步查询。
pub fn run_engine_future<T, Fut>(future: Fut) -> anyhow::Result<T>
where
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    build_runtime()?.block_on(future)
}

fn build_runtime() -> anyhow::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("创建 lowcode engine runtime 失败")
}
