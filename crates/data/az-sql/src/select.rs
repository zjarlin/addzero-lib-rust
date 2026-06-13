use anyhow::Result;
use az_derive_aliases::{apply, plain_clone_debug, plain_default_clone_debug};

use crate::identifier::{quote_identifier, require_table_name};
use crate::query::{JoinType, Query, SortOrder};

/// A SELECT query builder.
#[apply(plain_default_clone_debug)]
pub struct SelectQuery {
    distinct: bool,
    columns: Vec<String>,
    table: Option<String>,
    joins: Vec<JoinClause>,
    conditions: Vec<(String, Vec<String>)>,
    group_by: Vec<String>,
    having: Option<(String, Vec<String>)>,
    order_by: Vec<(String, SortOrder)>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[apply(plain_clone_debug)]
struct JoinClause {
    join_type: JoinType,
    table: String,
    on: String,
}

impl SelectQuery {
    /// Create a new empty SELECT query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Select specific columns.
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set SELECT DISTINCT.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Set the FROM table.
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

    /// Add an INNER JOIN.
    pub fn inner_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// Add a LEFT JOIN.
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// Add a GROUP BY clause.
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a HAVING clause with parameterized values.
    pub fn having(mut self, condition: &str, params: Vec<&str>) -> Self {
        self.having = Some((
            condition.to_string(),
            params.into_iter().map(String::from).collect(),
        ));
        self
    }

    /// Add an ORDER BY clause. `ascending = true` for ASC, `false` for DESC.
    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        self.order_by.push((
            column.to_string(),
            if ascending {
                SortOrder::Asc
            } else {
                SortOrder::Desc
            },
        ));
        self
    }

    /// Set the LIMIT.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Set the OFFSET.
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    /// Build and validate the query.
    pub fn try_build(&self) -> Result<(String, Vec<String>)> {
        self.build()
    }
}

impl Query for SelectQuery {
    fn build(&self) -> Result<(String, Vec<String>)> {
        let mut sql = String::new();
        let mut all_params: Vec<String> = Vec::new();
        let table = require_table_name(self.table.as_deref())?;

        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }
        if self.columns.is_empty() {
            sql.push('*');
        } else {
            let quoted: Vec<String> = self.columns.iter().map(|c| quote_identifier(c)).collect();
            sql.push_str(&quoted.join(", "));
        }

        sql.push_str(&format!(" FROM {}", quote_identifier(table)));

        for join in &self.joins {
            sql.push_str(&format!(
                " {} {} ON {}",
                join.join_type,
                quote_identifier(&join.table),
                join.on
            ));
        }

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

        if !self.group_by.is_empty() {
            let quoted: Vec<String> = self.group_by.iter().map(|c| quote_identifier(c)).collect();
            sql.push_str(&format!(" GROUP BY {}", quoted.join(", ")));
        }

        if let Some((ref cond, ref params)) = self.having {
            all_params.extend(params.iter().cloned());
            sql.push_str(&format!(" HAVING {}", cond));
        }

        if !self.order_by.is_empty() {
            let parts: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, order)| format!("{} {}", quote_identifier(col), order))
                .collect();
            sql.push_str(&format!(" ORDER BY {}", parts.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Ok((sql, all_params))
    }
}

#[cfg(test)]
mod tests {
    use crate::query::Query;
    use crate::select::SelectQuery;

    #[test]
    fn simple_select_all() {
        let q = SelectQuery::new().from("users");
        let (sql, params) = q.build().unwrap();
        assert_eq!(sql, "SELECT * FROM \"users\"");
        assert!(params.is_empty());
    }

    #[test]
    fn select_specific_columns() {
        let q = SelectQuery::new()
            .select(&["id", "name", "email"])
            .from("users");
        let (sql, _) = q.build().unwrap();
        assert_eq!(sql, "SELECT \"id\", \"name\", \"email\" FROM \"users\"");
    }

