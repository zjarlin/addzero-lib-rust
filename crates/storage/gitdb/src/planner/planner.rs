//! Query planner - converts SQL AST to logical and physical plans.
//!
//! The planner is the entry point for query optimization.

use std::sync::Arc;

use parking_lot::RwLock;

use super::error::{PlanError, PlanResult};
use super::logical::{
    AggregateExpr, AggregateFunction, ColumnRef, LogicalPlan, ProjectColumn, SortDirection,
    SortSpec,
};
use super::optimizer::Optimizer;
use super::physical::PhysicalPlan;
use crate::catalog::Catalog;
use crate::sql::{Expr, OrderBy, Select, SelectColumn, Statement};
use crate::storage::GitRepository;

/// The query planner.
pub struct QueryPlanner {
    catalog: Catalog,
    optimizer: Optimizer,
}

impl QueryPlanner {
    /// Create a new query planner.
    pub fn new(repo: Arc<RwLock<GitRepository>>) -> Self {
        Self {
            catalog: Catalog::new(repo),
            optimizer: Optimizer::new(),
        }
    }

    /// Create a planner with a custom optimizer.
    pub fn with_optimizer(repo: Arc<RwLock<GitRepository>>, optimizer: Optimizer) -> Self {
        Self {
            catalog: Catalog::new(repo),
            optimizer,
        }
    }

    /// Plan a SQL statement.
    pub fn plan(&self, stmt: &Statement) -> PlanResult<QueryPlan> {
        match stmt {
            Statement::Select(select) => {
                let logical = self.plan_select(select)?;
                let optimized = self.optimizer.optimize(logical)?;
                let physical = self.optimizer.to_physical(&optimized)?;

                Ok(QueryPlan {
                    logical: optimized,
                    physical,
                })
            }
            _ => Err(PlanError::Unsupported(
                "Only SELECT statements can be planned".into(),
            )),
        }
    }

    /// Create a logical plan for a SELECT statement.
    pub fn plan_select(&self, select: &Select) -> PlanResult<LogicalPlan> {
        // Start with table scan.
        let mut plan = self.plan_from(&select.from)?;

        // Add WHERE filter.
        if let Some(ref where_clause) = select.where_clause {
            plan = LogicalPlan::Filter {
                input: Box::new(plan),
                predicate: self.convert_expr(where_clause),
            };
        }

        // Check for aggregates.
        if self.has_aggregates(&select.columns) {
            let (aggregates, non_agg_columns) = self.extract_aggregates(&select.columns)?;

            plan = LogicalPlan::Aggregate {
                input: Box::new(plan),
                group_by: Vec::new(), // Would need GROUP BY in AST.
                aggregates,
            };

            // Project the result if needed.
            if !non_agg_columns.is_empty() {
                plan = LogicalPlan::Project {
                    input: Box::new(plan),
                    columns: non_agg_columns,
                };
            }
        } else {
            // Regular projection.
            let columns = self.convert_select_columns(&select.columns)?;
            if !columns.is_empty() && !self.is_star_only(&columns) {
                plan = LogicalPlan::Project {
                    input: Box::new(plan),
                    columns,
                };
            }
        }

        // Add ORDER BY.
        if !select.order_by.is_empty() {
            let order = self.convert_order_by(&select.order_by)?;
            plan = LogicalPlan::Sort {
                input: Box::new(plan),
                order,
            };
        }

        // Add LIMIT/OFFSET.
        if let Some(limit) = select.limit {
            plan = LogicalPlan::Limit {
                input: Box::new(plan),
                limit,
                offset: select.offset,
            };
        }

        Ok(plan)
    }

    fn plan_from(&self, table: &str) -> PlanResult<LogicalPlan> {
        // Verify table exists.
        if !self.catalog.table_exists(table) {
            return Err(PlanError::TableNotFound(table.to_string()));
        }

        Ok(LogicalPlan::Scan {
            table: table.to_string(),
            alias: None,
            columns: None,
        })
    }

