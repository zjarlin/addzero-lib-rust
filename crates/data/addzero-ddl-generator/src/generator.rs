use crate::DdlError;
use crate::column::{Column, ColumnType};
use crate::dialect::Dialect;
use crate::table::Table;

/// DDL statement generator.
///
/// Generates SQL DDL statements (CREATE TABLE, ALTER TABLE, CREATE INDEX, etc.)
/// tailored to the target database dialect.
pub struct DdlGenerator {
    dialect: Dialect,
}

impl DdlGenerator {
    /// Create a new DDL generator for the given dialect.
    pub fn new(dialect: Dialect) -> Self {
        Self { dialect }
    }

    /// Generate a CREATE TABLE statement from a [`Table`] definition.
    pub fn generate_create_table(&self, table: &Table) -> Result<String, DdlError> {
        if table.name.is_empty() {
            return Err(DdlError::InvalidTableName("(empty)".to_string()));
        }

        if table.columns.is_empty() {
            return Err(DdlError::EmptyTable(table.name.clone()));
        }

        // Check for duplicate column names
        let mut seen = std::collections::HashSet::new();
        for col in &table.columns {
            if !seen.insert(&col.name) {
                return Err(DdlError::DuplicateColumn(col.name.clone()));
            }
        }

        let mut sql = String::new();
        sql.push_str(&format!("CREATE TABLE {} (\n", table.name));

        let column_defs: Vec<String> = table
            .columns
            .iter()
            .map(|col| self.format_column_def(col))
            .collect();

        sql.push_str(&column_defs.join(",\n"));
        sql.push_str("\n)");

        // Add dialect-specific extras
        match self.dialect {
            Dialect::MySQL => {
                if let Some(ref comment) = table.comment {
                    sql.push_str(&format!(" COMMENT='{}'", comment));
                }
            }
            Dialect::PostgreSQL | Dialect::SQLite => {
                // PostgreSQL uses separate COMMENT ON statement
            }
        }

        sql.push(';');
        Ok(sql)
    }

    /// Generate an ALTER TABLE ADD COLUMN statement.
    pub fn generate_add_column(
        &self,
        table_name: &str,
        column: &Column,
    ) -> Result<String, DdlError> {
        if table_name.is_empty() {
            return Err(DdlError::InvalidTableName("(empty)".to_string()));
        }

        let col_def = self.format_column_def(column);
        Ok(format!(
            "ALTER TABLE {} ADD COLUMN {};",
            table_name, col_def
        ))
    }

    /// Generate a CREATE INDEX statement.
    pub fn generate_create_index(
        &self,
        index_name: &str,
        table_name: &str,
        columns: &[&str],
        unique: bool,
    ) -> Result<String, DdlError> {
        if index_name.is_empty() || table_name.is_empty() {
            return Err(DdlError::InvalidTableName(
                if index_name.is_empty() {
                    "(empty index name)"
                } else {
                    "(empty table name)"
                }
                .to_string(),
            ));
        }

        let unique_kw = if unique { "UNIQUE " } else { "" };
        Ok(format!(
            "CREATE {unique_kw}INDEX {index_name} ON {table_name} ({});",
            columns.join(", ")
        ))
    }

    /// Generate a statement to remove a table.
    pub fn generate_drop_table(
        &self,
        table_name: &str,
        if_exists: bool,
    ) -> Result<String, DdlError> {
        if table_name.is_empty() {
            return Err(DdlError::InvalidTableName("(empty)".to_string()));
        }

        let if_exists_kw = if if_exists { "IF EXISTS " } else { "" };
        Ok(format!("DROP TABLE {if_exists_kw}{table_name};"))
    }

    fn format_column_def(&self, col: &Column) -> String {
        let type_str = self.map_column_type(&col.column_type);
        let mut parts = vec![format!("  {} {}", col.name, type_str)];

        if col.primary_key {
            parts.push("PRIMARY KEY".to_string());
            if self.dialect == Dialect::SQLite {
                parts.push("AUTOINCREMENT".to_string());
            }
        }
        if col.not_null {
            parts.push("NOT NULL".to_string());
        }
        if col.unique {
            parts.push("UNIQUE".to_string());
        }
        if let Some(ref default) = col.default {
            parts.push(format!("DEFAULT {}", default));
        }

        parts.join(" ")
    }

