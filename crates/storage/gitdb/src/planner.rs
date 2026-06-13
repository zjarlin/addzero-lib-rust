//! Query planning and optimization.
//!
//! This module provides cost-based query planning that transforms SQL AST into
//! optimized physical execution plans.

mod error;
mod logical;
mod optimizer;
mod physical;
mod planner;

pub use error::{PlanError, PlanResult};
pub use logical::{JoinType, LogicalPlan};
pub use optimizer::{OptimizationRule, Optimizer};
pub use physical::{PhysicalOperator, PhysicalPlan};
pub use planner::QueryPlanner;
