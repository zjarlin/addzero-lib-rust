//! 物理查询计划表示。
//!
//! 物理计划描述查询“如何执行”，包括扫描方式、连接算法、聚合方式和估算成本。

use std::fmt;
use std::sync::Arc;

use super::logical::SortSpec;
use crate::sql::Expr;
use az_derive_aliases::{apply, plain_clone_debug, plain_code_display_no_default_enum};

/// 物理执行算子。
///
/// 这些枚举值对应查询实际执行时使用的算法或访问方式。
#[apply(plain_clone_debug)]
pub enum PhysicalOperator {
    /// 顺序表扫描。
    SeqScan {
        table: String,
        columns: Option<Vec<String>>,
        predicate: Option<Expr>,
    },

    /// 索引扫描；仅在索引存在时可用。
    IndexScan {
        table: String,
        index: String,
        key_range: KeyRange,
        columns: Option<Vec<String>>,
    },

    /// 过滤算子。
    Filter { predicate: Expr },

    /// 投影指定列。
    Project {
        columns: Vec<String>,
        expressions: Vec<(Expr, String)>,
    },

    /// 嵌套循环连接。
    NestedLoopJoin {
        join_type: JoinPhysicalType,
        condition: Option<Expr>,
    },

    /// 哈希连接，在右侧构建哈希表并用左侧探测。
    HashJoin {
        join_type: JoinPhysicalType,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
    },

    /// 归并连接，要求输入已排序。
    MergeJoin {
        join_type: JoinPhysicalType,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
    },

    /// 内存排序。
    Sort { order: Vec<SortSpec> },

    /// 外部排序，用于大数据集。
    ExternalSort {
        order: Vec<SortSpec>,
        memory_limit: usize,
    },

    /// 限制返回行数。
    Limit { limit: usize, offset: Option<usize> },

    /// 基于哈希表的聚合。
    HashAggregate {
        group_by: Vec<String>,
        aggregates: Vec<PhysicalAggregate>,
    },

    /// 流式聚合，要求输入已按分组键排序。
    StreamAggregate {
        group_by: Vec<String>,
        aggregates: Vec<PhysicalAggregate>,
    },

    /// 基于哈希表的去重。
    HashDistinct,

    /// 追加多个输入，用于 `UNION`。
    Append,
}

/// 索引扫描使用的键范围。
#[apply(plain_clone_debug)]
pub struct KeyRange {
    pub start: Option<KeyBound>,
    pub end: Option<KeyBound>,
}

/// 键范围边界。
#[apply(plain_clone_debug)]
pub struct KeyBound {
    pub value: String,
    pub inclusive: bool,
}

/// 带实现语义的物理连接类型。
#[apply(plain_code_display_no_default_enum)]
pub enum JoinPhysicalType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
    Cross,
}

/// 物理聚合表达式。
#[apply(plain_clone_debug)]
pub struct PhysicalAggregate {
    pub function: AggregatePhysical,
    pub input_column: Option<String>,
    pub output_column: String,
    pub distinct: bool,
}

/// 物理聚合函数。
#[apply(plain_code_display_no_default_enum)]
pub enum AggregatePhysical {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// 物理计划节点。
#[apply(plain_clone_debug)]
pub struct PhysicalPlanNode {
    pub operator: PhysicalOperator,
    pub children: Vec<Arc<PhysicalPlanNode>>,
    pub estimated_cost: f64,
    pub estimated_rows: usize,
}

impl PhysicalPlanNode {
    /// 创建物理计划节点。
    pub fn new(operator: PhysicalOperator) -> Self {
        Self {
            operator,
            children: Vec::new(),
            estimated_cost: 0.0,
            estimated_rows: 0,
        }
    }

    /// 添加一个子节点。
    pub fn with_child(mut self, child: Arc<PhysicalPlanNode>) -> Self {
        self.children.push(child);
        self
    }

    /// 添加多个子节点。
    pub fn with_children(mut self, children: Vec<Arc<PhysicalPlanNode>>) -> Self {
        self.children = children;
        self
    }

    /// 设置节点估算成本。
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost = cost;
        self
    }

    /// 设置节点估算输出行数。
    pub fn with_rows(mut self, rows: usize) -> Self {
        self.estimated_rows = rows;
        self
    }

    /// 计算包含子树在内的总估算成本。
    pub fn total_cost(&self) -> f64 {
        let child_cost: f64 = self.children.iter().map(|c| c.total_cost()).sum();
        self.estimated_cost + child_cost
    }
}

/// 完整的物理查询计划。
#[apply(plain_clone_debug)]
pub struct PhysicalPlan {
    pub root: Arc<PhysicalPlanNode>,
}

impl PhysicalPlan {
    /// 创建物理查询计划。
    pub fn new(root: PhysicalPlanNode) -> Self {
        Self {
            root: Arc::new(root),
        }
    }

    /// 返回总估算成本。
    pub fn total_cost(&self) -> f64 {
        self.root.total_cost()
    }

    /// 返回估算输出行数。
    pub fn estimated_rows(&self) -> usize {
        self.root.estimated_rows
    }
}

impl fmt::Display for PhysicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Physical Plan (estimated cost: {:.2}):",
            self.total_cost()
        )?;
        self.format_node(f, &self.root, 0)
    }
}

