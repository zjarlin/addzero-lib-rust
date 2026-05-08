use std::{
    collections::BTreeMap,
    env,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr};
use sea_orm_migration::prelude::*;
use thiserror::Error;

const WORKSPACE_ENV_FILE: &str = ".env";
const LOCAL_ENV_FILE: &str = ".config/aio/aio.env";
static WORKSPACE_MIGRATIONS_DONE: AtomicBool = AtomicBool::new(false);
static WORKSPACE_MIGRATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Shared database context for workspace services.
#[derive(Clone)]
pub struct PersistenceContext {
    database_url: String,
    db: DatabaseConnection,
}

impl PersistenceContext {
    pub async fn connect() -> Result<Self, PersistenceError> {
        let database_url = database_url().ok_or(PersistenceError::MissingDatabaseUrl)?;
        Self::connect_with_url(&database_url).await
    }

    pub async fn connect_with_url(database_url: &str) -> Result<Self, PersistenceError> {
        let mut options = ConnectOptions::new(database_url.to_owned());
        options
            .max_connections(8)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .sqlx_logging(false);

        let db = Database::connect(options)
            .await
            .map_err(PersistenceError::Connect)?;

        if !WORKSPACE_MIGRATIONS_DONE.load(Ordering::Acquire) {
            let _guard = WORKSPACE_MIGRATION_LOCK.lock().await;
            if !WORKSPACE_MIGRATIONS_DONE.load(Ordering::Acquire) {
                match WorkspaceMigrator::up(&db, None).await {
                    Ok(()) => {}
                    Err(err) if is_concurrent_migration_conflict(&err) => {}
                    Err(err) => return Err(PersistenceError::Migrate(err)),
                }
                WORKSPACE_MIGRATIONS_DONE.store(true, Ordering::Release);
            }
        }

        db.execute_unprepared("SELECT 1")
            .await
            .map_err(PersistenceError::Ping)?;

        Ok(Self {
            database_url: database_url.to_owned(),
            db,
        })
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn into_connection(self) -> DatabaseConnection {
        self.db
    }
}

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("missing MSC_AIO_DATABASE_URL / repo .env / DATABASE_URL / ~/.config/aio/aio.env")]
    MissingDatabaseUrl,
    #[error("connect to postgres: {0}")]
    Connect(#[source] DbErr),
    #[error("run workspace migrations: {0}")]
    Migrate(#[source] DbErr),
    #[error("ping postgres: {0}")]
    Ping(#[source] DbErr),
}

pub fn database_url() -> Option<String> {
    env::var("MSC_AIO_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(read_database_url_from_workspace_env)
        .or_else(|| {
            env::var("DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(read_database_url_from_local_env)
}

pub fn workspace_env_path() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    workspace_env_path_from(&cwd)
}

pub fn local_env_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(LOCAL_ENV_FILE))
}

fn read_database_url_from_workspace_env() -> Option<String> {
    let path = workspace_env_path()?;
    read_database_url_from_path(&path)
}

fn read_database_url_from_local_env() -> Option<String> {
    let path = local_env_path()?;
    read_database_url_from_path(&path)
}

fn read_database_url_from_path(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let vars = parse_env_pairs(&content);
    vars.get("MSC_AIO_DATABASE_URL")
        .or_else(|| vars.get("DATABASE_URL"))
        .cloned()
        .filter(|value| !value.trim().is_empty())
}

fn workspace_env_path_from(start_dir: &Path) -> Option<PathBuf> {
    start_dir
        .ancestors()
        .map(|dir| dir.join(WORKSPACE_ENV_FILE))
        .find(|path| path.is_file())
}

fn parse_env_pairs(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn home_dir() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn is_concurrent_migration_conflict(err: &DbErr) -> bool {
    let message = err.to_string();
    message.contains("seaql_migrations_pkey")
        || message.contains("duplicate key value violates unique constraint")
            && message.contains("seaql_migrations")
}

#[cfg(test)]
mod tests {
    use super::{read_database_url_from_path, workspace_env_path_from};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn workspace_env_path_finds_ancestor_env() {
        let root = unique_temp_dir("workspace-env-path");
        let nested = root.join("apps/aio/backend");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join(".env"), "DATABASE_URL=postgresql://root\n").unwrap();

        let found = workspace_env_path_from(&nested).unwrap();
        assert_eq!(found, root.join(".env"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn read_database_url_prefers_msc_key_from_env_file() {
        let dir = unique_temp_dir("workspace-env-read");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        fs::write(
            &path,
            "# comment\nDATABASE_URL=postgresql://fallback\nMSC_AIO_DATABASE_URL=postgresql://preferred\n",
        )
        .unwrap();

        let value = read_database_url_from_path(&path).unwrap();
        assert_eq!(value, "postgresql://preferred");

        fs::remove_dir_all(dir).unwrap();
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("addzero-persistence-{prefix}-{unique}"))
    }
}

pub struct WorkspaceMigrator;

#[async_trait::async_trait]
impl MigratorTrait for WorkspaceMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CliMarketSchemaMigration),
            Box::new(AdminAssetGraphSchemaMigration),
            Box::new(AdminSoftwareCatalogSchemaMigration),
            Box::new(AdminBrandingSettingsSchemaMigration),
            Box::new(AssetSchemaMigration),
            Box::new(KnowledgeSchemaMigration),
            Box::new(SkillSchemaMigration),
            Box::new(RemoveAgentRuntimeSchemaMigration),
        ]
    }
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        manager.get_connection().execute_unprepared(trimmed).await?;
    }
    Ok(())
}

#[derive(DeriveMigrationName)]
struct CliMarketSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for CliMarketSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/aio/backend/src/server/migrations/0002_clianything_market.sql"
            ),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct AdminAssetGraphSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for AdminAssetGraphSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/aio/backend/src/server/migrations/0003_admin_asset_graph.sql"
            ),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct AdminSoftwareCatalogSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for AdminSoftwareCatalogSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../addzero-software-catalog/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct AdminBrandingSettingsSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for AdminBrandingSettingsSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/aio/backend/src/server/migrations/0006_admin_branding_settings.sql"
            ),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct AssetSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for AssetSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../addzero-assets/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct KnowledgeSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for KnowledgeSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../addzero-knowledge/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct SkillSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for SkillSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../addzero-skills/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct RemoveAgentRuntimeSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for RemoveAgentRuntimeSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/aio/backend/src/server/migrations/0009_remove_agent_runtime.sql"
            ),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct AdminMenuSystemSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for AdminMenuSystemSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/aio/backend/src/server/migrations/0011_admin_menu_system.sql"
            ),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct UnifiedResourceSystemSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for UnifiedResourceSystemSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/aio/backend/src/server/migrations/0012_unified_resource_system.sql"
            ),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
