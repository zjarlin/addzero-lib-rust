//! Query optimizer with rule-based and cost-based optimizations.
//!
//! The optimizer transforms logical plans to improve execution efficiency.

use std::sync::Arc;


use super::logical::{LogicalPlan, ProjectColumn};
use super::physical::{
    AggregatePhysical, JoinPhysicalType, PhysicalAggregate, PhysicalOperator, PhysicalPlan,
    PhysicalPlanNode,
};
use crate::sql::Expr;

/// Cost model constants.
mod cost {
    /// Cost per row for sequential scan.
    pub const SEQ_SCAN_PER_ROW: f64 = 1.0;
    /// Cost per row for filter evaluation.
    pub const FILTER_PER_ROW: f64 = 0.1;
    /// Cost per row for projection.
    pub const PROJECT_PER_ROW: f64 = 0.05;
    /// Cost per row for sorting (n log n).
    pub const SORT_PER_ROW: f64 = 2.0;
    /// Cost per row for hash join probe.
    pub const HASH_JOIN_PER_ROW: f64 = 0.5;
    /// Cost to build hash table per row.
    pub const HASH_BUILD_PER_ROW: f64 = 1.5;
    /// Cost per row for nested loop join.
    pub const NESTED_LOOP_PER_ROW: f64 = 10.0;
    /// Cost per row for hash aggregation.
    pub const HASH_AGG_PER_ROW: f64 = 0.8;
    /// Threshold for choosing external sort.
    pub const EXTERNAL_SORT_THRESHOLD: usize = 100_000;
}

/// Optimization rule trait.
pub trait OptimizationRule: Send + Sync {
    /// Name of the rule.
    fn name(&self) -> &str;

    /// Apply the rule to a logical plan, returning a potentially optimized plan.
    fn apply(&self, plan: &LogicalPlan) -> Option<LogicalPlan>;
}

/// Predicate pushdown rule - pushes filters closer to data sources.
pub struct PredicatePushdown;

