use std::{env, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use toasty::{ModelSet, sql};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    env_file::{LOCAL_ENV_FILE, read_database_url_from_path, workspace_env_path_from},
    migration::workspace_sql_migrations,
    sql_split::split_sql_statements,
};

/// 可在多个 repository 之间共享的 Toasty 数据库句柄。
#[derive(Clone)]
pub struct PersistenceDb {
    inner: Arc<Mutex<toasty::Db>>,
}

impl PersistenceDb {
    fn new(db: toasty::Db) -> Self {
        Self {
            inner: Arc::new(Mutex::new(db)),
        }
    }

    /// 获取 Toasty 数据库的独占执行守卫。
    pub async fn lock(&self) -> MutexGuard<'_, toasty::Db> {
        self.inner.lock().await
    }
}

/// 统一管理 Toasty 模型、PostgreSQL 连接和 workspace SQL 迁移。
#[derive(Clone)]
pub struct PersistenceContext {
    database_url: String,
    db: PersistenceDb,
}

impl PersistenceContext {
    /// 使用环境中的数据库连接和明确的 Toasty 模型集合建立上下文。
    pub async fn connect(models: ModelSet) -> Result<Self> {
        let database_url = database_url()
            .context("缺少 MSC_AIO_DATABASE_URL / DATABASE_URL / ~/.config/aio/aio.env")?;
        Self::connect_with_url(&database_url, models).await
    }

    /// 使用指定 PostgreSQL 连接和 Toasty 模型集合建立上下文。
    pub async fn connect_with_url(database_url: &str, models: ModelSet) -> Result<Self> {
        let database_url = verify_database_url(database_url)?;
        let mut db = toasty::Db::builder()
            .models(models)
            .connect(database_url)
            .await
            .with_context(|| format!("连接 PostgreSQL 失败: {database_url}"))?;

        run_workspace_migrations(&mut db)
            .await
            .context("执行 workspace SQL 迁移失败")?;
        sql::statement("SELECT 1")
            .exec(&mut db)
            .await
            .context("检查 PostgreSQL 连接失败")?;

        Ok(Self {
            database_url: database_url.to_owned(),
            db: PersistenceDb::new(db),
        })
    }

    /// 返回共享 Toasty 数据库句柄。
    pub fn db(&self) -> &PersistenceDb {
        &self.db
    }

    /// 返回当前 PostgreSQL 连接串。
    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}

/// 从环境变量或本地配置文件读取 PostgreSQL 连接串。
pub fn database_url() -> Option<String> {
    env::var("MSC_AIO_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(read_database_url_from_local_env)
}

#[deprecated(note = "AIO desktop configuration is stored in ~/.config/aio/aio.env")]
pub fn workspace_env_path() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    workspace_env_path_from(&cwd)
}

pub fn local_env_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(LOCAL_ENV_FILE))
}

async fn run_workspace_migrations(db: &mut toasty::Db) -> Result<()> {
    sql::statement(
        "CREATE TABLE IF NOT EXISTS toasty_workspace_migrations (name TEXT PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
    )
    .exec(&mut *db)
    .await
    .context("创建 Toasty 迁移记录表失败")?;
    for migration in workspace_sql_migrations() {
        let mut transaction = db
            .transaction()
            .await
            .with_context(|| format!("开启迁移事务失败: {}", migration.name))?;
        sql::statement("LOCK TABLE toasty_workspace_migrations IN EXCLUSIVE MODE")
            .exec(&mut transaction)
            .await
            .with_context(|| format!("锁定迁移记录失败: {}", migration.name))?;
        let applied = sql::query(
            "SELECT 1 FROM toasty_workspace_migrations WHERE name = $1",
        )
        .bind(migration.name)
        .exec(&mut transaction)
        .await
        .with_context(|| format!("查询迁移记录失败: {}", migration.name))?;
        if applied.is_empty() {
            for statement in split_sql_statements(migration.sql) {
                sql::statement(statement)
                    .exec(&mut transaction)
                    .await
                    .with_context(|| format!("执行迁移失败: {}", migration.name))?;
            }
            sql::statement(
                "INSERT INTO toasty_workspace_migrations (name) VALUES ($1)",
            )
            .bind(migration.name)
            .exec(&mut transaction)
            .await
            .with_context(|| format!("记录迁移失败: {}", migration.name))?;
        }
        transaction
            .commit()
            .await
            .with_context(|| format!("提交迁移失败: {}", migration.name))?;
    }
    Ok(())
}

fn verify_database_url(value: &str) -> Result<&str> {
    let value = value.trim();
    if value.starts_with("postgresql://") || value.starts_with("postgres://") {
        return Ok(value);
    }
    anyhow::bail!("正式持久化只接受 PostgreSQL 连接串: {value}")
}

fn read_database_url_from_local_env() -> Option<String> {
    let path = local_env_path()?;
    read_database_url_from_path(&path)
}

fn home_dir() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}
