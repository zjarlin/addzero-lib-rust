//! 逻辑查询计划表示。
//!
//! 逻辑计划描述查询“要做什么”，不指定具体执行算法。优化器会先改写这棵树，
//! 再把它转换为物理计划。

use std::collections::HashSet;
use std::fmt;


// Re-export Expr from sql for use in plans.
pub use crate::sql::Expr;

/// planner 支持的逻辑连接类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum JoinType {
    #[display("INNER")]
    Inner,
    #[display("LEFT")]
    Left,
    #[display("RIGHT")]
    Right,
    #[display("FULL")]
    Full,
    #[display("CROSS")]
    Cross,
}

impl JoinType {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 逻辑聚合函数类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum AggregateFunction {
    #[display("COUNT")]
    Count,
    #[display("SUM")]
    Sum,
    #[display("AVG")]
    Avg,
    #[display("MIN")]
    Min,
    #[display("MAX")]
    Max,
    #[display("COUNT_DISTINCT")]
    CountDistinct,
}

impl AggregateFunction {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 排序方向。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum SortDirection {
    #[display("ASC")]
    Ascending,
    #[display("DESC")]
    Descending,
}

impl SortDirection {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 排序规则。
#[derive(Clone, Debug, PartialEq)]
pub struct SortSpec {
    pub column: String,
    pub direction: SortDirection,
    pub nulls_first: bool,
}

/// 可带表别名的列引用。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// 逻辑查询计划树。
///
/// 每个节点表达一种关系代数操作，并对其输入节点进行变换。
#[derive(Clone, Debug, PartialEq)]
pub enum LogicalPlan {
    /// 扫描表，返回所有行。
    Scan {
        table: String,
        alias: Option<String>,
        /// 要包含的列；`None` 表示全部列。
        columns: Option<Vec<String>>,
    },

    /// 根据谓词过滤行。
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },

    /// 投影指定列或表达式。
    Project {
        input: Box<LogicalPlan>,
        columns: Vec<ProjectColumn>,
    },

    /// 连接两个输入。
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: JoinType,
        on: Option<Expr>,
    },

    /// 按列排序。
    Sort {
        input: Box<LogicalPlan>,
        order: Vec<SortSpec>,
    },

    /// 限制返回行数。
    Limit {
        input: Box<LogicalPlan>,
        limit: usize,
        offset: Option<usize>,
    },

    /// 分组并计算聚合。
    Aggregate {
        input: Box<LogicalPlan>,
        group_by: Vec<String>,
        aggregates: Vec<AggregateExpr>,
    },

    /// 去除重复行。
    Distinct { input: Box<LogicalPlan> },

    /// 合并两个输入并去重。
    Union {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
    },

    /// 返回空结果。
    Empty { columns: Vec<String> },
}

/// 投影中的一列。
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectColumn {
    /// 输入中的所有列。
    Star,
    /// 指定表的所有列。
    TableStar(String),
    /// 命名列引用。
    Column(ColumnRef),
    /// 表达式及可选别名。
    Expr { expr: Expr, alias: Option<String> },
}

/// 聚合表达式。
#[derive(Clone, Debug, PartialEq)]
pub struct AggregateExpr {
    pub function: AggregateFunction,
    pub column: Option<String>,
    pub alias: String,
}

impl LogicalPlan {
    /// 推导该计划的输出列。
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

    /// 收集该计划引用到的表。
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
                    .map(|s| format!("{} {}", s.column, s.direction))
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
    fn test_join_type_code_and_display() {
        assert_eq!(JoinType::Inner.to_string(), "INNER");
        assert_eq!(JoinType::Left.to_string(), "LEFT");
        assert_eq!(JoinType::Right.to_string(), "RIGHT");
        assert_eq!(JoinType::Inner.code(), "inner");
        assert_eq!(JoinType::from_code("cross"), Some(JoinType::Cross));
        assert_eq!(JoinType::ALL.len(), 5);
    }

    #[test]
    fn test_aggregate_function_code_and_display() {
        assert_eq!(
            AggregateFunction::CountDistinct.to_string(),
            "COUNT_DISTINCT"
        );
        assert_eq!(AggregateFunction::CountDistinct.code(), "count_distinct");
        assert_eq!(
            AggregateFunction::from_code("avg"),
            Some(AggregateFunction::Avg)
        );
        assert_eq!(AggregateFunction::ALL.len(), 6);
    }

    #[test]
    fn test_sort_direction_code_and_display() {
        assert_eq!(SortDirection::Ascending.to_string(), "ASC");
        assert_eq!(SortDirection::Descending.to_string(), "DESC");
        assert_eq!(SortDirection::Ascending.code(), "ascending");
        assert_eq!(
            SortDirection::from_code("descending"),
            Some(SortDirection::Descending)
        );
        assert_eq!(SortDirection::ALL.len(), 2);
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