impl OptimizationRule for PredicatePushdown {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn apply(&self, plan: &LogicalPlan) -> Option<LogicalPlan> {
        match plan {
            // Push filter through projection if possible.
            LogicalPlan::Filter { input, predicate } => {
                match input.as_ref() {
                    // Push filter below projection.
                    LogicalPlan::Project {
                        input: proj_input,
                        columns,
                    } => Some(LogicalPlan::Project {
                        input: Box::new(LogicalPlan::Filter {
                            input: proj_input.clone(),
                            predicate: predicate.clone(),
                        }),
                        columns: columns.clone(),
                    }),
                    // Merge consecutive filters.
                    LogicalPlan::Filter {
                        input: inner_input,
                        predicate: inner_pred,
                    } => Some(LogicalPlan::Filter {
                        input: inner_input.clone(),
                        predicate: Expr::BinaryOp {
                            left: Box::new(predicate.clone()),
                            op: crate::sql::BinaryOperator::And,
                            right: Box::new(inner_pred.clone()),
                        },
                    }),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Projection pushdown - pushes projections closer to data sources.
pub struct ProjectionPushdown;

impl OptimizationRule for ProjectionPushdown {
    fn name(&self) -> &str {
        "ProjectionPushdown"
    }

    fn apply(&self, plan: &LogicalPlan) -> Option<LogicalPlan> {
        match plan {
            // Eliminate redundant projections.
            LogicalPlan::Project { input, columns } => {
                match input.as_ref() {
                    LogicalPlan::Project {
                        input: inner_input, ..
                    } => {
                        // Replace with single projection.
                        Some(LogicalPlan::Project {
                            input: inner_input.clone(),
                            columns: columns.clone(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Limit pushdown - pushes limits as close to sources as possible.
pub struct LimitPushdown;

impl OptimizationRule for LimitPushdown {
    fn name(&self) -> &str {
        "LimitPushdown"
    }

    fn apply(&self, plan: &LogicalPlan) -> Option<LogicalPlan> {
        match plan {
            // Push limit through projection.
            LogicalPlan::Limit {
                input,
                limit,
                offset,
            } => match input.as_ref() {
                LogicalPlan::Project {
                    input: proj_input,
                    columns,
                } => Some(LogicalPlan::Project {
                    input: Box::new(LogicalPlan::Limit {
                        input: proj_input.clone(),
                        limit: *limit,
                        offset: *offset,
                    }),
                    columns: columns.clone(),
                }),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Constant folding - evaluates constant expressions at plan time.
pub struct ConstantFolding;

impl OptimizationRule for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, plan: &LogicalPlan) -> Option<LogicalPlan> {
        match plan {
            // If filter is constant true, eliminate it.
            LogicalPlan::Filter { input, predicate } => {
                match predicate {
                    Expr::Literal(crate::sql::LiteralValue::Boolean(true)) => {
                        Some(input.as_ref().clone())
                    }
                    // Constant false means empty result.
                    Expr::Literal(crate::sql::LiteralValue::Boolean(false)) => {
                        Some(LogicalPlan::Empty {
                            columns: input.output_columns(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// The query optimizer.
pub struct Optimizer {
    rules: Vec<Box<dyn OptimizationRule>>,
    max_iterations: usize,
}

impl Default for Optimizer {
    fn default() -> Self {
        Optimizer::new()
    }
}

impl Optimizer {
    /// Create a new optimizer with default rules.
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(ConstantFolding),
                Box::new(PredicatePushdown),
                Box::new(ProjectionPushdown),
                Box::new(LimitPushdown),
            ],
            max_iterations: 10,
        }
    }

    /// Add a custom optimization rule.
    pub fn add_rule(&mut self, rule: Box<dyn OptimizationRule>) {
        self.rules.push(rule);
    }

    /// Optimize a logical plan.
    pub fn optimize(&self, plan: LogicalPlan) -> anyhow::Result<LogicalPlan> {
        let mut current = plan;

        for _ in 0..self.max_iterations {
            let mut changed = false;

            // Apply rules recursively to the tree.
            let optimized = self.apply_rules_recursive(&current, &mut changed);

            if !changed {
                break;
            }

            current = optimized;
        }

        Ok(current)
    }

    fn apply_rules_recursive(&self, plan: &LogicalPlan, changed: &mut bool) -> LogicalPlan {
        // First, try to apply rules to this node.
        let mut current = plan.clone();

        for rule in &self.rules {
            if let Some(optimized) = rule.apply(&current) {
                *changed = true;
                current = optimized;
            }
        }

        // Then, recursively optimize children.
        match current {
            LogicalPlan::Filter { input, predicate } => LogicalPlan::Filter {
                input: Box::new(self.apply_rules_recursive(&input, changed)),
                predicate,
            },
            LogicalPlan::Project { input, columns } => LogicalPlan::Project {
                input: Box::new(self.apply_rules_recursive(&input, changed)),
                columns,
            },
            LogicalPlan::Join {
                left,
                right,
                join_type,
                on,
            } => LogicalPlan::Join {
                left: Box::new(self.apply_rules_recursive(&left, changed)),
                right: Box::new(self.apply_rules_recursive(&right, changed)),
                join_type,
                on,
            },
            LogicalPlan::Sort { input, order } => LogicalPlan::Sort {
                input: Box::new(self.apply_rules_recursive(&input, changed)),
                order,
            },
            LogicalPlan::Limit {
                input,
                limit,
                offset,
            } => LogicalPlan::Limit {
                input: Box::new(self.apply_rules_recursive(&input, changed)),
                limit,
                offset,
            },
            LogicalPlan::Aggregate {
                input,
                group_by,
                aggregates,
            } => LogicalPlan::Aggregate {
                input: Box::new(self.apply_rules_recursive(&input, changed)),
                group_by,
                aggregates,
            },
            LogicalPlan::Distinct { input } => LogicalPlan::Distinct {
                input: Box::new(self.apply_rules_recursive(&input, changed)),
            },
            LogicalPlan::Union { left, right } => LogicalPlan::Union {
                left: Box::new(self.apply_rules_recursive(&left, changed)),
                right: Box::new(self.apply_rules_recursive(&right, changed)),
            },
            // Leaf nodes stay the same.
            other => other,
        }
    }

    /// Convert a logical plan to a physical plan.
    pub fn to_physical(&self, plan: &LogicalPlan) -> anyhow::Result<PhysicalPlan> {
        let root = self.logical_to_physical(plan)?;
        Ok(PhysicalPlan::new(root))
    }

    fn logical_to_physical(&self, plan: &LogicalPlan) -> anyhow::Result<PhysicalPlanNode> {
        match plan {
            LogicalPlan::Scan { table, columns, .. } => {
                let estimated_rows = 1000; // Default estimate.
                let node = PhysicalPlanNode::new(PhysicalOperator::SeqScan {
                    table: table.clone(),
                    columns: columns.clone(),
                    predicate: None,
                })
                .with_cost(estimated_rows as f64 * cost::SEQ_SCAN_PER_ROW)
                .with_rows(estimated_rows);

                Ok(node)
            }

            LogicalPlan::Filter { input, predicate } => {
                let child = self.logical_to_physical(input)?;
                let input_rows = child.estimated_rows;
                let output_rows = input_rows / 3; // Selectivity estimate.

                let node = PhysicalPlanNode::new(PhysicalOperator::Filter {
                    predicate: predicate.clone(),
                })
                .with_cost(input_rows as f64 * cost::FILTER_PER_ROW)
                .with_rows(output_rows)
                .with_child(Arc::new(child));

                Ok(node)
            }

            LogicalPlan::Project { input, columns } => {
                let child = self.logical_to_physical(input)?;
                let rows = child.estimated_rows;

                // Extract column names from ProjectColumn.
                let col_names: Vec<String> = columns
                    .iter()
                    .filter_map(|c| match c {
                        ProjectColumn::Column(col) => Some(col.column.clone()),
                        ProjectColumn::Expr { alias, .. } => alias.clone(),
                        _ => None,
                    })
                    .collect();

                let node = PhysicalPlanNode::new(PhysicalOperator::Project {
                    columns: col_names,
                    expressions: Vec::new(),
                })
                .with_cost(rows as f64 * cost::PROJECT_PER_ROW)
                .with_rows(rows)
                .with_child(Arc::new(child));

                Ok(node)
            }

            LogicalPlan::Join {
                left,
                right,
                join_type,
                on,
            } => {
                let left_child = self.logical_to_physical(left)?;
                let right_child = self.logical_to_physical(right)?;

                let left_rows = left_child.estimated_rows;
                let right_rows = right_child.estimated_rows;

                // Choose join algorithm based on size.
                let (operator, join_cost) = if right_rows > 100 {
                    // Use hash join for larger right side.
                    let build_cost = right_rows as f64 * cost::HASH_BUILD_PER_ROW;
                    let probe_cost = left_rows as f64 * cost::HASH_JOIN_PER_ROW;

                    (
                        PhysicalOperator::HashJoin {
                            join_type: self.convert_join_type(join_type),
                            left_keys: vec![], // Would be extracted from ON condition.
                            right_keys: vec![],
                        },
                        build_cost + probe_cost,
                    )
                } else {
                    // Use nested loop for small right side.
                    let nested_cost = (left_rows * right_rows) as f64 * cost::NESTED_LOOP_PER_ROW;

                    (
                        PhysicalOperator::NestedLoopJoin {
                            join_type: self.convert_join_type(join_type),
                            condition: on.clone(),
                        },
                        nested_cost,
                    )
                };

                let output_rows = (left_rows * right_rows) / 100; // Selectivity estimate.

                let node = PhysicalPlanNode::new(operator)
                    .with_cost(join_cost)
                    .with_rows(output_rows)
                    .with_children(vec![Arc::new(left_child), Arc::new(right_child)]);

                Ok(node)
            }

            LogicalPlan::Sort { input, order } => {
                let child = self.logical_to_physical(input)?;
                let rows = child.estimated_rows;

                // Choose sort algorithm based on size.
                let (operator, sort_cost) = if rows > cost::EXTERNAL_SORT_THRESHOLD {
                    (
                        PhysicalOperator::ExternalSort {
                            order: order.clone(),
                            memory_limit: 1024 * 1024 * 100, // 100MB.
                        },
                        rows as f64 * cost::SORT_PER_ROW * 2.0,
                    )
                } else {
                    (
                        PhysicalOperator::Sort {
                            order: order.clone(),
                        },
                        rows as f64 * cost::SORT_PER_ROW * (rows as f64).log2(),
                    )
                };

                let node = PhysicalPlanNode::new(operator)
                    .with_cost(sort_cost)
                    .with_rows(rows)
                    .with_child(Arc::new(child));

                Ok(node)
            }

            LogicalPlan::Limit {
                input,
                limit,
                offset,
            } => {
                let child = self.logical_to_physical(input)?;
                let output_rows = (*limit).min(child.estimated_rows);

                let node = PhysicalPlanNode::new(PhysicalOperator::Limit {
                    limit: *limit,
                    offset: *offset,
                })
                .with_cost(output_rows as f64 * 0.01) // Very cheap.
                .with_rows(output_rows)
                .with_child(Arc::new(child));

                Ok(node)
            }

            LogicalPlan::Aggregate {
                input,
                group_by,
                aggregates,
            } => {
                let child = self.logical_to_physical(input)?;
                let input_rows = child.estimated_rows;

                let output_rows = if group_by.is_empty() {
                    1
                } else {
                    input_rows / 10 // Estimate distinct groups.
                };

                let physical_aggs: Vec<PhysicalAggregate> = aggregates
                    .iter()
                    .map(|a| PhysicalAggregate {
                        function: match a.function {
                            super::logical::AggregateFunction::Count => AggregatePhysical::Count,
                            super::logical::AggregateFunction::Sum => AggregatePhysical::Sum,
                            super::logical::AggregateFunction::Avg => AggregatePhysical::Avg,
                            super::logical::AggregateFunction::Min => AggregatePhysical::Min,
                            super::logical::AggregateFunction::Max => AggregatePhysical::Max,
                            super::logical::AggregateFunction::CountDistinct => {
                                AggregatePhysical::Count
                            }
                        },
                        input_column: a.column.clone(),
                        output_column: a.alias.clone(),
                        distinct: matches!(
                            a.function,
                            super::logical::AggregateFunction::CountDistinct
                        ),
                    })
                    .collect();

                let node = PhysicalPlanNode::new(PhysicalOperator::HashAggregate {
                    group_by: group_by.clone(),
                    aggregates: physical_aggs,
                })
                .with_cost(input_rows as f64 * cost::HASH_AGG_PER_ROW)
                .with_rows(output_rows)
                .with_child(Arc::new(child));

                Ok(node)
            }

            LogicalPlan::Distinct { input } => {
                let child = self.logical_to_physical(input)?;
                let rows = child.estimated_rows / 2;

                let node = PhysicalPlanNode::new(PhysicalOperator::HashDistinct)
                    .with_cost(child.estimated_rows as f64 * cost::HASH_AGG_PER_ROW)
                    .with_rows(rows)
                    .with_child(Arc::new(child));

                Ok(node)
            }

            LogicalPlan::Union { left, right } => {
                let left_child = self.logical_to_physical(left)?;
                let right_child = self.logical_to_physical(right)?;
                let rows = left_child.estimated_rows + right_child.estimated_rows;

                let node = PhysicalPlanNode::new(PhysicalOperator::Append)
                    .with_cost(rows as f64 * 0.01)
                    .with_rows(rows)
                    .with_children(vec![Arc::new(left_child), Arc::new(right_child)]);

                Ok(node)
            }

            LogicalPlan::Empty { .. } => {
                let node = PhysicalPlanNode::new(PhysicalOperator::SeqScan {
                    table: String::new(),
                    columns: None,
                    predicate: Some(Expr::Literal(crate::sql::LiteralValue::Boolean(false))),
                })
                .with_cost(0.0)
                .with_rows(0);

                Ok(node)
            }
        }
    }

    fn convert_join_type(&self, join_type: &super::logical::JoinType) -> JoinPhysicalType {
        match join_type {
            super::logical::JoinType::Inner => JoinPhysicalType::Inner,
            super::logical::JoinType::Left => JoinPhysicalType::LeftOuter,
            super::logical::JoinType::Right => JoinPhysicalType::RightOuter,
            super::logical::JoinType::Full => JoinPhysicalType::FullOuter,
            super::logical::JoinType::Cross => JoinPhysicalType::Cross,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_true() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let filter = LogicalPlan::Filter {
            input: Box::new(scan.clone()),
            predicate: Expr::Literal(crate::sql::LiteralValue::Boolean(true)),
        };

        let rule = ConstantFolding;
        let result = rule.apply(&filter).unwrap();

        // Filter with TRUE should be eliminated.
        assert!(matches!(result, LogicalPlan::Scan { .. }));
    }

    #[test]
    fn test_constant_folding_false() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let filter = LogicalPlan::Filter {
            input: Box::new(scan),
            predicate: Expr::Literal(crate::sql::LiteralValue::Boolean(false)),
        };

        let rule = ConstantFolding;
        let result = rule.apply(&filter).unwrap();

        // Filter with FALSE should become Empty.
        assert!(matches!(result, LogicalPlan::Empty { .. }));
    }

    #[test]
    fn test_optimizer_runs_rules() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let filter = LogicalPlan::Filter {
            input: Box::new(scan),
            predicate: Expr::Literal(crate::sql::LiteralValue::Boolean(true)),
        };

        let optimizer = Optimizer::new();
        let result = optimizer.optimize(filter).unwrap();

        // Constant folding should have eliminated the filter.
        assert!(matches!(result, LogicalPlan::Scan { .. }));
    }

    #[test]
    fn test_to_physical_scan() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };

        let optimizer = Optimizer::new();
        let physical = optimizer.to_physical(&scan).unwrap();

        assert!(physical.total_cost() > 0.0);
        assert!(physical.estimated_rows() > 0);
    }

    #[test]
    fn test_to_physical_filter() {
        let scan = LogicalPlan::Scan {
            table: "users".to_string(),
            alias: None,
            columns: None,
        };
        let filter = LogicalPlan::Filter {
            input: Box::new(scan),
            predicate: Expr::Column("active".into()),
        };

        let optimizer = Optimizer::new();
        let physical = optimizer.to_physical(&filter).unwrap();

        // Filter should reduce rows.
        assert!(physical.estimated_rows() < 1000);
    }
}