impl PhysicalPlan {
    fn format_node(
        &self,
        f: &mut fmt::Formatter<'_>,
        node: &PhysicalPlanNode,
        indent: usize,
    ) -> fmt::Result {
        let pad = "  ".repeat(indent);

        match &node.operator {
            PhysicalOperator::SeqScan {
                table, predicate, ..
            } => {
                write!(f, "{}SeqScan: {}", pad, table)?;
                if let Some(pred) = predicate {
                    write!(f, " WHERE {:?}", pred)?;
                }
            }
            PhysicalOperator::IndexScan { table, index, .. } => {
                write!(f, "{}IndexScan: {} using {}", pad, table, index)?;
            }
            PhysicalOperator::Filter { predicate } => {
                write!(f, "{}Filter: {:?}", pad, predicate)?;
            }
            PhysicalOperator::Project { columns, .. } => {
                write!(f, "{}Project: [{}]", pad, columns.join(", "))?;
            }
            PhysicalOperator::NestedLoopJoin { join_type, .. } => {
                write!(f, "{}NestedLoopJoin: {}", pad, join_type)?;
            }
            PhysicalOperator::HashJoin {
                join_type,
                left_keys,
                right_keys,
            } => {
                write!(
                    f,
                    "{}HashJoin: {} on {:?} = {:?}",
                    pad, join_type, left_keys, right_keys
                )?;
            }
            PhysicalOperator::MergeJoin { join_type, .. } => {
                write!(f, "{}MergeJoin: {}", pad, join_type)?;
            }
            PhysicalOperator::Sort { order } => {
                let cols: Vec<_> = order.iter().map(|s| &s.column).collect();
                write!(f, "{}Sort: {:?}", pad, cols)?;
            }
            PhysicalOperator::ExternalSort {
                order,
                memory_limit,
            } => {
                let cols: Vec<_> = order.iter().map(|s| &s.column).collect();
                write!(
                    f,
                    "{}ExternalSort: {:?} (limit: {})",
                    pad, cols, memory_limit
                )?;
            }
            PhysicalOperator::Limit { limit, offset } => {
                write!(f, "{}Limit: {}", pad, limit)?;
                if let Some(o) = offset {
                    write!(f, " OFFSET {}", o)?;
                }
            }
            PhysicalOperator::HashAggregate {
                group_by,
                aggregates,
            } => {
                let aggs: Vec<_> = aggregates.iter().map(|a| &a.output_column).collect();
                write!(
                    f,
                    "{}HashAggregate: group={:?}, aggs={:?}",
                    pad, group_by, aggs
                )?;
            }
            PhysicalOperator::StreamAggregate { group_by, .. } => {
                write!(f, "{}StreamAggregate: group={:?}", pad, group_by)?;
            }
            PhysicalOperator::HashDistinct => {
                write!(f, "{}HashDistinct", pad)?;
            }
            PhysicalOperator::Append => {
                write!(f, "{}Append", pad)?;
            }
        }

        writeln!(
            f,
            " (rows: {}, cost: {:.2})",
            node.estimated_rows, node.estimated_cost
        )?;

        for child in &node.children {
            self.format_node(f, child, indent + 1)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_plan_cost() {
        let scan = PhysicalPlanNode::new(PhysicalOperator::SeqScan {
            table: "users".to_string(),
            columns: None,
            predicate: None,
        })
        .with_cost(100.0)
        .with_rows(1000);

        let filter = PhysicalPlanNode::new(PhysicalOperator::Filter {
            predicate: Expr::Column("x".into()),
        })
        .with_cost(10.0)
        .with_rows(500)
        .with_child(Arc::new(scan));

        let plan = PhysicalPlan::new(filter);
        assert_eq!(plan.total_cost(), 110.0);
        assert_eq!(plan.estimated_rows(), 500);
    }

    #[test]
    fn test_physical_join() {
        let left = PhysicalPlanNode::new(PhysicalOperator::SeqScan {
            table: "users".to_string(),
            columns: None,
            predicate: None,
        })
        .with_cost(100.0)
        .with_rows(1000);

        let right = PhysicalPlanNode::new(PhysicalOperator::SeqScan {
            table: "orders".to_string(),
            columns: None,
            predicate: None,
        })
        .with_cost(200.0)
        .with_rows(5000);

        let join = PhysicalPlanNode::new(PhysicalOperator::HashJoin {
            join_type: JoinPhysicalType::Inner,
            left_keys: vec!["id".to_string()],
            right_keys: vec!["user_id".to_string()],
        })
        .with_cost(500.0)
        .with_rows(5000)
        .with_children(vec![Arc::new(left), Arc::new(right)]);

        let plan = PhysicalPlan::new(join);
        assert_eq!(plan.total_cost(), 800.0); // 100 + 200 + 500
    }

    #[test]
    fn test_physical_enum_codes_and_display() {
        assert_eq!(JoinPhysicalType::Inner.to_string(), "Inner");
        assert_eq!(JoinPhysicalType::LeftOuter.to_string(), "LeftOuter");
        assert_eq!(AggregatePhysical::Count.to_string(), "Count");
        assert_eq!(JoinPhysicalType::LeftOuter.code(), "left_outer");
        assert_eq!(
            JoinPhysicalType::from_code("full_outer"),
            Some(JoinPhysicalType::FullOuter)
        );
        assert_eq!(AggregatePhysical::Avg.code(), "avg");
        assert_eq!(
            AggregatePhysical::from_code("max"),
            Some(AggregatePhysical::Max)
        );
        assert_eq!(JoinPhysicalType::ALL.len(), 5);
        assert_eq!(AggregatePhysical::ALL.len(), 5);
    }
}