    fn map_column_type(&self, col_type: &ColumnType) -> String {
        match (col_type, self.dialect) {
            (ColumnType::Boolean, Dialect::MySQL) => "TINYINT(1)".to_string(),
            (ColumnType::Boolean, _) => "BOOLEAN".to_string(),

            (ColumnType::Integer, Dialect::MySQL) => "INT".to_string(),
            (ColumnType::Integer, _) => "INTEGER".to_string(),

            (ColumnType::BigInt, _) => "BIGINT".to_string(),

            (ColumnType::Float, _) => "FLOAT".to_string(),
            (ColumnType::Double, Dialect::PostgreSQL) => "DOUBLE PRECISION".to_string(),
            (ColumnType::Double, _) => "DOUBLE".to_string(),

            (ColumnType::Char(n), _) => format!("CHAR({})", n),
            (ColumnType::Varchar(n), _) => format!("VARCHAR({})", n),
            (ColumnType::Text, _) => "TEXT".to_string(),
            (ColumnType::Blob, Dialect::PostgreSQL) => "BYTEA".to_string(),
            (ColumnType::Blob, _) => "BLOB".to_string(),

            (ColumnType::Date, _) => "DATE".to_string(),
            (ColumnType::DateTime, Dialect::PostgreSQL) => "TIMESTAMP".to_string(),
            (ColumnType::DateTime, _) => "DATETIME".to_string(),
            (ColumnType::Timestamp, Dialect::PostgreSQL) => "TIMESTAMPTZ".to_string(),
            (ColumnType::Timestamp, _) => "TIMESTAMP".to_string(),

            (ColumnType::Json, Dialect::SQLite) => "TEXT".to_string(),
            (ColumnType::Json, Dialect::PostgreSQL) => "JSONB".to_string(),
            (ColumnType::Json, Dialect::MySQL) => "JSON".to_string(),

            (ColumnType::Uuid, Dialect::PostgreSQL) => "UUID".to_string(),
            (ColumnType::Uuid, Dialect::MySQL) => "CHAR(36)".to_string(),
            (ColumnType::Uuid, Dialect::SQLite) => "TEXT".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> Table {
        Table::new("users")
            .column(
                Column::new("id", ColumnType::BigInt)
                    .primary_key()
                    .not_null(),
            )
            .column(Column::new("name", ColumnType::Varchar(255)).not_null())
            .column(Column::new("email", ColumnType::Varchar(255)).unique())
            .column(Column::new("active", ColumnType::Boolean).default("true"))
    }

    #[test]
    fn create_table_postgresql() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let sql = ddl.generate_create_table(&sample_table()).unwrap();

        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("BIGINT"));
        assert!(sql.contains("VARCHAR(255)"));
        assert!(sql.contains("BOOLEAN"));
        assert!(sql.contains("PRIMARY KEY"));
        assert!(sql.contains("NOT NULL"));
        assert!(sql.contains("UNIQUE"));
        assert!(sql.contains("DEFAULT true"));
    }

    #[test]
    fn create_table_mysql_boolean_mapping() {
        let ddl = DdlGenerator::new(Dialect::MySQL);
        let table = Table::new("flags").column(Column::new("enabled", ColumnType::Boolean));
        let sql = ddl.generate_create_table(&table).unwrap();

        assert!(sql.contains("TINYINT(1)"));
        assert!(!sql.contains("BOOLEAN"));
    }

    #[test]
    fn create_table_empty_name_errors() {
        let ddl = DdlGenerator::new(Dialect::SQLite);
        let table = Table::new("").column(Column::new("id", ColumnType::Integer));
        let result = ddl.generate_create_table(&table);
        assert_eq!(
            result,
            Err(DdlError::InvalidTableName("(empty)".to_string()))
        );
    }

