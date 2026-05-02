//! SQL query builder with type-safe SQL construction.
//!
//! Provides a fluent API for building SELECT, INSERT, UPDATE, and DELETE
//! queries with parameterized values to prevent SQL injection.
//!
//! # Quick Start
//!
//! ```
//! use addzero_sql::{Query, QueryError, SelectQuery};
//!
//! fn main() -> Result<(), QueryError> {
//!
//! let query = SelectQuery::new()
//!     .select(&["id", "name", "email"])
//!     .from("users")
//!     .r#where("active = ?", vec!["true"])
//!     .order_by("name", true)
//!     .limit(10);
//!
//! let (sql, params) = query.build()?;
//! assert!(sql.contains("SELECT id, name, email"));
//! assert!(sql.contains("FROM users"));
//! # let _ = params;
//! # Ok(())
//! # }
//! ```

use thiserror::Error;

mod delete;
mod insert;
mod select;
mod update;

pub use delete::DeleteQuery;
pub use insert::InsertQuery;
pub use select::SelectQuery;
pub use update::UpdateQuery;

/// Errors that can occur during query building.
#[derive(Debug, Error, PartialEq)]
pub enum QueryError {
    /// No table specified for the query.
    #[error("no table specified")]
    NoTable,

    /// No columns or values specified for INSERT.
    #[error("no columns specified for insert")]
    NoColumns,

    /// No SET clauses specified for UPDATE.
    #[error("no set clauses specified for update")]
    NoSetClauses,

    /// Mismatched column/value count in INSERT.
    #[error("column count ({columns}) does not match value count ({values})")]
    ColumnValueMismatch { columns: usize, values: usize },
}

pub(crate) fn require_table_name(table: Option<&str>) -> Result<&str, QueryError> {
    match table {
        Some(table) if !table.trim().is_empty() => Ok(table),
        _ => Err(QueryError::NoTable),
    }
}

/// Trait for types that can build a parameterized SQL query string.
pub trait Query {
    /// Build the SQL string and return `(sql_string, params)`.
    fn build(&self) -> Result<(String, Vec<String>), QueryError>;

    /// Build just the SQL string, ignoring params.
    fn to_sql(&self) -> Result<String, QueryError> {
        self.build().map(|(sql, _)| sql)
    }
}

/// Represents a SQL ORDER BY clause direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Represents a SQL join type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    /// INNER JOIN.
    Inner,
    /// LEFT JOIN.
    Left,
    /// RIGHT JOIN.
    Right,
    /// FULL OUTER JOIN.
    FullOuter,
    /// CROSS JOIN.
    Cross,
}
