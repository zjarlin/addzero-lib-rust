use anyhow::{Result, bail};
use az_derive_aliases::{apply, plain_default_clone_debug};

use crate::identifier::{quote_identifier, require_table_name};
use crate::query::Query;

/// An INSERT query builder.
#[apply(plain_default_clone_debug)]
pub struct InsertQuery {
    table: Option<String>,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl InsertQuery {
    /// Create a new INSERT query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target table.
    pub fn into(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// Set the column names.
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a row of values.
    pub fn values(mut self, values: Vec<&str>) -> Self {
        self.rows
            .push(values.into_iter().map(String::from).collect());
        self
    }

    /// Build and validate the query, returning an error if invalid.
    pub fn try_build(&self) -> Result<(String, Vec<String>)> {
        require_table_name(self.table.as_deref())?;
        if self.columns.is_empty() {
            bail!("no columns specified for insert");
        }
        if self.rows.is_empty() {
            bail!("column count ({}) does not match value count (0)", self.columns.len());
        }
        let expected = self.columns.len();
        for row in &self.rows {
            if row.len() != expected {
                bail!(
                    "column count ({expected}) does not match value count ({})",
                    row.len()
                );
            }
        }
        self.build()
    }
}

impl Query for InsertQuery {
    fn build(&self) -> Result<(String, Vec<String>)> {
        let mut all_params: Vec<String> = Vec::new();

        let table = quote_identifier(require_table_name(self.table.as_deref())?);
        let quoted_cols: Vec<String> = self.columns.iter().map(|c| quote_identifier(c)).collect();
        let columns_str = quoted_cols.join(", ");

        let value_rows: Vec<String> = self
            .rows
            .iter()
            .map(|row| {
                all_params.extend(row.iter().cloned());
                let placeholders: Vec<String> = row.iter().map(|_| "?".to_string()).collect();
                format!("({})", placeholders.join(", "))
            })
            .collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {};",
            table,
            columns_str,
            value_rows.join(", ")
        );

        Ok((sql, all_params))
    }
}

#[cfg(test)]
mod tests {
    use crate::insert::InsertQuery;
    use crate::query::Query;

    #[test]
    fn single_row_insert() {
        let q = InsertQuery::new()
            .into("users")
            .columns(&["name", "email"])
            .values(vec!["Alice", "alice@example.com"]);
        let (sql, params) = q.build().unwrap();
        assert_eq!(
            sql,
            "INSERT INTO \"users\" (\"name\", \"email\") VALUES (?, ?);"
        );
        assert_eq!(params, vec!["Alice", "alice@example.com"]);
    }

    #[test]
    fn multi_row_insert() {
        let q = InsertQuery::new()
            .into("users")
            .columns(&["name", "email"])
            .values(vec!["Alice", "alice@example.com"])
            .values(vec!["Bob", "bob@example.com"]);
        let (sql, params) = q.build().unwrap();
        assert_eq!(
            sql,
            "INSERT INTO \"users\" (\"name\", \"email\") VALUES (?, ?), (?, ?);"
        );
        assert_eq!(
            params,
            vec!["Alice", "alice@example.com", "Bob", "bob@example.com"]
        );
    }

    #[test]
    fn try_build_no_table_errors() {
        let q = InsertQuery::new().columns(&["name"]).values(vec!["Alice"]);
        assert!(q.try_build().unwrap_err().to_string().contains("no table"));
    }

    #[test]
    fn try_build_no_columns_errors() {
        let q = InsertQuery::new().into("users").values(vec!["Alice"]);
        assert!(q.try_build().unwrap_err().to_string().contains("no columns"));
    }

    #[test]
    fn build_blank_table_errors() {
        let q = InsertQuery::new()
            .into("")
            .columns(&["name"])
            .values(vec!["Alice"]);
        assert!(q.build().unwrap_err().to_string().contains("no table"));
    }

    #[test]
    fn try_build_column_value_count_mismatch() {
        let q = InsertQuery::new()
            .into("users")
            .columns(&["name", "email", "age"])
            .values(vec!["Alice", "alice@example.com"]);
        assert!(q.try_build().unwrap_err().to_string().contains("column count"));
    }

    #[test]
    fn try_build_no_values_errors() {
        let q = InsertQuery::new().into("users").columns(&["name", "email"]);
        assert!(q.try_build().is_err());
    }
}
