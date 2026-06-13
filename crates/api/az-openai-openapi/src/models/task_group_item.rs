// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TaskGroupItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TaskGroupTask,
};

/// Collection of workflow tasks grouped together in the thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroupItem {
    /// Identifier of the thread item.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    /// Identifier of the parent thread.
    pub thread_id: String,
    /// Type discriminator that is always `chatkit.task_group`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Tasks included in the group.
    pub tasks: Vec<TaskGroupTask>,
}
