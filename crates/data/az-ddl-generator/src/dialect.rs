use az_derive_aliases::{apply, serde_eq_copy};

/// Supported SQL database dialects for DDL generation.
#[apply(serde_eq_copy)]
pub enum Dialect {
    /// MySQL / MariaDB.
    MySQL,
    /// PostgreSQL.
    PostgreSQL,
    /// SQLite.
    SQLite,
}
