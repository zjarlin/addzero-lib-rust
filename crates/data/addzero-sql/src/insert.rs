use crate::{Query, QueryError};

/// An INSERT query builder.
#[derive(Debug, Clone, Default)]
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
        self.rows.push(values.into_iter().map(String::from).collect());
        self
    }

    /// Build and validate the query, returning an error if invalid.
    pub fn try_build(&self) -> Result<(String, Vec<String>), QueryError> {
        if self.table.is_none() {
            return Err(QueryError::NoTable);
        }
        if self.columns.is_empty() {
            return Err(QueryError::NoColumns);
        }
        if !self.rows.is_empty() {
            let first_len = self.rows[0].len();
            for row in &self.rows {
                if row.len() != first_len {
                    return Err(QueryError::ColumnValueMismatch {
                        columns: self.columns.len(),
                        values: row.len(),
                    });
                }
            }
        }
        Ok(self.build())
    }
}

impl Query for InsertQuery {
    fn build(&self) -> (String, Vec<String>) {
        let mut all_params: Vec<String> = Vec::new();

        let table = self.table.as_deref().unwrap_or("unknown");
        let columns_str = self.columns.join(", ");

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

        (sql, all_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_row_insert() {
        let q = InsertQuery::new()
            .into("users")
            .columns(&["name", "email"])
            .values(vec!["Alice", "alice@example.com"]);
        let (sql, params) = q.build();
        assert_eq!(sql, "INSERT INTO users (name, email) VALUES (?, ?);");
        assert_eq!(params, vec!["Alice", "alice@example.com"]);
    }

    #[test]
    fn multi_row_insert() {
        let q = InsertQuery::new()
            .into("users")
            .columns(&["name", "email"])
            .values(vec!["Alice", "alice@example.com"])
            .values(vec!["Bob", "bob@example.com"]);
        let (sql, params) = q.build();
        assert_eq!(
            sql,
            "INSERT INTO users (name, email) VALUES (?, ?), (?, ?);"
        );
        assert_eq!(
            params,
            vec!["Alice", "alice@example.com", "Bob", "bob@example.com"]
        );
    }

    #[test]
    fn try_build_no_table_errors() {
        let q = InsertQuery::new().columns(&["name"]).values(vec!["Alice"]);
        assert_eq!(q.try_build(), Err(QueryError::NoTable));
    }

    #[test]
    fn try_build_no_columns_errors() {
        let q = InsertQuery::new().into("users").values(vec!["Alice"]);
        assert_eq!(q.try_build(), Err(QueryError::NoColumns));
    }
}
