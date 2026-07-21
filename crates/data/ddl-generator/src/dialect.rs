
/// Supported SQL database dialects for DDL generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Dialect {
    /// MySQL / MariaDB.
    #[display("MySQL")]
    #[serde(rename = "mysql")]
    #[strum(serialize = "mysql")]
    MySQL,
    /// PostgreSQL.
    #[display("PostgreSQL")]
    #[serde(rename = "postgresql")]
    #[strum(serialize = "postgresql")]
    PostgreSQL,
    /// SQLite.
    #[display("SQLite")]
    #[serde(rename = "sqlite")]
    #[strum(serialize = "sqlite")]
    SQLite,
}

impl Dialect {
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

#[cfg(test)]
mod tests {
    use super::Dialect;

    #[test]
    fn dialect_code_and_display_are_separate() {
        assert_eq!(Dialect::PostgreSQL.code(), "postgresql");
        assert_eq!(Dialect::PostgreSQL.to_string(), "PostgreSQL");
        assert_eq!(
            serde_json::to_string(&Dialect::MySQL).expect("serialize"),
            "\"mysql\""
        );
    }
}
