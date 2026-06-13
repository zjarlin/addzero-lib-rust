//! 支持多种数据库方言的 DDL 语句生成器。
//!
//! 提供类型安全的 API，用于在不同 SQL 方言（MySQL、PostgreSQL、SQLite）
//! 之间生成 `CREATE TABLE`、`ALTER TABLE`、`CREATE INDEX` 及其他 DDL 语句。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_ddl_generator::{DdlGenerator, Table, Column, ColumnType, Dialect};
//!
//! let table = Table::new("users")
//!     .column(Column::new("id", ColumnType::BigInt).primary_key().not_null())
//!     .column(Column::new("name", ColumnType::Varchar(255)).not_null())
//!     .column(Column::new("email", ColumnType::Varchar(255)).unique());
//!
//! let ddl = DdlGenerator::new(Dialect::PostgreSQL).generate_create_table(&table).unwrap();
//! assert!(ddl.contains("CREATE TABLE"));
//! assert!(ddl.contains("users"));
//! ```

use az_derive_aliases::{apply, error_eq};

automod::dir!("src");

pub use column::{Column, ColumnType};
pub use dialect::Dialect;
pub use generator::{DdlGenerator, quote_identifier};
pub use table::Table;

/// Errors that can occur during DDL generation.
#[apply(error_eq)]
pub enum DdlError {
    /// The table name is empty or invalid.
    #[error("invalid table name: {0}")]
    InvalidTableName(String),

    /// The table has no columns defined.
    #[error("table '{0}' has no columns")]
    EmptyTable(String),

    /// Duplicate column name detected.
    #[error("duplicate column name: '{0}'")]
    DuplicateColumn(String),
}
