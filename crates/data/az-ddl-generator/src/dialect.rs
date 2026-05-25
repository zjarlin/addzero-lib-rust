use az_derive_aliases::{apply, serde_code_display_enum};

/// Supported SQL database dialects for DDL generation.
#[apply(serde_code_display_enum)]
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
