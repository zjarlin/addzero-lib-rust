//! Logical query plan representation.
//!
//! Logical plans represent *what* the query does, not *how* it will be executed.
//! They are tree structures that can be optimized before conversion to physical plans.

use std::collections::HashSet;
use std::fmt;

use az_derive_aliases::{apply, plain_copy_eq, plain_eq_hash, plain_partial_eq};

// Re-export Expr from sql for use in plans.
pub use crate::sql::Expr;

/// Join types supported by the planner.
#[apply(plain_copy_eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER"),
            JoinType::Left => write!(f, "LEFT"),
            JoinType::Right => write!(f, "RIGHT"),
            JoinType::Full => write!(f, "FULL"),
            JoinType::Cross => write!(f, "CROSS"),
        }
    }
}

/// Aggregate function types.
#[apply(plain_copy_eq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    CountDistinct,
}

impl fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateFunction::Count => write!(f, "COUNT"),
            AggregateFunction::Sum => write!(f, "SUM"),
            AggregateFunction::Avg => write!(f, "AVG"),
            AggregateFunction::Min => write!(f, "MIN"),
            AggregateFunction::Max => write!(f, "MAX"),
            AggregateFunction::CountDistinct => write!(f, "COUNT_DISTINCT"),
        }
    }
}

/// Sort direction.
#[apply(plain_copy_eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Sort specification.
#[apply(plain_partial_eq)]
pub struct SortSpec {
    pub column: String,
    pub direction: SortDirection,
    pub nulls_first: bool,
}

/// Column reference with optional table alias.
#[apply(plain_eq_hash)]
pub struct ColumnRef {
    pub table: Option<String>,
    pub column: String,
}

impl ColumnRef {
    pub fn new(column: impl Into<String>) -> Self {
        Self {
            table: None,
            column: column.into(),
        }
    }

    pub fn qualified(table: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            table: Some(table.into()),
            column: column.into(),
        }
    }
}

impl fmt::Display for ColumnRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref table) = self.table {
            write!(f, "{}.{}", table, self.column)
        } else {
            write!(f, "{}", self.column)
        }
    }
}

/// Logical query plan.
///
/// This is a tree structure representing the logical operations of a query.
/// Each node transforms its input(s) in some way.
#[apply(plain_partial_eq)]
pub enum LogicalPlan {
    /// Scan a table, returning all rows.
    Scan {
        table: String,
        alias: Option<String>,
        /// Which columns to include (None = all).
        columns: Option<Vec<String>>,
    },

    /// Filter rows based on a predicate.
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },

    /// Project specific columns.
    Project {
        input: Box<LogicalPlan>,
        columns: Vec<ProjectColumn>,
    },

    /// Join two inputs.
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: JoinType,
        on: Option<Expr>,
    },

    /// Sort by columns.
    Sort {
        input: Box<LogicalPlan>,
        order: Vec<SortSpec>,
    },

    /// Limit number of rows.
    Limit {
        input: Box<LogicalPlan>,
        limit: usize,
        offset: Option<usize>,
    },

    /// Group by with aggregates.
    Aggregate {
        input: Box<LogicalPlan>,
        group_by: Vec<String>,
        aggregates: Vec<AggregateExpr>,
    },

    /// Remove duplicate rows.
    Distinct { input: Box<LogicalPlan> },

    /// Union of two inputs (removes duplicates).
    Union {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
    },

    /// Return no rows.
    Empty { columns: Vec<String> },
}

/// A column in a projection.
#[apply(plain_partial_eq)]
pub enum ProjectColumn {
    /// All columns from input.
    Star,
    /// All columns from a specific table.
    TableStar(String),
    /// Named column reference.
    Column(ColumnRef),
    /// Expression with optional alias.
    Expr { expr: Expr, alias: Option<String> },
}

/// An aggregate expression.
#[apply(plain_partial_eq)]
pub struct AggregateExpr {
    pub function: AggregateFunction,
    pub column: Option<String>,
    pub alias: String,
}

