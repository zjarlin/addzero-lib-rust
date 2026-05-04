use crate::{JoinType, Query, QueryError, SortOrder, require_table_name};

/// A SELECT query builder.
#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone)]
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
    pub fn try_build(&self) -> Result<(String, Vec<String>), QueryError> {
        self.build()
    }
}

impl Query for SelectQuery {
    fn build(&self) -> Result<(String, Vec<String>), QueryError> {
        let mut sql = String::new();
        let mut all_params: Vec<String> = Vec::new();
        let table = require_table_name(self.table.as_deref())?;

        // SELECT clause
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }
        if self.columns.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.columns.join(", "));
        }

        // FROM clause
        sql.push_str(&format!(" FROM {}", table));

        // JOIN clauses
        for join in &self.joins {
            let join_kw = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::FullOuter => "FULL OUTER JOIN",
                JoinType::Cross => "CROSS JOIN",
            };
            sql.push_str(&format!(" {} {} ON {}", join_kw, join.table, join.on));
        }

        // WHERE clause
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

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING
        if let Some((ref cond, ref params)) = self.having {
            all_params.extend(params.iter().cloned());
            sql.push_str(&format!(" HAVING {}", cond));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            let parts: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, order)| {
                    let dir = match order {
                        SortOrder::Asc => "ASC",
                        SortOrder::Desc => "DESC",
                    };
                    format!("{} {}", col, dir)
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", parts.join(", ")));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Ok((sql, all_params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_select_all() {
        let q = SelectQuery::new().from("users");
        let (sql, params) = q.build().unwrap();
        assert_eq!(sql, "SELECT * FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn select_specific_columns() {
        let q = SelectQuery::new()
            .select(&["id", "name", "email"])
            .from("users");
        let (sql, _) = q.build().unwrap();
        assert_eq!(sql, "SELECT id, name, email FROM users");
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
        assert!(sql.starts_with("SELECT DISTINCT country"));
    }

    #[test]
    fn select_with_join() {
        let q = SelectQuery::new()
            .select(&["users.name", "orders.total"])
            .from("users")
            .inner_join("orders", "users.id = orders.user_id");
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("INNER JOIN orders ON users.id = orders.user_id"));
    }

    #[test]
    fn select_with_left_join() {
        let q = SelectQuery::new()
            .select(&["users.name", "profiles.bio"])
            .from("users")
            .left_join("profiles", "users.id = profiles.user_id");
        let (sql, _) = q.build().unwrap();
        assert!(sql.contains("LEFT JOIN profiles ON users.id = profiles.user_id"));
    }

    #[test]
    fn select_group_by_having() {
        let q = SelectQuery::new()
            .select(&["department", "COUNT(*)"])
            .from("employees")
            .group_by(&["department"])
            .having("COUNT(*) > ?", vec!["5"]);
        let (sql, params) = q.build().unwrap();
        assert!(sql.contains("GROUP BY department"));
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
        assert!(sql.contains("ORDER BY name ASC, id DESC"));
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
        assert!(sql.contains("FROM users u"));
        assert!(sql.contains("INNER JOIN orders o"));
        assert!(sql.contains("ORDER BY o.total DESC"));
        assert!(sql.contains("LIMIT 5"));
        assert_eq!(params, vec!["100", "true"]);
    }

    #[test]
    fn to_sql_convenience_method() {
        let q = SelectQuery::new()
            .from("users")
            .r#where("id = ?", vec!["1"]);
        let sql = q.to_sql().unwrap();
        assert!(sql.contains("SELECT * FROM users WHERE id = ?"));
    }

    #[test]
    fn try_build_no_table_errors() {
        let q = SelectQuery::new().select(&["id"]);
        assert_eq!(q.try_build(), Err(QueryError::NoTable));
    }

    #[test]
    fn build_blank_table_errors() {
        let q = SelectQuery::new().from("   ");
        assert_eq!(q.build(), Err(QueryError::NoTable));
    }
}
