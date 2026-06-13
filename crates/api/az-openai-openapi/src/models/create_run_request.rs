// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateRunRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantsApiResponseFormatOption,
    CreateMessageRequest,
    CreateRunRequestModel,
    CreateRunRequestTool,
    CreateRunRequestToolChoice,
    CreateRunRequestTruncationStrategy,
    Metadata,
    ParallelToolCalls,
    ReasoningEffort,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunRequest {
    /// The ID of the [assistant](/docs/api-reference/assistants) to use to execute this run.
    pub assistant_id: String,
    /// The ID of the [Model](/docs/api-reference/models) to be used to execute this run. If a value is
    /// provided here, it will override the model associated with the assistant. If not, the model
    /// associated with the assistant will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<CreateRunRequestModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Overrides the [instructions](/docs/api-reference/assistants/createAssistant) of the assistant. This
    /// is useful for modifying the behavior on a per-run basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Appends additional instructions at the end of the instructions for the run. This is useful for
    /// modifying the behavior on a per-run basis without overriding other instructions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_instructions: Option<String>,
    /// Adds additional messages to the thread before creating the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_messages: Option<Vec<CreateMessageRequest>>,
    /// Override the tools the assistant can use for this run. This is useful for modifying the behavior on
    /// a per-run basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<CreateRunRequestTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more
    /// random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the
    /// results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top
    /// 10% probability mass are considered. We generally recommend altering this or temperature but not
    /// both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// If `true`, returns a stream of events that happen during the Run as server-sent events, terminating
    /// when the Run enters a terminal state with a `data: [DONE]` message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// The maximum number of prompt tokens that may be used over the course of the run. The run will make a
    /// best effort to use only the number of prompt tokens specified, across multiple turns of the run. If
    /// the run exceeds the number of prompt tokens specified, the run will end with status `incomplete`.
    /// See `incomplete_details` for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<i32>,
    /// The maximum number of completion tokens that may be used over the course of the run. The run will
    /// make a best effort to use only the number of completion tokens specified, across multiple turns of
    /// the run. If the run exceeds the number of completion tokens specified, the run will end with status
    /// `incomplete`. See `incomplete_details` for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: Option<CreateRunRequestTruncationStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<CreateRunRequestToolChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<ParallelToolCalls>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantsApiResponseFormatOption>,
}