impl LogicalPlan {
    /// Get the output columns of this plan.
    pub fn output_columns(&self) -> Vec<String> {
        match self {
            LogicalPlan::Scan { columns, .. } => columns.clone().unwrap_or_default(),
            LogicalPlan::Filter { input, .. } => input.output_columns(),
            LogicalPlan::Project { columns, .. } => columns
                .iter()
                .filter_map(|c| match c {
                    ProjectColumn::Column(col) => Some(col.column.clone()),
                    ProjectColumn::Expr { alias, .. } => alias.clone(),
                    _ => None,
                })
                .collect(),
            LogicalPlan::Join { left, right, .. } => {
                let mut cols = left.output_columns();
                cols.extend(right.output_columns());
                cols
            }
            LogicalPlan::Sort { input, .. } => input.output_columns(),
            LogicalPlan::Limit { input, .. } => input.output_columns(),
            LogicalPlan::Aggregate {
                group_by,
                aggregates,
                ..
            } => {
                let mut cols: Vec<String> = group_by.clone();
                cols.extend(aggregates.iter().map(|a| a.alias.clone()));
                cols
            }
            LogicalPlan::Distinct { input } => input.output_columns(),
            LogicalPlan::Union { left, .. } => left.output_columns(),
            LogicalPlan::Empty { columns } => columns.clone(),
        }
    }

    /// Get the tables referenced by this plan.
    pub fn referenced_tables(&self) -> HashSet<String> {
        let mut tables = HashSet::new();
        self.collect_tables(&mut tables);
        tables
    }

    fn collect_tables(&self, tables: &mut HashSet<String>) {
        match self {
            LogicalPlan::Scan { table, .. } => {
                tables.insert(table.clone());
            }
            LogicalPlan::Filter { input, .. } => input.collect_tables(tables),
            LogicalPlan::Project { input, .. } => input.collect_tables(tables),
            LogicalPlan::Join { left, right, .. } => {
                left.collect_tables(tables);
                right.collect_tables(tables);
            }
            LogicalPlan::Sort { input, .. } => input.collect_tables(tables),
            LogicalPlan::Limit { input, .. } => input.collect_tables(tables),
            LogicalPlan::Aggregate { input, .. } => input.collect_tables(tables),
            LogicalPlan::Distinct { input } => input.collect_tables(tables),
            LogicalPlan::Union { left, right } => {
                left.collect_tables(tables);
                right.collect_tables(tables);
            }
            LogicalPlan::Empty { .. } => {}
        }
    }

    /// Estimate the cardinality (number of rows) this plan will produce.
    /// This is a rough heuristic for cost estimation.
    pub fn estimated_cardinality(&self) -> usize {
        match self {
            // Assume 1000 rows per table as baseline
            LogicalPlan::Scan { .. } => 1000,
            // Filter typically reduces by 1/3
            LogicalPlan::Filter { input, .. } => input.estimated_cardinality() / 3,
            // Projection doesn't change row count
            LogicalPlan::Project { input, .. } => input.estimated_cardinality(),
            // Join multiplies cardinalities (pessimistic)
            LogicalPlan::Join { left, right, .. } => {
                (left.estimated_cardinality() * right.estimated_cardinality()) / 100
            }
            LogicalPlan::Sort { input, .. } => input.estimated_cardinality(),
            LogicalPlan::Limit { input, limit, .. } => (*limit).min(input.estimated_cardinality()),
            // Aggregation typically reduces significantly
            LogicalPlan::Aggregate {
                input, group_by, ..
            } => {
                if group_by.is_empty() {
                    1
                } else {
                    input.estimated_cardinality() / 10
                }
            }
            LogicalPlan::Distinct { input } => input.estimated_cardinality() / 2,
            LogicalPlan::Union { left, right } => {
                left.estimated_cardinality() + right.estimated_cardinality()
            }
            LogicalPlan::Empty { .. } => 0,
        }
    }
}

impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_indent(f, 0)
    }
}

