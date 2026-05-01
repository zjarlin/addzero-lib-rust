use std::{collections::BTreeMap, env, fs, path::PathBuf, time::Duration};

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr};
use sea_orm_migration::prelude::*;
use thiserror::Error;

const LOCAL_ENV_FILE: &str = ".config/msc-aio/msc-aio.env";

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

        WorkspaceMigrator::up(&db, None)
            .await
            .map_err(PersistenceError::Migrate)?;

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
    #[error("missing MSC_AIO_DATABASE_URL / DATABASE_URL / local env file")]
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
        .or_else(|| {
            env::var("DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(read_database_url_from_file)
}

pub fn local_env_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(LOCAL_ENV_FILE))
}

fn read_database_url_from_file() -> Option<String> {
    let path = local_env_path()?;
    let content = fs::read_to_string(path).ok()?;
    let vars = parse_env_pairs(&content);
    vars.get("MSC_AIO_DATABASE_URL")
        .or_else(|| vars.get("DATABASE_URL"))
        .cloned()
        .filter(|value| !value.trim().is_empty())
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

pub struct WorkspaceMigrator;

#[async_trait::async_trait]
impl MigratorTrait for WorkspaceMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(AgentRuntimeSchemaMigration),
            Box::new(CliMarketSchemaMigration),
            Box::new(AdminAssetGraphSchemaMigration),
            Box::new(AdminSoftwareCatalogSchemaMigration),
            Box::new(AssetSchemaMigration),
            Box::new(KnowledgeSchemaMigration),
            Box::new(SkillSchemaMigration),
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
struct AgentRuntimeSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for AgentRuntimeSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../../../apps/msc-aio/src/server/migrations/0001_agent_runtime.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveMigrationName)]
struct CliMarketSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for CliMarketSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!(
                "../../../../apps/msc-aio/src/server/migrations/0002_clianything_market.sql"
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
                "../../../../apps/msc-aio/src/server/migrations/0003_admin_asset_graph.sql"
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