    fn convert_select_columns(&self, columns: &[SelectColumn]) -> PlanResult<Vec<ProjectColumn>> {
        let mut result = Vec::new();

        for col in columns {
            match col {
                SelectColumn::Wildcard => {
                    result.push(ProjectColumn::Star);
                }
                SelectColumn::Column(name) => {
                    result.push(ProjectColumn::Column(ColumnRef::new(name.clone())));
                }
                SelectColumn::Expr { expr, alias } => {
                    let converted = self.convert_expr(expr);
                    if let Expr::Column(name) = &converted {
                        result.push(ProjectColumn::Column(ColumnRef::new(name.clone())));
                    } else {
                        result.push(ProjectColumn::Expr {
                            expr: converted,
                            alias: alias.clone(),
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    fn is_star_only(&self, columns: &[ProjectColumn]) -> bool {
        columns.len() == 1 && matches!(&columns[0], ProjectColumn::Star)
    }

    fn convert_order_by(&self, order_by: &[OrderBy]) -> PlanResult<Vec<SortSpec>> {
        order_by
            .iter()
            .map(|item| {
                Ok(SortSpec {
                    column: item.column.clone(),
                    direction: if item.ascending {
                        SortDirection::Ascending
                    } else {
                        SortDirection::Descending
                    },
                    nulls_first: false,
                })
            })
            .collect()
    }

    fn convert_expr(&self, expr: &crate::sql::Expr) -> Expr {
        // The planner uses the same Expr type as the SQL AST,
        // so we just clone it. This avoids complex conversions.
        expr.clone()
    }

    fn has_aggregates(&self, columns: &[SelectColumn]) -> bool {
        columns.iter().any(|col| {
            if let SelectColumn::Expr { expr, .. } = col {
                self.expr_has_aggregate(expr)
            } else {
                false
            }
        })
    }

    fn expr_has_aggregate(&self, expr: &crate::sql::Expr) -> bool {
        match expr {
            crate::sql::Expr::Function { name, .. } => {
                let upper = name.to_uppercase();
                matches!(upper.as_str(), "COUNT" | "SUM" | "AVG" | "MIN" | "MAX")
            }
            crate::sql::Expr::BinaryOp { left, right, .. } => {
                self.expr_has_aggregate(left) || self.expr_has_aggregate(right)
            }
            crate::sql::Expr::UnaryOp { expr, .. } => self.expr_has_aggregate(expr),
            crate::sql::Expr::IsNull { expr, .. } => self.expr_has_aggregate(expr),
            crate::sql::Expr::Between {
                expr, low, high, ..
            } => {
                self.expr_has_aggregate(expr)
                    || self.expr_has_aggregate(low)
                    || self.expr_has_aggregate(high)
            }
            crate::sql::Expr::InList { expr, list, .. } => {
                self.expr_has_aggregate(expr) || list.iter().any(|e| self.expr_has_aggregate(e))
            }
            _ => false,
        }
    }

    fn extract_aggregates(
        &self,
        columns: &[SelectColumn],
    ) -> PlanResult<(Vec<AggregateExpr>, Vec<ProjectColumn>)> {
        let mut aggregates = Vec::new();
        let mut non_agg = Vec::new();

        for (i, col) in columns.iter().enumerate() {
            if let SelectColumn::Expr { expr, alias } = col {
                if let crate::sql::Expr::Function { name, args } = expr {
                    let upper = name.to_uppercase();
                    let function = match upper.as_str() {
                        "COUNT" => AggregateFunction::Count,
                        "SUM" => AggregateFunction::Sum,
                        "AVG" => AggregateFunction::Avg,
                        "MIN" => AggregateFunction::Min,
                        "MAX" => AggregateFunction::Max,
                        _ => {
                            // Not an aggregate function.
                            non_agg.push(ProjectColumn::Expr {
                                expr: self.convert_expr(expr),
                                alias: alias.clone(),
                            });
                            continue;
                        }
                    };

                    let column = if args.is_empty() {
                        None
                    } else if let crate::sql::Expr::Column(col) = &args[0] {
                        Some(col.clone())
                    } else {
                        None
                    };

                    let alias_name = alias
                        .clone()
                        .unwrap_or_else(|| format!("{}_{}", name.to_lowercase(), i));

                    aggregates.push(AggregateExpr {
                        function,
                        column,
                        alias: alias_name,
                    });
                } else if let crate::sql::Expr::Column(name) = expr {
                    non_agg.push(ProjectColumn::Column(ColumnRef::new(name.clone())));
                } else {
                    non_agg.push(ProjectColumn::Expr {
                        expr: self.convert_expr(expr),
                        alias: alias.clone(),
                    });
                }
            } else if let SelectColumn::Column(name) = col {
                non_agg.push(ProjectColumn::Column(ColumnRef::new(name.clone())));
            }
        }

        Ok((aggregates, non_agg))
    }

    /// Explain a query plan.
    pub fn explain(&self, stmt: &Statement) -> PlanResult<String> {
        let plan = self.plan(stmt)?;
        Ok(format!(
            "=== Logical Plan ===\n{}\n=== Physical Plan ===\n{}",
            plan.logical, plan.physical
        ))
    }
}

/// A complete query plan with both logical and physical representations.
pub struct QueryPlan {
    pub logical: LogicalPlan,
    pub physical: PhysicalPlan,
}

impl QueryPlan {
    /// Get the estimated cost.
    pub fn estimated_cost(&self) -> f64 {
        self.physical.total_cost()
    }

    /// Get the estimated row count.
    pub fn estimated_rows(&self) -> usize {
        self.physical.estimated_rows()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Parser;
    use tempfile::TempDir;

    fn setup() -> (QueryPlanner, TempDir) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::open_or_init(dir.path()).unwrap();
        let shared_repo = Arc::new(RwLock::new(repo));

        // Create a test table.
        let catalog = Catalog::new(shared_repo.clone());
        catalog
            .create_table(
                crate::catalog::SchemaBuilder::new("users")
                    .add_column("id", crate::catalog::DataType::Text)
                    .add_column("name", crate::catalog::DataType::Text)
                    .add_column("age", crate::catalog::DataType::Integer)
                    .primary_key("id")
                    .build()
                    .unwrap(),
            )
            .unwrap();

        let planner = QueryPlanner::new(shared_repo);
        (planner, dir)
    }

    #[test]
    fn test_plan_simple_select() {
        let (planner, _dir) = setup();

        let stmt = Parser::parse("SELECT * FROM users").unwrap();
        let plan = planner.plan(&stmt).unwrap();

        assert!(plan.estimated_cost() > 0.0);
    }

    #[test]
    fn test_plan_select_with_where() {
        let (planner, _dir) = setup();

        let stmt = Parser::parse("SELECT * FROM users WHERE age > 21").unwrap();
        let plan = planner.plan(&stmt).unwrap();

        // Filter should reduce estimated rows.
        assert!(plan.estimated_rows() < 1000);
    }

    #[test]
    fn test_plan_select_with_order() {
        let (planner, _dir) = setup();

        let stmt = Parser::parse("SELECT * FROM users ORDER BY age DESC").unwrap();
        let plan = planner.plan(&stmt).unwrap();

        // Should have sort in the plan.
        assert!(plan.estimated_cost() > 0.0);
    }

    #[test]
    fn test_plan_select_with_limit() {
        let (planner, _dir) = setup();

        let stmt = Parser::parse("SELECT * FROM users LIMIT 10").unwrap();
        let plan = planner.plan(&stmt).unwrap();

        assert_eq!(plan.estimated_rows(), 10);
    }

    #[test]
    fn test_explain() {
        let (planner, _dir) = setup();

        let stmt =
            Parser::parse("SELECT * FROM users WHERE age > 21 ORDER BY name LIMIT 5").unwrap();
        let explanation = planner.explain(&stmt).unwrap();

        assert!(explanation.contains("Logical Plan"));
        assert!(explanation.contains("Physical Plan"));
    }

    #[test]
    fn test_table_not_found() {
        let (planner, _dir) = setup();

        let stmt = Parser::parse("SELECT * FROM nonexistent").unwrap();
        let result = planner.plan(&stmt);

        assert!(matches!(result, Err(PlanError::TableNotFound(_))));
    }
}
