// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TaskGroupTask` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TaskType,
};

/// Task entry that appears within a TaskGroup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroupTask {
    /// Subtype for the grouped task.
    #[serde(rename = "type")]
    pub type_value: TaskType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
