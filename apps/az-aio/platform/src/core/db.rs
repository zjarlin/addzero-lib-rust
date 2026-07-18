//! 共享数据库底座。
//!
//! 为所有 插件提供统一的 `toasty::Db` 连接管理与工具函数，
//! 消除每个插件各自重复的 `Arc<Mutex<Db>>`、URL 校验、UUID 和时间戳逻辑。

use std::sync::Arc;

use anyhow::{Context, anyhow};
use toasty::{ModelSet, sql};
use tokio::sync::Mutex;

/// Rudi-collected Toasty model contribution.
///
/// Each plugin exposes its persistent models as one contribution. The app
/// entrypoint merges these contributions before constructing the shared
/// `toasty::Db`, so the web shell does not hard-code every plugin model.
#[derive(Clone, Debug)]
pub struct ToastyModelContribution {
    models: ModelSet,
}

impl ToastyModelContribution {
    pub fn new(models: ModelSet) -> Self {
        Self { models }
    }

    pub fn into_model_set(self) -> ModelSet {
        self.models
    }
}

/// 共享数据库柄。
///
/// 持有 `toasty::Db` 的共享引用，所有插件复用同一连接池。
/// 平台启动时统一注册所有 Toasty 模型，插件 store 只持有该共享句柄。
#[derive(Clone)]
pub struct Db {
    db: Arc<Mutex<toasty::Db>>,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Db").finish_non_exhaustive()
    }
}

impl Db {
    /// 从已配置好的 `toasty::Db` 创建共享包装。
    ///
    /// `toasty::Db` 的构造（包括 `.models(...)`、`.connect(...)` 和 `push_schema()`）
    /// 由平台启动入口统一完成，
    /// `Db` 只负责提供共享的 `Arc<Mutex<>>` 访问。
    pub fn new(db: toasty::Db) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }

    /// 使用给定的模型集合建立 PostgreSQL 连接并执行 schema 迁移。
    pub async fn connect_with_models(
        database_url: &str,
        models: ModelSet,
        bootstrap_sql: &[&str],
    ) -> anyhow::Result<Self> {
        let database_url = verify_database_url(database_url)?;
        let mut db = toasty::Db::builder()
            .models(models)
            .connect(database_url)
            .await
            .with_context(|| format!("连接数据库失败: {database_url}"))?;
        push_or_bootstrap_schema(&mut db, bootstrap_sql)
            .await
            .context("数据库 schema 迁移失败")?;
        Ok(Self::new(db))
    }

    /// 使用给定的 PostgreSQL 连接串建立连接并执行 schema 迁移。
    ///
    /// 获取内部 `toasty::Db` 的锁守卫。
    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, toasty::Db> {
        self.db.lock().await
    }

    /// Returns the shared Toasty handle for stores that already own an executor wrapper.
    pub fn shared_handle(&self) -> Arc<Mutex<toasty::Db>> {
        Arc::clone(&self.db)
    }
}

/// 把共享数据库作为 Rudi singleton 写入容器。
pub async fn install_shared_db_singleton(
    di: &mut rudi::Context,
    database_url: Option<&str>,
    models: ModelSet,
    bootstrap_sql: &[&str],
) -> anyhow::Result<Option<Db>> {
    let Some(database_url) = database_url.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let db = Db::connect_with_models(database_url, models, bootstrap_sql).await?;
    di.insert_singleton(db.clone());
    Ok(Some(db))
}

/// Merge all Rudi-collected Toasty model contributions into one model set.
pub fn collect_toasty_models(di: &mut rudi::Context) -> ModelSet {
    let mut models = ModelSet::new();
    for contribution in di.resolve_by_type::<ToastyModelContribution>() {
        for model in contribution.into_model_set() {
            models.add(model);
        }
    }
    models
}

async fn push_or_bootstrap_schema(
    db: &mut toasty::Db,
    bootstrap_sql: &[&str],
) -> anyhow::Result<()> {
    match db.push_schema().await {
        Ok(()) => Ok(()),
        Err(error)
            if is_relation_already_exists(&error.to_string()) && !bootstrap_sql.is_empty() =>
        {
            for statement in bootstrap_sql {
                sql::statement(*statement).exec(db).await?;
            }
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

/// Detects PostgreSQL duplicate-table/index failures emitted by Toasty schema push.
pub fn is_relation_already_exists(message: &str) -> bool {
    message.contains("already exists") || message.contains("relation") && message.contains("exists")
}

/// 校验并规范化数据库连接串。
pub fn verify_database_url(value: &str) -> anyhow::Result<&str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(anyhow!("数据库连接串未配置"));
    }
    if !value.starts_with("postgresql://") && !value.starts_with("postgres://") {
        return Err(anyhow!(
            "正式持久化只接受 PostgreSQL 连接串，请改用 postgresql://...，当前值: {value}"
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

/// 生成毫秒级 Unix 时间戳。
pub fn timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
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
    fn timestamp_ms_uses_millisecond_precision() {
        // 管理 API 统一返回毫秒时间点，避免前端再猜测时间单位。
        assert!(timestamp_ms() > 1_000_000_000_000);
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
    fn detects_existing_relation_errors() {
        assert!(is_relation_already_exists("relation demo already exists"));
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
