//! DDL statement generator supporting multiple database dialects.
//!
//! Provides a type-safe API for generating `CREATE TABLE`, `ALTER TABLE`,
//! `CREATE INDEX`, and other DDL statements across different SQL dialects
//! (MySQL, PostgreSQL, SQLite).
//!
//! # Quick Start
//!
//! ```no_run
//! use addzero_ddl_generator::{DdlGenerator, Table, Column, ColumnType, Dialect};
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

use thiserror::Error;

mod column;
mod dialect;
mod generator;
mod table;

pub use column::{Column, ColumnType};
pub use dialect::Dialect;
pub use generator::DdlGenerator;
pub use table::Table;

/// Errors that can occur during DDL generation.
#[derive(Debug, Error, PartialEq)]
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