    #[test]
    fn select_with_where() {
        let q = SelectQuery::new()
            .select(&["id", "name"])
            .from("users")
            .r#where("age > ?", vec!["18"])
            .r#where("active = ?", vec!["true"]);
        let (sql, params) = q.build().unwrap();
        assert!(sql.contains("WHERE age > ? AND active = ?"));
        assert_eq!(params, vec!["18", "true"]);
    }

    #[test]
    fn select_distinct() {
        let q = SelectQuery::new()
            .select(&["country"])
            .from("users")
            .distinct();
        let (sql, _) = q.build().unwrap();
        assert!(sql.starts_with("SELECT DISTINCT \"country\""));
    }

    #[test]
    fn select_with_join() {
        let q = SelectQuery::new()
            .select(&["users.name", "orders.total"])
            .from("users")
            .inner_join("orders", "users.id = orders.user_id");
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("INNER JOIN \"orders\" ON users.id = orders.user_id"));
    }

    #[test]
    fn select_with_left_join() {
        let q = SelectQuery::new()
            .select(&["users.name", "profiles.bio"])
            .from("users")
            .left_join("profiles", "users.id = profiles.user_id");
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("LEFT JOIN \"profiles\" ON users.id = profiles.user_id"));
    }

    #[test]
    fn select_group_by_having() {
        let q = SelectQuery::new()
            .select(&["department", "COUNT(*)"])
            .from("employees")
            .group_by(&["department"])
            .having("COUNT(*) > ?", vec!["5"]);
        let (sql, params) = q.build().unwrap();
        assert!(sql.contains("GROUP BY \"department\""));
        assert!(sql.contains("HAVING COUNT(*) > ?"));
        assert_eq!(params, vec!["5"]);
    }

    #[test]
    fn select_order_by_limit_offset() {
        let q = SelectQuery::new()
            .select(&["id"])
            .from("users")
            .order_by("name", true)
            .order_by("id", false)
            .limit(10)
            .offset(20);
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("ORDER BY \"name\" ASC, \"id\" DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
    }

    #[test]
    fn complex_query() {
        let q = SelectQuery::new()
            .select(&["u.name", "o.total"])
            .from("users u")
            .inner_join("orders o", "u.id = o.user_id")
            .r#where("o.total > ?", vec!["100"])
            .r#where("u.active = ?", vec!["true"])
            .order_by("o.total", false)
            .limit(5);
        let (sql, params) = q.build().unwrap();
        assert!(sql.contains("FROM \"users u\""));
        assert!(sql.contains("INNER JOIN \"orders o\""));
        assert!(sql.contains("ORDER BY \"o.total\" DESC"));
        assert!(sql.contains("LIMIT 5"));
        assert_eq!(params, vec!["100", "true"]);
    }

    #[test]
    fn to_sql_convenience_method() {
        let q = SelectQuery::new()
            .from("users")
            .r#where("id = ?", vec!["1"]);
        let sql = q.to_sql().unwrap();
        assert!(sql.contains("SELECT * FROM \"users\" WHERE id = ?"));
    }

    // ── Injection prevention tests ──────────────────────────────────

    #[test]
    fn select_quotes_table_name_with_injection_attempt() {
        let q = SelectQuery::new().from("users; DROP TABLE users; --");
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("FROM \"users; DROP TABLE users; --\""));
    }

    #[test]
    fn select_escapes_double_quotes_in_identifier() {
        let q = SelectQuery::new().from("my\"table");
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("FROM \"my\"\"table\""));
    }

    #[test]
    fn try_build_no_table_errors() {
        let q = SelectQuery::new().select(&["id"]);
        assert!(q.try_build().unwrap_err().to_string().contains("no table"));
    }

    #[test]
    fn build_blank_table_errors() {
        let q = SelectQuery::new().from("   ");
        assert!(q.build().unwrap_err().to_string().contains("no table"));
    }
}
