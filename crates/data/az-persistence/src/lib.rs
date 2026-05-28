//! Workspace 级 PostgreSQL 持久化基础设施。
//!
//! 本 crate 提供统一的数据库连接建立、URL 发现和 schema 迁移管理，
//! 是所有依赖 SeaORM 的 crate 的共同入口。
//!
//! ## 核心能力
//!
//! - **数据库 URL 发现**：按优先级从 `MSC_AIO_DATABASE_URL`、`DATABASE_URL` 环境变量
//!   或 `~/.config/aio/aio.env` 配置文件中读取连接地址。
//! - **连接管理**：通过 [`PersistenceContext`] 封装 SeaORM `DatabaseConnection`，
//!   自动配置连接池参数（最大/最小连接数、超时时间）。
//! - **Workspace 迁移**：[`WorkspaceMigrator`] 在首次连接时自动执行全部 workspace 级
//!   SQL 迁移，使用 `AtomicBool` + tokio `Mutex` 保证并发安全。
//! - **错误处理**：[`PersistenceError`] 统一封装连接、迁移、ping 等阶段的错误。
//!
//! ## 关键类型
//!
//! - [`PersistenceContext`] — 持有数据库连接和 URL 的上下文，提供 `connect()` / `connect_with_url()`。
//! - [`WorkspaceMigrator`] — 实现 `MigratorTrait`，聚合所有 workspace 级迁移。
//! - [`PersistenceError`] — 统一错误枚举。

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use az_derive_aliases::{apply, error, plain_clone};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr};
use sea_orm_migration::prelude::*;

const WORKSPACE_ENV_FILE: &str = ".env";
const LOCAL_ENV_FILE: &str = ".config/aio/aio.env";
static WORKSPACE_MIGRATIONS_DONE: AtomicBool = AtomicBool::new(false);
static WORKSPACE_MIGRATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A SQL migration embedded in the persistence crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddedSqlMigration {
    /// Stable file-like migration name used for logs and manual runners.
    pub name: &'static str,
    /// SQL body to execute.
    pub sql: &'static str,
}

/// CLI market schema used by the workspace migration runner and AIO services.
pub const CLI_MARKET_SCHEMA_SQL: &str = include_str!("../migrations/0002_clianything_market.sql");
/// Legacy admin asset graph schema used by AIO services.
pub const ADMIN_ASSET_GRAPH_SCHEMA_SQL: &str =
    include_str!("../migrations/0003_admin_asset_graph.sql");
/// Software catalog schema shared with the software catalog crate.
pub const ADMIN_SOFTWARE_CATALOG_SCHEMA_SQL: &str =
    include_str!("../../az-software-catalog/migrations/0001_init.sql");
/// Legacy admin knowledge graph schema used by AIO services.
pub const ADMIN_KNOWLEDGE_GRAPH_SCHEMA_SQL: &str =
    include_str!("../migrations/0005_admin_knowledge_graph.sql");
/// Branding settings schema used by AIO services.
pub const ADMIN_BRANDING_SETTINGS_SCHEMA_SQL: &str =
    include_str!("../migrations/0006_admin_branding_settings.sql");
/// System management schema used by AIO admin modules.
pub const SYSTEM_MANAGEMENT_SCHEMA_SQL: &str =
    include_str!("../migrations/0007_system_management.sql");
/// Department and dictionary schema used by AIO admin modules.
pub const DEPARTMENTS_DICTIONARIES_SCHEMA_SQL: &str =
    include_str!("../migrations/0008_departments_dictionaries.sql");
/// Agent runtime cleanup migration.
pub const REMOVE_AGENT_RUNTIME_SCHEMA_SQL: &str =
    include_str!("../migrations/0009_remove_agent_runtime.sql");
/// Download Station schema used by AIO admin modules.
pub const DOWNLOAD_STATION_SCHEMA_SQL: &str =
    include_str!("../migrations/0010_download_station.sql");
/// Admin menu schema used by AIO admin modules.
pub const ADMIN_MENU_SYSTEM_SCHEMA_SQL: &str =
    include_str!("../migrations/0011_admin_menu_system.sql");
/// Unified resource schema used by AIO admin modules.
pub const UNIFIED_RESOURCE_SYSTEM_SCHEMA_SQL: &str =
    include_str!("../migrations/0012_unified_resource_system.sql");
/// API key schema used by AIO auth modules.
pub const API_KEYS_SCHEMA_SQL: &str = include_str!("../migrations/0013_api_keys.sql");

