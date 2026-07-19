use anyhow::Result;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum SortOrder {
    /// Ascending order.
    #[display("ASC")]
    Asc,
    /// Descending order.
    #[display("DESC")]
    Desc,
}

impl SortOrder {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// Represents a SQL join type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
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

impl JoinType {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}
