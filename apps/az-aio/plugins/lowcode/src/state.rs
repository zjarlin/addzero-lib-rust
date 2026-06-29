//! lowcode 插件的 engine 运行时状态。

use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use az_aio_platform::core::db::Db;
use az_engine::EngineStore;

static STORE: OnceLock<EngineStore> = OnceLock::new();

/// 使用应用级共享 Toasty 单例初始化 engine store。
pub fn store_from_shared_db(shared_db: Db) -> EngineStore {
    EngineStore::from_shared_db(shared_db.shared_handle())
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