/// SQL migrations available to manual workspace migration commands.
pub const WORKSPACE_SQL_MIGRATIONS: &[EmbeddedSqlMigration] = &[
    EmbeddedSqlMigration {
        name: "0002_clianything_market.sql",
        sql: CLI_MARKET_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0003_admin_asset_graph.sql",
        sql: ADMIN_ASSET_GRAPH_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0004_admin_software_catalog.sql",
        sql: ADMIN_SOFTWARE_CATALOG_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0005_admin_knowledge_graph.sql",
        sql: ADMIN_KNOWLEDGE_GRAPH_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0006_admin_branding_settings.sql",
        sql: ADMIN_BRANDING_SETTINGS_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0007_system_management.sql",
        sql: SYSTEM_MANAGEMENT_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0008_departments_dictionaries.sql",
        sql: DEPARTMENTS_DICTIONARIES_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0009_remove_agent_runtime.sql",
        sql: REMOVE_AGENT_RUNTIME_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0010_download_station.sql",
        sql: DOWNLOAD_STATION_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0011_admin_menu_system.sql",
        sql: ADMIN_MENU_SYSTEM_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0012_unified_resource_system.sql",
        sql: UNIFIED_RESOURCE_SYSTEM_SCHEMA_SQL,
    },
    EmbeddedSqlMigration {
        name: "0013_api_keys.sql",
        sql: API_KEYS_SCHEMA_SQL,
    },
];

/// Returns the embedded SQL migrations in execution order.
pub fn workspace_sql_migrations() -> &'static [EmbeddedSqlMigration] {
    WORKSPACE_SQL_MIGRATIONS
}

/// Shared database context for workspace services.
#[apply(plain_clone)]
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
                cleanup_invalid_migration_records(&db)
                    .await
                    .map_err(PersistenceError::Migrate)?;
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

#[apply(error)]
pub enum PersistenceError {
    #[error("missing MSC_AIO_DATABASE_URL / DATABASE_URL / ~/.config/aio/aio.env")]
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

async fn cleanup_invalid_migration_records(db: &DatabaseConnection) -> Result<(), DbErr> {
    WorkspaceMigrator::install(db).await?;
    db.execute_unprepared("DELETE FROM seaql_migrations WHERE version = 'lib'")
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        WorkspaceMigrator, read_database_url_from_path, split_sql_statements,
        workspace_env_path_from,
    };
    use sea_orm_migration::prelude::MigratorTrait;
    use std::{
        collections::HashSet,
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

    #[test]
    fn workspace_migration_names_are_stable_and_unique() {
        let names = WorkspaceMigrator::migrations()
            .into_iter()
            .map(|migration| migration.name().to_string())
            .collect::<Vec<_>>();
        let unique = names.iter().collect::<HashSet<_>>();

        assert_eq!(
            unique.len(),
            names.len(),
            "each migration needs its own seaql_migrations version"
        );
        assert!(
            !names.iter().any(|name| name == "lib"),
            "file-level derived migration names collapse all inline migrations to `lib`"
        );
        assert!(names.contains(&"0002_clianything_market".to_string()));
        assert!(names.contains(&"0012_unified_resource_system".to_string()));
    }

    #[test]
    fn split_sql_statements_keeps_dollar_quoted_blocks_together() {
        let sql = r#"
            DO $$
            BEGIN
                IF TRUE THEN
                    RAISE NOTICE 'contains; semicolon';
                END IF;
            END $$;
            CREATE TABLE IF NOT EXISTS demo (id INTEGER);
        "#;

        let statements = split_sql_statements(sql);

        assert_eq!(
            statements.len(),
            2,
            "the DO block must remain one executable statement"
        );
        assert!(statements[0].contains("RAISE NOTICE"));
        assert!(statements[1].starts_with("CREATE TABLE"));
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("az-persistence-{prefix}-{unique}"))
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
            Box::new(AdminKnowledgeGraphSchemaMigration),
            Box::new(AdminBrandingSettingsSchemaMigration),
            Box::new(SystemManagementSchemaMigration),
            Box::new(DepartmentsDictionariesSchemaMigration),
            Box::new(AssetSchemaMigration),
            Box::new(KnowledgeSchemaMigration),
            Box::new(SkillSchemaMigration),
            Box::new(RemoveAgentRuntimeSchemaMigration),
            Box::new(DownloadStationSchemaMigration),
            Box::new(AdminMenuSystemSchemaMigration),
            Box::new(UnifiedResourceSystemSchemaMigration),
            Box::new(ApiKeysSchemaMigration),
        ]
    }
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    for statement in split_sql_statements(sql) {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        manager.get_connection().execute_unprepared(trimmed).await?;
    }
    Ok(())
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let bytes = sql.as_bytes();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut dollar_tag: Option<String> = None;

    while index < bytes.len() {
        let ch = bytes[index] as char;

        if let Some(tag) = dollar_tag.as_deref() {
            if ch == '$' && sql[index..].starts_with(tag) {
                index += tag.len();
                dollar_tag = None;
            } else {
                index += 1;
            }
            continue;
        }

        if in_single_quote {
            if ch == '\'' {
                if bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                    continue;
                }
                in_single_quote = false;
            }
            index += 1;
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                in_double_quote = false;
            }
            index += 1;
            continue;
        }

        match ch {
            '\'' => {
                in_single_quote = true;
                index += 1;
            }
            '"' => {
                in_double_quote = true;
                index += 1;
            }
            '$' => {
                if let Some(tag) = read_dollar_tag(&sql[index..]) {
                    index += tag.len();
                    dollar_tag = Some(tag);
                } else {
                    index += 1;
                }
            }
            ';' => {
                let statement = sql[start..index].trim();
                if !statement.is_empty() {
                    statements.push(statement.to_string());
                }
                index += 1;
                start = index;
            }
            _ => {
                index += 1;
            }
        }
    }

    let statement = sql[start..].trim();
    if !statement.is_empty() {
        statements.push(statement.to_string());
    }

    statements
}

