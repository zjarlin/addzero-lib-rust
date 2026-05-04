use crate::{Query, QueryError, require_table_name};

/// A DELETE query builder.
#[derive(Debug, Clone, Default)]
pub struct DeleteQuery {
    table: Option<String>,
    conditions: Vec<(String, Vec<String>)>,
    limit: Option<usize>,
}

impl DeleteQuery {
    /// Create a new DELETE query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target table.
    pub fn from(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// Add a WHERE condition with parameterized values.
    pub fn r#where(mut self, condition: &str, params: Vec<&str>) -> Self {
        self.conditions.push((
            condition.to_string(),
            params.into_iter().map(String::from).collect(),
        ));
        self
    }

    /// Set the LIMIT.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Build and validate the query.
    pub fn try_build(&self) -> Result<(String, Vec<String>), QueryError> {
        require_table_name(self.table.as_deref())?;
        self.build()
    }
}

impl Query for DeleteQuery {
    fn build(&self) -> Result<(String, Vec<String>), QueryError> {
        let mut all_params: Vec<String> = Vec::new();
        let table = require_table_name(self.table.as_deref())?;

        let mut sql = format!("DELETE FROM {}", table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let cond_parts: Vec<String> = self
                .conditions
                .iter()
                .map(|(cond, params)| {
                    all_params.extend(params.iter().cloned());
                    cond.clone()
                })
                .collect();
            sql.push_str(&cond_parts.join(" AND "));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        sql.push(';');
        Ok((sql, all_params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_with_where() {
        let q = DeleteQuery::new()
            .from("users")
            .r#where("id = ?", vec!["42"]);
        let (sql, params) = q.build().unwrap();
        assert_eq!(sql, "DELETE FROM users WHERE id = ?;");
        assert_eq!(params, vec!["42"]);
    }

    #[test]
    fn delete_all() {
        let q = DeleteQuery::new().from("sessions");
        let (sql, params) = q.build().unwrap();
        assert_eq!(sql, "DELETE FROM sessions;");
        assert!(params.is_empty());
    }

    #[test]
    fn delete_with_multiple_conditions() {
        let q = DeleteQuery::new()
            .from("logs")
            .r#where("created_at < ?", vec!["2024-01-01"])
            .r#where("level = ?", vec!["DEBUG"])
            .limit(1000);
        let (sql, params) = q.build().unwrap();
        assert!(sql.contains("WHERE created_at < ? AND level = ?"));
        assert!(sql.contains("LIMIT 1000"));
        assert_eq!(params, vec!["2024-01-01", "DEBUG"]);
    }

    #[test]
    fn try_build_no_table_errors() {
        let q = DeleteQuery::new().r#where("id = ?", vec!["1"]);
        assert_eq!(q.try_build(), Err(QueryError::NoTable));
    }

    #[test]
    fn build_blank_table_errors() {
        let q = DeleteQuery::new().from("");
        assert_eq!(q.build(), Err(QueryError::NoTable));
    }
}