impl LogicalPlan {
    fn format_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let pad = "  ".repeat(indent);
        match self {
            LogicalPlan::Scan {
                table,
                alias,
                columns,
            } => {
                write!(f, "{}Scan: {}", pad, table)?;
                if let Some(a) = alias {
                    write!(f, " AS {}", a)?;
                }
                if let Some(cols) = columns {
                    write!(f, " [{}]", cols.join(", "))?;
                }
                writeln!(f)
            }
            LogicalPlan::Filter { input, predicate } => {
                writeln!(f, "{}Filter: {:?}", pad, predicate)?;
                input.format_indent(f, indent + 1)
            }
            LogicalPlan::Project { input, columns } => {
                let cols: Vec<String> = columns.iter().map(|c| format!("{:?}", c)).collect();
                writeln!(f, "{}Project: [{}]", pad, cols.join(", "))?;
                input.format_indent(f, indent + 1)
            }
            LogicalPlan::Join {
                left,
                right,
                join_type,
                on,
            } => {
                write!(f, "{}Join: {}", pad, join_type)?;
                if let Some(cond) = on {
                    write!(f, " ON {:?}", cond)?;
                }
                writeln!(f)?;
                left.format_indent(f, indent + 1)?;
                right.format_indent(f, indent + 1)
            }
            LogicalPlan::Sort { input, order } => {
                let ord: Vec<String> = order
                    .iter()
                    .map(|s| format!("{} {:?}", s.column, s.direction))
                    .collect();
                writeln!(f, "{}Sort: [{}]", pad, ord.join(", "))?;
                input.format_indent(f, indent + 1)
            }
            LogicalPlan::Limit {
                input,
                limit,
                offset,
            } => {
                write!(f, "{}Limit: {}", pad, limit)?;
                if let Some(o) = offset {
                    write!(f, " OFFSET {}", o)?;
                }
                writeln!(f)?;
                input.format_indent(f, indent + 1)
            }
            LogicalPlan::Aggregate {
                input,
                group_by,
                aggregates,
            } => {
                let aggs: Vec<String> = aggregates
                    .iter()
                    .map(|a| format!("{}({:?})", a.function, a.column))
                    .collect();
                writeln!(
                    f,
                    "{}Aggregate: group=[{}], aggs=[{}]",
                    pad,
                    group_by.join(", "),
                    aggs.join(", ")
                )?;
                input.format_indent(f, indent + 1)
            }
            LogicalPlan::Distinct { input } => {
                writeln!(f, "{}Distinct", pad)?;
                input.format_indent(f, indent + 1)
            }
            LogicalPlan::Union { left, right } => {
                writeln!(f, "{}Union", pad)?;
                left.format_indent(f, indent + 1)?;
                right.format_indent(f, indent + 1)
            }
            LogicalPlan::Empty { columns } => {
                writeln!(f, "{}Empty: [{}]", pad, columns.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_type_display() {
        assert_eq!(JoinType::Inner.to_string(), "INNER");
        assert_eq!(JoinType::Left.to_string(), "LEFT");
        assert_eq!(JoinType::Right.to_string(), "RIGHT");
    }

    #[test]
    fn test_column_ref() {
        let col = ColumnRef::new("name");
        assert_eq!(col.to_string(), "name");

        let qualified = ColumnRef::qualified("users", "id");
        assert_eq!(qualified.to_string(), "users.id");
    }

    #[test]
    fn test_scan_estimated_cardinality() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        assert_eq!(scan.estimated_cardinality(), 1000);
    }

    #[test]
    fn test_filter_reduces_cardinality() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let filter = LogicalPlan::Filter {
            input: Box::new(scan),
            predicate: Expr::Column("x".into()),
        };
        assert!(filter.estimated_cardinality() < 1000);
    }

    #[test]
    fn test_limit_caps_cardinality() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let limit = LogicalPlan::Limit {
            input: Box::new(scan),
            limit: 10,
            offset: None,
        };
        assert_eq!(limit.estimated_cardinality(), 10);
    }

    #[test]
    fn test_referenced_tables() {
        let left = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let right = LogicalPlan::Scan {
            table: "orders".to_string(),
            alias: None,
            columns: None,
        };
        let join = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Inner,
            on: None,
        };

        let tables = join.referenced_tables();
        assert!(tables.contains("users"));
        assert!(tables.contains("orders"));
        assert_eq!(tables.len(), 2);
    }
}
