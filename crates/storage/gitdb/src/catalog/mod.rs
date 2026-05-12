//! Catalog module for schema management.
//!
//! The catalog stores table schemas in the `_schemas` directory of the repository,
//! providing schema validation and migration capabilities.

mod manager;
mod schema;
mod types;

pub use manager::Catalog;
pub use schema::{SchemaBuilder, SchemaError, SchemaVersion, TableSchema};
pub use types::{ColumnDef, Constraint, DataType};
