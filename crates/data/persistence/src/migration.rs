/// 内嵌到持久化 crate 的幂等 SQL 迁移。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddedSqlMigration {
    /// 用于错误定位和人工执行的稳定文件名。
    pub name: &'static str,
    /// 需要执行的 SQL 内容。
    pub sql: &'static str,
}

pub const CLI_MARKET_SCHEMA_SQL: &str = include_str!("../migrations/0002_clianything_market.sql");
pub const ADMIN_ASSET_GRAPH_SCHEMA_SQL: &str =
    include_str!("../migrations/0003_admin_asset_graph.sql");
pub const ADMIN_SOFTWARE_CATALOG_SCHEMA_SQL: &str =
    include_str!("../../software-catalog/migrations/0001_init.sql");
pub const ADMIN_KNOWLEDGE_GRAPH_SCHEMA_SQL: &str =
    include_str!("../migrations/0005_admin_knowledge_graph.sql");
pub const ADMIN_BRANDING_SETTINGS_SCHEMA_SQL: &str =
    include_str!("../migrations/0006_admin_branding_settings.sql");
pub const SYSTEM_MANAGEMENT_SCHEMA_SQL: &str =
    include_str!("../migrations/0007_system_management.sql");
pub const DEPARTMENTS_DICTIONARIES_SCHEMA_SQL: &str =
    include_str!("../migrations/0008_departments_dictionaries.sql");
pub const REMOVE_AGENT_RUNTIME_SCHEMA_SQL: &str =
    include_str!("../migrations/0009_remove_agent_runtime.sql");
pub const DOWNLOAD_STATION_SCHEMA_SQL: &str =
    include_str!("../migrations/0010_download_station.sql");
pub const ADMIN_MENU_SYSTEM_SCHEMA_SQL: &str =
    include_str!("../migrations/0011_admin_menu_system.sql");
pub const UNIFIED_RESOURCE_SYSTEM_SCHEMA_SQL: &str =
    include_str!("../migrations/0012_unified_resource_system.sql");
pub const API_KEYS_SCHEMA_SQL: &str = include_str!("../migrations/0013_api_keys.sql");
pub const ASSET_SCHEMA_SQL: &str = include_str!("../../assets/migrations/0001_init.sql");
pub const KNOWLEDGE_SCHEMA_SQL: &str = include_str!("../../knowledge/migrations/0001_init.sql");
pub const SKILL_SCHEMA_SQL: &str = include_str!("../../skills/migrations/0001_init.sql");

pub const WORKSPACE_SQL_MIGRATIONS: &[EmbeddedSqlMigration] = &[
    EmbeddedSqlMigration { name: "0002_clianything_market.sql", sql: CLI_MARKET_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0003_admin_asset_graph.sql", sql: ADMIN_ASSET_GRAPH_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0004_admin_software_catalog.sql", sql: ADMIN_SOFTWARE_CATALOG_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0005_admin_knowledge_graph.sql", sql: ADMIN_KNOWLEDGE_GRAPH_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0006_admin_branding_settings.sql", sql: ADMIN_BRANDING_SETTINGS_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0007_system_management.sql", sql: SYSTEM_MANAGEMENT_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0008_departments_dictionaries.sql", sql: DEPARTMENTS_DICTIONARIES_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0001_assets.sql", sql: ASSET_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0001_knowledge.sql", sql: KNOWLEDGE_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0001_skills.sql", sql: SKILL_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0009_remove_agent_runtime.sql", sql: REMOVE_AGENT_RUNTIME_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0010_download_station.sql", sql: DOWNLOAD_STATION_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0011_admin_menu_system.sql", sql: ADMIN_MENU_SYSTEM_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0012_unified_resource_system.sql", sql: UNIFIED_RESOURCE_SYSTEM_SCHEMA_SQL },
    EmbeddedSqlMigration { name: "0013_api_keys.sql", sql: API_KEYS_SCHEMA_SQL },
];

/// 返回按执行顺序排列的 workspace SQL 迁移。
pub fn workspace_sql_migrations() -> &'static [EmbeddedSqlMigration] {
    WORKSPACE_SQL_MIGRATIONS
}
