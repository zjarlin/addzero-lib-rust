//! Catalog module for schema management.
//!
//! The catalog stores table schemas in the `_schemas` directory of the repository,
//! providing schema validation and migration capabilities.

automod::dir!("src/catalog");

pub use manager::Catalog;
pub use schema::{SchemaBuilder, SchemaError, SchemaVersion, TableSchema};
pub use types::{ColumnDef, Constraint, DataType};
