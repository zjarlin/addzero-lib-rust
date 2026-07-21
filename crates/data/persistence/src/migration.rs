use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::prelude::*;

use crate::sql_split::split_sql_statements;

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
    include_str!("../../software-catalog/migrations/0001_init.sql");
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
            include_str!("../../assets/migrations/0001_init.sql"),
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
            include_str!("../../knowledge/migrations/0001_init.sql"),
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
            include_str!("../../skills/migrations/0001_init.sql"),
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
