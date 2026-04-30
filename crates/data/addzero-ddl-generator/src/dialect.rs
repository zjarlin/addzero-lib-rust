use serde::{Deserialize, Serialize};

/// Supported SQL database dialects for DDL generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dialect {
    /// MySQL / MariaDB.
    MySQL,
    /// PostgreSQL.
    PostgreSQL,
    /// SQLite.
    SQLite,
}
