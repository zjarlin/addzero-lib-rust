use crate::{Query, QueryError};

/// An UPDATE query builder.
#[derive(Debug, Clone, Default)]
pub struct UpdateQuery {
    table: Option<String>,
    set_clauses: Vec<(String, String)>,
    conditions: Vec<(String, Vec<String>)>,
    limit: Option<usize>,
}

impl UpdateQuery {
    /// Create a new UPDATE query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target table.
    pub fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// Add a SET clause.
    pub fn set(mut self, column: &str, value: &str) -> Self {
        self.set_clauses
            .push((column.to_string(), value.to_string()));
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
        if self.table.is_none() {
            return Err(QueryError::NoTable);
        }
        if self.set_clauses.is_empty() {
            return Err(QueryError::NoSetClauses);
        }
        Ok(self.build())
    }
}

impl Query for UpdateQuery {
    fn build(&self) -> (String, Vec<String>) {
        let mut all_params: Vec<String> = Vec::new();
        let table = self.table.as_deref().unwrap_or("unknown");

        let set_parts: Vec<String> = self
            .set_clauses
            .iter()
            .map(|(col, val)| {
                all_params.push(val.clone());
                format!("{} = ?", col)
            })
            .collect();

        let mut sql = format!("UPDATE {} SET {}", table, set_parts.join(", "));

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
        (sql, all_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_update() {
        let q = UpdateQuery::new()
            .table("users")
            .set("name", "Alice")
            .set("email", "alice@new.com")
            .r#where("id = ?", vec!["1"]);
        let (sql, params) = q.build();
        assert!(sql.contains("UPDATE users SET name = ?, email = ?"));
        assert!(sql.contains("WHERE id = ?"));
        assert_eq!(params, vec!["Alice", "alice@new.com", "1"]);
    }

    #[test]
    fn update_with_limit() {
        let q = UpdateQuery::new()
            .table("posts")
            .set("active", "false")
            .limit(100);
        let (sql, _) = q.build();
        assert!(sql.contains("LIMIT 100"));
    }

    #[test]
    fn try_build_no_table_errors() {
        let q = UpdateQuery::new().set("name", "Alice");
        assert_eq!(q.try_build(), Err(QueryError::NoTable));
    }

    #[test]
    fn try_build_no_set_errors() {
        let q = UpdateQuery::new()
            .table("users")
            .r#where("id = ?", vec!["1"]);
        assert_eq!(q.try_build(), Err(QueryError::NoSetClauses));
    }
}
