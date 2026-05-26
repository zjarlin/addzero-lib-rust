//! Physical query plan representation.
//!
//! Physical plans specify *how* the query will actually be executed,
//! including specific algorithms and access methods.

use std::fmt;
use std::sync::Arc;

use super::logical::SortSpec;
use crate::sql::Expr;
use az_derive_aliases::{apply, plain_clone_debug, plain_copy_eq_display};

/// Physical execution operators.
///
/// These specify the actual algorithms used to execute the query.
#[apply(plain_clone_debug)]
pub enum PhysicalOperator {
    /// Sequential table scan.
    SeqScan {
        table: String,
        columns: Option<Vec<String>>,
        predicate: Option<Expr>,
    },

    /// Index scan (if indexes exist).
    IndexScan {
        table: String,
        index: String,
        key_range: KeyRange,
        columns: Option<Vec<String>>,
    },

    /// Filter operator.
    Filter { predicate: Expr },

    /// Project specific columns.
    Project {
        columns: Vec<String>,
        expressions: Vec<(Expr, String)>,
    },

    /// Nested loop join.
    NestedLoopJoin {
        join_type: JoinPhysicalType,
        condition: Option<Expr>,
    },

    /// Hash join (builds hash table on right, probes with left).
    HashJoin {
        join_type: JoinPhysicalType,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
    },

    /// Merge join (requires sorted inputs).
    MergeJoin {
        join_type: JoinPhysicalType,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
    },

    /// In-memory sort.
    Sort { order: Vec<SortSpec> },

    /// External sort (for large datasets).
    ExternalSort {
        order: Vec<SortSpec>,
        memory_limit: usize,
    },

    /// Limit rows.
    Limit { limit: usize, offset: Option<usize> },

    /// Hash-based aggregation.
    HashAggregate {
        group_by: Vec<String>,
        aggregates: Vec<PhysicalAggregate>,
    },

    /// Streaming aggregation (requires sorted input).
    StreamAggregate {
        group_by: Vec<String>,
        aggregates: Vec<PhysicalAggregate>,
    },

    /// Hash-based distinct.
    HashDistinct,

    /// Append multiple inputs (for UNION).
    Append,
}

/// Key range for index scans.
#[apply(plain_clone_debug)]
pub struct KeyRange {
    pub start: Option<KeyBound>,
    pub end: Option<KeyBound>,
}

/// Bound for key range.
#[apply(plain_clone_debug)]
pub struct KeyBound {
    pub value: String,
    pub inclusive: bool,
}

/// Physical join types with implementation details.
#[apply(plain_copy_eq_display)]
pub enum JoinPhysicalType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
    Cross,
}

/// Physical aggregate expression.
#[apply(plain_clone_debug)]
pub struct PhysicalAggregate {
    pub function: AggregatePhysical,
    pub input_column: Option<String>,
    pub output_column: String,
    pub distinct: bool,
}

/// Physical aggregate functions.
#[apply(plain_copy_eq_display)]
pub enum AggregatePhysical {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// A physical plan node.
#[apply(plain_clone_debug)]
pub struct PhysicalPlanNode {
    pub operator: PhysicalOperator,
    pub children: Vec<Arc<PhysicalPlanNode>>,
    pub estimated_cost: f64,
    pub estimated_rows: usize,
}

impl PhysicalPlanNode {
    /// Create a new physical plan node.
    pub fn new(operator: PhysicalOperator) -> Self {
        Self {
            operator,
            children: Vec::new(),
            estimated_cost: 0.0,
            estimated_rows: 0,
        }
    }

    /// Add a child node.
    pub fn with_child(mut self, child: Arc<PhysicalPlanNode>) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children.
    pub fn with_children(mut self, children: Vec<Arc<PhysicalPlanNode>>) -> Self {
        self.children = children;
        self
    }

    /// Set estimated cost.
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost = cost;
        self
    }

    /// Set estimated rows.
    pub fn with_rows(mut self, rows: usize) -> Self {
        self.estimated_rows = rows;
        self
    }

    /// Get the total cost of this plan including children.
    pub fn total_cost(&self) -> f64 {
        let child_cost: f64 = self.children.iter().map(|c| c.total_cost()).sum();
        self.estimated_cost + child_cost
    }
}

/// A complete physical query plan.
#[apply(plain_clone_debug)]
pub struct PhysicalPlan {
    pub root: Arc<PhysicalPlanNode>,
}

impl PhysicalPlan {
    /// Create a new physical plan.
    pub fn new(root: PhysicalPlanNode) -> Self {
        Self {
            root: Arc::new(root),
        }
    }

    /// Get the total estimated cost.
    pub fn total_cost(&self) -> f64 {
        self.root.total_cost()
    }

    /// Get the estimated output rows.
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
    fn test_join_physical_type_display() {
        assert_eq!(JoinPhysicalType::Inner.to_string(), "Inner");
        assert_eq!(JoinPhysicalType::LeftOuter.to_string(), "LeftOuter");
        assert_eq!(AggregatePhysical::Count.to_string(), "Count");
    }
}