    #[test]
    fn create_table_no_columns_errors() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let table = Table::new("empty_table");
        let result = ddl.generate_create_table(&table);
        assert_eq!(result, Err(DdlError::EmptyTable("empty_table".to_string())));
    }

    #[test]
    fn create_table_duplicate_column_errors() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let table = Table::new("dup")
            .column(Column::new("id", ColumnType::Integer))
            .column(Column::new("id", ColumnType::Text));
        let result = ddl.generate_create_table(&table);
        assert_eq!(result, Err(DdlError::DuplicateColumn("id".to_string())));
    }

    #[test]
    fn add_column_statement() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let col = Column::new("age", ColumnType::Integer)
            .not_null()
            .default("0");
        let sql = ddl.generate_add_column("users", &col).unwrap();

        assert!(sql.contains("ALTER TABLE users ADD COLUMN"));
        assert!(sql.contains("INTEGER"));
        assert!(sql.contains("NOT NULL"));
        assert!(sql.contains("DEFAULT 0"));
    }

    #[test]
    fn create_index_statement() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let sql = ddl
            .generate_create_index("idx_users_email", "users", &["email"], true)
            .unwrap();

        assert!(sql.contains("CREATE UNIQUE INDEX"));
        assert!(sql.contains("idx_users_email"));
        assert!(sql.contains("ON users"));
    }

    #[test]
    fn drop_table_if_exists() {
        let ddl = DdlGenerator::new(Dialect::SQLite);
        let sql = ddl.generate_drop_table("old_table", true).unwrap();
        assert_eq!(sql, "DROP TABLE IF EXISTS old_table;");
    }

    #[test]
    fn sqlite_autoincrement() {
        let ddl = DdlGenerator::new(Dialect::SQLite);
        let table = Table::new("items").column(
            Column::new("id", ColumnType::Integer)
                .primary_key()
                .not_null(),
        );
        let sql = ddl.generate_create_table(&table).unwrap();

        assert!(sql.contains("AUTOINCREMENT"));
    }

    #[test]
    fn postgresql_blob_is_bytea() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let table = Table::new("files").column(Column::new("data", ColumnType::Blob).not_null());
        let sql = ddl.generate_create_table(&table).unwrap();

        assert!(sql.contains("BYTEA"));
        assert!(!sql.contains("BLOB"));
    }

    #[test]
    fn mysql_comment_support() {
        let ddl = DdlGenerator::new(Dialect::MySQL);
        let table = Table::new("users")
            .column(Column::new("id", ColumnType::BigInt).primary_key())
            .comment("User accounts");
        let sql = ddl.generate_create_table(&table).unwrap();

        assert!(sql.contains("COMMENT='User accounts'"));
    }

    #[test]
    fn uuid_dialect_mapping() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let table = Table::new("t").column(Column::new("uuid_col", ColumnType::Uuid));
        let pg_sql = ddl.generate_create_table(&table).unwrap();
        assert!(pg_sql.contains("UUID"));

        let ddl = DdlGenerator::new(Dialect::MySQL);
        let mysql_sql = ddl.generate_create_table(&table).unwrap();
        assert!(mysql_sql.contains("CHAR(36)"));

        let ddl = DdlGenerator::new(Dialect::SQLite);
        let sqlite_sql = ddl.generate_create_table(&table).unwrap();
        assert!(sqlite_sql.contains("TEXT"));
    }

    #[test]
    fn json_dialect_mapping() {
        let ddl = DdlGenerator::new(Dialect::PostgreSQL);
        let t = Table::new("j").column(Column::new("data", ColumnType::Json));
        assert!(ddl.generate_create_table(&t).unwrap().contains("JSONB"));

        let ddl = DdlGenerator::new(Dialect::MySQL);
        assert!(ddl.generate_create_table(&t).unwrap().contains("JSON"));

        let ddl = DdlGenerator::new(Dialect::SQLite);
        assert!(ddl.generate_create_table(&t).unwrap().contains("TEXT"));
    }
}
