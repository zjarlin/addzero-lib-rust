// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
    RunStepCompletionUsage,
    RunStepObjectLastError,
    RunStepObjectStepDetails,
};

/// Represents a step in execution of a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepObject {
    /// The identifier of the run step, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `thread.run.step`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the run step was created.
    pub created_at: i64,
    /// The ID of the [assistant](/docs/api-reference/assistants) associated with the run step.
    pub assistant_id: String,
    /// The ID of the [thread](/docs/api-reference/threads) that was run.
    pub thread_id: String,
    /// The ID of the [run](/docs/api-reference/runs) that this run step is a part of.
    pub run_id: String,
    /// The type of run step, which can be either `message_creation` or `tool_calls`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The status of the run step, which can be either `in_progress`, `cancelled`, `failed`, `completed`,
    /// or `expired`.
    pub status: String,
    /// The details of the run step.
    pub step_details: RunStepObjectStepDetails,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<RunStepObjectLastError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<RunStepCompletionUsage>,
}
