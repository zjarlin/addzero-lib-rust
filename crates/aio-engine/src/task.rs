//! 任务流编排引擎。
//!
//! 支持串行/并行/条件分支/定时触发/依赖编排。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A node in a task flow graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub label: String,
    /// Script to execute, or pluggable node type.
    pub action: TaskAction,
    /// Dependencies (must complete before this node runs).
    pub depends_on: Vec<String>,
    /// Retry configuration.
    pub retry: Option<TaskRetry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskAction {
    RunScript {
        lang: String,
        source: String,
    },
    PluginNode {
        plugin_id: String,
        node_type: String,
        config: serde_json::Value,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRetry {
    pub max_attempts: u32,
    pub delay_ms: u64,
}

/// Task flow execution engine trait.
pub trait TaskFlow: Send + Sync {
    fn execute(&self, nodes: Vec<TaskNode>, vars: BTreeMap<String, String>) -> TaskResult;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub node_results: BTreeMap<String, serde_json::Value>,
    pub duration_ms: u64,
}