fn read_dollar_tag(input: &str) -> Option<String> {
    let rest = input.strip_prefix('$')?;
    let end = rest.find('$')?;
    let tag = &rest[..end];
    if tag
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        Some(format!("${tag}$"))
    } else {
        None
    }
}

macro_rules! migration_name {
    ($migration:ty, $name:literal) => {
        impl MigrationName for $migration {
            fn name(&self) -> &str {
                $name
            }
        }
    };
}

struct CliMarketSchemaMigration;
migration_name!(CliMarketSchemaMigration, "0002_clianything_market");

#[async_trait::async_trait]
impl MigrationTrait for CliMarketSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, CLI_MARKET_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct AdminAssetGraphSchemaMigration;
migration_name!(AdminAssetGraphSchemaMigration, "0003_admin_asset_graph");

#[async_trait::async_trait]
impl MigrationTrait for AdminAssetGraphSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, ADMIN_ASSET_GRAPH_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct AdminSoftwareCatalogSchemaMigration;
migration_name!(
    AdminSoftwareCatalogSchemaMigration,
    "0004_admin_software_catalog"
);

#[async_trait::async_trait]
impl MigrationTrait for AdminSoftwareCatalogSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, ADMIN_SOFTWARE_CATALOG_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct AdminKnowledgeGraphSchemaMigration;
migration_name!(
    AdminKnowledgeGraphSchemaMigration,
    "0005_admin_knowledge_graph"
);

#[async_trait::async_trait]
impl MigrationTrait for AdminKnowledgeGraphSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, ADMIN_KNOWLEDGE_GRAPH_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct AdminBrandingSettingsSchemaMigration;
migration_name!(
    AdminBrandingSettingsSchemaMigration,
    "0006_admin_branding_settings"
);

#[async_trait::async_trait]
impl MigrationTrait for AdminBrandingSettingsSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, ADMIN_BRANDING_SETTINGS_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct SystemManagementSchemaMigration;
migration_name!(SystemManagementSchemaMigration, "0007_system_management");

#[async_trait::async_trait]
impl MigrationTrait for SystemManagementSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, SYSTEM_MANAGEMENT_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct DepartmentsDictionariesSchemaMigration;
migration_name!(
    DepartmentsDictionariesSchemaMigration,
    "0008_departments_dictionaries"
);

#[async_trait::async_trait]
impl MigrationTrait for DepartmentsDictionariesSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, DEPARTMENTS_DICTIONARIES_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct AssetSchemaMigration;
migration_name!(AssetSchemaMigration, "0001_az_assets_init");

#[async_trait::async_trait]
impl MigrationTrait for AssetSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../az-assets/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct KnowledgeSchemaMigration;
migration_name!(KnowledgeSchemaMigration, "0001_az_knowledge_init");

#[async_trait::async_trait]
impl MigrationTrait for KnowledgeSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../az-knowledge/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct SkillSchemaMigration;
migration_name!(SkillSchemaMigration, "0001_az_skills_init");

#[async_trait::async_trait]
impl MigrationTrait for SkillSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(
            manager,
            include_str!("../../az-skills/migrations/0001_init.sql"),
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct RemoveAgentRuntimeSchemaMigration;
migration_name!(
    RemoveAgentRuntimeSchemaMigration,
    "0009_remove_agent_runtime"
);

#[async_trait::async_trait]
impl MigrationTrait for RemoveAgentRuntimeSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, REMOVE_AGENT_RUNTIME_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct DownloadStationSchemaMigration;
migration_name!(DownloadStationSchemaMigration, "0010_download_station");

#[async_trait::async_trait]
impl MigrationTrait for DownloadStationSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, DOWNLOAD_STATION_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct AdminMenuSystemSchemaMigration;
migration_name!(AdminMenuSystemSchemaMigration, "0011_admin_menu_system");

#[async_trait::async_trait]
impl MigrationTrait for AdminMenuSystemSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, ADMIN_MENU_SYSTEM_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct UnifiedResourceSystemSchemaMigration;
migration_name!(
    UnifiedResourceSystemSchemaMigration,
    "0012_unified_resource_system"
);

#[async_trait::async_trait]
impl MigrationTrait for UnifiedResourceSystemSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, UNIFIED_RESOURCE_SYSTEM_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct ApiKeysSchemaMigration;
migration_name!(ApiKeysSchemaMigration, "0013_api_keys");

#[async_trait::async_trait]
impl MigrationTrait for ApiKeysSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_sql(manager, API_KEYS_SCHEMA_SQL).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
