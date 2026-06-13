// Generated from OpenAPI spec. Do not edit by hand.
//! `TaskItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TaskType,
};

/// Task emitted by the workflow to show progress and status updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    /// Identifier of the thread item.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    /// Identifier of the parent thread.
    pub thread_id: String,
    /// Type discriminator that is always `chatkit.task`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Subtype for the task.
    pub task_type: TaskType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
