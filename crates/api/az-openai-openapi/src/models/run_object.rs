// Generated from OpenAPI spec. Do not edit by hand.
//! `RunObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantsApiResponseFormatOption,
    Metadata,
    ParallelToolCalls,
    RunCompletionUsage,
    RunObjectIncompleteDetails,
    RunObjectLastError,
    RunObjectRequiredAction,
    RunObjectTool,
    RunObjectToolChoice,
    RunObjectTruncationStrategy,
};

/// Represents an execution run on a [thread](/docs/api-reference/threads).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `thread.run`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the run was created.
    pub created_at: i64,
    /// The ID of the [thread](/docs/api-reference/threads) that was executed on as a part of this run.
    pub thread_id: String,
    /// The ID of the [assistant](/docs/api-reference/assistants) used for execution of this run.
    pub assistant_id: String,
    /// The status of the run, which can be either `queued`, `in_progress`, `requires_action`, `cancelling`,
    /// `cancelled`, `failed`, `completed`, `incomplete`, or `expired`.
    pub status: String,
    /// Details on the action required to continue the run. Will be `null` if no action is required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_action: Option<RunObjectRequiredAction>,
    /// The last error associated with this run. Will be `null` if there are no errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<RunObjectLastError>,
    /// The Unix timestamp (in seconds) for when the run will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the run was started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the run was cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the run failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the run was completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// Details on why the run is incomplete. Will be `null` if the run is not incomplete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<RunObjectIncompleteDetails>,
    /// The model that the [assistant](/docs/api-reference/assistants) used for this run.
    pub model: String,
    /// The instructions that the [assistant](/docs/api-reference/assistants) used for this run.
    pub instructions: String,
    /// The list of tools that the [assistant](/docs/api-reference/assistants) used for this run.
    pub tools: Vec<RunObjectTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<RunCompletionUsage>,
    /// The sampling temperature used for this run. If not set, defaults to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// The nucleus sampling value used for this run. If not set, defaults to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// The maximum number of prompt tokens specified to have been used over the course of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<i32>,
    /// The maximum number of completion tokens specified to have been used over the course of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i32>,
    pub truncation_strategy: RunObjectTruncationStrategy,
    pub tool_choice: RunObjectToolChoice,
    pub parallel_tool_calls: ParallelToolCalls,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantsApiResponseFormatOption>,
}
