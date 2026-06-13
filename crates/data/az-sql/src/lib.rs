//! 带类型安全 SQL 构建的 SQL 查询构建器。
//!
//! 提供流式 API，用于构建 SELECT、INSERT、UPDATE 和 DELETE 查询，
//! 并使用参数化值来防止 SQL 注入。
//!
//! # 快速开始
//!
//! ```
//! use az_sql::{Query, QueryError, SelectQuery};
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
//! assert!(sql.contains(r#"SELECT "id", "name", "email""#));
//! assert!(sql.contains(r#"FROM "users""#));
//! # let _ = params;
//! # Ok(())
//! # }
//! ```

use az_derive_aliases::{apply, error_eq, plain_code_display_no_default_enum};

automod::dir!("src");

pub use delete::DeleteQuery;
pub use insert::InsertQuery;
pub use select::SelectQuery;
pub use update::UpdateQuery;

/// Errors that can occur during query building.
#[apply(error_eq)]
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
#[apply(plain_code_display_no_default_enum)]
pub enum SortOrder {
    /// Ascending order.
    #[display("ASC")]
    Asc,
    /// Descending order.
    #[display("DESC")]
    Desc,
}

/// Represents a SQL join type.
#[apply(plain_code_display_no_default_enum)]
pub enum JoinType {
    /// INNER JOIN.
    #[display("INNER JOIN")]
    Inner,
    /// LEFT JOIN.
    #[display("LEFT JOIN")]
    Left,
    /// RIGHT JOIN.
    #[display("RIGHT JOIN")]
    Right,
    /// FULL OUTER JOIN.
    #[display("FULL OUTER JOIN")]
    FullOuter,
    /// CROSS JOIN.
    #[display("CROSS JOIN")]
    Cross,
}

/// Quote a SQL identifier using ANSI SQL double-quote convention.
///
/// Escapes embedded double quotes by doubling them (`"` → `""`).
/// This prevents SQL injection through identifier positions (table names,
/// column names, etc.).
pub fn quote_identifier(identifier: &str) -> String {
    let escaped = identifier.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}
