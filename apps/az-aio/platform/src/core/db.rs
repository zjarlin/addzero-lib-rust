//! 共享数据库底座。
//!
//! 为所有 AZ AIO 插件提供统一的 `toasty::Db` 连接管理与工具函数，
//! 消除每个插件各自重复的 `Arc<Mutex<Db>>`、URL 校验、UUID 和时间戳逻辑。

use std::sync::Arc;

use anyhow::{Context, anyhow};
use tokio::sync::Mutex;

/// 共享数据库柄。
///
/// 持有 `toasty::Db` 的共享引用，所有插件复用同一连接池。
/// 插件负责用自身的模型注册 `toasty::Db`，然后传入 `SharedDb` 进行包装。
#[derive(Clone)]
pub struct SharedDb {
    db: Arc<Mutex<toasty::Db>>,
}

impl SharedDb {
    /// 从已配置好的 `toasty::Db` 创建共享包装。
    ///
    /// `toasty::Db` 的构造（包括 `.models(...)`、`.table_name_prefix(...)`、
    /// `.connect(...)` 和 `push_schema()`）由调用方（插件）完成，
    /// `SharedDb` 只负责提供共享的 `Arc<Mutex<>>` 访问。
    pub fn new(db: toasty::Db) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }

    /// 使用给定的 PostgreSQL 连接串建立连接并执行 schema 迁移。
    ///
    /// 适合不需要自定义模型注册的简单场景。
    pub async fn connect_raw(database_url: &str, table_prefix: &str) -> anyhow::Result<Self> {
        let database_url = verify_database_url(database_url)?;
        let db = toasty::Db::builder()
            .table_name_prefix(table_prefix)
            .connect(database_url)
            .await
            .with_context(|| format!("连接数据库失败: {database_url}"))?;
        db.push_schema().await.context("数据库 schema 迁移失败")?;
        Ok(Self::new(db))
    }

    /// 获取内部 `toasty::Db` 的锁守卫。
    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, toasty::Db> {
        self.db.lock().await
    }
}

/// 校验并规范化数据库连接串。
pub fn verify_database_url(value: &str) -> anyhow::Result<&str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(anyhow!("数据库连接串未配置"));
    }
    if !value.starts_with("postgresql://") && !value.starts_with("postgres://") {
        return Err(anyhow!(
            "AZ AIO 正式持久化只接受 PostgreSQL 连接串，请改用 postgresql://...，当前值: {value}"
        ));
    }
    Ok(value)
}

/// 生成 UUID v4 字符串。
pub fn new_uuid_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 生成秒级 Unix 时间戳字符串。
pub fn timestamp_secs() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_url() {
        assert!(verify_database_url("").is_err());
        assert!(verify_database_url("   ").is_err());
    }

    #[test]
    fn accepts_valid_url() {
        assert_eq!(
            verify_database_url("postgresql://localhost/test").unwrap(),
            "postgresql://localhost/test"
        );
        assert_eq!(
            verify_database_url("postgres://localhost/test").unwrap(),
            "postgres://localhost/test"
        );
    }

    #[test]
    fn rejects_sqlite_url() {
        let error = verify_database_url("sqlite:az-aio.db?mode=rwc").unwrap_err();
        // 防止正式 admin 数据误落到本地 SQLite，统一走 Toasty PG。
        assert!(error.to_string().contains("PostgreSQL"));

        let error = verify_database_url("sqlite:sync.db?mode=rwc").unwrap_err();
        // sync.db 也不再作为正式持久化落点。
        assert!(error.to_string().contains("PostgreSQL"));
    }

    #[test]
    fn uuid_has_expected_length() {
        assert_eq!(new_uuid_id().len(), 36);
    }

    #[test]
    fn timestamp_is_non_empty() {
        assert!(!timestamp_secs().is_empty());
    }
}
