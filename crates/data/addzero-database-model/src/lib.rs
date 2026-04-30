//! Database model and schema definitions for code generation.
//!
//! Provides data structures to represent database schemas, tables, columns,
//! relationships, and indexes in a dialect-agnostic way.
//!
//! # Quick Start
//!
//! ```no_run
//! use addzero_database_model::{Schema, Table, Column, DataType, Relation, RelationKind};
//!
//! let users = Table::new("users")
//!     .column(Column::new("id", DataType::BigInt).primary_key().auto_increment())
//!     .column(Column::new("name", DataType::Varchar(255)).not_null())
//!     .column(Column::new("email", DataType::Varchar(255)).unique());
//!
//! let schema = Schema::new("myapp").table(users);
//!
//! assert_eq!(schema.tables.len(), 1);
//! assert_eq!(schema.tables[0].name, "users");
//! ```

use thiserror::Error;

mod column;
mod index;
mod relation;
mod schema;
mod table;

pub use column::{Column, DataType};
pub use index::Index;
pub use relation::{Relation, RelationKind};
pub use schema::Schema;
pub use table::Table;

/// Errors that can occur during schema validation.
#[derive(Debug, Error, PartialEq)]
pub enum ModelError {
    /// The schema name is empty.
    #[error("empty schema name")]
    EmptySchemaName,

    /// A table name is empty.
    #[error("empty table name in schema '{schema}'")]
    EmptyTableName { schema: String },

    /// A column name is empty.
    #[error("empty column name in table '{table}'")]
    EmptyColumnName { table: String },

    /// Duplicate table name.
    #[error("duplicate table name: '{0}'")]
    DuplicateTable(String),

    /// Duplicate column name within a table.
    #[error("duplicate column '{column}' in table '{table}'")]
    DuplicateColumn { table: String, column: String },

    /// A relation references a non-existent table.
    #[error("relation references unknown table '{0}'")]
    UnknownTable(String),

    /// A relation references a non-existent column.
    #[error("relation references unknown column '{column}' in table '{table}'")]
    UnknownColumn { table: String, column: String },

    /// An index references a non-existent column.
    #[error("index '{index}' references unknown column '{column}' in table '{table}'")]
    UnknownIndexColumn {
        index: String,
        table: String,
        column: String,
    },
}
