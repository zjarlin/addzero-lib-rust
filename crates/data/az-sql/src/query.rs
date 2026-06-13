use anyhow::Result;
use az_derive_aliases::{apply, plain_code_display_no_default_enum};

/// Trait for types that can build a parameterized SQL query string.
pub trait Query {
    /// Build the SQL string and return `(sql_string, params)`.
    fn build(&self) -> Result<(String, Vec<String>)>;

    /// Build just the SQL string, ignoring params.
    fn to_sql(&self) -> Result<String> {
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
